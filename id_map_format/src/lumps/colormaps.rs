use crate::{lump_from_namespace, LumpNamespace, Wad, WadError};

/// LUT for converting colors to their closest darker color, based on lighting.
/// Returns an index into the palette.
///
/// The first 0..32 entries are for brightness 0..256, diving by 8.
///
/// How to use:
/// `new_index = colormap[(brightness >> 3) * 256 + index]``
pub type Colormaps = Vec<u8>;

impl Wad {
    pub fn parse_colormaps(&self) -> Result<Colormaps, WadError> {
        let lump = lump_from_namespace(&LumpNamespace::Global, "COLORMAP", self)?;
        let lump_bytes = lump.bytes();

        if lump.size % 256 != 0 {
            return Err(WadError::CorruptedLump(lump.name.clone()));
        }

        let num_colormaps = lump.size / 256;
        if num_colormaps < 32 {
            return Err(WadError::NotEnoughColormaps);
        }

        Ok(lump_bytes.to_vec())
    }
}
