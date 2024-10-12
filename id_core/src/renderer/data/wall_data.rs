use std::collections::HashMap;

use id_map_format::{Linedef, Map, Sidedef, Texture, Wad};

use anyhow::Result;
use encase::ShaderType;
use lazy_static::lazy_static;
use regex::Regex;
use ultraviolet::Vec2;
use wgpu::BufferUsages;

use super::PaletteImage;
use crate::renderer::helpers::gpu::{GpuStorageBuffer, GpuU8StorageBuffer, GpuVertexBuffer};

const WALL_TYPE_MIDDLE: u32 = 0;
const WALL_TYPE_UPPER: u32 = 1;
const WALL_TYPE_LOWER: u32 = 2;

#[derive(ShaderType)]
pub struct WallStorageData {
    // 0 == middle, 1 == upper, 2 == lower
    pub wall_type: u32,

    pub start_vert_index: u32,
    pub end_vert_index: u32,

    pub sector_index: u32,
    /// If the wall is one-sided, then this will be u32::MAX.
    /// Otherwise, this will be the back sector index.
    pub back_sector_index: u32,

    pub patch_index: u32,

    pub x_offset: f32,
    pub y_offset: f32,
}

#[derive(ShaderType)]
pub struct PatchHeaderStorageData {
    pub width: u32,
    pub height: u32,
    pub buffer_idx: u32,
}

/// Walls are rendered totally instanced; we have a single quad that we render
/// twice: one for middle walls, and one for edge walls.
///
/// The vertex shader will take care of transforming the quad into the correct
/// position.
pub struct WallData {
    /// The basic quad that we'll render for walls.
    pub wall_quad_vertex_buf: GpuVertexBuffer<Vec2>,

    /// Stores 2D vertices.
    pub vertices_storage_buffer: GpuStorageBuffer<Vec2>,

    /// Stores middle, upper, and lower walls.
    pub wall_storage_buffer: GpuStorageBuffer<WallStorageData>,

    /// Stores texture headers.
    /// The sky texture will always be stored in the first index.
    pub patch_headers_storage_data: GpuStorageBuffer<PatchHeaderStorageData>,
    /// Stores textures.
    pub patch_storage_buffer: GpuU8StorageBuffer,
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

struct WallPatchBuilder<'a> {
    pub patch_headers: Vec<PatchHeaderStorageData>,
    pub patch_buf: Vec<u8>,

    wad: &'a Wad,
    textures: HashMap<String, Texture>,

    patches_by_name: HashMap<String, PaletteImage>,
    patch_headers_by_name: HashMap<String, usize>,
}

impl<'a> WallPatchBuilder<'a> {
    pub fn new(wad: &'a Wad) -> Result<Self> {
        Ok(Self {
            patch_headers: Vec::new(),
            patch_buf: Vec::new(),
            wad,
            textures: wad.parse_textures()?,
            patches_by_name: HashMap::new(),
            patch_headers_by_name: HashMap::new(),
        })
    }

    pub fn get_patch_header(&mut self, texture_name: &str) -> Result<usize> {
        if let Some(header_idx) = self.patch_headers_by_name.get(texture_name) {
            return Ok(*header_idx);
        }

        let texture = match self.textures.get(&texture_name.to_uppercase()) {
            Some(texture) => texture,
            None => return Err(anyhow::anyhow!("Texture not found: {}", texture_name)),
        };

        let width = texture.width as u32;
        let height = texture.height as u32;

        assert!(width > 0);
        assert!(height > 0);

        let mut texture_image = PaletteImage::new(width, height);
        for entry in &texture.patch_entry {
            let patch = match self.patches_by_name.get(&entry.patch_name) {
                Some(patch) => patch,
                None => {
                    let patch = self.wad.parse_patch(&entry.patch_name)?;
                    let patch_image = PaletteImage::from_patch(&patch);

                    self.patches_by_name
                        .insert(entry.patch_name.clone(), patch_image);
                    &self.patches_by_name[&entry.patch_name]
                }
            };

            texture_image.copy_from(patch, entry.x_offset as isize, entry.y_offset as isize);
        }

        let pointer = self.patch_buf.len();
        self.patch_buf.extend_from_slice(&texture_image.buf);

        assert!(self.patch_buf.len() - texture_image.buf.len() == pointer);
        assert!(self.patch_buf.len() % 2 == 0);

        let header = PatchHeaderStorageData {
            width,
            height,
            buffer_idx: pointer as u32,
        };

        self.patch_headers.push(header);
        self.patch_headers_by_name
            .insert(texture_name.to_string(), self.patch_headers.len() - 1);

        Ok(self.patch_headers.len() - 1)
    }
}

fn _parse_wall(
    map: &Map,
    linedef: &Linedef,
    sidedef: &Sidedef,
    other: Option<&Sidedef>,
    flip_vertices: bool,
    walls: &mut Vec<WallStorageData>,
    patch_builder: &mut WallPatchBuilder,
) -> Result<()> {
    let start_vert_index = match flip_vertices {
        true => linedef.end_vertex_idx as u32,
        false => linedef.start_vertex_idx as u32,
    };

    let end_vert_index = match flip_vertices {
        true => linedef.start_vertex_idx as u32,
        false => linedef.end_vertex_idx as u32,
    };

    if sidedef.middle_texture != "-" {
        let patch_index = patch_builder.get_patch_header(sidedef.middle_texture.as_str())?;
        walls.push(WallStorageData {
            wall_type: WALL_TYPE_MIDDLE,
            start_vert_index,
            end_vert_index,
            sector_index: sidedef.sector_idx as u32,
            // If there's no other sidedef, then it's a one-sided wall.
            back_sector_index: if let Some(other) = other {
                other.sector_idx as u32
            } else {
                u32::MAX
            },
            patch_index: patch_index as u32,
            x_offset: sidedef.x_offset as f32,
            y_offset: sidedef.y_offset as f32,
        });
    }

    if let Some(other) = other {
        if sidedef.lower_texture != "-" {
            let patch_index = patch_builder.get_patch_header(sidedef.lower_texture.as_str())?;
            walls.push(WallStorageData {
                wall_type: WALL_TYPE_LOWER,
                start_vert_index,
                end_vert_index,
                sector_index: sidedef.sector_idx as u32,
                back_sector_index: other.sector_idx as u32,
                patch_index: patch_index as u32,
                x_offset: sidedef.x_offset as f32,
                y_offset: sidedef.y_offset as f32,
            });
        }

        let mut upper_texture = sidedef.upper_texture.as_str();

        let sidedef_sector = &map.sectors[sidedef.sector_idx as usize];
        let other_sector = &map.sectors[other.sector_idx as usize];

        // If the other sector has a sky ceiling, then we should render
        // the upper texture as the sky.
        if sidedef.upper_texture == "-"
            && other_sector.ceiling_height < sidedef_sector.ceiling_height
            && other_sector.ceiling_flat == "F_SKY1"
        {
            upper_texture = _get_sky_texture(map);
        }

        // https://doomwiki.org/wiki/Sky_hack
        if sidedef_sector.ceiling_flat == "F_SKY1" && other_sector.ceiling_flat == "F_SKY1" {
            upper_texture = _get_sky_texture(map);
        }

        if upper_texture != "-" {
            let patch_index = patch_builder.get_patch_header(upper_texture)?;
            walls.push(WallStorageData {
                wall_type: WALL_TYPE_UPPER,
                start_vert_index,
                end_vert_index,
                sector_index: sidedef.sector_idx as u32,
                back_sector_index: other.sector_idx as u32,
                patch_index: patch_index as u32,
                x_offset: sidedef.x_offset as f32,
                y_offset: sidedef.y_offset as f32,
            });
        }
    }

    Ok(())
}

impl WallData {
    pub fn new(device: &wgpu::Device, wad: &Wad, map: &Map) -> Result<Self> {
        let wall_quad_vertex_buf = GpuVertexBuffer::new_vec(
            BufferUsages::VERTEX,
            device,
            vec![
                Vec2::new(0., 0.),
                Vec2::new(1., 0.),
                Vec2::new(0., 1.),
                Vec2::new(0., 1.),
                Vec2::new(1., 0.),
                Vec2::new(1., 1.),
            ],
            Some("WallData::wall_quad_vertex_buf"),
        )?;

        let vertices_storage_buffer = {
            let mut vertices = Vec::new();
            for vertex in map.vertices.iter() {
                vertices.push(Vec2::new(vertex.x as f32, vertex.y as f32));
            }
            GpuStorageBuffer::new_vec(
                BufferUsages::STORAGE,
                device,
                vertices,
                Some("WallData::vertices_storage_buffer"),
            )?
        };

        let mut walls: Vec<WallStorageData> = Vec::new();
        let mut patch_builder = WallPatchBuilder::new(wad)?;

        // Build the sky texture first.
        let sky_texture = _get_sky_texture(map);
        patch_builder.get_patch_header(sky_texture)?;

        // Traverse through each linedef.
        for linedef in &map.linedefs {
            let left_sidedef_opt = linedef
                .left_sidedef_idx
                .map(|idx| &map.sidedefs[idx as usize]);

            let right_sidedef_opt = linedef
                .right_sidedef_idx
                .map(|idx| &map.sidedefs[idx as usize]);

            if let Some(left_sidedef) = left_sidedef_opt {
                _parse_wall(
                    map,
                    linedef,
                    left_sidedef,
                    right_sidedef_opt,
                    true, // Flip vertices.
                    &mut walls,
                    &mut patch_builder,
                )?;
            }

            if let Some(right_sidedef) = right_sidedef_opt {
                _parse_wall(
                    map,
                    linedef,
                    right_sidedef,
                    left_sidedef_opt,
                    false, // Don't flip vertices.
                    &mut walls,
                    &mut patch_builder,
                )?;
            }
        }

        Ok(Self {
            wall_quad_vertex_buf,
            vertices_storage_buffer,

            wall_storage_buffer: GpuStorageBuffer::new_vec(
                BufferUsages::STORAGE,
                device,
                walls,
                Some("WallData::wall_storage_buffer"),
            )?,

            patch_headers_storage_data: GpuStorageBuffer::new_vec(
                BufferUsages::STORAGE,
                device,
                patch_builder.patch_headers,
                Some("WallData::patch_headers_storage_data"),
            )?,

            patch_storage_buffer: GpuU8StorageBuffer::new_vec(
                BufferUsages::STORAGE,
                device,
                patch_builder.patch_buf,
                Some("WallData::patch_storage_buffer"),
            )?,
        })
    }
}
