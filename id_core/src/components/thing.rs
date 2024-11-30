use ultraviolet::Vec3;

use crate::helpers::Movable;

#[derive(Debug)]
pub struct CWorldPos {
    pub pos: Vec3,
    /// Between 0 and 360.
    pub yaw: f32,
    /// Between -90 and 90 deg.
    pub pitch: f32,
}

impl Movable for CWorldPos {
    fn pos(&self) -> Vec3 {
        self.pos
    }

    fn translate(&mut self, delta: Vec3) {
        let delta_target = self.rotor() * delta;
        self.pos += delta_target;
    }

    fn translate_xz(&mut self, delta: Vec3) {
        let mut delta_target = self.rotor() * delta;
        delta_target.y = 0.0;

        self.pos += delta_target;
    }

    fn yaw(&self) -> f32 {
        self.yaw
    }
    fn pitch(&self) -> f32 {
        self.pitch
    }

    fn rotate_pitch_yaw(&mut self, pitch: f32, yaw: f32) {
        self.pitch = (self.pitch - pitch).clamp(-89.0, 89.0);
        self.yaw = (self.yaw - yaw) % 360.0;
    }
}
