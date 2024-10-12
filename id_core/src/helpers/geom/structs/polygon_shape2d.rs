use anyhow::Result;
use lyon_tessellation::{
    geom::point, geometry_builder::simple_builder, math::Point, path::Path, FillOptions,
    FillTessellator, VertexBuffers,
};
use ultraviolet::Vec2;

use super::{Polygon2d, PolygonPointIndex};

/// PolygonShape2d is a polygon with holes.
#[derive(Debug, Clone)]
pub struct PolygonShape2d {
    pub polygon: Polygon2d,
    pub holes: Vec<Polygon2d>,
}

struct _PolygonShape2dNode {
    index: PolygonPointIndex,
    holes: Vec<PolygonPointIndex>,
}

fn _polygon_shape_node_dfs(
    nodes_by_id: &Vec<_PolygonShape2dNode>,
    polygons: &Vec<Polygon2d>,
    index: PolygonPointIndex,
    candidate: &Polygon2d,
) -> Option<PolygonPointIndex> {
    let node = &nodes_by_id[index];
    let polygon = &polygons[index];

    if candidate.is_inside(polygon) {
        // Recursively see if we're in any holes.
        for hole in &node.holes {
            if let Some(k) = _polygon_shape_node_dfs(nodes_by_id, polygons, *hole, candidate) {
                return Some(k);
            }
        }

        // If not, return the node.
        return Some(node.index);
    }

    None
}

fn _create_polygon_shapes_dfs(
    nodes_by_id: &Vec<_PolygonShape2dNode>,
    polygons: &Vec<Polygon2d>,
    node: PolygonPointIndex,
    depth: u32,
    out: &mut Vec<PolygonShape2d>,
) {
    let node = &nodes_by_id[node];
    let polygon = &polygons[node.index];

    // If depth is even, create a new shape.
    if depth % 2 == 0 {
        out.push(PolygonShape2d {
            polygon: polygon.clone(),
            holes: node.holes.iter().map(|&i| polygons[i].clone()).collect(),
        });
    }

    // Recursively create shapes for each hole.
    for hole in &node.holes {
        _create_polygon_shapes_dfs(nodes_by_id, polygons, *hole, depth + 1, out);
    }
}

impl PolygonShape2d {
    pub fn from_polygons(polygons: &Vec<Polygon2d>) -> Vec<Self> {
        // Simple (and unoptimized) algorithm to detect holes, and holes-within-holes, etc...
        // We break holes-within-holes into separate polygon shapes.

        // Create a DAG of polygons.
        let mut nodes_by_id: Vec<_PolygonShape2dNode> = Vec::with_capacity(polygons.len());

        // For each polygon, create a node.
        for i in 0..polygons.len() {
            nodes_by_id.push(_PolygonShape2dNode {
                index: i,
                holes: Vec::new(),
            });
        }

        let mut node_is_hole = vec![false; polygons.len()];

        // For each polygon, find the first polygon that contains it.
        for (i, polygon) in polygons.iter().enumerate() {
            'inner: for (j, ..) in polygons.iter().enumerate() {
                if i == j {
                    continue 'inner;
                }

                // Start a DFS search to see if we're inside any holes.
                if let Some(k) = _polygon_shape_node_dfs(&nodes_by_id, polygons, j, polygon) {
                    nodes_by_id[k].holes.push(i);
                    node_is_hole[i] = true;
                    break 'inner;
                }
            }
        }

        let mut shapes: Vec<PolygonShape2d> = Vec::new();
        for (i, ..) in nodes_by_id.iter().enumerate() {
            if !node_is_hole[i] {
                _create_polygon_shapes_dfs(&nodes_by_id, polygons, i, 0, &mut shapes);
            }
        }

        shapes
    }

    pub fn tessellate(&self) -> Result<(Vec<Vec2>, Vec<u32>)> {
        let mut path_builder = Path::builder();
        let mut i: usize = 0;

        self.polygon.points.iter().for_each(|p| {
            if i == 0 {
                path_builder.begin(point(p.x, p.y));
            } else {
                path_builder.line_to(point(p.x, p.y));
            }
            i += 1;
        });
        path_builder.end(false);

        self.holes.iter().for_each(|h| {
            let mut j: usize = 0;
            h.points.iter().for_each(|p| {
                if j == 0 {
                    path_builder.begin(point(p.x, p.y));
                } else {
                    path_builder.line_to(point(p.x, p.y));
                }
                j += 1;
            });
            path_builder.end(false);
        });

        // Create the tessellator.
        let mut tessellator = FillTessellator::new();

        let path = path_builder.build();
        let mut buffers: VertexBuffers<Point, u16> = VertexBuffers::new();
        let mut vertex_builder = simple_builder(&mut buffers);

        // Compute the tessellation.
        tessellator.tessellate_path(&path, &FillOptions::default(), &mut vertex_builder)?;

        Ok((
            buffers
                .vertices
                .iter()
                .map(|v| Vec2::new(v.x, v.y))
                .collect(),
            buffers.indices.iter().map(|n| *n as u32).collect(),
        ))
    }
}
