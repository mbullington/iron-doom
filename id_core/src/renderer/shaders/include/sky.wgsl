#include "uniforms.wgsl"
#include "helpers.wgsl"

#define PI (3.14159)

// DOOM textures are weird in that they expect no vertical wrapping, as
// mouse-look originally didn't exist.
//
// There's not a "standard" solution ports do:
//
// - Most ports just stretch the texture vertically, and allow y axis
//   skybox movement.
//
// - Russian DOOM has "tall sky textures" for this purpose.
//   Reference:
//   https://github.com/Russian-Doom/russian-doom/issues/1
//
// - GZDoom uses a spherical mapping, and "fades" to a solid color at the top.
//
// For now, we **don't tile vertically**, and use a cylindrical mapping.
//
// This means the sky pretends we don't have mouselook, which can be strange,
// but preserves the original textures.
fn draw_sky(
    position: vec4f,
    world_pos: vec3f
) -> vec4f {
    let norm_x = position.x / ubo.camera_info.screen_size.x;
    let norm_y = position.y / ubo.camera_info.screen_size.y;
    let rotation_rad = ubo.camera_info.rotation_rad;

    // Cylinder mapping.
    // From the center, the edges should encompass a 85deg FOV.
    let fov = ubo.cvar_uniforms.r_fov;
    let cos_x = cos((-2.0 * norm_x - 1.0) * (PI * fov / 360)) * 0.5;

    let palette_image_index = u32(8);
    let dims = vec2f(get_image_width_height(palette_image_index));

    let world_u = i32(f32(dims.x) * (rotation_rad / PI + cos_x));
    let world_v = i32(f32(dims.y) * norm_y);

    let palette_index = sample_image(palette_image_index, world_u, world_v);
    let color = palette[palette_index] / 256.0;
    return vec4f(color, 1.0);
}