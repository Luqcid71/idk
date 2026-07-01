use std::sync::Arc;
use wgpu::util::DeviceExt;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop, OwnedDisplayHandle},
    window::{Window, WindowId},
};
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex{
    position: [f32; 3],
}
impl Vertex{
    fn desc() -> wgpu::VertexBufferLayout<'static>{
        wgpu::VertexBufferLayout{
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute{
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                }
            ],
        }
    }
}
const TRIANGLE_VERTICIES: &[Vertex] = &[
    Vertex{position: [0.0, 0.5, 0.0]},
    Vertex{position: [-0.5, -0.5, 0.0]},
    Vertex{position: [0.5, -0.5, 0.0]}
];
const SQUARE_VERTICIES: & [Vertex] =&[
    Vertex{position: [0.0, 0.0, 0.0]},
    Vertex{position: [0.0, 0.5, 0.0]},
    Vertex{position: [0.5, 0.5, 0.0]},
    Vertex{position: [0.0, 0.0, 0.0]},
    Vertex{position: [0.5, 0.5, 0.0]},
    Vertex{position: [0.5, 0.0, 0.0]},

];
const OCTAGON_VERTICES: & [Vertex] =&[
    Vertex{position: [-0.5, 0.5, 0.0]},
    Vertex{position: [-0.25, 1.0, 0.0]},
    Vertex{position: [-0.5, 0.75, 0.0]},
    Vertex{position: [-0.5, 0.5, 0.0]},
    Vertex{position: [-0.25, 0.25, 0.0]},
    Vertex{position: [-0.25, 1.0, 0.0]},
    Vertex{position: [-0.25, 0.25, 0.0]},
    Vertex{position: [-0.25, 1.0, 0.0]},
    Vertex{position: [0.25, 1.0, 0.0]},
    Vertex{position: [-0.25, 0.25, 0.0]},
    Vertex{position: [0.25, 0.25, 0.0]},
    Vertex{position: [0.25, 1.0, 0.0]},
    Vertex{position: [0.25, 0.25, 0.0]},
    Vertex{position: [0.5, 0.5, 0.0]},
    Vertex{position: [0.5, 0.75, 0.0]},
    Vertex{position: [0.25, 0.25, 0.0]},
    Vertex{position: [0.5, 0.75, 0.0]},
    Vertex{position: [0.25, 1.0, 0.0]},


];
struct State {
    instance: wgpu::Instance,
    window: Arc<Window>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    size: winit::dpi::PhysicalSize<u32>,
    surface: wgpu::Surface<'static>,
    surface_format: wgpu::TextureFormat,
    shaders: Vec<wgpu::ShaderModule>,
    pipelines: Vec<wgpu::RenderPipeline>,
    vertex_buffers: Vec<wgpu::Buffer>,
}

impl State {
    fn create_pipeline(
        device: &wgpu::Device,
        layout: &wgpu::PipelineLayout,
        shader: &wgpu::ShaderModule,
        surface_format: wgpu::TextureFormat,
        label: &str,
    ) -> wgpu::RenderPipeline{
        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor { 
             label: Some(label),
             layout: Some(layout),

             vertex: wgpu::VertexState{
                module: shader,
                entry_point: Some("vs_main"),
                buffers: &[Vertex::desc()],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
             },
             fragment: Some(wgpu::FragmentState{
                module: shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState{
                    format: surface_format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
             }),
             primitive: wgpu::PrimitiveState{
                topology: wgpu::PrimitiveTopology::TriangleList,
                ..Default::default()
             },
             depth_stencil: None,
             multisample: wgpu::MultisampleState::default(),
             multiview_mask: None,
             cache: None,
             })
    }
    async fn new(display: OwnedDisplayHandle, window: Arc<Window>) -> State {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_with_display_handle(
            Box::new(display),
        ));
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions::default())
            .await
            .unwrap();
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor::default())
            .await
            .unwrap();

        let size = window.inner_size();

        let surface = instance.create_surface(window.clone()).unwrap();
        let cap = surface.get_capabilities(&adapter);
        let surface_format = cap
    .formats
    .iter()
    .copied()
    .find(|f| f.is_srgb())
    .unwrap_or(cap.formats[0]);
//add pipelines and shaders here
        let triangle_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor{
            label : Some("triangle shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/triangle.wgsl").into()),
        });
        let square_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor{
            label : Some("triangle shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/Square.wgsl").into()),
        });
        let octagon_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor{
            label : Some("triangle shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/Octagon.wgsl").into()),
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor{
            label: Some("pipeline layout"),
            bind_group_layouts: &[],
            immediate_size: 0,
        });
        let triangle_pipeline = Self::create_pipeline(
            &device,
            &pipeline_layout,
            &triangle_shader,
            surface_format,
            "triangle pipeline",
        );
        let square_pipeline = Self::create_pipeline(
            &device,
            &pipeline_layout,
            &square_shader,
            surface_format,
            "square pipeline",
        );
        let octagon_pipeline = Self::create_pipeline(
            &device,
            &pipeline_layout,
            &octagon_shader,
            surface_format,
            "octagon pipeline",
        );
        let triangle_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor{
            label: Some("triangle vertex buffer"),
            contents: bytemuck::cast_slice(TRIANGLE_VERTICIES),
            usage:wgpu::BufferUsages::VERTEX,
        });
        let square_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor{
            label: Some("square vertex buffer"),
            contents: bytemuck::cast_slice(SQUARE_VERTICIES),
            usage:wgpu::BufferUsages::VERTEX,
        });
        let octagon_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor{
            label: Some("square vertex buffer"),
            contents: bytemuck::cast_slice(OCTAGON_VERTICES),
            usage:wgpu::BufferUsages::VERTEX,
        });



        let state = State {
            instance,
            window,
            device,
            queue,
            size,
            surface,
            surface_format,
            shaders: vec![triangle_shader, square_shader, octagon_shader],
            pipelines: vec![triangle_pipeline, square_pipeline, octagon_pipeline],
            vertex_buffers: vec![triangle_buffer, square_buffer, octagon_buffer],
        };

        // Configure surface for the first time
        state.configure_surface();

        state
    }

    fn get_window(&self) -> &Window {
        &self.window
    }

    fn configure_surface(&self) {
        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: self.surface_format,
            // Request compatibility with the sRGB-format texture view we‘re going to create later.
            view_formats: vec![self.surface_format.add_srgb_suffix()],
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            width: self.size.width,
            height: self.size.height,
            desired_maximum_frame_latency: 2,
            present_mode: wgpu::PresentMode::AutoVsync,
        };
        self.surface.configure(&self.device, &surface_config);
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.size = new_size;

        // reconfigure the surface
        self.configure_surface();
    }

    fn render(&mut self) {
        // Create texture view.
        // NOTE: We must handle Timeout because the surface may be unavailable
        // (e.g., when the window is occluded on macOS).
        let surface_texture = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(texture) => texture,
            wgpu::CurrentSurfaceTexture::Occluded | wgpu::CurrentSurfaceTexture::Timeout => return,
            wgpu::CurrentSurfaceTexture::Suboptimal(texture) => {
                drop(texture);
                self.configure_surface();
                return;
            }
            wgpu::CurrentSurfaceTexture::Outdated => {
                self.configure_surface();
                return;
            }
            wgpu::CurrentSurfaceTexture::Validation => {
                unreachable!("No error scope registered, so validation errors will panic")
            }
            wgpu::CurrentSurfaceTexture::Lost => {
                self.surface = self.instance.create_surface(self.window.clone()).unwrap();
                self.configure_surface();
                return;
            }
        };
        let texture_view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor {
                // Without add_srgb_suffix() the image we will be working with
                // might not be "gamma correct".
                format: Some(self.surface_format.add_srgb_suffix()),
                ..Default::default()
            });

        // Renders a GREEN screen
        let mut encoder = self.device.create_command_encoder(&Default::default());
        // Create the renderpass which will clear the screen.
        let mut renderpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &texture_view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color{
                        r: 0.1,
                        g: 0.1,
                        b: 0.1,
                        a: 1.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });

        // If you wanted to call any drawing commands, they would go here.
        renderpass.set_pipeline(&self.pipelines[0]);
        renderpass.set_vertex_buffer(0, self.vertex_buffers[0].slice(..));
        renderpass.draw(0..3, 0..1);
       
        renderpass.set_pipeline(&self.pipelines[1]);
        renderpass.set_vertex_buffer(0, self.vertex_buffers[1].slice(..));
        renderpass.draw(0..6, 0..1);
       
        renderpass.set_pipeline(&self.pipelines[2]);
        renderpass.set_vertex_buffer(0, self.vertex_buffers[2].slice(..));
        renderpass.draw(0..18, 0..1);
        // End the renderpass.
        for pipeline in &self.pipelines{

        }
        drop(renderpass);

        // Submit the command in the queue to execute
        self.queue.submit([encoder.finish()]);
        self.window.pre_present_notify();
        surface_texture.present();
    }
}

#[derive(Default)]
struct App {
    state: Option<State>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // Create window object
        let window = Arc::new(
            event_loop
                .create_window(Window::default_attributes())
                .unwrap(),
        );

        let state = pollster::block_on(State::new(
            event_loop.owned_display_handle(),
            window.clone(),
        ));
        self.state = Some(state);

        window.request_redraw();
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        let state = self.state.as_mut().unwrap();
        match event {
            WindowEvent::CloseRequested => {
                println!("The close button was pressed; stopping");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                state.render();
                // Emits a new redraw requested event.
                state.get_window().request_redraw();
            }
            WindowEvent::Resized(size) => {
                // Reconfigures the size of the surface. We do not re-render
                // here as this event is always followed up by redraw request.
                state.resize(size);
            }
            _ => (),
        }
    }
}

fn main() {
    // wgpu uses `log` for all of our logging, so we initialize a logger with the `env_logger` crate.
    //
    // To change the log level, set the `RUST_LOG` environment variable. See the `env_logger`
    // documentation for more information.
    env_logger::init();

    let event_loop = EventLoop::new().unwrap();

    // When the current loop iteration finishes, immediately begin a new
    // iteration regardless of whether or not new events are available to
    // process. Preferred for applications that want to render as fast as
    // possible, like games.
    event_loop.set_control_flow(ControlFlow::Poll);

    // When the current loop iteration finishes, suspend the thread until
    // another event arrives. Helps keeping CPU utilization low if nothing
    // is happening, which is preferred if the application might be idling in
    // the background.
    // event_loop.set_control_flow(ControlFlow::Wait);

    let mut app = App::default();
    event_loop.run_app(&mut app).unwrap();
}