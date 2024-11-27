use std::collections::HashMap;

use anyhow::Result;
use encase::ShaderType;
use ultraviolet::Vec2;
use wgpu::BufferUsages;

use crate::{
    components::CSector,
    renderer::helpers::gpu::{GpuIndexBuffer, GpuStorageBuffer, GpuVertexBuffer},
    world::World,
};

use super::PaletteImageData;

#[repr(C)]
#[derive(ShaderType)]
pub struct SectorVertexData {
    pub position: Vec2,
    /// This is the index in the sector storage buffer.
    pub storage_index: u32,
}

#[derive(ShaderType)]
pub struct SectorStorageData {
    pub floor_height: f32,
    pub ceiling_height: f32,

    // Indexes in the flat array.
    pub ceiling_palette_image_index: u32,
    pub floor_palette_image_index: u32,

    // Light level.
    pub light_level: u32,
}

pub struct SectorData {
    /// Can be used to draw either floors or ceilings.
    pub vertex_buf: GpuVertexBuffer<SectorVertexData>,
    /// Assumed to be wound with +y = CW.
    pub index_buf: GpuIndexBuffer,

    /// Stores auxiliary information about each sector.
    pub sector_buf: GpuStorageBuffer<SectorStorageData>,

    /// This converts "sector_index" (from the wad) into the index of the sector buffer.
    pub sector_index_by_index: HashMap<usize, u32>,
}

impl SectorData {
    pub fn new(
        device: &wgpu::Device,
        world: &World,
        palette_image_data: &PaletteImageData,
    ) -> Result<Self> {
        let mut sector_vertices: Vec<SectorVertexData> = Vec::new();
        let mut sector_indices: Vec<u32> = Vec::new();

        let mut sector_storage: Vec<SectorStorageData> = Vec::new();

        let mut sector_index_by_index: HashMap<usize, u32> = HashMap::new();

        for (id, c_sector) in &mut world.world.query::<&CSector>() {
            for triangle in &c_sector.triangles {
                let old_vertices_len = sector_vertices.len() as u32;
                sector_vertices.append(
                    &mut triangle
                        .points
                        .iter()
                        .map(|p| SectorVertexData {
                            position: *p,
                            storage_index: sector_storage.len() as u32,
                        })
                        .collect::<Vec<_>>(),
                );

                sector_indices.append(
                    &mut triangle
                        .indices
                        .iter()
                        .map(|i| i + old_vertices_len)
                        .collect::<Vec<_>>(),
                );
            }

            sector_index_by_index.insert(c_sector.sector_index, sector_storage.len() as u32);

            sector_storage.push(SectorStorageData {
                floor_height: c_sector.floor_height as f32,
                ceiling_height: c_sector.ceiling_height as f32,
                light_level: c_sector.light_level as u32,

                ceiling_palette_image_index: palette_image_data.lookup_texture(world, id)?,
                floor_palette_image_index: palette_image_data.lookup_texture_floor(world, id)?,
            });
        }

        Ok(Self {
            vertex_buf: GpuVertexBuffer::new_vec(
                BufferUsages::VERTEX,
                device,
                sector_vertices,
                Some("SectorData::vertex_buf"),
            )?,
            index_buf: GpuIndexBuffer::new_vec(
                BufferUsages::INDEX,
                device,
                sector_indices,
                Some("SectorData::index_buf"),
            )?,
            sector_buf: GpuStorageBuffer::new_vec(
                BufferUsages::STORAGE,
                device,
                sector_storage,
                Some("SectorData::sector_buf"),
            )?,
            sector_index_by_index,
        })
    }
}
