use nalgebra_glm::{look_at_lh, vec3, Vec3};

pub struct Camera {
    pub position: Vec3,
    pub ratio: f32,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraRaw {
    view_proj: [[f32; 4]; 4],
}

impl From<&'_ Camera> for CameraRaw {
    fn from(value: &'_ Camera) -> Self {
        CameraRaw {
            view_proj: {
                let forward = vec3(0., 0., 1.);
                let view = look_at_lh(&value.position, &forward, &vec3(0., 1., 0.));

                view.into()
            },
        }
    }
}
