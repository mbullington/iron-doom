use std::f32::consts::FRAC_PI_2;

use id_game_config::ThingFlags;
use id_map_format::Thing;

use ultraviolet::{Vec2, Vec3};

use crate::{helpers::Movable, SectorAccel};

use super::CSector;

#[derive(Debug)]
pub struct CThing {
    pub thing_type: u16,
    pub spawn_flags: u16,

    /// These are static flags that are set per-thing.
    pub thing_flags: ThingFlags,

    pub radius: u32,
    pub height: u32,
}

/// [CWorldPos] is an "entity" in the world.
///
/// Currently reused for things (monsters, items, etc.) and players.
#[derive(Debug)]
pub struct CWorldPos {
    pub pos: Vec3,
    /// Between 0 and 360.
    pub yaw: f32,
    /// Between -90 and 90 deg.
    pub pitch: f32,
}

impl CWorldPos {
    pub fn from_thing(thing: &Thing, world: &hecs::World, sector_accel: &SectorAccel) -> Self {
        // Convert into camera space.
        let pos_xz = Vec2::new(thing.x as f32, thing.y as f32);

        let found_sector = sector_accel.query(world, pos_xz);
        let y = found_sector
            .map(|id| world.get::<&CSector>(id).unwrap().floor_height as f32)
            .unwrap_or(0.0);

        // Convert -CCW from East to +CW from North.
        let yaw = (-(thing.angle as f32).to_radians() - FRAC_PI_2).to_degrees();
        CWorldPos {
            pos: Vec3 {
                x: pos_xz.x,
                y,
                z: pos_xz.y,
            },
            yaw,
            pitch: 0.0,
        }
    }
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
