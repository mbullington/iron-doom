use indexmap::IndexMap;

use crate::{helpers::parse_bytes_cstr, lump_from_namespace, Lump, LumpNamespace, Wad, WadError};

#[derive(Debug)]
pub struct TexturePatchEntry {
    pub x_offset: i16,
    pub y_offset: i16,

    pub patch_name: String,
}

/// Textures are a list of patches and their offsets, which let you "build"
/// multiple textures from the same patch.
#[derive(Debug)]
pub struct Texture {
    pub name: String,
    pub priority: bool,

    pub width: u16,
    pub height: u16,

    pub patch_entry: Vec<TexturePatchEntry>,
}

impl Wad {
    fn parse_patch_names(&self) -> Result<Vec<String>, WadError> {
        let lump = lump_from_namespace(&LumpNamespace::Global, "PNAMES", self)?;
        let lump_bytes = lump.bytes();

        if lump.size % 8 != 4 {
            return Err(WadError::CorruptedLump(lump.name.clone()));
        }

        let num_patch_names = u32_le!(lump_bytes[0..4]) as usize;
        if lump.size != num_patch_names * 8 + 4 {
            return Err(WadError::CorruptedLump(lump.name.clone()));
        }

        let mut patch_names: Vec<String> = Vec::new();
        for i in 0..num_patch_names {
            let patch_name_offset = 4 + i * 8;
            let name = match parse_bytes_cstr(&lump_bytes[patch_name_offset..patch_name_offset + 8])
            {
                Ok(middle_texture) => match middle_texture.to_str() {
                    Ok(middle_texture) => middle_texture.to_string(),
                    Err(_) => return Err(WadError::CorruptedString),
                },
                Err(_) => return Err(WadError::CorruptedBytes),
            };

            patch_names.push(name);
        }

        Ok(patch_names)
    }

    pub fn parse_textures(&self) -> Result<IndexMap<String, Texture>, WadError> {
        // The point of the 1/2 distinction is just the shareware vs. full version of DOOM.
        //
        // Regardless, we want to parse them in the order they're presented in
        // by assigning the number as a priority.
        let mut lumps: Vec<&Lump> = Vec::new();
        if let Ok(lump) = lump_from_namespace(&LumpNamespace::Global, "TEXTURE1", self) {
            lumps.push(lump)
        }
        if let Ok(lump) = lump_from_namespace(&LumpNamespace::Global, "TEXTURE2", self) {
            lumps.push(lump)
        }

        if lumps.is_empty() {
            return Err(WadError::MissingLump("TEXTURE1".to_string()));
        }

        let patch_names = self.parse_patch_names()?;

        let mut textures: IndexMap<String, Texture> = IndexMap::new();
        for lump in lumps {
            let lump_bytes = lump.bytes();
            let is_priority = lump.name == "TEXTURE1";

            let num_textures = u32_le!(&lump_bytes[0..4]) as usize;
            let texture_offsets = &lump_bytes[4..4 + num_textures * 4];

            for i in 0..num_textures {
                let texture_offset = u32_le!(&texture_offsets[i * 4..i * 4 + 4]) as usize;
                let texture_bytes = &lump_bytes[texture_offset..];

                let name = match parse_bytes_cstr(&texture_bytes[0..8]) {
                    Ok(name) => match name.to_str() {
                        Ok(name) => name.to_string(),
                        Err(_) => return Err(WadError::CorruptedString),
                    },
                    Err(_) => return Err(WadError::CorruptedBytes),
                };

                // 8..10 and 10..12 are unused.

                let width = u16_le!(&texture_bytes[12..14]);
                let height = u16_le!(&texture_bytes[14..16]);

                // 16..20 is unused—and removed in the Strife specification.

                let num_patches = u16_le!(&texture_bytes[20..22]) as usize;
                let mut patch_entry = Vec::new();

                for j in 0..num_patches {
                    let patch_bytes = &texture_bytes[22 + (j * 10)..22 + (j * 10 + 10)];

                    let x_offset = i16_le!(&patch_bytes[0..2]);
                    let y_offset = i16_le!(&patch_bytes[2..4]);

                    let patch_idx = u16_le!(&patch_bytes[4..6]) as usize;
                    let patch_name = match patch_names.get(patch_idx) {
                        Some(patch) => patch.clone(),
                        None => return Err(WadError::TexturePatchNotFound),
                    };

                    // 6..10 is unused—and removed in the Strife specification.

                    patch_entry.push(TexturePatchEntry {
                        x_offset,
                        y_offset,
                        patch_name,
                    });
                }

                textures.insert(
                    name.to_uppercase().clone(),
                    Texture {
                        name,
                        priority: is_priority,
                        width,
                        height,
                        patch_entry,
                    },
                );
            }
        }

        Ok(textures)
    }
}
