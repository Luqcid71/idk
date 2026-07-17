use glam::{Mat4, Vec3, mat4, vec3};
mod chunk;
mod mesh_data;
mod chunk_mesh;
mod world_manager;
use image::GenericImageView;
use noise::{NoiseFn, Perlin};
use std::collections::HashMap;
use std::collections::hash_map;
use std::f32::consts::FRAC_PI_2;
use std::sync::Arc;
use wgpu::util::DeviceExt;
use winit::window::CursorGrabMode;
use winit::window::Fullscreen;
use winit::{
    application::ApplicationHandler,
    event::{DeviceEvent, ElementState, KeyEvent, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop, OwnedDisplayHandle},
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowId},
};

use crate::chunk::Chunk;

const CHUNK_DIRECTIONS: [[i32; 2]; 4] = [
    [1, 0],  // East (+X)
    [-1, 0], // West (-X)
    [0, 1],  // South (+Z)
    [0, -1], // North (-Z)
];
struct Camera {
    position: Vec3,
    yaw: f32,
    pitch: f32,
}
#[derive(Default)]
struct CameraController {
    is_forward_pressed: bool,
    is_backward_pressed: bool,
    is_left_pressed: bool,
    is_right_pressed: bool,
    is_up_pressed: bool,
    is_down_pressed: bool,
}
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 3],
    tex_coords: [f32; 2],
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Uniforms {
    transform: [[f32; 4]; 4],
}
impl Vertex {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        }
    }
}

const CHUNK_SIZE: i32 = 25;
const CHUNK_VOLUME: usize = (CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE) as usize;





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
    chunks: HashMap<(i32, i32), Chunk>,
    

    uniform_buffers: Vec<wgpu::Buffer>,
    uniform_bind_groups: Vec<wgpu::BindGroup>,
    diffuse_bind_group: wgpu::BindGroup,
    start_time: std::time::Instant,
    depth_texture_view: wgpu::TextureView,
    camera: Camera,
    camera_controller: CameraController,

    object_positions: Vec<Vec3>,
}
impl Camera {
    fn build_view_matrix(&self) -> Mat4 {
        let rotation = Mat4::from_euler(glam::EulerRot::YXZ, self.yaw, self.pitch, 0.0);
        let forward = rotation.transform_vector3(Vec3::new(0.0, 0.0, -1.0));

        Mat4::look_at_rh(self.position, self.position + forward, Vec3::Y)
    }
}

impl State {
    
    fn create_index_buffer(device: &wgpu::Device, indices: &[u16], label: &str) -> wgpu::Buffer {
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(label),
            contents: bytemuck::cast_slice(indices),
            usage: wgpu::BufferUsages::INDEX,
        })
    }

    pub fn update_camera(&mut self, speed: f32) {
        // Calculate the direction we are currently facing
        let rotation =
            Mat4::from_euler(glam::EulerRot::YXZ, self.camera.yaw, self.camera.pitch, 0.0);
        let forward = rotation.transform_vector3(Vec3::new(0.0, 0.0, -1.0));
        let upward = rotation.transform_vector3(Vec3::new(0.0, 1.0, 0.0));

        // Calculate the vector pointing directly to our right
        let right = rotation.transform_vector3(Vec3::new(1.0, 0.0, 0.0));

        // Apply movement based on which keys are held
        if self.camera_controller.is_forward_pressed {
            self.camera.position += forward * speed;
        }
        if self.camera_controller.is_backward_pressed {
            self.camera.position -= forward * speed;
        }
        if self.camera_controller.is_right_pressed {
            self.camera.position += right * speed;
        }
        if self.camera_controller.is_left_pressed {
            self.camera.position -= right * speed;
        }
        if self.camera_controller.is_up_pressed {
            self.camera.position += upward * speed
        }
        if self.camera_controller.is_down_pressed {
            self.camera.position -= upward * speed;
        }
    }
    fn create_pipeline(
        device: &wgpu::Device,
        layout: &wgpu::PipelineLayout,
        shader: &wgpu::ShaderModule,
        surface_format: wgpu::TextureFormat,
        label: &str,
    ) -> wgpu::RenderPipeline {
        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some(label),
            layout: Some(layout),

            vertex: wgpu::VertexState {
                module: shader,
                entry_point: Some("vs_main"),
                buffers: &[Vertex::desc()],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: Some(true),
                depth_compare: Some(wgpu::CompareFunction::Less),
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        })
    }
    fn create_depth_texture(
        device: &wgpu::Device,
        size: winit::dpi::PhysicalSize<u32>,
    ) -> wgpu::TextureView {
        let size = wgpu::Extent3d {
            width: size.width.max(1),
            height: size.height.max(1),
            depth_or_array_layers: 1,
        };
        let desc = wgpu::TextureDescriptor {
            label: Some("Depth Texture"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        };
        let texture = device.create_texture(&desc);
        texture.create_view(&wgpu::TextureViewDescriptor::default())
    }
    async fn new(display: OwnedDisplayHandle, window: Arc<Window>) -> State {
        // Inside State::new()
        let camera = Camera {
            position: Vec3::new(0.0, 0.0, 3.0),
            yaw: 0.0,
            pitch: 0.0,
        };

        let camera_controller = CameraController::default();

        // Add them to your final `State { ... }` return block
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

        // Inside your initialization code:
        let diffuse_bytes = include_bytes!("placeholder.png"); // Assuming you have a dirt.png
        let diffuse_image = image::load_from_memory(diffuse_bytes).unwrap();
        let diffuse_rgba = diffuse_image.to_rgba8();
        let dimensions = diffuse_image.dimensions();

        let texture_size = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1,
        };

        let diffuse_texture = device.create_texture(&wgpu::TextureDescriptor {
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            label: Some("diffuse_texture"),
            view_formats: &[],
        });

        // Write the image data to the GPU
        queue.write_texture(
            wgpu::TexelCopyTextureInfoBase {
                texture: &diffuse_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &diffuse_rgba,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * dimensions.0),
                rows_per_image: Some(dimensions.1),
            },
            texture_size,
        );

        let diffuse_texture_view =
            diffuse_texture.create_view(&wgpu::TextureViewDescriptor::default());

        // THE SAMPLER (Nearest Neighbor for crisp pixels)
        let diffuse_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest, // CRISP!
            min_filter: wgpu::FilterMode::Nearest, // CRISP!
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });

        let model = Mat4::from_rotation_y(1.5);

        let view = Mat4::look_at_rh(Vec3::new(0.0, 0.0, 3.0), Vec3::ZERO, Vec3::Y);
        let projection = Mat4::perspective_rh(
            45.0_f32.to_radians(),
            size.width as f32 / size.height as f32,
            0.1,
            10000.0,
        );
        let mvp = projection * view * model;
        let uniforms = Uniforms {
            transform: mvp.to_cols_array_2d(),
        };

        let uniform_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("uniform layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });
        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
                label: Some("texture_bind_group_layout"),
            });

        let diffuse_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&diffuse_texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&diffuse_sampler),
                },
            ],
            label: Some("diffuse_bind_group"),
        });
        let object_positions = vec![
            Vec3::new(-2.0, 0.0, 0.0),
            Vec3::new(2.0, 0.0, 0.0),
            Vec3::new(0.0, 2.0, 0.0),
            Vec3::new(0.0, -2.0, 0.0),
        ];
        let mut uniform_buffers = Vec::new();
        let mut uniform_bind_groups = Vec::new();

        for _ in &object_positions {
            let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("uniform buffer"),
                contents: bytemuck::bytes_of(&Uniforms {
                    transform: Mat4::IDENTITY.to_cols_array_2d(),
                }),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });
            let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("uniform bind group"),
                layout: &uniform_bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: buffer.as_entire_binding(),
                }],
            });
            uniform_buffers.push(buffer);
            uniform_bind_groups.push(bind_group);
        }

        let surface = instance.create_surface(window.clone()).unwrap();

        let cap = surface.get_capabilities(&adapter);

        let surface_format = cap
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(cap.formats[0]);
        //add pipelines and shaders here

        let triangle_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("triangle shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/triangle.wgsl").into()),
        });

       
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("pipeline layout"),
            bind_group_layouts: &[
                Some(&uniform_bind_group_layout),
                Some(&texture_bind_group_layout),
            ],
            immediate_size: 0,
        });

        let triangle_pipeline = Self::create_pipeline(
            &device,
            &pipeline_layout,
            &triangle_shader,
            surface_format,
            "triangle pipeline",
        );

        

        let depth_texture_view = Self::create_depth_texture(&device, size);

        // 1. Generate our initial terrain chunk
        let manager = world_manager::WorldManager::generate_world(&device);
        let chunks = manager.chunks;

        let state = State {
            instance,
            window,
            device,
            queue,
            size,
            surface,
            surface_format,
            shaders: vec![triangle_shader, ],
            pipelines: vec![
                triangle_pipeline,
                
            ],
            chunks,
            uniform_buffers,
            uniform_bind_groups,
            diffuse_bind_group,
            start_time: std::time::Instant::now(),
            depth_texture_view,
            camera,
            camera_controller,

            object_positions,
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
        self.depth_texture_view = Self::create_depth_texture(&self.device, self.size);
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
        let time = self.start_time.elapsed().as_secs_f32();
        //self.Rotate(time, time, time);
        self.update_camera(0.5);

        let view = self.camera.build_view_matrix();
        let projection = Mat4::perspective_rh(
            45.0_f32.to_radians(), // 45 degrees is standard! 90 stretches things weirdly.
            self.size.width as f32 / self.size.height as f32,
            0.1,
            10000.0,
        );
        let view_proj = projection * view;

        // 2. Loop through each object, translate it, and write to its specific buffer
        for (i, position) in self.object_positions.iter().enumerate() {
            let model = Mat4::from_translation(*position);
            let mvp = view_proj * model; // Combine camera and object position

            let uniforms = Uniforms {
                transform: mvp.to_cols_array_2d(),
            };

            // This is the missing link! Writing the math to the GPU buffer
            self.queue
                .write_buffer(&self.uniform_buffers[i], 0, bytemuck::bytes_of(&uniforms));
        }

        let mut encoder = self.device.create_command_encoder(&Default::default());
        // Create the renderpass which will clear the screen.
        let mut renderpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &texture_view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.25,
                        g: 0.6,
                        b: 1.0,
                        a: 1.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &self.depth_texture_view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });

        // If you wanted to call any drawing commands, they would go here.

        // Bind your camera math (just like you did before)
        renderpass.set_bind_group(0, &self.uniform_bind_groups[0], &[]);
        renderpass.set_bind_group(1, &self.diffuse_bind_group, &[]);

        // Bind your voxel shader pipeline
        renderpass.set_pipeline(&self.pipelines[0]);

        // Draw the chunk!
        /*if self.chunk_num_indices > 0 {
                    renderpass.set_vertex_buffer(0, self.chunk_vertex_buffer.slice(..));
                    renderpass.set_index_buffer(self.chunk_index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                    renderpass.draw_indexed(0..self.chunk_num_indices, 0, 0..1);
        }*/

        for (i, chunk) in self.chunks.values().enumerate() {
            if chunk.chunk_mesh.num_indices > 0 {
                renderpass.set_vertex_buffer(0, chunk.chunk_mesh.vertex_buffer.slice(..));
                renderpass.set_index_buffer(
                    chunk.chunk_mesh.index_buffer.slice(..),
                    wgpu::IndexFormat::Uint16,
                );
                renderpass.draw_indexed(0..chunk.chunk_mesh.num_indices, 0, 0..1);
            } else {
                println!("failed to render chunk: {i}");
            }
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
    fn device_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        device_id: winit::event::DeviceId,
        event: winit::event::DeviceEvent,
    ) {
        let state = self.state.as_mut().unwrap();
        if let DeviceEvent::MouseMotion { delta } = event {
            let sensitivitty = 0.002;
            state.camera.yaw -= (delta.0 as f32) * sensitivitty;
            state.camera.pitch -= (delta.1 as f32) * sensitivitty;

            let safe_pitch = FRAC_PI_2 - 0.001;
            state.camera.pitch = state.camera.pitch.clamp(-safe_pitch, safe_pitch);
        }
    }
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // Create window object
        let window = Arc::new(
            event_loop
                .create_window(
                    Window::default_attributes()
                        .with_fullscreen(Some(Fullscreen::Borderless((None)))),
                )
                .unwrap(),
        );

        let state = pollster::block_on(State::new(
            event_loop.owned_display_handle(),
            window.clone(),
        ));
        window.set_cursor_visible(false);

        if let Err(err) = window.set_cursor_grab(CursorGrabMode::Locked) {
            let _ = window.set_cursor_grab(CursorGrabMode::Confined);
        }
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
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(keycode),
                        state: key_state,
                        ..
                    },
                ..
            } => {
                let is_pressed = key_state == ElementState::Pressed;
                match keycode {
                    KeyCode::KeyW => state.camera_controller.is_forward_pressed = is_pressed,
                    KeyCode::KeyS => state.camera_controller.is_backward_pressed = is_pressed,
                    KeyCode::KeyA => state.camera_controller.is_left_pressed = is_pressed,
                    KeyCode::KeyD => state.camera_controller.is_right_pressed = is_pressed,
                    KeyCode::KeyE => state.camera_controller.is_up_pressed = is_pressed,
                    KeyCode::KeyQ => state.camera_controller.is_down_pressed = is_pressed,
                    _ => {}
                }
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
