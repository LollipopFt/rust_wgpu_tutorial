use wgpu::util::DeviceExt;
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
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    num_vertices: u32,
}

#[repr(C)]
// bytemuck traits needed for fn cast_slice()
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    pos: [f32; 3],
    color: [f32; 3],
}

const VERTICES: &[Vertex] = &[
    Vertex { pos: [0., 0.5, 0.], color: [1., 0., 0.] },
    Vertex { pos: [-0.5, -0.5, 0.], color: [0., 1., 0.] },
    Vertex { pos: [0.5, -0.5, 0.], color: [0., 0., 1.] },
];

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

        let shader =
            device.create_shader_module(wgpu::include_wgsl!("shader.wgsl"));

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[],
                push_constant_ranges: &[],
            });

        let render_pipeline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Render Pipeline"),
                layout: Some(&render_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: "vs_main",
                    // tells wgpu what types of vertices to pass to vertex shader:
                    // vertices specified in vertex shader itself so this is empty
                    buffers: &[Vertex::desc()],
                },
                primitive: wgpu::PrimitiveState {
                    // 3 vertices correspond to 1 triangle
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    // tells wgpu whether given triangle is forward facing:
                    // facing forward if vertices are counter-clockwise
                    front_face: wgpu::FrontFace::Ccw,
                    // triangles not facing forward are culled
                    cull_mode: Some(wgpu::Face::Back),
                    unclipped_depth: false,
                    polygon_mode: wgpu::PolygonMode::Fill,
                    conservative: false,
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState {
                    // how many samples pipeline will use
                    count: 1,
                    // which samples are active: all samples used
                    mask: !0,
                    // anti-aliasing
                    alpha_to_coverage_enabled: false,
                },
                // stores colour data to surface
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: "fs_main",
                    // tells wgpu what colour outputs to set up
                    targets: &[Some(wgpu::ColorTargetState {
                        format: config.format,
                        blend: Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrites::COLOR,
                    })],
                }),
                // how many array layers render attachments can have:
                // not using array textures
                multiview: None,
            });

        let vertex_buffer =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(VERTICES),
                usage: wgpu::BufferUsages::VERTEX,
            });

        let num_vertices = VERTICES.len() as u32;

        Self {
            surface,
            device,
            queue,
            config,
            size,
            render_pipeline,
            vertex_buffer,
            num_vertices,
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
        let mut render_pass =
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
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        // tells wgpu to draw something with 3 vertices & 1 instance
        render_pass.draw(0..self.num_vertices, 0..1);
        // dropped as encoder.finish() until mutable borrow here is released
        drop(render_pass);

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
        Ok(())
    }
}

impl Vertex {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            // array stride: how wide a vertex is
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            // whether element in array is per-vertex/per-instance
            step_mode: wgpu::VertexStepMode::Vertex,
            // 1:1 mapping of struct fields
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    // shape of attribute: Float32x3 = vec3<f32> -> max is 32x4
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>()
                        as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
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
