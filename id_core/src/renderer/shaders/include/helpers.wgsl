#include "uniforms.wgsl"

fn mod2(x: f32, y: f32) -> f32 {
    var x_mut = x;
    
    if abs(x) < 0.0001 {
        return 0.;
    }
    if abs(y) < 0.0001 {
        return 0.;
    }

    while x_mut >= y {
        x_mut = x_mut - y;
    }
    while x_mut < 0.0001 {
        x_mut = y + x_mut;
    }
    return x_mut;
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

fn get_image_width_height(idx: u32) -> vec2f {
    let idx_dims = idx / u32(4);
    let width = images[idx_dims];
    let height = images[idx_dims + u32(1)];
    return vec2f(f32(width), f32(height));
}

fn sample_image(idx: u32, x: f32, y: f32) -> u32 {
    let idx_dims = idx / u32(4);
    let width = images[idx_dims];
    let height = images[idx_dims + u32(1)];

    if width == u32(0) || height == u32(0) {
        discard;
    }

    let world_u = u32(mod2(x, f32(width)));
    let world_v = u32(mod2(y, f32(height)));

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