mod palette_colormap_data;
mod palette_image_data;
mod sector_data;
mod wall_data;

pub use palette_colormap_data::*;
pub use palette_image_data::*;
pub use sector_data::*;
pub use wall_data::*;

// Set custom limits for each buffer.
//
// The overhead is so we can dynamically add new sectors, etc...
// in a map editor context.
//
// These are very healthy margins: I based them as 2-3x the largest map I could find,
// SOS_Boom.wad (MAP32).
pub mod limits {
    pub const PALETTE_IMAGE_DATA_SIZE: usize = 75 * 1024 * 1024;
    pub const SECTOR_DATA_SIZE: usize = 10 * 1024 * 1024;
    pub const WALL_DATA_SIZE: usize = 10 * 1024 * 1024;
}
