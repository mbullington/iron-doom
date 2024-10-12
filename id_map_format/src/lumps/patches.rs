use crate::{lump_from_namespace, Lump, LumpNamespace, Texture, Wad, WadError};

#[derive(Debug)]
pub struct PatchColumnSpan {
    pub y_offset: u16,
    pub palette_indices: Vec<u8>,
}

#[derive(Debug)]
pub struct Patch {
    pub width: usize,
    pub height: usize,

    pub x_center: i16,
    pub y_center: i16,

    // Patch data is implemented in a column-major format.
    //
    // This is because in DOOM's original software renderer, the screen is drawn
    // in vertical strips.
    pub columns: Vec<Vec<PatchColumnSpan>>,
}

impl Wad {
    pub fn parse_patch(&self, patch_name: &str) -> Result<Patch, WadError> {
        // DOOM technically never reads the START/END lumps for patches,
        // so they're "not required."
        //
        // We get around this by searching the PATCH namespace first, then the
        // global namespace.
        let lump: &Lump =
            match lump_from_namespace(&LumpNamespace::Patch, &patch_name.to_uppercase(), self) {
                Ok(lump) => lump,
                Err(_) => match lump_from_namespace(
                    &LumpNamespace::Global,
                    &patch_name.to_uppercase(),
                    self,
                ) {
                    Ok(lump) => lump,
                    Err(_) => return Err(WadError::PatchDoesNotExist(patch_name.to_string())),
                },
            };

        let lump_bytes = lump.bytes();

        let width = u16_le!(&lump_bytes[0..2]) as usize;
        let height = u16_le!(&lump_bytes[2..4]) as usize;

        let x_center: i16 = i16_le!(&lump_bytes[4..6]);
        let y_center: i16 = i16_le!(&lump_bytes[6..8]);

        let mut columns: Vec<Vec<PatchColumnSpan>> = Vec::with_capacity(width);

        for i in 0..(width) {
            let col_lookup_offset = 8 + i * 4;
            let col_lookup =
                u32_le!(&lump_bytes[col_lookup_offset..col_lookup_offset + 4]) as usize;

            let mut colspans: Vec<PatchColumnSpan> = Vec::new();

            let mut curr_offset = col_lookup;
            loop {
                let span_bytes = &lump_bytes[curr_offset..];
                let y_offset = span_bytes[0];
                if y_offset == 255 {
                    break;
                }

                let num_palette_indicies = span_bytes[1] as usize;
                // 2 is a garbage byte.

                let palette_indices: Vec<u8> = span_bytes[3..3 + num_palette_indicies].to_vec();
                colspans.push(PatchColumnSpan {
                    y_offset: y_offset as u16,
                    palette_indices,
                });

                // Last byte is garbage.
                curr_offset += 4 + num_palette_indicies;
            }

            columns.push(colspans);
        }

        Ok(Patch {
            width,
            height,
            x_center,
            y_center,
            columns,
        })
    }

    pub fn parse_patches_for_texture(&self, texture: &Texture) -> Result<Vec<Patch>, WadError> {
        let mut patches: Vec<Patch> = Vec::new();
        for patch_entry in &texture.patch_entry {
            let patch = self.parse_patch(&patch_entry.patch_name)?;
            patches.push(patch);
        }

        Ok(patches)
    }
}
