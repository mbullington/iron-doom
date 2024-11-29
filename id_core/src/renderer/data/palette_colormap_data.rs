use anyhow::Result;
use id_map_format::Wad;
use ultraviolet::Vec3;
use wgpu::BufferUsages;

use crate::renderer::helpers::gpu::{GpuStorageBuffer, GpuU8StorageBuffer};

pub struct PaletteColormapData {
    pub palette_storage_buf: GpuStorageBuffer<Vec3>,
    pub colormap_storage_buf: GpuU8StorageBuffer,
}

impl PaletteColormapData {
    pub fn new(device: &wgpu::Device, wad: &Wad) -> Result<Self> {
        let palette = wad.parse_palettes()?[0]
            .iter()
            .map(|(r, g, b)| Vec3::new(*r as f32, *g as f32, *b as f32))
            .collect::<Vec<Vec3>>();

        let colormap = wad.parse_colormaps()?;

        Ok(Self {
            palette_storage_buf: GpuStorageBuffer::new_vec(
                BufferUsages::STORAGE,
                device,
                palette,
                None,
            )?,
            colormap_storage_buf: GpuU8StorageBuffer::new_vec(
                BufferUsages::STORAGE,
                device,
                colormap,
                None,
            )?,
        })
    }
}
