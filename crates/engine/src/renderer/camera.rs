use nalgebra_glm::{look_at_lh, ortho_lh, vec2, vec3, vec3_to_vec2, Mat4, Vec2, Vec3};

pub struct Camera {
    pub position: Vec3,
    pub ratio: f32,
    pub zoom: f32,
    pub objects_on_screen_cap: u64,
}

impl Camera {
    pub fn camera_culling_aabb(&self) -> (Vec2, Vec2) {
         /* TODO: proper fix of the edge culling */
        let corner = vec2(self.ratio / self.zoom, 1. / self.zoom,) * 1.2;
        let position =  vec3_to_vec2(&self.position);
        (position - corner, position + corner)
    }

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
