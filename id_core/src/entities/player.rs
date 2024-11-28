use std::f32::consts::FRAC_PI_2;

use id_map_format::Map;

use anyhow::Result;
use ultraviolet::{Vec2, Vec3};

use crate::{
    components::{CSector, CWorldPos},
    SectorAccel,
};

pub fn init_player_entities(
    world: &mut hecs::World,
    sector_accel: &SectorAccel,
    map: &Map,
) -> Result<hecs::Entity> {
    let player_start = map
        .things
        .iter()
        .find(|thing| thing.thing_type == 1)
        .ok_or(anyhow::anyhow!("No player start found!"))?;

    // Convert into camera space.
    let player_xz = Vec2::new(player_start.x as f32, player_start.y as f32);

    let found_sector = sector_accel.query(world, player_xz);
    let player_y = found_sector
        .map(|id| world.get::<&CSector>(id).unwrap().floor_height as f32)
        .ok_or(anyhow::anyhow!("Player start isn't in a sector!"))?;

    // Convert -CCW from East to +CW from North.
    let player_yaw = (-(player_start.angle as f32).to_radians() - FRAC_PI_2).to_degrees();
    let entity = world.spawn((CWorldPos {
        pos: Vec3 {
            x: player_xz.x,
            // TODO: Figure out actual height of player instead of guessing 16px.
            y: player_y + 16.0,
            z: player_xz.y,
        },
        yaw: player_yaw,
        pitch: 0.0,
    },));

    Ok(entity)
}
