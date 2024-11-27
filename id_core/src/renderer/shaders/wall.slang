#include "include/uniforms.slang"
#include "include/helpers.slang"
#include "include/sky.slang"

struct VSOutput {
    float4 position: SV_POSITION;
    float3 world_pos: TEXCOORD0;
    float2 uv: TEXCOORD1;

    uint wall_id: TEXCOORD2;
    float width: TEXCOORD3;
    float height: TEXCOORD4;

    int light_offset: TEXCOORD5;
};

[shader("vertex")]
VSOutput vs_main(
    float2 coord : register(POSITION0),
    uint instance_idx : SV_InstanceID
) {
    WadWall wall = walls[instance_idx];
    WadSector sector = sectors[wall.sector_index];

    float2 start_vert = wall.start_vert;
    float2 end_vert = wall.end_vert;
    float2 vert_vec = end_vert - start_vert;

    bool is_upper = wall.wall_type == 0;
    bool is_lower = wall.wall_type == 2;

    float floor_height = sector.floor_height;
    float ceiling_height = sector.ceiling_height;

    if (!is_upper && !is_lower && wall.back_sector_index != 0xFFFFFFFF) {
        WadSector back_sector = sectors[wall.back_sector_index];
        floor_height = max(sector.floor_height, back_sector.floor_height);
        ceiling_height = min(sector.ceiling_height, back_sector.ceiling_height);
    }

    if (is_upper) {
        WadSector back_sector = sectors[wall.back_sector_index];
        floor_height = back_sector.ceiling_height;
        ceiling_height = sector.ceiling_height;
    } else if (is_lower) {
        WadSector back_sector = sectors[wall.back_sector_index];
        floor_height = sector.floor_height;
        ceiling_height = back_sector.floor_height;
    }

    float width = sqrt(vert_vec.x * vert_vec.x + vert_vec.y * vert_vec.y);
    float height = ceiling_height - floor_height;

    float3 world_pos = float3(
        start_vert.x + vert_vec.x * coord.x,
        floor_height + (ceiling_height - floor_height) * coord.y,
        start_vert.y + vert_vec.y * coord.x
    );

    float4 position = mul(float4(world_pos, 1.0), ubo.camera_info.view_proj_mat);
    int light_offset = 0;
    
    if (abs(vert_vec.x) < 0.001) {
        light_offset = -2;
    } else if (abs(vert_vec.y) < 0.001) {
        light_offset = 1;
    }

    VSOutput output;
    output.position = position;
    output.world_pos = world_pos;
    output.wall_id = instance_idx;
    output.uv = coord;
    output.width = width;
    output.height = height;
    output.light_offset = light_offset;
    
    return output;
}

[shader("fragment")]
float4 ps_main(VSOutput input) : SV_Target {
    let wall = walls[input.wall_id];
    if (wall.palette_image_index == 8) {
        return draw_sky(input.position, input.world_pos);
    }

    float depth = 0.1 / input.position.w + 16.0;

    float world_u = input.uv.x * input.width + wall.x_offset;
    float world_v = (1.0 - input.uv.y) * input.height + wall.y_offset;

    uint palette_index = sample_image(wall.palette_image_index, world_u, world_v);
    uint light_index = get_light_index(
        sectors[wall.sector_index].light_level,
        input.light_offset,
        depth
    );
    
    palette_index = GET_U8(colormap, light_index * 256 + palette_index);
    float3 color = palette[palette_index] / 256.0;

    return srgb_to_linear(float4(color, 1.0));
}