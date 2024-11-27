use id_map_format::Map;

use anyhow::Result;
use ultraviolet::{Vec2, Vec3};

use crate::components::{CSector, CWorldPos};

pub fn init_player_entities(world: &mut hecs::World, map: &Map) -> Result<hecs::Entity> {
    let player_start = map
        .things
        .iter()
        .find(|thing| thing.thing_type == 1)
        .ok_or(anyhow::anyhow!("No player start found!"))?;

    // Convert into camera space.
    let player_xz = Vec2::new(player_start.x as f32, player_start.y as f32);

    // TODO: Improve this in two ways.
    // 1. Use acceleration structure for looking up sectors.
    // 2. Figure out actual height of player instead of guessing 16px.
    let mut found_sector: Option<hecs::Entity> = None;
    for (id, sector) in world.query_mut::<&CSector>() {
        if found_sector.is_some() {
            break;
        }
        for triangle in &sector.triangles {
            if triangle.bbox().has_point(player_xz) && triangle.has_point(player_xz) {
                found_sector = Some(id);
                break;
            }
        }
    }

    let player_y = found_sector
        .map(|id| world.get::<&CSector>(id).unwrap().floor_height as f32)
        .ok_or(anyhow::anyhow!("Player start isn't in a sector!"))?;

    let player_yaw = player_start.angle as f32;
    let entity = world.spawn((CWorldPos {
        pos: Vec3 {
            x: player_xz.x,
            y: player_y,
            z: player_xz.y,
        },
        yaw: player_yaw,
        pitch: 0.0,
    },));

    Ok(entity)
}
