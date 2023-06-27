use nalgebra_glm::{look_at_lh, ortho_lh, vec3, Mat4, Vec3};

pub struct Camera {
    pub position: Vec3,
    pub ratio: f32,
    pub zoom: f32,
}

impl Camera {
    pub fn view(&self) -> Mat4 {
        let forward = self.position + vec3(0., 0., 1.);
        look_at_lh(&self.position, &forward, &vec3(0., 1., 0.))
    }

    pub fn proj(&self) -> Mat4 {
        ortho_lh(
            -self.ratio / self.zoom,
            self.ratio / self.zoom,
            -1. / self.zoom,
            1. / self.zoom,
            -1.0,
            1.0,
        )
    }

    pub fn view_proj(&self) -> Mat4 {
        self.proj() * self.view()
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraRaw {
    view_proj: [[f32; 4]; 4],
}

impl From<&'_ Camera> for CameraRaw {
    fn from(value: &'_ Camera) -> Self {
        CameraRaw {
            view_proj: value.view_proj().into(),
        }
    }
}
