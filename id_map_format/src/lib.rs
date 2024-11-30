extern crate lazy_static;

#[macro_use]
mod helpers;
mod lumps;

use helpers::parse_bytes_cstr;
use lazy_static::lazy_static;

use std::{
    collections::{HashMap, HashSet},
    rc::Rc,
};

pub use helpers::WadError;
pub use lumps::*;

#[derive(Debug, Clone)]
pub struct Lump {
    pub name: String,
    pub offset: usize,
    pub size: usize,
    bytes: Rc<Vec<u8>>,
}

impl Lump {
    pub fn bytes(&self) -> &[u8] {
        &self.bytes[self.offset..self.offset + self.size]
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum LumpNamespace {
    Global,
    Map(String),
    Patch,
    Sprite,
    Flat,
}

pub struct Wad {
    pub is_iwad: bool,
    pub lump_names_in_order: Vec<String>,

    /// For more deterministic parsing, we parse the lumps into these namespaces.
    ///
    /// This is a list of all the lumps not in the "global" namespace.
    pub lump_namespaces: HashMap<LumpNamespace, HashMap<String, Lump>>,
}

pub fn lump_from_namespace<'a>(
    namespace: &LumpNamespace,
    lump_name: &str,
    wad: &'a Wad,
) -> Result<&'a Lump, WadError> {
    let lump_map = match wad.lump_namespaces.get(namespace) {
        Some(lump_map) => lump_map,
        None => {
            return Err(WadError::MissingLump(lump_name.to_string()));
        }
    };

    let lump = match lump_map.get(lump_name) {
        Some(lump) => lump,
        None => {
            return Err(WadError::MissingLump(lump_name.to_string()));
        }
    };

    Ok(lump)
}

lazy_static! {
    // These are the lumps we'll include inside the map's namespace if present.
    static ref ORDERED_MAP_LUMP_NAMES: Vec<&'static str> = vec![
        "THINGS", "LINEDEFS", "SIDEDEFS", "VERTEXES", "SECTORS",
    ];

    // This is a subset of ORDERED_MAP_LUMP_NAMES, and are the lumps we can safely
    // skip if not present.
    static ref OPTIONAL_MAP_LUMP_NAMES: HashSet<&'static str> =
        vec!["SEGS", "SSECTORS", "NODES", "REJECT", "BLOCKMAP",]
            .into_iter()
            .collect();
}

/// References:
/// - "The Unofficial Doom Specs": `docs/dmsp1666.txt`
/// - https://zdoom.org/wiki/WAD
impl Wad {
    pub fn new(bytes_raw: Vec<u8>) -> Result<Self, WadError> {
        // Parse header.

        let bytes = Rc::new(bytes_raw);

        let header = match parse_bytes_cstr(&bytes[0..4]) {
            Ok(header) => header,
            Err(_) => return Err(WadError::CorruptedBytes),
        };

        if header.as_c_str() != c"IWAD" && header.as_c_str() != c"PWAD" {
            return Err(WadError::InvalidHeader);
        }

        let is_iwad = header.as_c_str() == c"IWAD";
        let num_lumps = u32_le!(&bytes[4..8]) as usize;
        let dir_start_offset = u32_le!(&bytes[8..12]) as usize;

        // Parse directory, get lumps & indices.

        let mut lumps = Vec::<Lump>::with_capacity(num_lumps);

        for i in 0..num_lumps {
            let dir_offset = dir_start_offset + i * 16;

            let offset = u32_le!(&bytes[dir_offset..dir_offset + 4]) as usize;
            let size = u32_le!(&bytes[dir_offset + 4..dir_offset + 8]) as usize;

            let name_cstr = match parse_bytes_cstr(&bytes[dir_offset + 8..dir_offset + 16]) {
                Ok(name) => name,
                Err(_) => return Err(WadError::CorruptedBytes),
            };

            let name = match name_cstr.to_str() {
                Ok(name) => name.to_string(),
                Err(_) => return Err(WadError::CorruptedString),
            };

            lumps.push(Lump {
                name,
                bytes: bytes.clone(),
                offset,
                size,
            });
        }

        // Parse lumps into namespaces.

        let mut curr_namespace = LumpNamespace::Global;
        let mut map_iter_idx: i128 = 0;

        let mut lump_names_in_order = Vec::new();

        let mut lump_namespaces: HashMap<LumpNamespace, HashMap<String, Lump>> = HashMap::new();

        for (i, lump) in lumps.iter().enumerate() {
            lump_names_in_order.push(lump.name.clone());

            // Lookahead for the next lump's name—if it's THINGS, we're likely in a map.
            if i < lumps.len() - 1 && lumps[i + 1].name == "THINGS" {
                curr_namespace = LumpNamespace::Map(lump.name.clone());
                map_iter_idx = -1;
            }

            // Entering a patch block.
            if lumps[i].name == "P_START" || lumps[i].name == "PP_START" {
                curr_namespace = LumpNamespace::Patch;
            }

            // Entering a sprite block.
            if lumps[i].name == "S_START" || lumps[i].name == "SS_START" {
                curr_namespace = LumpNamespace::Sprite;
            }

            // Entering a flat block.
            if lumps[i].name == "F_START" || lumps[i].name == "FF_START" {
                curr_namespace = LumpNamespace::Flat;
            }

            if let LumpNamespace::Map(_) = curr_namespace {
                if map_iter_idx == -1 {
                    map_iter_idx = 0;
                } else {
                    let map_iter_idx_usize: usize = map_iter_idx as usize;

                    // If we're in a map, we can check the idx against the "canonical"
                    // order of map lumps.
                    if map_iter_idx_usize < ORDERED_MAP_LUMP_NAMES.len()
                        && lump.name == ORDERED_MAP_LUMP_NAMES[map_iter_idx_usize]
                    {
                        map_iter_idx += 1;
                    } else if !OPTIONAL_MAP_LUMP_NAMES.contains(lump.name.as_str()) {
                        if map_iter_idx != ORDERED_MAP_LUMP_NAMES.len() as i128 {
                            return Err(WadError::CorruptedLump(
                                ORDERED_MAP_LUMP_NAMES[map_iter_idx_usize]
                                    .to_string()
                                    .clone(),
                            ));
                        }

                        // If we're in a map, and the lump name doesn't match the
                        // canonical order, and it's not an optional lump, then we
                        // can assume we're not in a map anymore.
                        curr_namespace = LumpNamespace::Global;
                    }
                }
            }

            // Add to the current namespace.
            match lump_namespaces.get_mut(&curr_namespace) {
                Some(lump_map) => {
                    lump_map.insert(lump.name.to_uppercase().clone(), lump.clone());
                }
                None => {
                    let mut lump_map = HashMap::new();
                    lump_map.insert(lump.name.to_uppercase().clone(), lump.clone());
                    lump_namespaces.insert(curr_namespace.clone(), lump_map);
                }
            }

            // Leaving a patch block.
            if lumps[i].name == "P_END" || lumps[i].name == "PP_END" {
                curr_namespace = LumpNamespace::Global;
            }

            // Leaving a sprite block.
            if lumps[i].name == "S_END" || lumps[i].name == "SS_END" {
                curr_namespace = LumpNamespace::Global;
            }

            // Leaving a flat block.
            if lumps[i].name == "F_END" || lumps[i].name == "FF_END" {
                curr_namespace = LumpNamespace::Global;
            }
        }

        Ok(Self {
            is_iwad,
            lump_names_in_order,
            lump_namespaces,
        })
    }

    pub fn map_names(&self) -> Vec<String> {
        let map_names: Vec<String> = self
            .lump_namespaces
            .keys()
            .filter_map(|k| match k {
                LumpNamespace::Map(map_name) => Some(map_name.clone()),
                _ => None,
            })
            .collect();

        map_names
    }

    pub fn endoom_or_endtext(&self) -> Option<Vec<u8>> {
        let endoom = lump_from_namespace(&LumpNamespace::Global, "ENDOOM", self);
        let endtext = lump_from_namespace(&LumpNamespace::Global, "ENDTEXT", self);

        match endoom {
            Ok(lump) => Some(lump.bytes().to_vec()),
            Err(_) => match endtext {
                Ok(lump) => Some(lump.bytes().to_vec()),
                Err(_) => None,
            },
        }
    }
}
