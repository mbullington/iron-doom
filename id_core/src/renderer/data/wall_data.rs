use std::collections::HashMap;

use anyhow::Result;
use encase::ShaderType;
use offset_allocator::{Allocation, Allocator};
use ultraviolet::Vec2;
use wgpu::BufferUsages;

use crate::{
    components::{CWall, CWallTwoSided},
    renderer::helpers::gpu::{GpuStorageBuffer, GpuVertexBuffer, LenOrData},
    world::World,
};

use super::{limits::WALL_DATA_SIZE, PaletteImageData, SectorData};

#[derive(ShaderType, Default)]
pub struct WallStorageData {
    // 0 == invalid, 1 == upper, 2 == middle, 3 == lower
    pub wall_type: u32,

    pub start_vert: Vec2,
    pub end_vert: Vec2,

    pub flags: u32,

    pub sector_index: u32,
    /// If the wall is one-sided, then this will be u32::MAX.
    /// Otherwise, this will be the back sector index.
    pub back_sector_index: u32,

    pub palette_image_index: u32,

    pub x_offset: i32,
    pub y_offset: i32,
}

/// Walls are rendered totally instanced; we have a single quad that we render
/// twice: one for middle walls, and one for edge walls.
///
/// The vertex shader will take care of transforming the quad into the correct
/// position.
pub struct WallData {
    /// The basic quad that we'll render for walls.
    pub quad_vertex_buf: GpuVertexBuffer<Vec2>,

    /// Stores middle, upper, and lower walls.
    pub wall_buf: GpuStorageBuffer<WallStorageData>,
    pub highest_wall_index: u32,

    wall_alloc: Allocator,
    wall_alloc_by_entity: HashMap<hecs::Entity, Allocation>,
}

impl WallData {
    pub fn new(device: &wgpu::Device) -> Result<Self> {
        Ok(Self {
            quad_vertex_buf: GpuVertexBuffer::new_vec(
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
                Some("WallData::quad_vertex_buf"),
            )?,

            wall_buf: GpuStorageBuffer::new(
                BufferUsages::STORAGE,
                device,
                LenOrData::Len(WALL_DATA_SIZE as u64),
                Some("WallData::wall_buf"),
            )?,
            highest_wall_index: 0,

            wall_alloc: Allocator::new(WALL_DATA_SIZE as u32),
            wall_alloc_by_entity: HashMap::new(),
        })
    }

    pub fn think(
        &mut self,
        queue: &wgpu::Queue,
        world: &World,
        sector_data: &SectorData,
        palette_image_data: &PaletteImageData,
    ) -> Result<()> {
        let mut thing_writes = SparseVec::<ThingStorageData>::default();

        // First handle removed, so the allocator can free up space.
        for id in world.changed_set.removed() {
            if !world.world.satisfies::<&CWall>(*id)? {
                continue;
            }

            let alloc = *self
                .wall_alloc_by_entity
                .get(id)
                .ok_or(anyhow::anyhow!("Wall index not found."))?;

            self.wall_alloc.free(alloc);
        }

        // Next handle changed & spawned entities.
        for id in world
            .changed_set
            .changed()
            .iter()
            .chain(world.changed_set.spawned().iter())
        {
            if !world.world.satisfies::<&CWall>(*id)? {
                continue;
            }

            let c_wall = world.world.get::<&CWall>(*id)?;

            // Check if wall is double-sided.
            let back_sector_index = world
                .world
                .query_one::<&CWallTwoSided>(*id)?
                .get()
                .map(|two_sided| two_sided.back_sector_index);

            let wall = WallStorageData {
                // Add one so we're not invalid.
                wall_type: c_wall.wall_type.bits() + 1,

                start_vert: c_wall.start_vert,
                end_vert: c_wall.end_vert,

                flags: c_wall.flags,

                sector_index: sector_data
                    .sector_alloc_by_index
                    .get(&c_wall.sector_index)
                    .ok_or(anyhow::anyhow!("Sector index not found."))?
                    .offset,

                back_sector_index: if let Some(idx) = back_sector_index {
                    sector_data
                        .sector_alloc_by_index
                        .get(&idx)
                        .ok_or(anyhow::anyhow!("Back sector index not found."))?
                        .offset
                } else {
                    u32::MAX
                },

                palette_image_index: palette_image_data.lookup_texture(world, *id)?,

                x_offset: c_wall.x_offset as i32,
                y_offset: c_wall.y_offset as i32,
            };

            let alloc = self
                .wall_alloc_by_entity
                .get(id)
                .copied()
                .unwrap_or_else(|| self.wall_alloc.allocate(1).unwrap());

            self.wall_alloc_by_entity.insert(*id, alloc);
            self.highest_wall_index = self.highest_wall_index.max(alloc.offset);

            // Write the data to the buffer.
            self.wall_buf.write_to_offset(
                queue,
                wall,
                alloc.offset as usize * self.wall_buf.stride,
            )?;
        }

        Ok(())
    }
}
