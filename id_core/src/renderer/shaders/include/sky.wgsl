#include "uniforms.wgsl"
#include "utils.wgsl"

fn draw_sky(
    position: vec4f,
    world_pos: vec3f
) -> vec4f {
    let x = position.x;
    let y = position.y;
    let delta = world_pos - ubo.camera_info.camera_pos;

    let patch_header = patch_header_storage_data[0];

    // Because we allow for mouse look, we need to fix delta.x and delta.z somehow.    

    let norm_angle = mod2((atan2(delta.x, delta.z) + 3.14159) / (1.0 * 3.14159), 1.0);
    let world_u = u32(f32(patch_header.width) * norm_angle);
    let world_v = u32(f32(patch_header.height) * position.y / ubo.camera_info.screen_size.y);

    let idx = patch_header.buffer_idx + u32(2) * (world_u * patch_header.height + world_v);

    let is_transparent = GET_U8(patches, idx);
    if is_transparent == u32(0) {
        return vec4f(0.0, 0.0, 0.0, 0.0);
    }

    let palette_index = GET_U8(patches, idx + u32(1));
    let color = palette[palette_index] / 256.0;
    return srgb_to_linear(vec4f(color, 1.0));
}