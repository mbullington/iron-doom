#include "include/uniforms.wgsl"
#include "include/helpers.wgsl"
#include "include/sky.wgsl"

struct VsOutput {
    @builtin(position) position: vec4f,
    @location(0) world_pos: vec3f,
    @location(1) wall_idx: u32,

    @location(2) uv: vec2f,

    @location(3) width: u32,
    @location(4) height: u32,

    @location(5) x_offset: i32,
    @location(6) y_offset: i32,

    @location(7) light_offset: i32,
}

@vertex
fn vs_main(
    @location(0) coord: vec2f,
    @builtin(instance_index) instance_idx: u32,
) -> VsOutput {
    let wall = walls[instance_idx];
    let sector = sectors[wall.sector_idx];

    let start_vert = wall.start_vert;
    let end_vert = wall.end_vert;
    let vert_vec = end_vert - start_vert;

    var floor = sector.floor_height;
    var ceiling = sector.ceiling_height;

    var x_offset = wall.x_offset;
    var y_offset = wall.y_offset;

    let is_upper = wall.wall_type == WALL_TYPE_UPPER;
    let is_middle = wall.wall_type == WALL_TYPE_MIDDLE;
    let is_lower = wall.wall_type == WALL_TYPE_LOWER;

    let has_back_sector = wall.back_sector_idx != MAGIC_BACKSECTOR_INVALID;
    if has_back_sector {
        // This implements two-sided walls, with the option to be "unpegged" at
        // either the top or the bottom.
        //
        // Reference:
        // https://doomwiki.org/wiki/Texture_alignment
        let back_sector = sectors[wall.back_sector_idx];
        if is_middle {
            floor = max(sector.floor_height, back_sector.floor_height);
            ceiling = min(sector.ceiling_height, back_sector.ceiling_height);
        } else if is_upper {
            floor = back_sector.ceiling_height;
            ceiling = sector.ceiling_height;
        } else if is_lower {
            floor = sector.floor_height;
            ceiling = back_sector.floor_height;
        }

        if is_upper && FALSE(wall.flags & FLAGS_UPPER_UNPEGGED) {
            y_offset -= sector.ceiling_height - back_sector.ceiling_height;
        }
    }
    
    if (is_middle || is_lower) && TRUE(wall.flags & FLAGS_LOWER_UNPEGGED) {
        let height = u32(ceiling - floor);
        let dims = get_image_width_height(wall.palette_image_index);
        y_offset -= mod2i(i32(height), i32(dims.y));
    }

    let width = u32(ceil(sqrt(vert_vec.x * vert_vec.x + vert_vec.y * vert_vec.y)));
    let height = u32(ceiling - floor);

    let world_pos = vec3f(
        start_vert.x + vert_vec.x * coord.x,
        f32(floor) + f32(ceiling - floor) * coord.y,
        start_vert.y + vert_vec.y * coord.x
    );

    var position = ubo.camera_info.view_proj_mat * vec4f(world_pos, 1.0);

    // https://doomwiki.org/wiki/Fake_contrast
    var light_offset = i32(0);
    if abs(vert_vec.x) < 0.001 {
        light_offset = -2;
    } else if abs(vert_vec.y) < 0.001 {
        light_offset = 1;
    }

    return VsOutput(
        position,
        world_pos,
        instance_idx,
        coord, // uv
        width,
        height,
        x_offset,
        y_offset,
        light_offset
    );
}

@fragment
fn fs_main(
    @builtin(position) position: vec4f,
    @location(0) world_pos: vec3f,
    @location(1) wall_idx: u32,
    @location(2) uv: vec2f,
    @location(3) width: u32,
    @location(4) height: u32,
    @location(5) x_offset: i32,
    @location(6) y_offset: i32,
    @location(7) light_offset: i32,
) -> @location(0) vec4f {
    let wall = walls[wall_idx];
    let sector = sectors[wall.sector_idx];

    let has_back_sector = wall.back_sector_idx != MAGIC_BACKSECTOR_INVALID;

    // If the wall is a sky texture, we need to draw the sky instead.
    if wall.palette_image_index == MAGIC_OFFSET_SKY {
        return draw_sky(position, world_pos);
    }

    let depth = 0.1 / position.w + 16.0;

    // This is on the X/Z axis.
    var world_u = i32(uv.x * f32(width));
    var world_v = i32(f32(1.0 - uv.y) * f32(height));

    if wall.wall_type == WALL_TYPE_MIDDLE && has_back_sector {
        let dims = get_image_width_height(wall.palette_image_index);
        // Determine if to peg to lower or upper texture.
        if TRUE(wall.flags & FLAGS_LOWER_UNPEGGED) {
            if world_v < i32(height - dims.y) {
                discard;
            }
        } else {
            if world_v > i32(dims.y) {
                discard;
            }
        }
    }

    world_u += x_offset;
    world_v += y_offset;

    var palette_index = sample_image(wall.palette_image_index, world_u, world_v);
    let light_index = get_light_index(sector.light_level, light_offset, depth);

    palette_index = GET_U8(colormap, light_index * u32(256) + palette_index);
    let color = palette[palette_index] / 255.0;

    return vec4f(color, 1.0);
}