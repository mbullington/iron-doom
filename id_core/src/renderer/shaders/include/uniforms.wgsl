#define GET_U8(array, u8_index) \
    ((array[(u8_index) / u32(4)] >> (((u8_index) % u32(4)) * u32(8))) & u32(0xFF))

#define TRUE(u32_val) \
    ((u32_val) > u32(0))

#define FALSE(u32_val) \
    ((u32_val) == u32(0))

struct CameraInfo {
    view_proj_mat: mat4x4<f32>,
    screen_size: vec2<f32>,
    camera_pos: vec3<f32>,
}

struct CVarUniforms {
    // WGSL doesn't support boolean types, so we use a u32 instead.
    r_fullbright: u32,
    r_lightfalloff: f32,
}

struct UBO {
    camera_info: CameraInfo,
    cvar_uniforms: CVarUniforms,
}

struct WadSector {
    floor_height: f32,
    ceiling_height: f32,
    // Indexes in the flat array.
    ceiling_palette_image_index: u32,
    floor_palette_image_index: u32,
    // Light level.
    light_level: u32
}

const WALL_TYPE_UPPER = u32(0);
const WALL_TYPE_MIDDLE = u32(1);
const WALL_TYPE_LOWER = u32(2);

const MAGIC_BACKSECTOR_INVALID = 0xFFFFFFFFu;

const MAGIC_OFFSET_SKY = u32(8);
const MAGIC_OFFSET_INVALID = u32(0);

const FLAGS_TWOSIDED = u32(4);
const FLAGS_UPPER_UNPEGGED = u32(8);
const FLAGS_LOWER_UNPEGGED = u32(16);

struct WadWall {
    // 0 == upper, 1 == middle, 2 == lower
    wall_type: u32,

    start_vert: vec2f,
    end_vert: vec2f,

    flags: u32,

    sector_idx: u32,
    back_sector_idx: u32,

    palette_image_index: u32,
    
    x_offset: f32,
    y_offset: f32
}

struct PatchHeaderStorageData {
    width: u32,
    height: u32,
    buffer_idx: u32,
}

@binding(0) @group(0) var<uniform> ubo : UBO;

@binding(1) @group(0) var<storage> palette : array<vec3f>;
@binding(2) @group(0) var<storage> colormap : array<u32>; // u8
@binding(3) @group(0) var<storage> images : array<u32>; // u8 that are u32 aligned

@binding(4) @group(0) var<storage> sectors : array<WadSector>;
@binding(5) @group(0) var<storage> walls: array<WadWall>;
