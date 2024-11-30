use std::collections::HashMap;

use meshopt::{
    generate_vertex_remap, optimize_vertex_cache_in_place, optimize_vertex_fetch_in_place,
    remap_index_buffer, remap_vertex_buffer,
};

use anyhow::Result;
use encase::ShaderType;
use offset_allocator::{Allocation, Allocator};
use ultraviolet::Vec2;
use wgpu::BufferUsages;

use crate::{
    components::CSector,
    renderer::helpers::gpu::{GpuIndexBuffer, GpuStorageBuffer, GpuVertexBuffer, LenOrData},
    world::World,
};

use super::{limits::SECTOR_DATA_SIZE, PaletteImageData};

#[repr(C)]
#[derive(ShaderType, Clone, Default)]
pub struct SectorVertexData {
    pub position: Vec2,
    /// This is the index in the sector storage buffer.
    pub sector_index: u32,
}

#[derive(ShaderType)]
pub struct SectorStorageData {
    pub floor_height: i32,
    pub ceiling_height: i32,

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

    sector_alloc: Allocator,

    /// This converts "sector_index" (from the wad) into the index of the sector buffer.
    pub sector_alloc_by_index: HashMap<usize, Allocation>,
    sector_alloc_by_entity: HashMap<hecs::Entity, Allocation>,
}

impl SectorData {
    pub fn new(device: &wgpu::Device) -> Result<Self> {
        Ok(Self {
            // Instead of resizing "vertex_buf" and "index_buf",
            // we just recreate them later if we need to add more sectors.
            vertex_buf: GpuVertexBuffer::new_vec(
                BufferUsages::VERTEX,
                device,
                vec![],
                Some("SectorData::vertex_buf"),
            )?,
            index_buf: GpuIndexBuffer::new_vec(
                BufferUsages::INDEX,
                device,
                vec![],
                Some("SectorData::index_buf"),
            )?,

            sector_buf: GpuStorageBuffer::new(
                BufferUsages::STORAGE,
                device,
                LenOrData::Len(SECTOR_DATA_SIZE as u64),
                Some("SectorData::sector_buf"),
            )?,

            sector_alloc: Allocator::new(SECTOR_DATA_SIZE as u32),
            sector_alloc_by_index: HashMap::new(),
            sector_alloc_by_entity: HashMap::new(),
        })
    }

    pub fn think(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        world: &mut World,
        palette_image_data: &PaletteImageData,
    ) -> Result<()> {
        // TODO: We can't handle changing the sectors' triangles yet.
        // We should build a hash tree or something.
        let mut needs_recreated_mesh = false;

        // First, update existing sectors.
        for id in world.changed_set.changed() {
            if !world.world.satisfies::<&CSector>(*id)? {
                continue;
            }

            let mut c_sector = world.world.get::<&mut CSector>(*id)?;

            let sector = _create_sector(*id, &c_sector, world, palette_image_data)?;
            let sector_index = self
                .sector_alloc_by_entity
                .get(id)
                .ok_or(anyhow::anyhow!("Sector index not found."))?
                .offset as usize;

            // TODO: If the triangles are updated, we need to recreate the mesh.
            if c_sector.triangles.changed() {
                println!("Sector {} triangles changed.", sector_index);
                needs_recreated_mesh = true;
                c_sector.triangles.clear_changed();
            }

            // Write the data to the buffer.
            self.sector_buf.write_to_offset(
                queue,
                sector,
                sector_index as usize * self.sector_buf.stride,
            )?;
        }

        // Next, remove any sectors that were deleted.
        for id in world.changed_set.removed() {
            if !world.world.satisfies::<&CSector>(*id)? {
                continue;
            }

            // We need to recreate the mesh.
            needs_recreated_mesh = true;

            let sector_index = self
                .sector_alloc_by_entity
                .get(id)
                .ok_or(anyhow::anyhow!("Sector index not found."))?
                .offset as usize;

            self.sector_alloc.free(self.sector_alloc_by_entity[id]);

            self.sector_alloc_by_entity.remove(id);
            self.sector_alloc_by_index.remove(&sector_index);
        }

        // Lastly handle spawned, which will add new walls.
        for id in world.changed_set.spawned() {
            if !world.world.satisfies::<&CSector>(*id)? {
                continue;
            }

            // We need to recreate the mesh.
            needs_recreated_mesh = true;

            let c_sector = world.world.get::<&CSector>(*id)?;
            let sector = _create_sector(*id, &c_sector, world, palette_image_data)?;
            let alloc = self
                .sector_alloc
                .allocate(1)
                .ok_or(anyhow::anyhow!("Sector allocation failed, out of space!"))?;

            self.sector_alloc_by_index
                .insert(c_sector.sector_index, alloc);
            self.sector_alloc_by_entity.insert(*id, alloc);

            // Write the data to the buffer.
            self.sector_buf.write_to_offset(
                queue,
                sector,
                alloc.offset as usize * self.sector_buf.stride,
            )?;
        }

        if needs_recreated_mesh {
            let (sector_vertices, sector_indices) =
                _create_mesh(world, &self.sector_alloc_by_entity)?;

            self.vertex_buf = GpuVertexBuffer::new_vec(
                BufferUsages::VERTEX,
                device,
                sector_vertices,
                Some("SectorData::vertex_buf"),
            )?;
            self.index_buf = GpuIndexBuffer::new_vec(
                BufferUsages::INDEX,
                device,
                sector_indices,
                Some("SectorData::index_buf"),
            )?;
        }

        Ok(())
    }
}

fn _create_sector(
    id: hecs::Entity,
    c_sector: &CSector,
    world: &World,
    palette_image_data: &PaletteImageData,
) -> Result<SectorStorageData> {
    Ok(SectorStorageData {
        floor_height: c_sector.floor_height as i32,
        ceiling_height: c_sector.ceiling_height as i32,
        light_level: c_sector.light_level as u32,

        ceiling_palette_image_index: palette_image_data.lookup_texture(world, id)?,
        floor_palette_image_index: palette_image_data.lookup_texture_floor(world, id)?,
    })
}

fn _create_mesh(
    world: &World,
    sector_alloc_by_entity: &HashMap<hecs::Entity, Allocation>,
) -> Result<(Vec<SectorVertexData>, Vec<u32>)> {
    let mut sector_vertices: Vec<SectorVertexData> = Vec::new();
    let mut sector_indices: Vec<u32> = Vec::new();

    for (id, c_sector) in &mut world.world.query::<&CSector>() {
        for triangle in c_sector.triangles.iter() {
            let old_vertices_len = sector_vertices.len() as u32;

            let sector_index = sector_alloc_by_entity
                .get(&id)
                .ok_or(anyhow::anyhow!("Sector index not found."))?
                .offset;

            sector_vertices.append(
                &mut triangle
                    .points
                    .iter()
                    .map(|p| SectorVertexData {
                        position: *p,
                        sector_index,
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
    }

    // Use meshoptimizer.
    let (vertex_count, remap) = generate_vertex_remap(&sector_vertices, Some(&sector_indices));

    let mut sector_vertices = remap_vertex_buffer(&sector_vertices, vertex_count, &remap);
    let mut sector_indices = remap_index_buffer(Some(&sector_indices), vertex_count, &remap);

    optimize_vertex_cache_in_place(&mut sector_indices, vertex_count);
    optimize_vertex_fetch_in_place(&mut sector_indices, &mut sector_vertices);

    Ok((sector_vertices, sector_indices))
}
