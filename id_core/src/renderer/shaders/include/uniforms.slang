#ifndef UNIFORMS_H
#define UNIFORMS_H

struct CameraInfo {
    float4x4 view_proj_mat;
    float2 screen_size;
    float3 camera_pos;
};

struct CVarUniforms {
    uint r_fullbright;
    float r_lightfalloff;
};

struct UBO {
    CameraInfo camera_info;
    CVarUniforms cvar_uniforms;
};

struct WadSector {
    float floor_height;
    float ceiling_height;

    uint ceiling_palette_image_index;
    uint floor_palette_image_index;

    uint light_level;
};

struct WadWall {
    // 0 == upper, 1 == middle, 2 == lower
    uint wall_type;

    float2 start_vert;
    float2 end_vert;

    uint sector_index;
    uint back_sector_index;

    uint palette_image_index;

    float x_offset;
    float y_offset;
};

struct PatchHeaderStorageData {
    uint width;
    uint height;
    uint buffer_idx;
};

ConstantBuffer<UBO> ubo;

StructuredBuffer<float3> palette;
StructuredBuffer<uint> colormap;
StructuredBuffer<uint> images;

StructuredBuffer<WadSector> sectors;
StructuredBuffer<WadWall> walls;

#endif // UNIFORMS_H