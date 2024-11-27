#pragma once

#include "uniforms.slang"

#define GET_U8(array, u8_index) \
    ((array[(u8_index) / 4] >> (((u8_index) % 4) * 8)) & 0xFF)

float2 get_image_width_height(uint idx) {
    let idx_dims = idx / 4;
    let width = images[idx_dims];
    let height = images[idx_dims + 1];
    return float2(float(width), float(height));
}

uint sample_image(uint idx, float x, float y) {
    let idx_dims = idx / 4;
    let width = images[idx_dims];
    let height = images[idx_dims + 1];

    if (width == 0 || height == 0) {
        discard;
    }

    let world_u = uint(mod2(x, float(width)));
    let world_v = uint(mod2(y, float(height)));

    let idx_u8 = idx + /* magic number for width/height */ 8 + 2 * (world_u * height + world_v);

    var is_transparent = GET_U8(images, idx_u8);
    if (is_transparent == 0) {
        discard;
    }

    var palette_index = GET_U8(images, idx_u8 + 1);
    return palette_index;
}

uint get_light_index(uint light_level, int light_offset, float depth) {
    uint light_index = 0;
    if (ubo.cvar_uniforms.r_fullbright != 1) {
        light_index = 31 - (light_level >> 3);
        light_index = max(min(light_index + uint(depth / ubo.cvar_uniforms.r_lightfalloff), 31), min(6, light_index));
    }

    // This can be used to achieve DOOM's fake contrast:
    // https://doomwiki.org/wiki/Fake_contrast
    light_index = uint(clamp(int(light_index) + light_offset, 0, 31));
    return light_index;
}

// Helper function to calculate modulo
float mod2(float x, float y)
{
    // Handle near-zero case
    if (abs(x) < 0.0001f)
    {
        return 0.0f;
    }
    
    // Store mutable value
    float result = x;
    
    // Subtract y until result is less than y
    while (result >= y)
    {
        result = result - y;
    }
    
    // Add y until result is non-negative
    while (result < 0.0f)
    {
        result = y + result;
    }
    
    return result;
}

// Convert sRGB to linear color space
float4 srgb_to_linear(float4 coord)
{
    // Create linear color using gamma correction
    float4 linear = float4(
        pow(coord.r, 2.2f),
        pow(coord.g, 2.2f),
        pow(coord.b, 2.2f),
        coord.a
    );
    
    return linear;
}
