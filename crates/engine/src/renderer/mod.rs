use std::cell::{Cell, OnceCell, RefCell};

use self::egui::{Context as EguiContext, Renderer as EguiRenderer, Ui as EguiUI};
use bytemuck::{Pod, Zeroable};
use egui_wgpu::renderer::ScreenDescriptor;
use glm::{vec4, Mat4};
use nalgebra_glm as glm;
use nalgebra_glm::{vec3, Vec2};
use puffin_egui::puffin::{self, profile_function, profile_scope};
use wgpu::{util::DeviceExt, BindGroup, BufferUsages};
use winit::dpi::PhysicalPosition;
use winit::window::Window;

mod atlas;
mod camera;
mod vertex;

pub use atlas::*;
pub use camera::*;
pub use vertex::*;

pub mod egui {
    pub use egui::*;
    pub use egui_wgpu::Renderer;
}

#[derive(Debug)]
pub struct Instance {
    pub size: f32,
    pub pos: Vec2,
    pub layer: u16,

    // Clockwise rotation of the sprite in degrees
    pub angle: f32,
    pub tint: [u8; 3],
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct InstanceRaw {
    pub pos: [f32; 2],
    pub rotation: f32,
    pub scale: f32,
    pub tint: [f32; 3],
}

impl InstanceRaw {
    pub fn layout() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<InstanceRaw>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                    shader_location: 4,
                    format: wgpu::VertexFormat::Float32,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float32,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 6,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}

impl From<&'_ Instance> for InstanceRaw {
    fn from(value: &'_ Instance) -> Self {
        InstanceRaw {
            tint: value.tint.map(|x| x as f32 / 255.),
            pos: value.pos.into(),
            rotation: value.angle * glm::pi::<f32>() / 180.,
            scale: value.size,
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
    pub start_time: OnceCell<instant::Instant>,
    pub last_render_time: Option<instant::Instant>,
    pub time_since_start_seconds: f64,
    pub delta_time: f32,
    pub time_buffer: wgpu::Buffer,

    /* camera */
    pub camera: RefCell<Camera>,
    pub camera_buffer: wgpu::Buffer,

    /* misc */
    pub instances: wgpu::Buffer,

    /* bind groups */
    pub camera_bind_group: BindGroup,
    pub atlas_bind_layout: wgpu::BindGroupLayout,

    /* debugging */
    pub enable_puffin_gui: Cell<bool>,
    pub egui_input_state: egui_winit::State,
    pub egui_context: EguiContext,
    pub egui_renderer: EguiRenderer,
}

impl Renderer {
    pub async fn new(window: Window) -> Self {
        profile_function!();
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
            objects_on_screen_cap: 16 * 16 * 16,
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

        let atlas_bind_layout = Atlas::create_binding_layout(&device);

        let shader = device.create_shader_module(wgpu::include_wgsl!("shaders/main.wgsl"));
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Pipeline Layout"),
            bind_group_layouts: &[&atlas_bind_layout, &camera_bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[Vertex::layout(), InstanceRaw::layout()],
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

        let instances = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Instance Buffer"),
            size: std::mem::size_of::<InstanceRaw>() as u64 * camera.objects_on_screen_cap,
            usage: BufferUsages::COPY_DST | BufferUsages::VERTEX,
            mapped_at_creation: false,
        });

        let egui_renderer = EguiRenderer::new(&device, surface_format, None, 1);
        let egui_context = EguiContext::default();

        let egui_input_state = egui_winit::State::new(&window);

        Renderer {
            enable_puffin_gui: Cell::new(false),
            egui_context,
            egui_renderer,
            time_since_start_seconds: 0.,
            instances,
            camera_bind_group,
            surface,
            device,
            queue,
            config,
            size,
            window,
            pipeline,
            camera: RefCell::new(camera),
            camera_buffer,
            start_time: OnceCell::default(),
            delta_time: 0.,
            last_render_time: None,
            time_buffer,
            atlas_bind_layout,
            egui_input_state,
        }
    }

    pub fn set_camera(&mut self, camera: Camera) -> Camera {
        self.camera.replace(camera)
    }

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

    pub fn window(&self) -> &Window {
        &self.window
    }

    fn window_to_world_matrix(&self) -> Mat4 {
        let camera = self.camera.borrow();

        let proj = camera.proj();

        glm::translation(&vec3(-camera.ratio / camera.zoom, 1. / camera.zoom, 0.))
            * glm::scaling(&vec3(
                2. / self.size.width as f32,
                -2. / self.size.height as f32,
                0.,
            ))
            * glm::inverse(&proj)
    }

    pub fn window_space_to_world(&self, pos: &PhysicalPosition<f64>) -> Vec2 {
        glm::vec4_to_vec2(
            &(self.window_to_world_matrix() * vec4(pos.x as f32, pos.y as f32, 0., 1.)),
        ) + self.camera.borrow().position.xy()
    }

    pub fn begin_frame<'a>(&'a mut self, atlas: &'a Atlas) -> FrameBuilder<'a> {
        self.egui_context
            .begin_frame(self.egui_input_state.take_egui_input(&self.window));
        puffin::GlobalProfiler::lock().new_frame();
        FrameBuilder {
            debug_draws: vec![],
            renderer: self,
            atlas,
            command_queue: vec![],
        }
    }

    pub fn tick_timers(&mut self) {
        let now = instant::Instant::now();
        let start_time = self.start_time.get_or_init(|| now);
        let time_since_start_millis =
            // There is probably a more idiomatic way to do this
            (now.duration_since(start_time.to_owned()).as_millis() % u32::MAX as u128) as u32;
        let delta_time = now
            .duration_since(self.last_render_time.unwrap_or(now))
            .as_secs_f32();
        self.time_since_start_seconds = now.duration_since(start_time.to_owned()).as_secs_f64();

        self.last_render_time.replace(now);
        self.delta_time = delta_time;

        self.queue.write_buffer(
            &self.time_buffer,
            0,
            bytemuck::cast_slice(&[TimeRaw {
                delta_time,
                time_since_start_millis,
            }]),
        );
    }

    pub fn is_profiler_enabled(&self) -> bool {
        self.enable_puffin_gui.get()
    }
}

pub struct FrameBuilder<'a> {
    renderer: &'a mut Renderer,
    atlas: &'a Atlas,
    command_queue: Vec<(u32, Instance)>,
    debug_draws: Vec<Box<dyn FnOnce(&mut EguiUI)>>,
}

impl FrameBuilder<'_> {
    pub fn is_culled(&mut self, position: Vec2) -> bool {
        let (cull_min, cull_max) = self.renderer.camera.borrow().camera_culling_aabb();

        position.x < cull_min.x
            || position.x > cull_max.x
            || position.y > cull_max.y
            || position.y < cull_min.y
    }

    pub fn draw_sprite(&mut self, sprite_idx: u32, instance: Instance) -> &mut Self {
        if self.is_culled(instance.pos) {
            return self;
        }

        self.command_queue.push((sprite_idx, instance));

        self
    }

    pub fn draw_egui(&mut self, draw: impl FnOnce(&EguiContext)) {
        draw(&self.renderer.egui_context)
    }

    pub fn draw_debug(&mut self, draw: impl FnOnce(&mut EguiUI) + 'static) {
        self.debug_draws.push(Box::new(draw));
    }

    pub fn optimize(self) -> Self {
        self
    }

    fn sort_sprites(&mut self) {
        puffin::profile_function!();

        self.command_queue
            .sort_unstable_by(|(a_sprite, a_instance), (b_sprite, b_instance)| {
                match a_instance.layer.cmp(&b_instance.layer) {
                    std::cmp::Ordering::Equal => a_sprite.cmp(b_sprite),
                    x => x,
                }
            });
    }

    pub fn end_frame(mut self) -> Result<(), wgpu::SurfaceError> {
        profile_function!();

        self.sort_sprites();

        let FrameBuilder {
            renderer,
            command_queue,
            atlas,
            debug_draws,
            ..
        } = self;

        renderer.tick_timers();

        let output = renderer.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = renderer
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        if renderer.is_profiler_enabled() {
            puffin_egui::profiler_window(&renderer.egui_context);
        }

        egui::Window::new("Debug").show(&renderer.egui_context, |ui| {
            for debug_draw in debug_draws {
                debug_draw(ui);
            }
        });

        let egui_output = renderer.egui_context.end_frame();
        let egui_paint_job = renderer.egui_context.tessellate(egui_output.shapes);

        for (id, image_delta) in egui_output.textures_delta.set {
            renderer.egui_renderer.update_texture(
                &renderer.device,
                &renderer.queue,
                id,
                &image_delta,
            );
        }

        for id in egui_output.textures_delta.free {
            renderer.egui_renderer.free_texture(&id);
        }

        renderer.egui_renderer.update_buffers(
            &renderer.device,
            &renderer.queue,
            &mut encoder,
            &egui_paint_job,
            &ScreenDescriptor {
                pixels_per_point: 1.,
                size_in_pixels: [renderer.config.width, renderer.config.height],
            },
        );

        renderer.egui_input_state.handle_platform_output(
            &renderer.window,
            &renderer.egui_context,
            egui_output.platform_output,
        );

        {
            profile_scope!("build_render_pass");
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });

            render_pass.set_pipeline(&renderer.pipeline);
            render_pass.set_bind_group(0, &atlas.bind_group, &[]);
            render_pass.set_bind_group(1, &renderer.camera_bind_group, &[]);

            render_pass.set_vertex_buffer(0, atlas.vertex_buffer.slice(..));
            render_pass.set_vertex_buffer(1, renderer.instances.slice(..));

            render_pass.set_index_buffer(atlas.index_buffer.slice(..), wgpu::IndexFormat::Uint16);

            let camera = renderer.camera.borrow();
            let mut instances = vec![];

            let mut last_sprite_idx = 0;
            let mut same_sprite_len = 0;
            let mut sprite_seq_begin_idx = 0;
            for (sprite_idx, instance) in command_queue
                .into_iter()
                .take(camera.objects_on_screen_cap as usize)
            {
                if sprite_idx == last_sprite_idx {
                    same_sprite_len += 1;
                } else {
                    let target_sprite = &atlas.sprites[last_sprite_idx as usize];
                    render_pass.draw_indexed(
                        target_sprite.indices(),
                        0,
                        sprite_seq_begin_idx..sprite_seq_begin_idx + same_sprite_len,
                    );
                    last_sprite_idx = sprite_idx;
                    same_sprite_len = 1;
                    sprite_seq_begin_idx = instances.len() as u32;
                }

                instances.push(InstanceRaw::from(&instance));
            }

            if !instances.is_empty() {
                render_pass.draw_indexed(
                    atlas.sprites[last_sprite_idx as usize].indices(),
                    0,
                    sprite_seq_begin_idx..instances.len() as u32,
                );
            };

            renderer
                .queue
                .write_buffer(&renderer.instances, 0, bytemuck::cast_slice(&instances));

            renderer.egui_renderer.render(
                &mut render_pass,
                &egui_paint_job,
                &ScreenDescriptor {
                    pixels_per_point: renderer.egui_input_state.pixels_per_point(),
                    size_in_pixels: [renderer.config.width, renderer.config.height],
                },
            );
        }

        renderer.queue.submit([encoder.finish()]);
        output.present();

        Ok(())
    }
}
