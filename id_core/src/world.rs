use anyhow::Result;
use id_map_format::Wad;

use crate::{
    components::CWorldPos,
    cvars::{CVarsMap, DEFAULT_CVARS},
    entities::{init_player_entities, init_sector_entities, init_wall_entities},
    Stopwatch,
};

pub struct World {
    // Actual game state is maintained in the ECS "world".
    pub world: hecs::World,
    pub player: hecs::Entity,

    pub wad: Wad,
    pub map: id_map_format::Map,

    pub cvars: CVarsMap,
}

impl World {
    pub fn new(wad: Wad, map_name: &str) -> Result<Self> {
        let map = wad.parse_map(map_name)?;

        let mut world = hecs::World::new();

        // Time how long it takes to spawn the entities.
        let mut stopwatch = Stopwatch::new();

        // Add walls to the world.
        init_wall_entities(&mut world, &map);
        // Add sectors to the world.
        init_sector_entities(&mut world, &map);

        // Add entities to the world.
        // Requires we've already initialized sectors.
        let player = init_player_entities(&mut world, &map)?;

        let setup_time = stopwatch.lap();

        println!("Added {} entities to the world.", world.len());
        println!("Setup time: {:?}", setup_time);

        Ok(Self {
            world,
            player,

            wad,
            map,

            cvars: DEFAULT_CVARS.iter().copied().collect::<CVarsMap>(),
        })
    }

    pub fn with_player_pos<RT, F: FnOnce(&mut CWorldPos) -> RT>(
        &mut self,
        callback: F,
    ) -> Result<RT> {
        let mut player_pos = self.world.query_one_mut::<&mut CWorldPos>(self.player)?;
        Ok(callback(&mut player_pos))
    }
}
