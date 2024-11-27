#include "include/uniforms.wgsl"
#include "include/utils.wgsl"
#include "include/image.wgsl"
#include "include/sky.wgsl"

struct VsOutput {
    @builtin(position) position: vec4f,
    @location(0) world_pos: vec3f,
    @location(1) palette_image_index: u32,
    @location(2) sector_idx: u32,
    @location(3) uv: vec2f,
    @location(4) width: f32,
    @location(5) height: f32,
    @location(6) x_offset: f32,
    @location(7) y_offset: f32,
    @location(8) light_offset: i32,
}

@vertex
fn vs_main(
    @location(0) coord: vec2f,
    @builtin(instance_index) instance_idx: u32,
) -> VsOutput {
    let wall = walls[instance_idx];
    let sector = sectors[wall.sector_index];

    let start_vert = wall.start_vert;
    let end_vert = wall.end_vert;
    let vert_vec = end_vert - start_vert;

    // coord goes from 0:0 to 1:1 on the x/y axis.

    let is_upper = wall.wall_type == u32(0);
    let is_lower = wall.wall_type == u32(2);

    var floor = sector.floor_height;
    var ceiling = sector.ceiling_height;

    if !is_upper && !is_lower && wall.back_sector_index != 0xFFFFFFFFu {
        let back_sector = sectors[wall.back_sector_index];
        floor = max(sector.floor_height, back_sector.floor_height);
        ceiling = min(sector.ceiling_height, back_sector.ceiling_height);
    }

    if is_upper {
        let back_sector = sectors[wall.back_sector_index];
        floor = back_sector.ceiling_height;
        ceiling = sector.ceiling_height;
    } else if is_lower {
        let back_sector = sectors[wall.back_sector_index];
        floor = sector.floor_height;
        ceiling = back_sector.floor_height;
    }

    let width = f32(sqrt(vert_vec.x * vert_vec.x + vert_vec.y * vert_vec.y));
    let height = ceiling - floor;

    let world_pos = vec3f(
        start_vert.x + vert_vec.x * coord.x,
        f32(floor + (ceiling - floor) * coord.y),
        start_vert.y + vert_vec.y * coord.x
    );

    var position = ubo.camera_info.view_proj_mat * vec4f(world_pos, 1.0);
    var light_offset = i32(0);
    if abs(vert_vec.x) < 0.001 {
        light_offset = -2;
    } else if abs(vert_vec.y) < 0.001 {
        light_offset = 1;
    }

    return VsOutput(
        position,
        world_pos,
        wall.palette_image_index,
        wall.sector_index,
        coord, // uv
        width,
        height,
        wall.x_offset,
        wall.y_offset,
        light_offset
    );
}

@fragment
fn fs_main(
    @builtin(position) position: vec4f,
    @location(0) world_pos: vec3f,
    @location(1) palette_image_index: u32,
    @location(2) sector_idx: u32,
    @location(3) uv: vec2f,
    @location(4) width: f32,
    @location(5) height: f32,
    @location(6) x_offset: f32,
    @location(7) y_offset: f32,
    @location(8) light_offset: i32,
) -> @location(0) vec4f {
    if palette_image_index == u32(8) {
        return draw_sky(position, world_pos);
    }

    let depth = 0.1 / position.w + 16.0;

    // This is on the X/Z axis.
    var world_u = f32(uv.x) * width + x_offset;
    var world_v = f32(1.0 - uv.y) * height + y_offset;

    var palette_index = sample_image(palette_image_index, world_u, world_v);
    var light_index = u32(0);

    if ubo.cvar_uniforms.r_fullbright != u32(1) {
        // Adjust for light level.
        light_index = u32(31) - (sectors[sector_idx].light_level >> u32(3));

        // Adjust for depth. From eyeballing screenshots, it reduces by 1 every 8 units.
        light_index = min(light_index + u32(depth / ubo.cvar_uniforms.r_lightfalloff), u32(31));
    }

    // Adjust for light offset.
    light_index = u32(clamp(i32(light_index) + light_offset, i32(0), i32(31)));

    palette_index = GET_U8(colormap, light_index * u32(256) + palette_index);
    let color = palette[palette_index] / 255.0;

    return srgb_to_linear(vec4f(color, 1.0));
}