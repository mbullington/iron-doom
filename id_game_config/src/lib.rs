use id_map_format::Wad;

use serde::{Deserialize, Serialize};
use serde_json::Result;
use sha2::Digest;

#[derive(Debug, Clone, Copy)]
pub enum Game {
    /// Doom encompasses DOOM, DOOM II, and FreeDOOM.
    Doom,
    Heretic,
    Chex,
}

// These are all sha256 hashes of the ENDOOM, or ENDTEXT, lump in the IWAD.
// May not be a perfect system, but works for now.
mod hashes {
    pub(crate) const DOOM: &str =
        "6c37f5a1ad9cbb7110da91e26a52dd8b8021151cac148c83177bb6e78417fedf";
    pub(crate) const DOOM2: &str =
        "1cf281dbb13912b00b597bad4e84cfcef90c175f0869745a429a5e976a088cc2";
    pub(crate) const FREEDOOM: &str =
        "41d70f3fffaa451aa73be1729bdb0bc5c6c82871fb1e4e5659bd15387973d707";
    pub(crate) const HERETIC: &str =
        "b3f74949bba7ade6473164cfba8c4ca1e75771504f411dfceeef804d17f522ff";
    pub(crate) const CHEX: &str =
        "a1db19b007ec74054182283e1d6b1bc317d75e3d84a697530fe75614aecaa387";
}

impl Game {
    pub fn from_wad(wad: &Wad) -> Option<Game> {
        let endoom_or_endtext = wad.endoom_or_endtext()?;
        let hash = hex::encode(sha2::Sha256::digest(&endoom_or_endtext));

        if hash == hashes::DOOM || hash == hashes::DOOM2 || hash == hashes::FREEDOOM {
            return Some(Game::Doom);
        }
        if hash == hashes::HERETIC {
            return Some(Game::Heretic);
        }
        if hash == hashes::CHEX {
            return Some(Game::Chex);
        }
        None
    }

    pub fn name(&self) -> &str {
        match self {
            Game::Doom => "DOOM",
            Game::Heretic => "Heretic",
            Game::Chex => "Chex Quest",
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct GameConfig {
    pub walls: Vec<(String, String)>,
    pub flats: Vec<(String, String)>,
    pub things: Vec<ThingConfig>,
}

#[derive(Serialize, Deserialize)]
pub struct ThingConfig {
    thing_type: u32,

    radius: u32,
    height: u32,

    sprite: String,
    sequence: String,
    class: String,

    description: String,
}

impl GameConfig {
    pub fn from_game(game: Game) -> Result<Self> {
        let config_str = match game {
            Game::Doom => include_str!("../config/doom.json"),
            Game::Heretic => include_str!("../config/heretic.json"),
            Game::Chex => include_str!("../config/doom.json"),
        };

        serde_json::from_str(config_str)
    }
}
