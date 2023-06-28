use nalgebra_glm::Vec2;
use smallvec::{smallvec, SmallVec};
use std::ops::Range;

use hashbrown::HashMap;
use image::GenericImageView;
use wgpu::util::DeviceExt;

use crate::Vertex;

pub struct Atlas {
    pub textures: Vec<(wgpu::Texture, wgpu::TextureView)>,
    pub sampler: wgpu::Sampler,
    pub bind_group: wgpu::BindGroup,

    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,

    // List of all the sprites
    pub(crate) sprites: Vec<Sprite>,
    // Mapping of the actor template's resource_name
    // onto a default sprite (.0) and possibly variant sprites (.1)
    pub(crate) resource_name_to_sprite: HashMap<String, (u32, SmallVec<[u32; 2]>)>,
}

pub struct Sprite {
    pub sprite_index_range: (u16, u16),
}

impl Sprite {
    pub fn indices(&self) -> Range<u32> {
        self.sprite_index_range.0 as u32..self.sprite_index_range.1 as u32
    }
}

impl Atlas {
    pub fn create_binding_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
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
            label: Some("Atlas Bind Layout"),
        })
    }

    pub fn sampling_options() -> wgpu::SamplerDescriptor<'static> {
        wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        }
    }

    pub fn mesh_from_sprite(texture_topleft: Vec2, size: Vec2) -> (Vec<Vertex>, Vec<u16>) {
        let x = texture_topleft.x;
        let y = texture_topleft.y;
        let x_step = size.x;
        let y_step = size.y;

        let verts = [
            Vertex {
                position: [-0.5, -0.5, 0.0],
                tex_coords: [x, y + y_step],
            },
            Vertex {
                position: [0.5, -0.5, 0.0],
                tex_coords: [x + x_step, y + y_step],
            },
            Vertex {
                position: [-0.5, 0.5, 0.0],
                tex_coords: [x, y],
            },
            Vertex {
                position: [0.5, 0.5, 0.0],
                tex_coords: [x + x_step, y],
            },
        ];

        let inds = [0, 1, 2, 1, 3, 2];

        (verts.to_vec(), inds.to_vec())
    }

    pub fn default_from_device(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        binding_layout: &wgpu::BindGroupLayout,
    ) -> Self {
        let diffuse_bytes = include_bytes!("assets/tileset.png");
        let diffuse_image = image::load_from_memory(diffuse_bytes).unwrap();
        let diffuse_rgba = diffuse_image.to_rgba8();

        let dimensions = diffuse_image.dimensions();
        let size = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1,
        };

        let diffuse_texture = device.create_texture(&wgpu::TextureDescriptor {
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            label: Some("Atlas: tileset.png"),
            view_formats: &[],
        });

        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &diffuse_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &diffuse_rgba,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * dimensions.0),
                rows_per_image: Some(dimensions.1),
            },
            size,
        );

        let diffuse_texture_view =
            diffuse_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let sampler = device.create_sampler(&Self::sampling_options());

        let mut sprites = vec![];
        let mut global_indices: Vec<u16> = vec![];
        let mut global_vertices: Vec<Vertex> = vec![];

        let x_step = 1. / 16.;
        let y_step = 1. / 16.;

        for y in 0..16 {
            for x in 0..16 {
                let yy = y as f32 * y_step;
                let xx = x as f32 * x_step;

                let (verts, inds) =
                    Self::mesh_from_sprite(Vec2::new(xx, yy), Vec2::new(x_step, y_step));
                let inds = inds.into_iter().map(|x| x + global_vertices.len() as u16);

                let sprite = Sprite {
                    sprite_index_range: (
                        global_indices.len() as u16,
                        (global_indices.len() + inds.len()) as u16,
                    ),
                };

                sprites.push(sprite);
                global_indices.extend(inds);
                global_vertices.extend(verts);
            }
        }

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Atlas Index Buffer"),
            contents: bytemuck::cast_slice(&global_indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Atlas Vertex Buffer"),
            contents: bytemuck::cast_slice(&global_vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: binding_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&diffuse_texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
            label: Some("Atlas Bind Group"),
        });

        let resource_name_to_sprite = HashMap::from_iter([
            ("tile.floor".to_string(), (0, smallvec![])),
            ("tile.wall".to_string(), (1, smallvec![])),
            ("creature.snek".to_string(), (5, smallvec![])),
            ("creature.player".to_string(), (16, smallvec![])),
        ]);

        Self {
            resource_name_to_sprite,
            index_buffer,
            vertex_buffer,
            sprites,
            sampler,
            textures: vec![(diffuse_texture, diffuse_texture_view)],
            bind_group,
        }
    }

    pub fn resolve_resource(&self, name: &str) -> Option<&(u32, SmallVec<[u32; 2]>)> {
        self.resource_name_to_sprite.get(name)
    }
}
