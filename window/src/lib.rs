use winit::{
    event::*,
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

        // the instance is a handle to the GPU
        let instance = wgpu::Instance::new(wgpu::Backends::VULKAN);
        // unsafe to draw directly to the screen
        let surface = unsafe { instance.create_surface(window) };
        // handle to actual graphics card: info about graphics card
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
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
                None, // trace path
            )
            .await
            .unwrap();
        let config = wgpu::SurfaceConfiguration {
            // RENDER_ATTACHMENT: textures used to write to the screen
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            // how SurfaceTextures will be stored on the GPU
            format: surface.get_supported_formats(&adapter)[0],
            // width & height of SurfaceTexture, normally also window
            width: size.width,
            height: size.height,
            // Fifo: cap display rate at display's framerate (VSync)
            present_mode: wgpu::PresentMode::Fifo,
        };
        surface.configure(&device, &config);
        Self {
            surface,
            device,
            queue,
            config,
            size,
        }
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    // indicates whether an event has been fully processed:
    // true = main loop will not process event any further
    fn input(&mut self, _event: &WindowEvent) -> bool {
        false
    }

    // move around objects
    fn update(&mut self) {}

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        // surface provides new SurfaceTexture to render to
        let output = self.surface.get_current_texture()?;
        // TextureView with default settings:
        // control how render code interacts with texture
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        // create commands to send to gpu
        let mut encoder = self.device.create_command_encoder(
            &wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            },
        );
        // block needed to release mutable borrow of encoder by dropping any
        // variables within it
        {
            encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    // texture to save colours to
                    view: &view,
                    // texture receiving output: no need to specify = None
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        // store rendered results to Texture behind TextureView
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });
        }

        // finish command buffer, submit to gpu render queue
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
        Event::WindowEvent {
            ref event,
            window_id,
        } if window_id == window.id() => {
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
                        state.resize(*physical_size);
                    }
                    WindowEvent::ScaleFactorChanged {
                        new_inner_size, ..
                        // new_inner_size is &&mut so derefence twice
                    } => state.resize(**new_inner_size),
                    _ => {}
                }
            }
        }
        Event::RedrawRequested(window_id) if window_id == window.id() => {
            state.update();
            match state.render() {
                Ok(_) => {}
                // reconfig surface if lost
                Err(wgpu::SurfaceError::Lost) => state.resize(state.size),
                // system out of memory, wait
                Err(wgpu::SurfaceError::OutOfMemory) => {
                    *control_flow = ControlFlow::Exit
                }
                Err(e) => eprintln!("{e:?}"),
            }
        }
        Event::MainEventsCleared => {
            // RedrawRequested only triggers once unless manually request
            window.request_redraw();
        }
        _ => {}
    });
}
