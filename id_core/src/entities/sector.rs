use rayon::prelude::*;

use std::collections::HashMap;

use id_map_format::{Linedef, Map};

use hecs::{CommandBuffer, EntityBuilder};
use multimap::MultiMap;
use ultraviolet::Vec2;

use crate::{
    components::{CSector, CTexture, CTextureAnimated, CTextureFloor},
    helpers::{
        geom::{Graph2d, PolygonShape2d, Triangles2d},
        ChangedField,
    },
    AnimationStateMap,
};

use super::F_SKY1;

pub fn init_sector_entities(world: &mut hecs::World, map: &Map, animations: &AnimationStateMap) {
    let mut sidedefs_by_sector: MultiMap<usize, usize> = MultiMap::new();
    for (i, sidedef) in map.sidedefs.iter().enumerate() {
        sidedefs_by_sector.insert(sidedef.sector_idx as usize, i);
    }

    let mut linedefs_by_sidedef: MultiMap<usize, &Linedef> = MultiMap::new();
    for linedef in map.linedefs.iter() {
        if let Some(sidedef_idx) = linedef.left_sidedef_idx {
            linedefs_by_sidedef.insert(sidedef_idx as usize, linedef);
        }
        if let Some(sidedef_idx) = linedef.right_sidedef_idx {
            linedefs_by_sidedef.insert(sidedef_idx as usize, linedef);
        }
    }

    // Populate sectors.
    map.sectors
        .par_iter()
        .enumerate()
        .map(|(i, sector)| {
            let mut vertices: Vec<Vec2> = Vec::new();
            let mut edges: Vec<(usize, usize)> = Vec::new();

            let mut vert_wad_mapping: HashMap<id_map_format::Vertex, usize> = HashMap::new();

            let sidedefs = match sidedefs_by_sector.get_vec(&i) {
                Some(sidedefs) => sidedefs,
                None => return None,
            };

            for sidedef_idx in sidedefs {
                let linedefs = match linedefs_by_sidedef.get_vec(sidedef_idx) {
                    Some(linedefs) => linedefs,
                    None => return None,
                };

                for linedef in linedefs {
                    let start_vertex = map.vertices[linedef.start_vertex_idx as usize];
                    let start_vertex_idx = match vert_wad_mapping.get(&start_vertex) {
                        Some(idx) => *idx,
                        None => {
                            let idx = vertices.len();
                            vertices.push(Vec2::new(start_vertex.x as f32, start_vertex.y as f32));
                            vert_wad_mapping.insert(start_vertex, idx);
                            idx
                        }
                    };

                    let end_vertex = map.vertices[linedef.end_vertex_idx as usize];
                    let end_vertex_idx = match vert_wad_mapping.get(&end_vertex) {
                        Some(idx) => *idx,
                        None => {
                            let idx = vertices.len();
                            vertices.push(Vec2::new(end_vertex.x as f32, end_vertex.y as f32));
                            vert_wad_mapping.insert(end_vertex, idx);
                            idx
                        }
                    };

                    edges.push((start_vertex_idx, end_vertex_idx));
                }
            }

            let polygons = Graph2d::new(vertices, edges).detect_polygons();
            let polygon_shapes = PolygonShape2d::from_polygons(&polygons);

            let mut triangles: Vec<Triangles2d> = Vec::new();
            for shape in polygon_shapes {
                let (vertices, indices) = match shape.tessellate() {
                    Ok((vertices, indices)) => (vertices, indices),
                    Err(e) => panic!("{}", e.to_string()),
                };

                triangles.push(Triangles2d {
                    points: vertices,
                    indices,
                });
            }

            let mut builder = EntityBuilder::new();
            builder.add(CSector {
                triangles: ChangedField::new(triangles),
                sector_index: i,
                floor_height: sector.floor_height,
                ceiling_height: sector.ceiling_height,
                light_level: sector.light_level,
                special_type: sector.special_type,
                sector_tag: sector.sector_tag,
            });

            if sector.ceiling_flat != "-" {
                if sector.ceiling_flat == F_SKY1 {
                    builder.add(CTexture::Sky);
                } else {
                    let c_texture = CTexture::Flat(sector.ceiling_flat.clone());
                    if animations.contains_key(&c_texture) {
                        builder.add(CTextureAnimated {});
                    }

                    builder.add(c_texture);
                }
            }

            if sector.floor_flat != "-" {
                if sector.floor_flat == F_SKY1 {
                    builder.add(CTextureFloor(CTexture::Sky));
                } else {
                    let c_texture = CTexture::Flat(sector.floor_flat.clone());
                    if animations.contains_key(&c_texture) {
                        builder.add(CTextureAnimated {});
                    }
                    builder.add(CTextureFloor(c_texture));
                }
            }

            let mut cmd = CommandBuffer::new();
            cmd.spawn(builder.build());

            Some(cmd)
        })
        .collect::<Vec<Option<CommandBuffer>>>()
        .into_iter()
        .for_each(|cmd| {
            if let Some(mut cmd) = cmd {
                cmd.run_on(world);
            }
        });
}
