use std::collections::HashSet;

use id_game_config::{Game, GameConfig};
use id_map_format::{Texture, Wad};

use anyhow::Result;
use indexmap::IndexMap;

use crate::{
    components::CWorldPos,
    cvars::{CVarsMap, DEFAULT_CVARS},
    entities::{init_player_entities, init_sector_entities, init_wall_entities},
    AnimationStateMap, SectorAccel, Stopwatch,
};

pub struct World {
    pub game: Game,
    pub game_config: GameConfig,

    pub wad: Wad,
    pub map: id_map_format::Map,
    pub textures: IndexMap<String, Texture>,

    /// Actual game state is maintained in the ECS "world".
    pub world: hecs::World,
    pub player: hecs::Entity,

    /// This tracks which entities have been modified since the last tick.
    ///
    /// Right now insertion is manual: there's different approaches, like Bevy's
    /// use of DerefMut (with polling), or hecs' hashing system.
    ///
    /// Neither of these have the performance we want, so we're doing it manually.
    pub changed_set: HashSet<hecs::Entity>,

    pub sector_accel: SectorAccel,
    pub animations: AnimationStateMap,
    pub cvars: CVarsMap,

    frame_count_mod_20: u32,
}

impl World {
    pub fn new(wad: Wad, map_name: &str) -> Result<Self> {
        let game = Game::from_wad(&wad).ok_or(anyhow::anyhow!(
            "Game detection failed: is this DOOM/DOOM2/Heretic?"
        ))?;
        let game_config = GameConfig::from_game(game)?;

        let map = wad.parse_map(map_name)?;
        let textures = wad.parse_textures()?;

        let mut world = hecs::World::new();

        // Time how long it takes to spawn the entities.
        let mut stopwatch = Stopwatch::new();

        let animations = AnimationStateMap::from_game_config(&game_config, &wad, &textures);

        // Add walls to the world.
        init_wall_entities(&mut world, &map, &animations);
        // Add sectors to the world.
        init_sector_entities(&mut world, &map, &animations);

        // Build acceleration structure for sectors.
        let sector_accel = SectorAccel::new(&world);

        // Add entities to the world.
        // Requires we've already initialized sector accel.
        let player = init_player_entities(&mut world, &sector_accel, &map)?;

        let setup_time = stopwatch.lap();

        println!("Added {} entities to the world.", world.len());
        println!("Setup time: {:?}", setup_time);

        Ok(Self {
            game,
            game_config,

            wad,
            map,
            textures,

            world,
            player,

            changed_set: HashSet::new(),

            sector_accel,
            animations,
            cvars: DEFAULT_CVARS.iter().copied().collect::<CVarsMap>(),

            frame_count_mod_20: 0,
        })
    }

    pub fn think(&mut self) -> Result<()> {
        // Every 20 frames, animate the textures.
        self.frame_count_mod_20 = (self.frame_count_mod_20 + 1) % 20;
        if self.frame_count_mod_20 == 19 {
            self.animations
                .animate_world(&mut self.changed_set, &mut self.world);
        }

        Ok(())
    }

    pub fn think_end(&mut self) -> Result<()> {
        // Clear the changed set.
        self.changed_set.clear();
        Ok(())
    }

    pub fn with_player_pos<RT, F: FnOnce(&mut CWorldPos) -> RT>(
        &mut self,
        callback: F,
    ) -> Result<RT> {
        let player_pos = self.world.query_one_mut::<&mut CWorldPos>(self.player)?;
        Ok(callback(player_pos))
    }
}
