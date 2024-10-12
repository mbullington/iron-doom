use multimap::MultiMap;
use ultraviolet::Vec2;

use crate::helpers::geom::minimum_cycle_basis;

use super::Polygon2d;

#[derive(Debug, Clone)]
pub struct Graph2d {
    pub vertices: Vec<Vec2>,
    /// This is a multimap from vertex index to adjacent vertex indices.
    pub vert_adj_map: MultiMap<GraphVertIndex, GraphVertIndex>,
}

impl Graph2d {
    pub fn new(vertices: Vec<Vec2>, edges: Vec<(usize, usize)>) -> Self {
        let mut vert_adj_map = MultiMap::new();
        for (v1, v2) in edges.iter() {
            vert_adj_map.insert(*v1, *v2);
            vert_adj_map.insert(*v2, *v1);
        }

        Self {
            vertices,
            vert_adj_map,
        }
    }

    pub fn detect_polygons(&self) -> Vec<Polygon2d> {
        let cycles = minimum_cycle_basis(self);
        cycles
            .into_iter()
            .map(|cycle| Polygon2d::from_graph_cycle(self, &cycle))
            .collect()
    }
}

pub type GraphVertIndex = usize;
