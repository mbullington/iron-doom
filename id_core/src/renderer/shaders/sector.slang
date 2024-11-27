#include "include/uniforms.slang"
#include "include/helpers.slang"
#include "include/sky.slang"

struct VSOutput {
    float4 position: SV_POSITION;
    float3 world_pos: TEXCOORD0;
    uint sector_idx: TEXCOORD1;
    uint is_ceiling: TEXCOORD2;
};

[shader("vertex")]
VSOutput vs_main(
    float2 coord: POSITION,
    uint sector_idx: TEXCOORD0,
    uint is_ceiling: SV_InstanceID
) {
    float height = 0.0f;
    if (is_ceiling > 0) {
        height = (float)sectors[sector_idx].ceiling_height;
    } else {
        height = (float)sectors[sector_idx].floor_height;
    }

    float3 world_pos = float3(coord.x, height, coord.y);
    float4 position = mul(float4(world_pos, 1.0), ubo.camera_info.view_proj_mat);
    
    VSOutput output;
    output.position = position;
    output.world_pos = world_pos;
    output.sector_idx = sector_idx;
    output.is_ceiling = is_ceiling;
    return output;
}

[shader("fragment")]
float4 ps_main(VSOutput input): SV_Target {
    let sector = sectors[input.sector_idx];

    // Check for sky texture.
    if (input.is_ceiling == 1 && sector.ceiling_palette_image_index == 8) {
        return draw_sky(input.position, input.world_pos);
    }
    if (input.is_ceiling == 0 && sector.floor_palette_image_index == 8) {
        return draw_sky(input.position, input.world_pos);
    }

    float depth = 0.1 / input.position.w + 16.0;

    // Calculate UV coordinates for 64x64 tile grid.
    float u = frac(input.world_pos.x / 64.0);
    if (u < 0.0) {
        u = 1.0 + u;
    }

    float v = frac(input.world_pos.z / 64.0);
    if (v < 0.0) {
        v = 1.0 + v;
    }

    float u_index = u * 64.0;
    float v_index = v * 64.0;

    uint palette_image_index = 0;
    if (input.is_ceiling > 0) {
        palette_image_index = sector.ceiling_palette_image_index;
    } else {
        palette_image_index = sector.floor_palette_image_index;
    }

    uint palette_index = sample_image(palette_image_index, u_index, v_index);
    uint light_index = get_light_index(sector.light_level, 0, depth);

    palette_index = GET_U8(colormap, light_index * 256 + palette_index);
    float3 color = palette[palette_index] / 255.0;

    return srgb_to_linear(float4(color, 1.0));
}