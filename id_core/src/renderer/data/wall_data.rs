use anyhow::Result;
use encase::ShaderType;
use ultraviolet::Vec2;
use wgpu::BufferUsages;

use crate::{
    components::{CWall, CWallTwoSided},
    renderer::helpers::gpu::{GpuStorageBuffer, GpuVertexBuffer},
    world::World,
};

use super::{PaletteImageData, SectorData};

#[derive(ShaderType)]
pub struct WallStorageData {
    // 0 == upper, 1 == middle, 2 == lower
    pub wall_type: u32,

    pub start_vert: Vec2,
    pub end_vert: Vec2,

    pub sector_index: u32,
    /// If the wall is one-sided, then this will be u32::MAX.
    /// Otherwise, this will be the back sector index.
    pub back_sector_index: u32,

    pub palette_image_index: u32,

    pub x_offset: f32,
    pub y_offset: f32,
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
}

impl WallData {
    pub fn new(
        device: &wgpu::Device,
        world: &World,
        sector_data: &SectorData,
        palette_image_data: &PaletteImageData,
    ) -> Result<Self> {
        let mut walls: Vec<WallStorageData> = Vec::new();

        for (id, c_wall) in &mut world.world.query::<&CWall>() {
            // Check if wall is double-sided.
            let back_sector_index = world
                .world
                .query_one::<&CWallTwoSided>(id)?
                .get()
                .map(|two_sided| two_sided.back_sector_index);

            let wall = WallStorageData {
                wall_type: c_wall.wall_type.bits(),

                start_vert: c_wall.start_vert,
                end_vert: c_wall.end_vert,

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

                x_offset: c_wall.x_offset as f32,
                y_offset: c_wall.y_offset as f32,
            };

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
            )?,
        })
    }
}
