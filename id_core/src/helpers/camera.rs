use ultraviolet::{projection::perspective_reversed_infinite_z_wgpu_dx_gl, Mat4, UVec2, Vec3};

use super::Movable;

pub struct Camera<'a, M: Movable> {
    pub movable: &'a mut M,

    pub z_near: f32,
    pub fov: f32,
}

impl<M: Movable> Camera<'_, M> {
    pub fn view_matrix(&self) -> Mat4 {
        Mat4::look_at_lh(
            self.movable.pos(),
            self.movable.pos() + self.movable.look_at_vector(),
            Vec3 {
                x: 0.0,
                y: 1.0,
                z: 0.0,
            },
        )
    }

    /// Simple perspective projection matrix with a few tricks up its sleeve.
    ///
    /// Assumptions:
    /// - WebGPU, like DirectX, uses a left-handed coordinate system with Y pointing up.
    /// - Reverse Z with an infinite far plane.
    pub fn projection_matrix(&self, display_size: UVec2) -> Mat4 {
        let aspect_ratio = (display_size.x as f64 / display_size.y as f64) as f32;
        perspective_reversed_infinite_z_wgpu_dx_gl(self.fov.to_radians(), aspect_ratio, self.z_near)
    }
}
