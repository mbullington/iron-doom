use std::collections::HashMap;

use anyhow::Result;
use encase::ShaderType;
use ultraviolet::Vec2;
use wgpu::BufferUsages;

use crate::{
    components::{CWall, CWallTwoSided},
    renderer::helpers::gpu::{GpuStorageBuffer, GpuVertexBuffer},
    world::World,
};

use super::{limits::WALL_DATA_SIZE, PaletteImageData, SectorData};

#[derive(ShaderType)]
pub struct WallStorageData {
    // 0 == upper, 1 == middle, 2 == lower
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
    pub wall_quad_vertex_buf: GpuVertexBuffer<Vec2>,

    /// Stores middle, upper, and lower walls.
    pub wall_buf: GpuStorageBuffer<WallStorageData>,

    pub wall_index_by_entity: HashMap<hecs::Entity, usize>,
}

impl WallData {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        world: &World,
        sector_data: &SectorData,
        palette_image_data: &PaletteImageData,
    ) -> Result<Self> {
        let mut walls: Vec<WallStorageData> = Vec::new();
        let mut wall_index_by_entity: HashMap<hecs::Entity, usize> = HashMap::new();

        for (id, c_wall) in &mut world.world.query::<&CWall>() {
            let wall = _create_wall(id, c_wall, world, sector_data, palette_image_data)?;

            wall_index_by_entity.insert(id, walls.len());
            walls.push(wall)
        }

        Ok(Self {
            wall_quad_vertex_buf: GpuVertexBuffer::new_vec(
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
            )?,

            wall_buf: GpuStorageBuffer::new_vec(
                BufferUsages::STORAGE,
                device,
                walls,
                Some("WallData::wall_buf"),
            )?
            // Resize so we can add more walls later.
            .resize(device, queue, WALL_DATA_SIZE)?,

            wall_index_by_entity,
        })
    }

    pub fn think(
        &mut self,
        queue: &wgpu::Queue,
        world: &World,
        sector_data: &SectorData,
        palette_image_data: &PaletteImageData,
    ) -> Result<()> {
        // TODO: Right now we only update auxiliary information.
        //
        // We can't handle:
        // - Removing a wall.
        // - Adding a wall.

        for id in world.changed_set.changed() {
            if !world.world.satisfies::<&CWall>(*id)? {
                continue;
            }

            let c_wall = world.world.get::<&CWall>(*id)?;
            let wall_index = *self
                .wall_index_by_entity
                .get(id)
                .ok_or(anyhow::anyhow!("Wall index not found."))?;

            let wall = _create_wall(*id, &c_wall, world, sector_data, palette_image_data)?;

            // Write the data to the buffer.
            self.wall_buf.write_to_offset(
                queue,
                wall,
                wall_index as usize * self.wall_buf.stride,
            )?;
        }

        Ok(())
    }
}

fn _create_wall(
    id: hecs::Entity,
    c_wall: &CWall,
    world: &World,
    sector_data: &SectorData,
    palette_image_data: &PaletteImageData,
) -> Result<WallStorageData> {
    // Check if wall is double-sided.
    let back_sector_index = world
        .world
        .query_one::<&CWallTwoSided>(id)?
        .get()
        .map(|two_sided| two_sided.back_sector_index);

    Ok(WallStorageData {
        wall_type: c_wall.wall_type.bits(),

        start_vert: c_wall.start_vert,
        end_vert: c_wall.end_vert,

        flags: c_wall.flags,

        sector_index: *sector_data
            .sector_index_by_index
            .get(&c_wall.sector_index)
            .ok_or(anyhow::anyhow!("Sector index not found."))?,

        back_sector_index: if let Some(idx) = back_sector_index {
            *sector_data
                .sector_index_by_index
                .get(&idx)
                .ok_or(anyhow::anyhow!("Back sector index not found."))?
        } else {
            u32::MAX
        },

        palette_image_index: palette_image_data.lookup_texture(world, id)?,

        x_offset: c_wall.x_offset as i32,
        y_offset: c_wall.y_offset as i32,
    })
}
