use id_game_config::{Game, GameConfig};
use id_map_format::{lump_from_namespace, Lump, LumpNamespace, Patch, Texture, Wad};

use anyhow::Result;
use indexmap::IndexMap;
use ultraviolet::Vec3;

use crate::{
    components::CWorldPos,
    cvars::{CVarsMap, DEFAULT_CVARS},
    entities::{
        init_player_entities, init_sector_entities, init_thing_entities, init_wall_entities,
    },
    helpers::ChangedSet,
    AnimationStateMap, SectorAccel, Stopwatch,
};

pub struct World {
    iwad: Wad,
    pwad: Vec<Wad>,

    pub game: Game,
    pub game_config: GameConfig,

    pub map: id_map_format::Map,
    pub palette: Vec<Vec3>,
    pub colormap: Vec<u8>,
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
    pub changed_set: ChangedSet<hecs::Entity>,

    pub sector_accel: SectorAccel,
    pub animations: AnimationStateMap,
    pub cvars: CVarsMap,

    frame_count_mod_20: u32,
}

impl World {
    pub fn new(iwad: Wad, pwad: Vec<Wad>, map_name: &str) -> Result<Self> {
        let game = Game::from_wad(&iwad).ok_or(anyhow::anyhow!(
            "Game detection failed: is this DOOM/DOOM2/Heretic?"
        ))?;
        let game_config = GameConfig::from_game(game)?;

        // If the map is in the PWAD, use that.
        let map = pwad
            .iter()
            .rev()
            .find_map(|pwad| pwad.parse_map(map_name).ok())
            .unwrap_or_else(|| iwad.parse_map(map_name).unwrap());

        // If the palette is in the PWAD, use that.
        let palette = pwad
            .iter()
            .rev()
            .find_map(|pwad| pwad.parse_palettes().ok())
            .unwrap_or_else(|| iwad.parse_palettes().unwrap())[0]
            .iter()
            .map(|(r, g, b)| Vec3::new(*r as f32, *g as f32, *b as f32))
            .collect::<Vec<Vec3>>();

        // If the colormap is in the PWAD, use that.
        let colormap = pwad
            .iter()
            .rev()
            .find_map(|pwad| pwad.parse_colormaps().ok())
            .unwrap_or_else(|| iwad.parse_colormaps().unwrap());

        let textures = {
            let mut textures = IndexMap::new();

            let mut patch_names = iwad.parse_patch_names()?;
            textures.extend(iwad.parse_textures(&patch_names)?);

            // For each PWAD, add the textures.
            for pwad in pwad.iter() {
                // Just overwrite the patch names, since they're the same.
                patch_names = pwad.parse_patch_names()?;
                textures.extend(pwad.parse_textures(&patch_names)?);
            }

            textures
        };

        let mut world = hecs::World::new();
        let mut changed_set = ChangedSet::<hecs::Entity>::default();

        // Time how long it takes to spawn the entities.
        let mut stopwatch = Stopwatch::new();

        let animations = AnimationStateMap::from_game_config(&game_config, &iwad, &pwad, &textures);

        // Add walls to the world.
        init_wall_entities(&mut world, &map, &animations);
        // Add sectors to the world.
        init_sector_entities(&mut world, &map, &animations);

        // Build acceleration structure for sectors.
        let sector_accel = SectorAccel::new(&world);

        // Add things to the world.
        // Requires we've already initialized sector accel.
        init_thing_entities(&mut world, &game_config, &sector_accel, &map);
        let player = init_player_entities(&mut world, &sector_accel, &map)?;

        let setup_time = stopwatch.lap();

        println!("Added {} entities to the world.", world.len());
        println!("Setup time: {:?}", setup_time);

        // Add all entities to the changed_set.
        for entity_ref in world.iter() {
            changed_set.spawn(entity_ref.entity());
        }

        Ok(Self {
            iwad,
            pwad,

            game,
            game_config,

            map,
            palette,
            colormap,
            textures,

            world,
            player,
            changed_set,

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

    pub fn with_lump<RT, F: FnOnce(&Lump) -> RT>(
        &self,
        namespace: &LumpNamespace,
        lump_name: &str,
        callback: F,
    ) -> Result<RT> {
        // If the map is in the PWAD, use that.
        for pwad in self.pwad.iter().rev() {
            if let Ok(lump) = lump_from_namespace(namespace, lump_name, pwad) {
                return Ok(callback(lump));
            }
        }

        if let Ok(lump) = lump_from_namespace(namespace, lump_name, &self.iwad) {
            return Ok(callback(lump));
        }

        Err(anyhow::anyhow!("Lump not found: {}", lump_name))
    }

    pub fn with_patch<RT, F: FnOnce(Patch) -> RT>(
        &self,
        patch_name: &str,
        callback: F,
    ) -> Result<RT> {
        // If the map is in the PWAD, use that.
        for pwad in self.pwad.iter().rev() {
            if let Ok(patch) = pwad.parse_patch(patch_name) {
                return Ok(callback(patch));
            }
        }

        if let Ok(patch) = self.iwad.parse_patch(patch_name) {
            return Ok(callback(patch));
        }

        Err(anyhow::anyhow!("Patch not found: {}", patch_name))
    }
}
