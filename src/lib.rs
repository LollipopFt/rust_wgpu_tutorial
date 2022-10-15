use winit::{
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

struct State {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
}

impl State {
    async fn new(window: &Window) -> Self {
        let size = window.inner_size();

        // instance is handle to GPU: creates Adapters & Surfaces
        let instance = wgpu::Instance::new(wgpu::Backends::VULKAN);
        // surface is used to draw to window; needs to implement
        // raw-window-handle, thus it is unsafe
        let surface = unsafe { instance.create_surface(window) };
        // adapter is handle to graphics card: get info, create Device & Queue
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::default(),
                    label: None,
                },
                None,
            )
            .await
            .unwrap();

        // config is to define surface creation of SurfaceTexture
        let config = wgpu::SurfaceConfiguration {
            // how SurfaceTextures are used
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT, // to write to window
            // how SurfaceTextures are stored
            format: surface.get_supported_formats(&adapter)[0],
            // width & height cannot be 0
            width: size.width,
            height: size.height,
            // how to sync surface with display
            present_mode: wgpu::PresentMode::Fifo, // vsync
            alpha_mode: wgpu::CompositeAlphaMode::Opaque,
        };
        surface.configure(&device, &config);

        Self { surface, device, queue, config, size }
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    fn input(&mut self, event: &WindowEvent) -> bool {
        false
    }

    fn update(&mut self) {}

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        // wait for surface to provide SurfaceTexture to render to
        let output = self.surface.get_current_texture()?;
        let view =
            output.texture.create_view(&wgpu::TextureViewDescriptor::default());
        // builds command buffer to store commands to send to GPU
        let mut encoder = self.device.create_command_encoder(
            &wgpu::CommandEncoderDescriptor { label: Some("Render Encoder") },
        );
        // clearing the screen
        // use encoder to create RenderPass which has methods for actual drawing
        let render_pass =
            encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.,
                            g: 0.,
                            b: 0.,
                            a: 1.,
                        }),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });
        // dropped as encoder.finish() until mutable borrow here is released
        drop(render_pass);

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
        Ok(())
    }
}

pub async fn run() {
    env_logger::init();
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    let mut state = State::new(&window).await;

    event_loop.run(move |event, _, control_flow| match event {
        Event::RedrawRequested(window_id) if window_id == window.id() => {
            state.update();
            match state.render() {
                Ok(_) => {}
                // reconfigure surface if lost
                Err(wgpu::SurfaceError::Lost) => state.resize(state.size),
                // system out of mem, quit
                Err(wgpu::SurfaceError::OutOfMemory) => {
                    *control_flow = ControlFlow::Exit
                }
                Err(e) => eprintln!("{:?}", e),
            }
        }

        Event::MainEventsCleared => {
            // RedrawRequested only triggers once unless manually requested
            window.request_redraw();
        }

        Event::WindowEvent { ref event, window_id }
            if window_id == window.id() =>
        {
            if !state.input(event) {
                match event {
                    WindowEvent::CloseRequested
                    | WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(VirtualKeyCode::Escape),
                                ..
                            },
                        ..
                    } => *control_flow = ControlFlow::Exit,

                    WindowEvent::Resized(physical_size) => {
                        state.resize(physical_size.cast());
                    }
                    WindowEvent::ScaleFactorChanged {
                        new_inner_size, ..
                    } => state.resize(new_inner_size.cast()),
                    _ => {}
                }
            }
        }
        _ => {}
    });
}
