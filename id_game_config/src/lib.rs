use id_map_format::Wad;

use bitflags::bitflags;
use serde::Deserialize;
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

#[derive(Deserialize)]
pub struct GameConfig {
    pub walls: Vec<(String, String)>,
    pub flats: Vec<(String, String)>,
    pub things: Vec<ThingConfig>,
}

#[derive(Deserialize)]
pub struct ThingConfig {
    pub thing_type: u32,

    pub flags: ThingFlags,

    pub radius: u32,
    pub height: u32,

    pub sprite: String,
    pub sequence: ThingSequence,

    pub description: String,
}

bitflags! {
    /// These map to the "Class" attribute on DoomWiki.
    /// https://doomwiki.org/wiki/Thing_types#Monsters
    #[derive(Debug, Deserialize, Copy, Clone)]
    #[serde(try_from = "String")]
    pub struct ThingFlags: u32 {
        /// Artifact item. Counts toward ITEMS percentage at the end of a level
        ///
        /// String code: A
        const Aritfact = 0b00000001;
        /// Pickup. Player can pick the thing up by walking over it
        ///
        /// String code: P
        const Pickup = 0b00000010;
        /// Weapon.
        ///
        /// String code: W
        const Weapon = 0b00000100;
        /// Monster. Counts towards kill percentage, hidden when using the nomonsters parameter
        ///
        /// String code: M
        const Monster = 0b00001000;
        /// Obstacle. Players and monsters must walk around
        ///
        /// String code: O
        const Obstacle = 0b00010000;
        /// Shootable. Player can attack and destroy monster or object
        ///
        /// String code: *
        const Shootable = 0b00100000;
        /// Hangs from ceiling, or floats if a monster
        ///
        /// String code: ^
        const UpperPegged = 0b01000000;
    }
}

impl TryFrom<String> for ThingFlags {
    type Error = &'static str;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        let mut flags = ThingFlags::empty();
        for c in s.chars() {
            match c {
                'A' => flags |= ThingFlags::Aritfact,
                'P' => flags |= ThingFlags::Pickup,
                'W' => flags |= ThingFlags::Weapon,
                'M' => flags |= ThingFlags::Monster,
                'O' => flags |= ThingFlags::Obstacle,
                '*' => flags |= ThingFlags::Shootable,
                '^' => flags |= ThingFlags::UpperPegged,
                _ => return Err("Invalid flag"),
            }
        }

        Ok(flags)
    }
}

#[derive(Deserialize)]
#[serde(try_from = "String")]
pub struct ThingSequence {
    pub sequence: Vec<String>,
    pub has_gameplay_frame: bool,
}

impl TryFrom<String> for ThingSequence {
    type Error = &'static str;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        // We get a sequence like ABCD, where each letter is a frame.
        // If a + is present at the end, it means a gameplay frame is present.

        let mut sequence = Vec::new();
        let mut has_gameplay_frame = false;

        for c in s.chars() {
            if c == '+' {
                has_gameplay_frame = true;
                continue;
            }

            sequence.push(c.to_string());
        }

        Ok(ThingSequence {
            sequence,
            has_gameplay_frame,
        })
    }
}

impl GameConfig {
    pub fn from_game(game: Game) -> serde_json::Result<Self> {
        let config_str = match game {
            Game::Doom => include_str!("../config/doom.json"),
            Game::Heretic => include_str!("../config/heretic.json"),
            Game::Chex => include_str!("../config/doom.json"),
        };

        serde_json::from_str(config_str)
    }
}
