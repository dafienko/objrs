// https://sotrh.github.io/learn-wgpu/

mod camera;
mod model;

use cgmath::Matrix4;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::{WindowBuilder, Window},
};

use wgpu::util::DeviceExt;
use camera::{Camera, MatrixUniform};

use model::{Mesh, Vertex};

struct State {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
	render_pipeline: wgpu::RenderPipeline,
    window: Window,

	camera: Camera,
	camera_buffer: wgpu::Buffer,
	camera_uniform: MatrixUniform,
	camera_bind_group: wgpu::BindGroup,
	triangle: Mesh,
	tree: Mesh,
}

impl State {
    async fn new(window: Window) -> Self {
		let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
			backends: wgpu::Backends::all(),
            ..Default::default()
        });
        
        let surface = unsafe { instance.create_surface(&window) }.unwrap();
		
        let adapter = instance.request_adapter(
			&wgpu::RequestAdapterOptions {
				power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            },
        ).await.unwrap();
		
		let (device, queue) = adapter.request_device(
			&wgpu::DeviceDescriptor {
				features: wgpu::Features::empty(),
				limits: wgpu::Limits::default(),
				label: None,
			},
			None,
		).await.unwrap();
		
		let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps.formats.iter()
			.copied()
			.filter(|f| f.is_srgb())
			.next()
			.unwrap_or(surface_caps.formats[0]);
	
		let size = window.inner_size();
		let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
        };

        surface.configure(&device, &config);

		let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
			label: Some("Shader"),
			source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
		});

		let camera = Camera::new(
			Matrix4::from_translation((0.0, 0.0, 3.0).into()),
			// cgmath::Matrix4::look_at_lh((0.0, 0.0, -3.0).into(), (0.0, 0.0, 0.0).into(), (0.0, 1.0, 0.0).into()),
			config.width as f32 / config.height as f32,
			70.0,
			0.1,
			100.0
		);

		let camera_uniform = MatrixUniform::from_Matrix4(camera.view_proj());

		let camera_buffer = device.create_buffer_init(
			&wgpu::util::BufferInitDescriptor {
				label: Some("Camera Buffer"),
				contents: bytemuck::cast_slice(&[camera_uniform]),
				usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
			}
		);

		let camera_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
			entries: &[
				wgpu::BindGroupLayoutEntry {
					binding: 0,
					visibility: wgpu::ShaderStages::VERTEX,
					ty: wgpu::BindingType::Buffer {
						ty: wgpu::BufferBindingType::Uniform,
						has_dynamic_offset: false,
						min_binding_size: None,
					},
					count: None,
				}
			],
			label: Some("camera_bind_group_layout"),
		});

		let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
			layout: &camera_bind_group_layout,
			entries: &[
				wgpu::BindGroupEntry {
					binding: 0,
					resource: camera_buffer.as_entire_binding(),
				}
			],
			label: Some("camera_bind_group"),
		});

		let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
			label: Some("Render Pipeline Layout"),
			bind_group_layouts: &[
				&camera_bind_group_layout
			],
			push_constant_ranges: &[],
		});

		let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
			label: Some("Render Pipeline"),
			layout: Some(&render_pipeline_layout),
			vertex: wgpu::VertexState {
				module: &shader,
				entry_point: "vs_main", 
				buffers: &[
					Vertex::desc()
				],
			},
			fragment: Some(wgpu::FragmentState {
				module: &shader,
				entry_point: "fs_main",
				targets: &[Some(wgpu::ColorTargetState {
					format: config.format,
					blend: Some(wgpu::BlendState::REPLACE),
					write_mask: wgpu::ColorWrites::ALL,
				})],
			}),
			primitive: wgpu::PrimitiveState {
				topology: wgpu::PrimitiveTopology::TriangleList, 
				strip_index_format: None,
				front_face: wgpu::FrontFace::Ccw, 
				cull_mode: Some(wgpu::Face::Back),
				polygon_mode: wgpu::PolygonMode::Fill,
				unclipped_depth: false,
				conservative: false,
			},
			depth_stencil: None, 
			multisample: wgpu::MultisampleState {
				count: 1, 
				mask: !0, 
				alpha_to_coverage_enabled: false, 
			},
			multiview: None,
		});

		Self {
            window,
            surface,
			triangle: Mesh::new(&device),
			tree: Mesh::from_obj(&device).unwrap(),
            device,
            queue,
            config,
            size,
			render_pipeline,
			camera,
			camera_buffer,
			camera_uniform,
			camera_bind_group,
        }
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
		if new_size.width > 0 && new_size.height > 0 {
			self.size = new_size;
			self.config.width = new_size.width;
			self.config.height = new_size.height;
			self.surface.configure(&self.device, &self.config);
			self.camera.aspect = self.config.width as f32 / self.config.height as f32;
		}
	}

    fn input(&mut self, event: &WindowEvent) -> bool { 
		self.camera.input(event);
		false
	}

    fn update(&mut self) {
		self.camera.update();
		
        self.camera_uniform = MatrixUniform::from_Matrix4(self.camera.view_proj());
        self.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[self.camera_uniform]),
        );
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
		let output = self.surface.get_current_texture()?;
		let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
		let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
			label: Some("Render Encoder"),
		});

		{
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            render_pass.set_pipeline(&self.render_pipeline);
			render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
			// self.triangle.draw(&mut render_pass);
			self.tree.draw(&mut render_pass);
        }
	
		self.queue.submit(std::iter::once(encoder.finish()));
		output.present();
	
		Ok(())
    }
}

pub async fn run() {
    env_logger::init();

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

	let mut state = State::new(window).await;

    event_loop.run(move |event, _, control_flow| {
		match event {
			Event::RedrawRequested(window_id) if window_id == state.window().id() => {
				state.update();
				match state.render() {
					Ok(_) => {}
					Err(wgpu::SurfaceError::Lost) => state.resize(state.size),
					Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
					Err(e) => eprintln!("{:?}", e),
				}
			}
			Event::MainEventsCleared => {
				state.window().request_redraw();
			}
			Event::WindowEvent {
				ref event,
				window_id,
			} if window_id == state.window().id() => if !state.input(event) {
				match event {
					WindowEvent::CloseRequested | WindowEvent::KeyboardInput {
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
					WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
						state.resize(**new_inner_size);
					}
					_ => {}
				}
			}
			_ => {}
		}
    });
}

