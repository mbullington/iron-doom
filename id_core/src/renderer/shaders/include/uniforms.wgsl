#define GET_U8(array, u8_index) \
    ((array[(u8_index) / u32(4)] >> (((u8_index) % u32(4)) * u32(8))) & u32(0xFF))

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
    floor_flat_index: u32,
    ceiling_flat_index: u32,
    // Light level.
    light_level: u32
}

struct WallStorageData {
    // 0 == middle, 1 == upper, 2 == lower
    wall_type: u32,

    start_vert_index: u32,
    end_vert_index: u32,

    sector_index: u32,
    back_sector_index: u32,
    patch_index: u32,
    
    x_offset: f32,
    y_offset: f32
}

struct PatchHeaderStorageData {
    width: u32,
    height: u32,
    buffer_idx: u32,
}

@binding(0) @group(0) var<uniform> ubo : UBO;
@binding(1) @group(0) var<storage> sectors : array<WadSector>;
@binding(2) @group(0) var<storage> flats : array<u32>; // u8
@binding(3) @group(0) var<storage> vertices: array<vec2f>;
@binding(4) @group(0) var<storage> wall_storage_data: array<WallStorageData>;
@binding(5) @group(0) var<storage> patch_header_storage_data: array<PatchHeaderStorageData>;
@binding(6) @group(0) var<storage> patches : array<u32>; // u8
@binding(7) @group(0) var<storage> palette : array<vec3f>;
@binding(8) @group(0) var<storage> colormap : array<u32>; // u8

