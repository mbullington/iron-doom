mod gpu_buffer;
mod gpu_frame_texture;
mod gpu_u8_buffer;

pub use gpu_buffer::*;
pub use gpu_frame_texture::*;
pub use gpu_u8_buffer::*;

#[derive(Debug)]
pub enum LenOrData<DataType> {
    Len(u64),
    Data(DataType),
}
