use std::collections::HashMap;

use id_game_config::GameConfig;
use id_map_format::{Texture, Wad};

use indexmap::IndexMap;

use crate::components::CTexturePurpose;

pub struct AnimationStateMap {
    states: HashMap<(CTexturePurpose, String), String>,
}

impl AnimationStateMap {
    pub fn from_game_config(
        game_config: &GameConfig,
        wad: &Wad,
        textures: &IndexMap<String, Texture>,
    ) -> Self {
        let mut states = HashMap::<(CTexturePurpose, String), String>::new();

        // Animation state transitions are defined as pointers in the WAD lump.
        // Their names are only by convention.
        //
        // Reference:
        // https://doomwiki.org/wiki/Animated_flat

        for (start, end) in &game_config.flats {
            // Flats are stored in the wad as individual lumps.
            let start_idx = wad
                .lump_names_in_order
                .iter()
                .position(|name| name == start);

            let end_idx = wad.lump_names_in_order.iter().position(|name| name == end);

            if let (Some(start_idx), Some(end_idx)) = (start_idx, end_idx) {
                // Create state transitions for every flat between start_idx and end_idx.
                for i in start_idx..end_idx {
                    let start_name = &wad.lump_names_in_order[i];
                    let end_name =
                        &wad.lump_names_in_order[if i + 1 > end_idx { start_idx } else { i + 1 }];

                    states.insert(
                        (CTexturePurpose::Flat, start_name.clone()),
                        end_name.clone(),
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
                for i in start_idx..end_idx {
                    let start_name = keys[i];
                    let end_name = keys[if i + 1 > end_idx { start_idx } else { i + 1 }];

                    states.insert(
                        (CTexturePurpose::Texture, start_name.clone()),
                        end_name.clone(),
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

    pub fn contains_key(&self, purpose: CTexturePurpose, name: &str) -> bool {
        self.states.contains_key(&(purpose, name.to_string()))
    }

    pub fn get(&self, purpose: CTexturePurpose, name: &str) -> Option<String> {
        self.states.get(&(purpose, name.to_string())).cloned()
    }

    pub fn keys(&self) -> impl Iterator<Item = &(CTexturePurpose, String)> {
        self.states.keys()
    }
}
