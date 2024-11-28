#include "include/uniforms.wgsl"
#include "include/helpers.wgsl"
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
    // We render the sector as a single mesh, instanced twice.
    // The first time is for the floor, and the second time is for the ceiling.
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
    let sector = sectors[sector_idx];

    // If the wall is a sky texture, we need to draw the sky instead.
    if TRUE(is_ceiling) && sector.ceiling_palette_image_index == MAGIC_OFFSET_SKY {
        return draw_sky(position, world_pos);
    }
    if FALSE(is_ceiling) && sector.floor_palette_image_index == MAGIC_OFFSET_SKY {
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

    var u_index = i32(u * 64.0);
    var v_index = i32(64.0 - v * 64.0);

    var palette_image_index = u32(0);
    if TRUE(is_ceiling) {
        palette_image_index = sector.ceiling_palette_image_index;
    } else {
        palette_image_index = sector.floor_palette_image_index;
    }

    var palette_index = sample_image(palette_image_index, u_index, v_index);
    var light_index = get_light_index(sector.light_level, i32(0), depth);

    palette_index = GET_U8(colormap, light_index * u32(256) + palette_index);
    var color = palette[palette_index] / 255.0;

    return srgb_to_linear(vec4f(color, 1.0));
}