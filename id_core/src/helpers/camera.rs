use ultraviolet::{
    projection::perspective_reversed_infinite_z_wgpu_dx_gl, Mat4, Rotor3, UVec2, Vec3,
};

use super::Movable;

pub struct Camera {
    pub pos: Vec3,
    pub z_near: f32,
    pub fov: f32,

    pub yaw: f32,
    pub pitch: f32,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            pos: Vec3::zero(),
            yaw: 0.0,
            pitch: 0.0,

            fov: 85.0,
            z_near: 1.0,
        }
    }
}

impl Camera {
    pub fn translate(&mut self, delta: Vec3) {
        let delta_target = self.get_rotor() * delta;
        self.pos += delta_target;
    }

    pub fn translate_xz(&mut self, delta: Vec3) {
        let mut delta_target = self.get_rotor() * delta;
        delta_target.y = 0.0;

        self.pos += delta_target;
    }

    pub fn rotate_pitch_yaw(&mut self, pitch: f32, yaw: f32) {
        self.pitch = (self.pitch - pitch).clamp(-89.0, 89.0);
        self.yaw = (self.yaw - yaw) % 360.0;
    }

    fn get_rotor(&self) -> Rotor3 {
        let mut rotor =
            Rotor3::from_euler_angles(0.0, self.pitch.to_radians(), self.yaw.to_radians());
        rotor.normalize();
        rotor
    }

    pub fn get_look_at_vector(&self) -> Vec3 {
        // By default, we look down the positive Z axis (forward).
        let look_at_vector = Vec3 {
            x: 0.0,
            y: 0.0,
            z: 1.0,
        };

        self.get_rotor() * look_at_vector
    }

    pub fn get_view_matrix(&self) -> Mat4 {
        Mat4::look_at_lh(
            self.pos,
            self.pos + self.get_look_at_vector(),
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
    pub fn get_projection_matrix(&self, display_size: UVec2) -> Mat4 {
        let aspect_ratio = (display_size.x as f64 / display_size.y as f64) as f32;
        perspective_reversed_infinite_z_wgpu_dx_gl(self.fov.to_radians(), aspect_ratio, self.z_near)
    }
}

impl Movable for Camera {
    fn move_premul(&mut self, delta: Vec3) {
        // Split apart the XY and Z components of the delta vector.
        let delta_xy = Vec3 {
            x: delta.x,
            y: delta.y,
            z: 0.,
        };
        let delta_z = Vec3 {
            x: 0.,
            y: 0.,
            z: delta.z,
        };

        self.translate_xz(delta_xy);
        self.translate(delta_z);
    }
}
