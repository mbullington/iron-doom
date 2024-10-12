mod palette_colormap_data;
mod sector_data;
mod wall_data;

use id_map_format::Patch;

pub use palette_colormap_data::*;
pub use sector_data::*;
pub use wall_data::*;

/// Version of mod that wraps negative values around to the end of the range.
pub fn _mod2(x: isize, y: usize) -> usize {
    if x >= y as isize {
        return _mod2(x - y as isize, y);
    }
    if x < 0 {
        return _mod2(y as isize + x, y);
    }
    x as usize
}

/// Represents a palette image.
/// The image is stored in a column-major order.
///
/// The overall size is 2*width*height.
/// - The first bit is the "valid bit", which denotes if it's transparent or not
/// - The second bit is the palette color if valid, or garbage.
struct PaletteImage {
    pub width: u32,
    pub height: u32,
    pub buf: Vec<u8>,
}

impl PaletteImage {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            buf: vec![0; (2 * width * height) as usize],
        }
    }

    pub fn from_patch(patch: &Patch) -> Self {
        let mut image = PaletteImage::new(patch.width as u32, patch.height as u32);

        // For each patch column, mark those bits as valid & their colors.
        for (i, column_list) in patch.columns.iter().enumerate() {
            for column in column_list {
                let x = _mod2(i as isize, patch.width);
                let y_offset = column.y_offset as isize;

                for y in y_offset..(y_offset + column.palette_indices.len() as isize) {
                    let y_mod = _mod2(y, patch.height);

                    image.set(x, y_mod, column.palette_indices[(y - y_offset) as usize]);
                }
            }
        }

        image
    }

    pub fn set(&mut self, x: usize, y: usize, palette_index: u8) {
        assert!(x < self.width as usize);
        assert!(y < self.height as usize);

        // Stored in column-major order.
        let image_buf_idx = 2 * (x * self.height as usize + y);

        self.buf[image_buf_idx] = 1;
        self.buf[image_buf_idx + 1] = palette_index;
    }

    pub fn get(&self, x: usize, y: usize) -> Option<u8> {
        // Stored in column-major order.
        let image_buf_idx = 2 * (x * self.height as usize + y);

        // If the first bit is 0, it's not valid.
        if self.buf[image_buf_idx] == 0 {
            return None;
        }
        Some(self.buf[image_buf_idx + 1])
    }

    pub fn copy_from(&mut self, other: &PaletteImage, dx: isize, dy: isize) {
        for x in 0..other.width {
            for y in 0..other.height {
                if let Some(palette_index) = other.get(x as usize, y as usize) {
                    let x2 = dx + x as isize;
                    let y2 = dy + y as isize;

                    if x2 < 0 || x2 >= self.width as isize {
                        continue;
                    }
                    if y2 < 0 || y2 >= self.height as isize {
                        continue;
                    }

                    self.set(x2 as usize, y2 as usize, palette_index);
                }
            }
        }
    }
}
