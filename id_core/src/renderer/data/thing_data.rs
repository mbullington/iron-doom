use std::collections::HashMap;

use anyhow::Result;
use encase::ShaderType;
use offset_allocator::{Allocation, Allocator};
use ultraviolet::Vec2;
use wgpu::BufferUsages;

use crate::{
    components::{CThing, CWorldPos},
    renderer::helpers::gpu::{GpuStorageBuffer, GpuVertexBuffer},
    world::World,
};

use super::{limits::THING_DATA_SIZE, PaletteImageData};

#[derive(ShaderType)]
pub struct ThingStorageData {
    pub vert: Vec2,

    pub thing_type: u32,
    pub spawn_flags: u32,

    /// These are static flags that are set per-thing.
    pub thing_flags: u32,

    pub radius: u32,
    pub height: u32,

    // Currently unused.
    pub palette_image_index: u32,
}

/// Things (monsters, entities, etc...) are rendered totally instanced:
/// one instance per thing.
///
/// The vertex shader will take care of transforming the quad into the correct
/// position.
pub struct ThingData {
    /// The basic quad that we'll render for things.
    pub quad_vertex_buf: GpuVertexBuffer<Vec2>,

    pub thing_buf: GpuStorageBuffer<ThingStorageData>,

    pub highest_thing_index: u32,

    thing_alloc: Allocator,
    thing_alloc_by_entity: HashMap<hecs::Entity, Allocation>,
}

impl ThingData {
    pub fn new(
        device: &wgpu::Device,
        world: &World,
        palette_image_data: &PaletteImageData,
    ) -> Result<Self> {
        let mut things: Vec<ThingStorageData> = Vec::new();

        let mut highest_thing_index: u32 = 0;

        let mut thing_alloc = Allocator::new(THING_DATA_SIZE as u32);
        let mut thing_alloc_by_entity: HashMap<hecs::Entity, Allocation> = HashMap::new();

        for (id, (c_thing, c_world_pos)) in &mut world.world.query::<(&CThing, &CWorldPos)>() {
            let thing = _create_thing(id, c_thing, c_world_pos, palette_image_data)?;
            let alloc = thing_alloc
                .allocate(1)
                .ok_or(anyhow::anyhow!("Thing allocation failed, out of space!"))?;

            // Ensure "push" is correct.
            assert!(alloc.offset == things.len() as u32);

            thing_alloc_by_entity.insert(id, alloc);
            highest_thing_index = highest_thing_index.max(alloc.offset);

            things.push(thing)
        }

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
                None,
                Some("ThingData::quad_vertex_buf"),
            )?,

            thing_buf: GpuStorageBuffer::new_vec(
                BufferUsages::STORAGE,
                device,
                things,
                Some(THING_DATA_SIZE as u64),
                Some("ThingData::thing_buf"),
            )?,

            highest_thing_index,

            thing_alloc,
            thing_alloc_by_entity,
        })
    }

    pub fn think(
        &mut self,
        queue: &wgpu::Queue,
        world: &World,
        palette_image_data: &PaletteImageData,
    ) -> Result<()> {
        // First handle changed, which will update the data.
        for id in world.changed_set.changed() {
            if !world.world.satisfies::<&CThing>(*id)? {
                continue;
            }

            let mut query = world.world.query_one::<(&CThing, &CWorldPos)>(*id)?;
            let (c_thing, c_world_pos) = query.get().unwrap();

            let alloc = *self
                .thing_alloc_by_entity
                .get(id)
                .ok_or(anyhow::anyhow!("Thing index not found."))?;

            let thing = _create_thing(*id, c_thing, c_world_pos, palette_image_data)?;

            // Write the data to the buffer.
            self.thing_buf.write_to_offset(
                queue,
                thing,
                alloc.offset as usize * self.thing_buf.stride,
            )?;
        }

        // Next handle removed, so the allocator can free up space.
        for id in world.changed_set.removed() {
            if !world.world.satisfies::<&CThing>(*id)? {
                continue;
            }

            let alloc = *self
                .thing_alloc_by_entity
                .get(id)
                .ok_or(anyhow::anyhow!("Thing index not found."))?;

            self.thing_alloc.free(alloc);
        }

        // Lastly handle spawned, which will add new things.
        for id in world.changed_set.spawned() {
            if !world.world.satisfies::<&CThing>(*id)? {
                continue;
            }

            let mut query = world.world.query_one::<(&CThing, &CWorldPos)>(*id)?;
            let (c_thing, c_world_pos) = query.get().unwrap();

            let thing = _create_thing(*id, c_thing, c_world_pos, palette_image_data)?;
            let alloc = self
                .thing_alloc
                .allocate(1)
                .ok_or(anyhow::anyhow!("Thing allocation failed, out of space!"))?;

            self.thing_alloc_by_entity.insert(*id, alloc);
            self.highest_thing_index = self.highest_thing_index.max(alloc.offset);

            // Write the data to the buffer.
            self.thing_buf.write_to_offset(
                queue,
                thing,
                alloc.offset as usize * self.thing_buf.stride,
            )?;
        }

        Ok(())
    }
}

fn _create_thing(
    _id: hecs::Entity,
    c_thing: &CThing,
    c_world_pos: &CWorldPos,
    _palette_image_data: &PaletteImageData,
) -> Result<ThingStorageData> {
    Ok(ThingStorageData {
        vert: Vec2 {
            x: c_world_pos.pos.x,
            y: c_world_pos.pos.z,
        },

        thing_type: c_thing.thing_type as u32,
        spawn_flags: c_thing.spawn_flags as u32,
        thing_flags: c_thing.thing_flags.bits(),
        radius: c_thing.radius,
        height: c_thing.height,

        // TODO: Replace this with something.
        palette_image_index: 0,
    })
}
