use std::ops::Range;

use hashbrown::HashMap;
use image::GenericImageView;
use wgpu::util::DeviceExt;

use crate::Vertex;

pub struct Atlas {
    pub textures: Vec<(wgpu::Texture, wgpu::TextureView)>,
    pub sampler: wgpu::Sampler,

    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,

    pub sprites: Vec<Sprite>,
    pub named_sprites: HashMap<String, (u32, Vec<u32>)>,
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
    pub fn default_from_device(device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
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

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let mut sprites = vec![];
        let mut global_indices: Vec<u16> = vec![];
        let mut global_vertices: Vec<Vertex> = vec![];

        let x_step = 0.25;
        let y_step = 0.25;

        for y in 0..4 {
            for x in 0..4 {
                let yy = y as f32 * y_step;
                let xx = x as f32 * x_step;

                let verts = [
                    Vertex {
                        position: [-0.5, -0.5, 0.0],
                        tex_coords: [xx, yy + y_step],
                    },
                    Vertex {
                        position: [0.5, -0.5, 0.0],
                        tex_coords: [xx + x_step, yy + y_step],
                    },
                    Vertex {
                        position: [-0.5, 0.5, 0.0],
                        tex_coords: [xx, yy],
                    },
                    Vertex {
                        position: [0.5, 0.5, 0.0],
                        tex_coords: [xx + x_step, yy],
                    },
                ]; // quad

                let inds = [0, 1, 2, 1, 3, 2].map(|x| (x + global_vertices.len()) as u16); // quad

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

        Self {
            named_sprites: Default::default(),
            index_buffer,
            vertex_buffer,
            sprites,
            sampler,
            textures: vec![(diffuse_texture, diffuse_texture_view)],
        }
    }
}
