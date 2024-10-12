#include "include/uniforms.wgsl"
#include "include/utils.wgsl"
#include "include/sky.wgsl"

struct VsOutput {
    @builtin(position) position: vec4f,
    @location(0) world_pos: vec3f,
    @location(1) sector_idx: u32,
    @location(2) is_ceiling: u32
}

@vertex
fn vs_main(
    @location(0) coord: vec2f,
    @location(1) sector_idx: u32,
    @builtin(instance_index) is_ceiling: u32,
) -> VsOutput {
    var height: f32 = 0.0;
    if is_ceiling > u32(0) {
        height = f32(sectors[sector_idx].ceiling_height);
    } else {
        height = f32(sectors[sector_idx].floor_height);
    }

    let world_pos = vec3f(coord.x, height, coord.y);
    var position = ubo.camera_info.view_proj_mat * vec4f(world_pos, 1.0);
    return VsOutput(position, world_pos, sector_idx, is_ceiling);
}

@fragment
fn fs_main(
    @builtin(position) position: vec4f,
    @location(0) world_pos: vec3f,
    @location(1) sector_idx: u32,
    @location(2) is_ceiling: u32,
) -> @location(0) vec4f {
    // If it's the second patch index, it's the sky.
    if is_ceiling == u32(1) && sectors[sector_idx].ceiling_flat_index == u32(1) {
        return draw_sky(position, world_pos);
    }

    let depth = 0.1 / position.w + 16.0;

    // Turn world_pos into a UV coordinate for a 64-by-64 tile grid on the ground.
    var u: f32 = modf(world_pos.x / 64.0).fract;
    if u < 0.0 {
        u = 1.0 + u;
    }

    var v: f32 = modf(world_pos.z / 64.0).fract;
    if v < 0.0 {
        v = 1.0 + v;
    }

    var u_index = u32(u * 64.0);
    var v_index = u32(v * 64.0);

    var flat_index = u32(0);
    if is_ceiling > u32(0) {
        flat_index = sectors[sector_idx].ceiling_flat_index;
    } else {
        flat_index = sectors[sector_idx].floor_flat_index;
    }

    var palette_index = GET_U8(flats, flat_index * u32(4096) + u_index * u32(64) + v_index);
    var light_index = u32(0);

    if ubo.cvar_uniforms.r_fullbright != u32(1) {
        // Adjust for light level.
        light_index = u32(31) - (sectors[sector_idx].light_level >> u32(3));

        // Adjust for depth. From eyeballing screenshots, it reduces by 1 every 8 units.
        light_index = min(light_index + u32(depth / ubo.cvar_uniforms.r_lightfalloff), u32(31));
    }

    palette_index = GET_U8(colormap, light_index * u32(256) + palette_index);
    let color = palette[palette_index] / 255.0;

    return srgb_to_linear(vec4f(color, 1.0));
}