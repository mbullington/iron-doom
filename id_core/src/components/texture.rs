/// We handle textures as part of a unified system.
///
/// Sources can be:
/// - flat (floor/ceiling)
/// - texture (wall)
/// - sprite (things)
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum CTexture {
    Sky,
    /// Used for sectors (floor/ceiling).
    Flat(String),
    /// Used for walls.
    Texture(String),
    // Used for things.
    Sprite(String),
}

/// [CTextureAnimated] is a hint to our ECS system that the texture
/// should be animated on frame update.
///
/// Normally, an entity would add this after consulting [AnimationStateMap].
#[derive(Debug)]
pub struct CTextureAnimated {}

/// [CTextureOrdinal] is a hint to our ECS system that the texture
/// should be changed based on player's ordinal direction.
///
/// Normally, an entity would add this after consulting [AnimationStateMap].
///
/// This only has an effect for sprites.
#[derive(Debug)]
pub struct CTextureOrdinal {}

/// We cannot have two [CTexture] in the same entity, however
/// this is necesary for sectors which have a floor and ceiling.
///
/// [CTextureFloor] is thus used for the floor, and [CTexture] for the ceiling.
pub struct CTextureFloor(pub CTexture);
