#pragma once

#include "uniforms.slang"
#include "helpers.slang"

#define PI (3.14159)

float4 draw_sky(float4 position, float3 world_pos) {
    let delta = world_pos - ubo.camera_info.camera_pos;

    uint palette_image_index = 8;
    let dims = get_image_width_height(palette_image_index);

    // Because we allow for mouse look,
    // we need to fix delta.x and delta.z somehow.    

    let norm_angle = mod2((atan2(delta.x, delta.z) + PI) / PI, 1.0);
    float world_u = dims.x * norm_angle;
    float world_v = dims.y * position.y / ubo.camera_info.screen_size.y;

    let palette_index = sample_image(palette_image_index, world_u, world_v);
    let color = palette[palette_index] / 256.0;
    return srgb_to_linear(float4(color, 1.0));
}
