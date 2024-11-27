mod sector;
mod wall;

use ultraviolet::Vec3;

pub use sector::*;
pub use wall::*;

use crate::helpers::Movable;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum CTexturePurpose {
    /// Used for flats (walls).
    Flat,
    /// Used for walls.
    Texture,
    // Used for things.
    // Sprite,
}

/// Asset has either a flat (floor/ceiling), texture (wall), or sprite (things).
///
/// We handle them all the same.
#[derive(Debug)]
pub struct CTexture {
    pub purpose: CTexturePurpose,
    pub texture_name: String,
}

// We assume for flats, ceiling is default.
// Floor is specified via a wrapper type.
pub struct CTextureFloor(pub CTexture);

/// Asset is either a flat (ceiling) or texture (wall).
///
/// Render the quad as the sky texture.
#[derive(Debug)]
pub struct CTextureSky {}

// We assume for flats, ceiling is default.
// Floor is specified via a wrapper type.
pub struct CTextureSkyFloor(pub CTextureSky);

#[derive(Debug)]
pub struct CWorldPos {
    pub pos: Vec3,
    /// Between 0 and 360.
    pub yaw: f32,
    /// Between -90 and 90 deg.
    pub pitch: f32,
}

impl Movable for CWorldPos {
    fn get_pos(&self) -> Vec3 {
        self.pos
    }

    fn translate(&mut self, delta: Vec3) {
        let delta_target = self.get_rotor() * delta;
        self.pos += delta_target;
    }

    fn translate_xz(&mut self, delta: Vec3) {
        let mut delta_target = self.get_rotor() * delta;
        delta_target.y = 0.0;

        self.pos += delta_target;
    }

    fn get_yaw(&self) -> f32 {
        self.yaw
    }
    fn get_pitch(&self) -> f32 {
        self.pitch
    }

    fn rotate_pitch_yaw(&mut self, pitch: f32, yaw: f32) {
        self.pitch = (self.pitch - pitch).clamp(-89.0, 89.0);
        self.yaw = (self.yaw - yaw) % 360.0;
    }
}
