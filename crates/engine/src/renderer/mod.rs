use std::cell::{Cell, OnceCell};

use bytemuck::{Pod, Zeroable};
use nalgebra_glm as glm;
use nalgebra_glm::{vec3, Vec2};
use wgpu::{util::DeviceExt, BindGroup, BufferUsages};
use winit::{event::WindowEvent, window::Window};

mod atlas;
mod camera;
mod vertex;

pub use atlas::*;
pub use camera::*;
pub use vertex::*;

pub struct Instance {
    pub size: f32,
    pub pos: Vec2,
    pub rotation: f32,
    pub tint: [u8; 3],
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct InstanceRaw {
    pub tint: [f32; 3],
    pub model: [[f32; 3]; 3],
}

impl From<&'_ Instance> for InstanceRaw {
    fn from(value: &'_ Instance) -> Self {
        let model = glm::translation2d(&value.pos) * glm::rotation2d(value.rotation);

        InstanceRaw {
            tint: value.tint.map(|x| x as f32 / 255.),
            model: model.into(),
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
struct TimeRaw {
    pub delta_time: f32,
    pub time_since_start_millis: u32,
}

pub struct Renderer {
    /* wgpu */
    pub window: Window,
    pub surface: wgpu::Surface,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,
    pub pipeline: wgpu::RenderPipeline,

    /* timers */
    pub start_time: OnceCell<std::time::Instant>,
    pub last_render_time: Option<std::time::Instant>,
    pub delta_time: f32,
    pub time_buffer: wgpu::Buffer,

    /* camera */
    pub camera: Cell<Camera>,
    pub camera_buffer: wgpu::Buffer,

    /* misc */
    pub atlas: Atlas,
    pub instances: wgpu::Buffer,

    /* bind groups */
    pub camera_bind_group: BindGroup,
    pub static_bind_group: BindGroup,
}

impl Renderer {
    pub async fn new(window: Window) -> Self {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            dx12_shader_compiler: Default::default(),
        });

        let surface = unsafe { instance.create_surface(&window) }.unwrap();

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
                    limits: if cfg!(target_arch = "wasm32") {
                        wgpu::Limits::downlevel_webgl2_defaults()
                    } else {
                        wgpu::Limits::default()
                    },
                    label: None,
                },
                None,
            )
            .await
            .unwrap();

        let camera = Camera {
            position: vec3(0., 0., -1.),
            ratio: 16f32 / 9f32,
            zoom: 1. / 10.,
        };

        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[CameraRaw::from(&camera)]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
                label: Some("Camera Bind Group Layout"),
            });

        let time_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Time Buffer"),
            contents: bytemuck::cast_slice(&[TimeRaw {
                delta_time: 0.,
                time_since_start_millis: 0,
            }]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: camera_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: time_buffer.as_entire_binding(),
                },
            ],
            label: Some("Camera Bind Group"),
        });

        let surface_caps = surface.get_capabilities(&adapter);

        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);

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

        let atlas = Atlas::default_from_device(&device, &queue);

        let static_bind_layout =
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
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
                        count: None,
                    },
                ],
                label: Some("Static Bind Layout"),
            });

        let shader = device.create_shader_module(wgpu::include_wgsl!("shaders/main.wgsl"));
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Pipeline Layout"),
            bind_group_layouts: &[&static_bind_layout, &camera_bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[Vertex::layout()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
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

        let static_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &static_bind_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&atlas.textures[0].1),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&atlas.sampler),
                },
            ],
            label: Some("Static Bind Group"),
        });

        let instances = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Instance Buffer"),
            size: std::mem::size_of::<InstanceRaw>() as u64 * 96,
            usage: BufferUsages::STORAGE,
            mapped_at_creation: false,
        });

        Renderer {
            instances,
            camera_bind_group,
            static_bind_group,
            atlas,
            surface,
            device,
            queue,
            config,
            size,
            window,
            pipeline,
            camera: Cell::new(camera),
            camera_buffer,
            start_time: OnceCell::default(),
            delta_time: 0.,
            last_render_time: None,
            time_buffer,
        }
    }

    pub fn update(&mut self) {}

    pub fn refresh_camera(&mut self) {
        self.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[CameraRaw::from(&*self.camera.get_mut())]),
        )
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.camera.get_mut().ratio = new_size.width as f32 / new_size.height as f32;
            self.refresh_camera();

            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    pub fn input(&mut self, event: &WindowEvent) -> bool {
        false
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    pub fn begin_frame<'a>(&'a mut self) -> FrameBuilder<'a> {
        FrameBuilder {
            renderer: self,
            command_queue: vec![],
        }
    }
}

pub struct FrameBuilder<'a> {
    renderer: &'a mut Renderer,
    command_queue: Vec<RenderCommand>,
}

impl FrameBuilder<'_> {
    pub fn draw_sprite(mut self, sprite_idx: u32, instance: Instance) -> Self {
        self.command_queue.push(RenderCommand::DrawSprite {
            sprite_idx,
            instance,
        });

        self
    }

    pub fn end_frame(mut self) -> Result<(), wgpu::SurfaceError> {
        let FrameBuilder {
            renderer,
            command_queue,
        } = self;

        let now = std::time::Instant::now();
        let start_time = renderer.start_time.get_or_init(|| now);
        let time_since_start_millis =
            // There is probably a more idiomatic way to do this
            (now.duration_since(start_time.to_owned()).as_millis() % u32::MAX as u128) as u32;
        let delta_time = now
            .duration_since(renderer.last_render_time.unwrap_or(now))
            .as_secs_f32();
        renderer.queue.write_buffer(
            &renderer.time_buffer,
            0,
            bytemuck::cast_slice(&[TimeRaw {
                delta_time,
                time_since_start_millis,
            }]),
        );

        let output = renderer.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = renderer
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
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
                            g: 0.1,
                            b: 0.1,
                            a: 1.0,
                        }),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });

            render_pass.set_pipeline(&renderer.pipeline);
            render_pass.set_bind_group(0, &renderer.static_bind_group, &[]);
            render_pass.set_bind_group(1, &renderer.camera_bind_group, &[]);

            render_pass.set_vertex_buffer(0, renderer.atlas.vertex_buffer.slice(..));
            render_pass.set_index_buffer(
                renderer.atlas.index_buffer.slice(..),
                wgpu::IndexFormat::Uint16,
            );

            for cmd in command_queue {
                match cmd {
                    RenderCommand::DrawSprite {
                        sprite_idx,
                        instance,
                    } => {
                        let target_sprite = &renderer.atlas.sprites[sprite_idx as usize];
                        render_pass.draw_indexed(target_sprite.indices(), 0, 0..1)
                    }
                }
            }
        }

        renderer.queue.submit([encoder.finish()]);
        output.present();

        Ok(())
    }
}

pub enum RenderCommand {
    DrawSprite { sprite_idx: u32, instance: Instance },
}
