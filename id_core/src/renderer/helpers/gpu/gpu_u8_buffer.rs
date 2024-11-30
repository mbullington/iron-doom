use anyhow::Result;
use bytemuck::cast_slice;
use wgpu::{Buffer, BufferUsages};

use super::GpuBuffer;

/// GpuU8Buffer is a wrapper around [GpuBuffer] that takes in u8 data, compacting
/// them into a u32 buffer for use in WGSL.
///
/// This is because u32 is the smallest data type that can be used.
pub struct GpuU8Buffer {
    pub gpu_buffer: GpuBuffer<u32>,
}

pub type GpuU8StorageBuffer = GpuU8Buffer;

impl GpuU8Buffer {
    pub fn with_inner_buf<F: FnOnce(&Buffer)>(&mut self, f: F) {
        self.gpu_buffer.with_inner_buf(f)
    }

    // new & write are omitted as they don't make sense for GpuU8Buffer.

    pub fn new_vec(
        usage: BufferUsages,
        device: &wgpu::Device,
        mut data: Vec<u8>,
        label: Option<&'static str>,
    ) -> Result<Self> {
        // Make sure the data is padded correctly.
        let size = data.len();
        let padded_size = size + (4 - (size % 4));
        // Make sure "data" is at least this length. Fill with zeros.
        let mut padding = vec![0u8; padded_size - size];
        data.append(&mut padding);

        let u32_data = cast_slice::<u8, u32>(&data).to_vec();
        let gpu_buffer = GpuBuffer::new_vec(usage, device, u32_data, label)?;

        Ok(GpuU8Buffer { gpu_buffer })
    }

    pub fn write_vec(&mut self, queue: &wgpu::Queue, mut data: Vec<u8>) -> Result<()> {
        // Make sure the data is padded correctly.
        let size = data.len();
        let padded_size = size + (4 - (size % 4));
        // Make sure "data" is at least this length. Fill with zeros.
        let mut padding = vec![0u8; padded_size - size];
        data.append(&mut padding);

        let u32_data = cast_slice::<u8, u32>(&data).to_vec();
        self.gpu_buffer.write_vec(queue, u32_data)
    }

    pub fn bind_group_layout_entry(
        &self,
        binding: u32,
        visibility: wgpu::ShaderStages,
    ) -> wgpu::BindGroupLayoutEntry {
        self.gpu_buffer.bind_group_layout_entry(binding, visibility)
    }

    pub fn bind_group_descriptor_entry(&self, binding: u32) -> wgpu::BindGroupEntry {
        self.gpu_buffer.bind_group_descriptor_entry(binding)
    }
}
