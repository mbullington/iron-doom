#include "uniforms.wgsl"
#include "image.wgsl"
#include "utils.wgsl"

fn draw_sky(
    position: vec4f,
    world_pos: vec3f
) -> vec4f {
    let x = position.x;
    let y = position.y;
    let delta = world_pos - ubo.camera_info.camera_pos;

    let palette_image_index = u32(8);
    let dims = sample_image_width_height(palette_image_index);

    // Because we allow for mouse look, we need to fix delta.x and delta.z somehow.    

    let norm_angle = mod2((atan2(delta.x, delta.z) + 3.14159) / (1.0 * 3.14159), 1.0);
    let world_u = f32(dims.x * norm_angle);
    let world_v = f32(dims.y * position.y / ubo.camera_info.screen_size.y);

    let palette_index = sample_image(palette_image_index, world_u, world_v);
    let color = palette[palette_index] / 256.0;
    return srgb_to_linear(vec4f(color, 1.0));
}