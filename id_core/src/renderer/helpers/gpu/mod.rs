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

enum _LenOrMappedData {
    Len(u64),
    /// Second value is the "stride" of the data, if an array.
    ///
    /// Otherwise, it is the same as the length.
    MappedData(Vec<u8>, u64),
}
