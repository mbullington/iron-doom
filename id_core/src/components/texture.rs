#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum CTexturePurpose {
    /// Used for flats (walls).
    Flat,
    /// Used for walls.
    Texture,
    // Used for things.
    // Sprite,
}

/// Asset has either a flat (floor/ceiling), texture (wall), or sprite (things).
///
/// We handle them all the same.
#[derive(Debug)]
pub struct CTexture {
    pub purpose: CTexturePurpose,
    pub texture_name: String,
}

pub struct CTextureFloor(pub CTexture);

/// Asset is either a flat (ceiling) or texture (wall).
///
/// Render the quad as the sky texture.
#[derive(Debug)]
pub struct CTextureSky {}

pub struct CTextureSkyFloor(pub CTextureSky);

#[derive(Debug)]
pub struct CTextureAnimated {}
