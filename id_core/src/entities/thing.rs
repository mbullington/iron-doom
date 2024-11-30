use std::collections::HashMap;

use id_game_config::GameConfig;
use id_map_format::Map;

use crate::{
    components::{CThing, CWorldPos},
    SectorAccel,
};

pub fn init_thing_entities(
    world: &mut hecs::World,
    game_config: &GameConfig,
    sector_accel: &SectorAccel,
    map: &Map,
) {
    // Get all the game config things for lookup.
    let mut things_by_thing_type = HashMap::new();
    for thing in game_config.things.iter() {
        things_by_thing_type.insert(thing.thing_type, thing);
    }

    for thing in map.things.iter() {
        if let Some(thing_config) = things_by_thing_type.get(&(thing.thing_type as u32)) {
            let c_thing = CThing {
                thing_type: thing.thing_type,
                spawn_flags: thing.spawn_flags,

                thing_flags: thing_config.flags,

                radius: thing_config.radius,
                height: thing_config.height,
            };

            world.spawn((c_thing, CWorldPos::from_thing(thing, world, sector_accel)));
        }
    }
}
