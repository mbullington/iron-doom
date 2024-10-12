use crate::{lump_from_namespace, LumpNamespace, Wad, WadError};

/// This is a list of all the palettes in the WAD.
/// The first one, index 0, is the "normal" one except during different game states.
pub type Palettes = Vec<[(u8, u8, u8); 256]>;

impl Wad {
    pub fn parse_palettes(&self) -> Result<Palettes, WadError> {
        let lump = lump_from_namespace(&LumpNamespace::Global, "PLAYPAL", self)?;
        let lump_bytes = lump.bytes();

        if lump.size % (256 * 3) != 0 {
            return Err(WadError::CorruptedLump(lump.name.clone()));
        }

        let num_palettes = lump.size / (256 * 3);
        if num_palettes < 1 {
            return Err(WadError::NotEnoughPalettes);
        }

        let mut palettes: Palettes = Vec::with_capacity(num_palettes);

        for i in 0..num_palettes {
            let palette_offset = i * 256 * 3;
            let mut palette = [(0, 0, 0); 256];
            for j in 0..256 {
                let color_offset = palette_offset + j * 3;
                palette[j] = (
                    lump_bytes[color_offset],
                    lump_bytes[color_offset + 1],
                    lump_bytes[color_offset + 2],
                );
            }

            palettes.push(palette);
        }

        Ok(palettes)
    }
}
