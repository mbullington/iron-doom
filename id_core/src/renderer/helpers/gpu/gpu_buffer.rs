use std::{cell::RefCell, marker::PhantomData};

use anyhow::Result;
use encase::{internal::WriteInto, ShaderSize, ShaderType};
use internal::{create_buf_encase, write_buf_encase};
use wgpu::{Buffer, BufferUsages};

use super::LenOrData;

/// GpuBuffer is a lightweight wrapper around [Buffer] that automatically
/// handles coersing byte alignment.
pub struct GpuBuffer<DataType: ShaderType + WriteInto + ShaderSize> {
    _phantom: PhantomData<DataType>,
    pub buf: Buffer,
    /// Encase needs its own staging buffer, but it immediately gets passed off
    /// to WGPU's staging buffer.
    ///
    /// So we do this battle where we initialize this once, and reuse it with
    /// every write.
    vec: RefCell<Vec<u8>>,

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
        let (buf, vec) = create_buf_encase(device, usage, len_or_data, label)?;

        let stride = vec.len();
        let size = vec.len();

        Ok(GpuBuffer {
            _phantom: PhantomData,
            buf,
            vec: RefCell::new(vec),
            usage,
            stride,
            size,
        })
    }

    pub fn write(&mut self, queue: &wgpu::Queue, data: DataType) -> anyhow::Result<()> {
        write_buf_encase(&mut self.vec, &self.buf, queue, data)
    }

    pub fn new_vec(
        usage: BufferUsages,
        device: &wgpu::Device,
        data: Vec<DataType>,
        label: Option<&'static str>,
    ) -> Result<Self> {
        let usage = usage | BufferUsages::COPY_DST;
        let (buf, vec) = create_buf_encase(device, usage, LenOrData::Data(&data), label)?;

        let stride = vec.len().checked_div(data.len()).unwrap_or(0);
        let size = vec.len();

        Ok(GpuBuffer {
            _phantom: PhantomData,
            buf,
            vec: RefCell::new(vec),
            usage,
            stride,
            size,
        })
    }

    pub fn write_vec(&mut self, queue: &wgpu::Queue, data: Vec<DataType>) -> Result<()> {
        write_buf_encase(&mut self.vec, &self.buf, queue, &data)
    }

    /// This is not unsafe per se, but this should be used sparingly to avoid
    /// UB in the shader code itself.
    pub unsafe fn write_to_offset(
        &mut self,
        queue: &wgpu::Queue,
        data: &[u8],
        offset: u64,
    ) -> Result<()> {
        self.with_inner_buf(|buf| {
            queue.write_buffer(buf, offset, data);
        });

        Ok(())
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

mod internal {
    use std::cell::RefCell;

    use anyhow::Result;
    use encase::{
        internal::{WriteInto, Writer},
        ShaderType,
    };

    use crate::renderer::helpers::gpu::LenOrData;

    pub enum LenOrMappedData {
        Len(u64),
        MappedData(Vec<u8>),
    }

    pub fn create_buf_encase<DataType: ShaderType + WriteInto>(
        device: &wgpu::Device,
        usage: wgpu::BufferUsages,
        len_or_data: LenOrData<DataType>,
        label: Option<&'static str>,
    ) -> Result<(wgpu::Buffer, Vec<u8>)> {
        let len_or_mapped_data = match len_or_data {
            LenOrData::Len(len) => LenOrMappedData::Len(len),
            LenOrData::Data(data) => {
                let mut inner = Vec::new();
                let mut writer = Writer::new(&data, &mut inner, 0)?;
                data.write_into(&mut writer);

                LenOrMappedData::MappedData(inner)
            }
        };

        create_buf(device, usage, len_or_mapped_data, label)
    }

    pub fn write_buf_encase<DataType: ShaderType + WriteInto>(
        vec: &mut RefCell<Vec<u8>>,
        buffer: &wgpu::Buffer,
        queue: &wgpu::Queue,
        data: DataType,
    ) -> Result<()> {
        {
            let mut borrowed = vec.borrow_mut();
            let mut writer: Writer<&mut Vec<u8>> = Writer::new(&data, borrowed.as_mut(), 0)?;
            data.write_into(&mut writer);
        }

        queue.write_buffer(buffer, 0, &vec.borrow());
        Ok(())
    }

    pub fn create_buf(
        device: &wgpu::Device,
        usage: wgpu::BufferUsages,
        len_or_mapped_data: LenOrMappedData,
        label: Option<&'static str>,
    ) -> Result<(wgpu::Buffer, Vec<u8>)> {
        match len_or_mapped_data {
            LenOrMappedData::Len(len) => {
                let buffer = device.create_buffer(&wgpu::BufferDescriptor {
                    label,
                    size: len,
                    usage,
                    mapped_at_creation: false,
                });

                Ok((buffer, Vec::with_capacity(len as usize)))
            }

            LenOrMappedData::MappedData(data) => {
                let mut size = data.len() as u64;
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

                buffer.slice(..).get_mapped_range_mut()[..size as usize].copy_from_slice(&data);
                buffer.unmap();

                Ok((buffer, data))
            }
        }
    }
}
