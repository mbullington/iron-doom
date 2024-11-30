use anyhow::Result;
use ultraviolet::Vec3;
use wgpu::BufferUsages;

use crate::{
    renderer::helpers::gpu::{GpuStorageBuffer, GpuU8StorageBuffer},
    world::World,
};

pub struct PaletteColormapData {
    pub palette_storage_buf: GpuStorageBuffer<Vec3>,
    pub colormap_storage_buf: GpuU8StorageBuffer,
}

impl PaletteColormapData {
    pub fn new(device: &wgpu::Device, world: &World) -> Result<Self> {
        Ok(Self {
            palette_storage_buf: GpuStorageBuffer::new_vec(
                BufferUsages::STORAGE,
                device,
                world.palette.clone(),
                None,
                None,
            )?,
            colormap_storage_buf: GpuU8StorageBuffer::new_vec(
                BufferUsages::STORAGE,
                device,
                world.colormap.clone(),
                None,
                None,
            )?,
        })
    }
}
