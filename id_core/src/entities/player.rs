use id_map_format::Map;

use anyhow::Result;

use crate::{components::CWorldPos, SectorAccel};

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

    let entity = world.spawn((CWorldPos::from_thing(player_start, world, sector_accel),));
    Ok(entity)
}
