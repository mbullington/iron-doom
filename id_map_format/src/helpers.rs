use std::ffi::{CString, NulError};

use thiserror::Error;

macro_rules! i16_le {
    ($bytes:expr) => {
        i16::from_le_bytes($bytes.try_into().unwrap())
    };
}

macro_rules! u32_le {
    ($bytes:expr) => {
        u32::from_le_bytes($bytes.try_into().unwrap())
    };
}

macro_rules! u16_le {
    ($bytes:expr) => {
        u16::from_le_bytes($bytes.try_into().unwrap())
    };
}

/// Adds an extra byte to the end of the string.
///
/// This is so if a string is exactly 4 bytes long without a null terminator,
/// it will still be parsed correctly.
pub fn parse_bytes_cstr(bytes: &[u8]) -> Result<CString, NulError> {
    let mut str = Vec::new();
    for byte in bytes {
        if *byte == 0 {
            break;
        }
        str.push(*byte);
    }

    CString::new(str)
}

#[derive(Debug, Error)]
pub enum WadError {
    #[error("Invalid header")]
    InvalidHeader,
    #[error("Corrupted bytes")]
    CorruptedBytes,
    #[error("Corrupted string")]
    CorruptedString,

    #[error("Missing lump {0}")]
    MissingLump(String),
    #[error("Missing namespace {0}")]
    CorruptedLump(String),

    #[error("Not enough palettes in PLAYPAL lump.")]
    NotEnoughPalettes,
    #[error("Not enough colormaps in COLORMAP lump.")]
    NotEnoughColormaps,

    #[error("Requested map {0} not found.")]
    MapDoesNotExist(String),

    #[error("Texture by index not found.")]
    TexturePatchNotFound,
    #[error("Patch {0} not found.")]
    PatchDoesNotExist(String),
}
