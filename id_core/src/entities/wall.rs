use id_map_format::{Linedef, Map, Sidedef, Vertex};

use hecs::EntityBuilder;
use ultraviolet::Vec2;

use crate::{
    components::{
        CTexture, CTextureAnimated, CTexturePurpose, CTextureSky, CWall, CWallTwoSided, CWallType,
    },
    AnimationStateMap,
};

use super::{F_SKY1, SKY1};

pub fn init_wall_entities(world: &mut hecs::World, map: &Map, animations: &AnimationStateMap) {
    let mut parse_wall = |linedef: &Linedef,
                          vertices: &[Vertex],
                          sidedef: &Sidedef,
                          other: Option<&Sidedef>,
                          flip_vertices: bool| {
        let start_vert_index = match flip_vertices {
            true => linedef.end_vertex_idx as u32,
            false => linedef.start_vertex_idx as u32,
        };

        let end_vert_index = match flip_vertices {
            true => linedef.start_vertex_idx as u32,
            false => linedef.end_vertex_idx as u32,
        };

        let start_vert = Vec2::new(
            vertices[start_vert_index as usize].x as f32,
            vertices[start_vert_index as usize].y as f32,
        );

        let end_vert = Vec2::new(
            vertices[end_vert_index as usize].x as f32,
            vertices[end_vert_index as usize].y as f32,
        );

        let mut spawn = |wall_type: CWallType, texture: &str| {
            if texture == "-" {
                return;
            }

            let mut builder = EntityBuilder::new();
            builder.add(CWall {
                wall_type,
                start_vert,
                end_vert,
                flags: linedef.flags as u32,
                sector_index: sidedef.sector_idx as usize,
                x_offset: sidedef.x_offset,
                y_offset: sidedef.y_offset,
            });

            // If we're a two-sided wall.
            if let Some(other) = other {
                builder.add(CWallTwoSided {
                    back_sector_index: other.sector_idx as usize,
                });
            }

            // Add either texture, or sky.
            if texture == SKY1 {
                builder.add(CTextureSky {});
            } else {
                builder.add(CTexture {
                    purpose: CTexturePurpose::Texture,
                    texture_name: texture.to_string(),
                });
            }

            if animations.contains_key(CTexturePurpose::Texture, texture) {
                builder.add(CTextureAnimated {});
            }

            world.spawn(builder.build());
        };

        // Spawn the middle texture.
        spawn(CWallType::Middle, &sidedef.middle_texture);

        if let Some(other) = other {
            // Spawn the lower texture.
            // Lower/upper textures are only allowed to be two-sided.
            spawn(CWallType::Lower, &sidedef.lower_texture);

            // Spawn the upper texture.
            // First: modify the upper texture to incorporate the sky hack.

            let mut upper_texture = sidedef.upper_texture.as_str();

            let sidedef_sector = &map.sectors[sidedef.sector_idx as usize];
            let other_sector = &map.sectors[other.sector_idx as usize];

            // If the other sector has a sky ceiling, then we should render
            // the upper texture as the sky.
            if sidedef.upper_texture == "-"
                && other_sector.ceiling_height < sidedef_sector.ceiling_height
                && other_sector.ceiling_flat == F_SKY1
            {
                upper_texture = SKY1;
            }
            // https://doomwiki.org/wiki/Sky_hack
            if sidedef_sector.ceiling_flat == F_SKY1 && other_sector.ceiling_flat == F_SKY1 {
                upper_texture = SKY1;
            }

            spawn(CWallType::Upper, upper_texture);
        };
    };

    // Traverse through each linedef.
    for linedef in &map.linedefs {
        let left_sidedef_opt = linedef
            .left_sidedef_idx
            .map(|idx| &map.sidedefs[idx as usize]);

        let right_sidedef_opt = linedef
            .right_sidedef_idx
            .map(|idx| &map.sidedefs[idx as usize]);

        if let Some(left_sidedef) = left_sidedef_opt {
            parse_wall(
                linedef,
                &map.vertices,
                left_sidedef,
                right_sidedef_opt,
                true, // Flip vertices.
            );
        }

        if let Some(right_sidedef) = right_sidedef_opt {
            parse_wall(
                linedef,
                &map.vertices,
                right_sidedef,
                left_sidedef_opt,
                false, // Don't flip vertices.
            );
        }
    }
}
