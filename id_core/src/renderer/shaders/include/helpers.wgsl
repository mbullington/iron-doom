#include "uniforms.wgsl"


fn mod2i(a: i32, b: i32) -> i32 {
    var m = a % b;
    if (m < 0) {
        if (b < 0) {
            m -= b;
        } else {
            m += b;
        }
    }
    return m;
}

fn srgb_to_linear(
    coord: vec4f
) -> vec4f {
    let linear = vec4f(
        pow(coord.r, 2.2),
        pow(coord.g, 2.2),
        pow(coord.b, 2.2),
        coord.a
    );
    return linear;
}

fn get_image_width_height(idx: u32) -> vec2u {
    let idx_dims = idx / u32(4);
    let width = images[idx_dims];
    let height = images[idx_dims + u32(1)];
    return vec2u(width, height);
}

fn sample_image(idx: u32, x: i32, y: i32) -> u32 {
    let idx_dims = idx / u32(4);
    let width = images[idx_dims];
    let height = images[idx_dims + u32(1)];

    if width == u32(0) || height == u32(0) {
        discard;
    }

    let world_u = u32(mod2i(x, i32(width)));
    let world_v = u32(mod2i(y, i32(height)));

    let idx_u8 = idx + u32(8) + u32(2) * (world_u * height + world_v);

    var is_transparent = GET_U8(images, idx_u8);
    if is_transparent == u32(0) {
        discard;
    }

    var palette_index = GET_U8(images, idx_u8 + u32(1));
    return palette_index;
}

fn get_light_index(light_level: u32, light_offset: i32, depth: f32) -> u32 {
    var light_index = u32(0);
    if (ubo.cvar_uniforms.r_fullbright != 1) {
        light_index = u32(31) - (light_level >> 3);
        light_index = max(min(light_index + u32(depth / ubo.cvar_uniforms.r_lightfalloff), u32(31)), min(u32(6), light_index));
    }

    light_index = u32(clamp(i32(light_index) + light_offset, i32(0), i32(31)));
    return light_index;
}