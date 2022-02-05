use std::detect::__is_feature_detected::sha;
use wgpu::{Buffer, Device, include_wgsl};
use wgpu::util::DeviceExt;
use winit::dpi::PhysicalSize;
use winit::event::{ElementState, Event, KeyboardInput, WindowEvent, VirtualKeyCode};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::platform::run_return::EventLoopExtRunReturn;
use winit::window::{Window, WindowBuilder};
use crate::gpu::base::{GPUBackend, Vert};

pub struct WgpuBackend {
    event_loop: EventLoop<()>,
    window: Window,
    is_open: bool,
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    vertex_count: usize,
}

impl WgpuBackend {
    pub fn new(window_label: &str, width: usize, height: usize) -> Self {
        env_logger::init();

        let event_loop = EventLoop::new();
        let window: Window = WindowBuilder::new()
            .with_title(window_label)
            .with_resizable(false)
            .with_inner_size(PhysicalSize::new(width as u32, height as u32))
            .build(&event_loop).unwrap();
        let instance = wgpu::Instance::new(wgpu::Backends::all());
        let surface = unsafe { instance.create_surface(&window) };

        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        })).unwrap();

        let (device, queue) = pollster::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                features: wgpu::Features::empty(),
                limits: wgpu::Limits::default(),
                label: None,
            },
            None,
        )).unwrap();

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_preferred_format(&adapter).unwrap(),
            width: width as u32,
            height: height as u32,
            present_mode: wgpu::PresentMode::Fifo,
        };
        surface.configure(&device, &config);

        let size = window.inner_size();

        let shader = device.create_shader_module(&include_wgsl!("shaders/shader.wgsl"));
        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });


        let vertex_buffer = Self::vertex_to_buffer(&device, &[]);
        let vertex_count = 0usize;

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<Vert>() as wgpu::BufferAddress,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &[
                        wgpu::VertexAttribute {
                            offset: 0,
                            shader_location: 0,
                            format: wgpu::VertexFormat::Float32x4,
                        },
                        wgpu::VertexAttribute {
                            offset: std::mem::size_of::<[f32;4]>() as wgpu::BufferAddress,
                            shader_location: 1,
                            format: wgpu::VertexFormat::Float32x4,
                        },
                    ]
                }],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                }],
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
            event_loop,
            window,
            is_open: true,
            surface,
            device,
            queue,
            config,
            size,
            render_pipeline,
            vertex_buffer,
            vertex_count
        }
    }

    fn vertex_to_buffer(device: &Device, vertex: &[Vert]) -> Buffer {
        let vertex_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(vertex),
                usage: wgpu::BufferUsages::VERTEX,
            }
        );
        vertex_buffer
    }

    fn render(&mut self) {
        let output = self.surface.get_current_texture().expect("Couldn't access surface texture");

        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Command Encoder for WgpuBackend")
        });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass for WgpuBackend triangles"),
                color_attachments: &[wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 1.0,
                        }),
                        store: true,
                    },
                }],
                depth_stencil_attachment: None,
            });


            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_pipeline(&self.render_pipeline);

            render_pass.draw(0..(self.vertex_count as u32), 0..1);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
    }
}

impl GPUBackend for WgpuBackend {
    fn update(&mut self) {
        self.render();

        self.event_loop.run_return(|event, _, control_flow| {
            match event {
                Event::RedrawEventsCleared => {
                    *control_flow = ControlFlow::Exit;
                }
                Event::WindowEvent { ref event, window_id } if window_id == self.window.id() => match event {
                    WindowEvent::CloseRequested |
                    WindowEvent::KeyboardInput {
                        input: KeyboardInput {
                            state: ElementState::Pressed,
                            virtual_keycode: Some(VirtualKeyCode::Escape),
                            ..
                        }, ..
                    } => {
                        self.is_open = false;
                        *control_flow = ControlFlow::Exit;
                    }
                    _ => {}
                }
                _ => (),
            }
        })
    }

    fn load_vertices(&mut self, vertices: Vec<Vert>) {
        self.vertex_buffer.destroy();
        self.vertex_buffer = Self::vertex_to_buffer(&self.device, vertices.as_slice());
        self.vertex_count = vertices.len();
    }

    fn is_open(&self) -> bool {
        self.is_open
    }
}