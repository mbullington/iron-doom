use std::marker::PhantomData;

use anyhow::Result;
use encase::{
    internal::{WriteInto, Writer},
    ShaderSize, ShaderType,
};
use wgpu::{Buffer, BufferUsages};

use super::LenOrData;

/// GpuBuffer is a lightweight wrapper around [Buffer] that automatically
/// handles coersing byte alignment.
pub struct GpuBuffer<DataType: ShaderType + WriteInto + ShaderSize> {
    _phantom: PhantomData<DataType>,
    pub buf: Buffer,

    pub usage: BufferUsages,
    pub stride: usize,
    pub size: usize,
}

pub type GpuStorageBuffer<DataType: ShaderType + WriteInto + ShaderSize> = GpuBuffer<DataType>;
pub type GpuVertexBuffer<DataType: ShaderType + WriteInto + ShaderSize> = GpuBuffer<DataType>;
pub type GpuUniformBuffer<DataType: ShaderType + WriteInto + ShaderSize> = GpuBuffer<DataType>;
pub type GpuIndexBuffer = GpuBuffer<u32>;

impl<DataType> GpuBuffer<DataType>
where
    DataType: ShaderType + WriteInto + ShaderSize,
{
    pub fn with_inner_buf<F: FnOnce(&Buffer)>(&mut self, f: F) {
        f(&self.buf)
    }

    pub fn new(
        usage: BufferUsages,
        device: &wgpu::Device,
        len_or_data: LenOrData<DataType>,
        label: Option<&'static str>,
    ) -> Result<Self> {
        let usage = usage | BufferUsages::COPY_DST;
        let buf = _create_buf_encase(device, usage, len_or_data, label)?;

        let shader_size = DataType::SHADER_SIZE;
        let stride = shader_size.get()
            + DataType::METADATA
                .alignment()
                .padding_needed_for(shader_size.get());

        let stride = stride as usize;
        let size = buf.size() as usize;

        Ok(GpuBuffer {
            _phantom: PhantomData,
            buf,
            usage,
            stride,
            size,
        })
    }

    pub fn new_vec(
        usage: BufferUsages,
        device: &wgpu::Device,
        data: Vec<DataType>,
        label: Option<&'static str>,
    ) -> Result<Self> {
        let usage = usage | BufferUsages::COPY_DST;
        let buf = _create_buf_encase(device, usage, LenOrData::Data(&data), label)?;

        let shader_size = DataType::SHADER_SIZE;
        let stride = shader_size.get()
            + DataType::METADATA
                .alignment()
                .padding_needed_for(shader_size.get());

        let stride = stride as usize;
        let size = stride * data.len();

        Ok(GpuBuffer {
            _phantom: PhantomData,
            buf,
            usage,
            stride,
            size,
        })
    }

    pub fn write(&mut self, queue: &wgpu::Queue, data: DataType) -> anyhow::Result<()> {
        _write_buf_encase(&self.buf, queue, data, 0)
    }

    pub fn write_vec(&mut self, queue: &wgpu::Queue, data: &[DataType]) -> Result<()> {
        if data.len() > self.size / self.stride {
            return Err(anyhow::anyhow!(
                "Data length {} is greater than buffer size {}",
                data.len(),
                self.size
            ));
        }

        _write_buf_encase(&self.buf, queue, data, 0)
    }

    pub fn write_to_offset(
        &mut self,
        queue: &wgpu::Queue,
        data: DataType,
        offset: usize,
    ) -> Result<()> {
        if offset > self.size {
            return Err(anyhow::anyhow!(
                "Offset {} is greater than buffer size {}",
                offset,
                self.size
            ));
        }

        _write_buf_encase(&self.buf, queue, &data, offset as u64)
    }

    pub fn write_vec_to_index(
        &mut self,
        queue: &wgpu::Queue,
        data: &[DataType],
        index: usize,
    ) -> Result<()> {
        if data.len() + index > self.size / self.stride {
            return Err(anyhow::anyhow!(
                "Data length {} is greater than buffer size {}",
                data.len(),
                self.size
            ));
        }

        _write_buf_encase(&self.buf, queue, &data, (index * self.stride) as u64)
    }

    pub fn bind_group_layout_entry(
        &self,
        binding: u32,
        visibility: wgpu::ShaderStages,
    ) -> wgpu::BindGroupLayoutEntry {
        let usage = self.usage;

        wgpu::BindGroupLayoutEntry {
            binding,
            visibility,
            ty: wgpu::BindingType::Buffer {
                ty: {
                    if usage.contains(BufferUsages::UNIFORM) {
                        wgpu::BufferBindingType::Uniform {}
                    } else if usage.contains(BufferUsages::STORAGE) {
                        wgpu::BufferBindingType::Storage { read_only: true }
                    } else {
                        unreachable!()
                    }
                },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }
    }

    pub fn bind_group_descriptor_entry(&self, binding: u32) -> wgpu::BindGroupEntry {
        wgpu::BindGroupEntry {
            binding,
            resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                buffer: &self.buf,
                offset: 0,
                size: None,
            }),
        }
    }
}

fn _create_buf_encase<DataType: ShaderType + WriteInto>(
    device: &wgpu::Device,
    usage: wgpu::BufferUsages,
    len_or_data: LenOrData<DataType>,
    label: Option<&'static str>,
) -> Result<wgpu::Buffer> {
    match len_or_data {
        LenOrData::Len(len) => {
            let buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label,
                size: len,
                usage,
                mapped_at_creation: false,
            });

            Ok(buffer)
        }
        LenOrData::Data(data) => {
            let mut inner = Vec::new();
            let mut writer = Writer::new(&data, &mut inner, 0)?;
            data.write_into(&mut writer);

            let mut size = inner.len() as u64;
            // Make sure size is a multiple of COPY_BUFFER_ALIGNMENT.
            if size % wgpu::COPY_BUFFER_ALIGNMENT != 0 {
                size += wgpu::COPY_BUFFER_ALIGNMENT - (size % wgpu::COPY_BUFFER_ALIGNMENT);
            }

            let buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label,
                size,
                usage,
                mapped_at_creation: true,
            });

            buffer.slice(..).get_mapped_range_mut()[..size as usize].copy_from_slice(&inner);
            buffer.unmap();

            Ok(buffer)
        }
    }
}

fn _write_buf_encase<DataType: ShaderType + WriteInto>(
    buffer: &wgpu::Buffer,
    queue: &wgpu::Queue,
    data: DataType,
    offset: u64,
) -> Result<()> {
    let mut view = queue
        .write_buffer_with(buffer, offset, data.size())
        .ok_or(anyhow::anyhow!(
            "Failed to write buffer with offset {} and size {}",
            offset,
            data.size()
        ))?;

    let mut writer = Writer::new(&data, view.as_mut(), 0)?;
    data.write_into(&mut writer);

    Ok(())
}
