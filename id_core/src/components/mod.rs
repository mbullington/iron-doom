mod sector;
mod wall;

pub use sector::*;
pub use wall::*;

#[derive(Debug)]
pub enum CTexturePurpose {
    Flat,
    Texture,
    Sprite,
}

/// Asset has either a flat (floor/ceiling), texture (wall), or sprite (things).
///
/// We handle them all the same.
#[derive(Debug)]
pub struct CTexture {
    pub purpose: CTexturePurpose,
    pub texture_name: String,
    /// If the entity is a floor/ceiling, this flag determines if it is a ceiling.
    ///
    /// Otherwise, it has no effect.
    pub is_ceiling: bool,
}

/// Asset is either a flat (ceiling) or texture (wall).
///
/// Render the quad as the sky texture.
#[derive(Debug)]
pub struct CTextureSky {
    /// If the entity is a floor/ceiling, this flag determines if it is a ceiling.
    ///
    /// Otherwise, it has no effect.
    pub is_ceiling: bool,
}
