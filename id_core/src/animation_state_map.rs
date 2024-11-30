use std::collections::HashMap;

use id_game_config::GameConfig;
use id_map_format::{Texture, Wad};

use indexmap::IndexMap;

use crate::{
    components::{CTexture, CTextureAnimated, CTextureFloor},
    helpers::ChangedSet,
};

pub struct AnimationStateMap {
    states: HashMap<CTexture, CTexture>,
}

impl AnimationStateMap {
    pub fn from_game_config(
        game_config: &GameConfig,
        wad: &Wad,
        pwad: &[Wad],
        textures: &IndexMap<String, Texture>,
    ) -> Self {
        let mut states = HashMap::<CTexture, CTexture>::new();

        // Animation state transitions are defined as pointers in the WAD lump.
        // Their names are only by convention.
        //
        // Reference:
        // https://doomwiki.org/wiki/Animated_flat

        let mut lump_names_in_order = wad.lump_names_in_order.clone();
        for pwad in pwad {
            lump_names_in_order.extend_from_slice(&pwad.lump_names_in_order);
        }

        for (start, end) in &game_config.flats {
            // Flats are stored in the wad as individual lumps.
            let start_idx = lump_names_in_order.iter().position(|name| name == start);
            let end_idx = lump_names_in_order.iter().position(|name| name == end);

            if let (Some(start_idx), Some(end_idx)) = (start_idx, end_idx) {
                // Create state transitions for every flat between start_idx and end_idx.
                for i in start_idx..end_idx + 1 {
                    let start_name = &lump_names_in_order[i];
                    let end_name =
                        &lump_names_in_order[if i + 1 > end_idx { start_idx } else { i + 1 }];

                    states.insert(
                        CTexture::Flat(start_name.clone()),
                        CTexture::Flat(end_name.clone()),
                    );
                }
            } else {
                eprintln!(
                    "Failed to find animation states for flats: {} -> {}",
                    start, end
                );
            }
        }

        for (start, end) in &game_config.walls {
            let keys = textures.keys().collect::<Vec<_>>();

            // Textures are stored in the texture lump.
            let start_idx = keys.iter().position(|name| *name == start);
            let end_idx = keys.iter().position(|name| *name == end);

            if let (Some(start_idx), Some(end_idx)) = (start_idx, end_idx) {
                // Create state transitions for every wall between start_idx and end_idx.
                for i in start_idx..end_idx + 1 {
                    let start_name = keys[i];
                    let end_name = keys[if i + 1 > end_idx { start_idx } else { i + 1 }];

                    states.insert(
                        CTexture::Texture(start_name.clone()),
                        CTexture::Texture(end_name.clone()),
                    );
                }
            } else {
                eprintln!(
                    "Failed to find animation states for walls: {} -> {}",
                    start, end
                );
            }
        }

        Self { states }
    }

    pub fn contains_key(&self, c_texture: &CTexture) -> bool {
        self.states.contains_key(c_texture)
    }

    pub fn get(&self, c_texture: &CTexture) -> Option<CTexture> {
        self.states.get(c_texture).cloned()
    }

    pub fn keys(&self) -> impl Iterator<Item = &CTexture> {
        self.states.keys()
    }

    pub fn animate_world(
        &self,
        changed_set: &mut ChangedSet<hecs::Entity>,
        world: &mut hecs::World,
    ) {
        // Animate all textures.
        for (id, (c_texture, _c_texture_anim)) in
            world.query_mut::<(&mut CTexture, &CTextureAnimated)>()
        {
            if let Some(next) = self.get(c_texture) {
                *c_texture = next;
                changed_set.change(id);
            }
        }

        // Animate all floor textures.
        for (id, (c_texture, _c_texture_anim)) in
            world.query_mut::<(&mut CTextureFloor, &CTextureAnimated)>()
        {
            if let Some(next) = self.get(&c_texture.0) {
                c_texture.0 = next;
                changed_set.change(id);
            }
        }
    }
}
