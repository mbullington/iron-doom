use std::collections::HashMap;

use anyhow::Result;
use bytemuck::cast_slice;
use lazy_static::lazy_static;
use regex::Regex;
use wgpu::BufferUsages;

use id_map_format::{LumpNamespace, Map, Patch};

use crate::{
    components::{CTexture, CTextureFloor},
    renderer::helpers::gpu::GpuU8StorageBuffer,
    world::World,
};

pub const MAGIC_OFFSET_INVALID: u32 = 0;
pub const MAGIC_OFFSET_SKY: u32 = 8;

pub struct PaletteImageData {
    pub image_storage_buf: GpuU8StorageBuffer,
    pub image_storage_by_index: HashMap<CTexture, usize>,
}

impl PaletteImageData {
    pub fn new(device: &wgpu::Device, world: &World) -> Result<Self> {
        let textures = &world.textures;

        let mut image_storage_buf: Vec<u8> = Vec::new();
        let mut image_storage_by_index: HashMap<CTexture, usize> = HashMap::new();

        image_storage_by_index.insert(
            CTexture::Flat("-".to_string()),
            MAGIC_OFFSET_INVALID as usize,
        );
        image_storage_by_index.insert(
            CTexture::Texture("-".to_string()),
            MAGIC_OFFSET_INVALID as usize,
        );

        // Sky texture is stored at index 8.
        image_storage_by_index.insert(CTexture::Sky, MAGIC_OFFSET_SKY as usize);

        // Used for walls.
        let mut patches_by_name: HashMap<String, PaletteImage> = HashMap::new();

        // Storage i=0 is reserved for empty.
        image_storage_buf.append(&mut vec![0u8; 8]);

        let mut parse_texture = |c_texture: &CTexture| -> Result<()> {
            if image_storage_by_index.contains_key(c_texture) {
                return Ok(());
            }

            let mut palette_image = match c_texture {
                CTexture::Flat(texture_name) => {
                    world.with_lump(&LumpNamespace::Flat, texture_name, |lump| {
                        // Fill the rest of the buffer with zeros, if it's not a multiple of 4096.
                        let bytes = &mut lump.bytes().to_vec();
                        if bytes.len() < 4096 {
                            let remaining = 4096 - bytes.len();
                            bytes.append(&mut vec![0u8; remaining]);
                        }

                        PaletteImage::from_flat(bytes)
                    })?
                }

                CTexture::Texture(texture_name) | CTexture::Sprite(texture_name) => {
                    let texture = match textures.get(&texture_name.to_uppercase()) {
                        Some(texture) => texture,
                        None => return Err(anyhow::anyhow!("Texture not found: {}", texture_name)),
                    };

                    let mut palette_image =
                        PaletteImage::new(texture.width as u32, texture.height as u32);

                    for entry in &texture.patch_entry {
                        let patch = match patches_by_name.get(&entry.patch_name) {
                            Some(patch) => patch,
                            None => world.with_patch(&entry.patch_name, |patch| {
                                let patch_image = PaletteImage::from_patch(&patch);

                                patches_by_name.insert(entry.patch_name.clone(), patch_image);
                                &patches_by_name[&entry.patch_name]
                            })?,
                        };

                        palette_image.copy_from(
                            patch,
                            entry.x_offset as isize,
                            entry.y_offset as isize,
                        );
                    }

                    palette_image
                }

                CTexture::Sky => {
                    panic!("Sky texture should be handled above.");
                }
            };

            // Make sure each entry is 32bit aligned.
            let len = image_storage_buf.len();
            let len_aligned = len + ((4 - len % 4) % 4);
            image_storage_buf.append(&mut vec![0u8; len_aligned - len]);

            // Add the width & height to the buf as u32.
            let mut dims =
                cast_slice::<u32, u8>(&[palette_image.width, palette_image.height]).to_vec();
            image_storage_buf.append(&mut dims);
            image_storage_buf.append(&mut palette_image.buf);

            // Add the aligned len to the index.
            image_storage_by_index.insert(c_texture.clone(), len_aligned);

            Ok(())
        };

        // Storage i=8 is reserved for the sky texture.
        let sky_texture = _get_sky_texture(&world.map);
        parse_texture(&CTexture::Texture(sky_texture.to_string()))?;

        // Parse textures.
        for (_id, c_texture) in &mut world.world.query::<&CTexture>() {
            parse_texture(c_texture)?;
        }
        // Parse textures for floor flats.
        for (_id, c_texture) in &mut world.world.query::<&CTextureFloor>() {
            parse_texture(&c_texture.0)?;
        }
        // Parse textures for animation states.
        for c_texture in world.animations.keys() {
            parse_texture(c_texture)?;
        }

        Ok(PaletteImageData {
            image_storage_buf: GpuU8StorageBuffer::new_vec(
                BufferUsages::STORAGE,
                device,
                image_storage_buf,
                Some("PaletteImageData::image_storage_buf"),
            )?,
            image_storage_by_index,
        })
    }

    pub fn lookup_texture(&self, world: &World, id: hecs::Entity) -> Result<u32> {
        Ok(match world.world.query_one::<&CTexture>(id)?.get() {
            Some(texture) => self
                .image_storage_by_index
                .get(texture)
                .map(|x| *x as u32)
                .unwrap_or_else(|| {
                    eprintln!("Texture not found: {:?}", texture);
                    MAGIC_OFFSET_INVALID
                }),
            None => MAGIC_OFFSET_INVALID,
        })
    }

    pub fn lookup_texture_floor(&self, world: &World, id: hecs::Entity) -> Result<u32> {
        Ok(match world.world.query_one::<&CTextureFloor>(id)?.get() {
            Some(texture) => self
                .image_storage_by_index
                .get(&texture.0)
                .map(|x| *x as u32)
                .unwrap_or_else(|| {
                    eprintln!("Texture not found: {:?}", texture.0);
                    MAGIC_OFFSET_INVALID
                }),
            None => MAGIC_OFFSET_INVALID,
        })
    }
}

/// Represents a palette image.
/// The image is stored in a column-major order.
///
/// The overall size is `2*width*height`.
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
        let mut image = PaletteImage::new(patch.width, patch.height);

        // For each patch column, mark those bits as valid & their colors.
        for (i, column_list) in patch.columns.iter().enumerate() {
            for column in column_list {
                let x = _mod2(i as isize, patch.width as usize);
                let y_offset = column.y_offset as isize;

                for y in y_offset..(y_offset + column.palette_indices.len() as isize) {
                    let y_mod = _mod2(y, patch.height as usize);

                    image.set(x, y_mod, column.palette_indices[(y - y_offset) as usize]);
                }
            }
        }

        image
    }

    pub fn from_flat(flat: &[u8]) -> PaletteImage {
        let mut image = PaletteImage::new(64, 64);
        // Maybe there's a faster way of doing this.
        for i in 0..64 {
            for j in 0..64 {
                image.set(i, j, flat[i + j * 64]);
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

lazy_static! {
    static ref REGEX_E1M1: Regex = Regex::new(r"^E([0-9])M([0-9])$").unwrap();
    static ref REGEX_MAP01: Regex = Regex::new(r"^MAP([0-9]{2})$").unwrap();
}

/// Gets the sky texture given the current map. This is somewhat hardcoded between
/// games, i.e. DOOM II has a scheme that's different from DOOM I, and these aren't
/// defined in code.
///
/// Reference:
/// https://doomwiki.org/wiki/Sky
fn _get_sky_texture(map: &Map) -> &'static str {
    // For episodes in E1M1 format, convert the episode (2nd capture group)
    // into SKY1, SKY2, or SKY3.
    if let Some(captures) = REGEX_E1M1.captures(&map.name) {
        let episode = captures.get(1).unwrap().as_str();
        return match episode {
            "1" => "SKY1",
            "2" => "SKY2",
            "3" => "SKY3",
            // Used in Hexen, and Ultimate DOOM.
            "4" => "SKY4",
            _ => "SKY1",
        };
    }

    // For maps in MAP01 format:
    // - MAP01 to MAP11: SKY1
    // - MAP12 to MAP20: SKY2
    // - MAP21 to MAP32: SKY3
    if let Some(captures) = REGEX_MAP01.captures(&map.name) {
        let map_num = captures.get(1).unwrap().as_str().parse::<u32>().unwrap();
        return match map_num {
            1..=11 => "SKY1",
            12..=20 => "SKY2",
            21..=32 => "SKY3",
            _ => "SKY1",
        };
    }

    // Likely a ZDoom map. Just return SKY1 for now.
    "SKY1"
}
