#![allow(unused)]

use ultraviolet::Vec2;

use crate::helpers::geom::point_in_polygon;

use super::{Bounds2d, Graph2d, GraphVertIndex};

/// [Polygon2d] is a set of points that make a closed "cycle" in 2D space.
///
/// This data type makes no other assumptions about the polygon. It is likely complex.
#[derive(Debug, Clone)]
pub struct Polygon2d {
    pub points: Vec<Vec2>,
}

pub type PolygonPointIndex = usize;

impl Polygon2d {
    pub fn from_graph_cycle(graph: &Graph2d, cycle: &Vec<GraphVertIndex>) -> Self {
        let mut points = Vec::with_capacity(cycle.len());
        for &vertex_index in cycle {
            points.push(graph.vertices[vertex_index]);
        }

        Self { points }
    }

    pub fn bbox(&self) -> Bounds2d {
        let mut min_x = f32::MAX;
        let mut min_y = f32::MAX;
        let mut max_x = f32::MIN;
        let mut max_y = f32::MIN;

        for point in &self.points {
            min_x = min_x.min(point.x);
            min_y = min_y.min(point.y);
            max_x = max_x.max(point.x);
            max_y = max_y.max(point.y);
        }

        Bounds2d {
            min: Vec2::new(min_x, min_y),
            max: Vec2::new(max_x, max_y),
        }
    }

    pub fn has_point(&self, point: Vec2) -> bool {
        point_in_polygon(&point, self)
    }

    pub fn is_inside(&self, other: &Self) -> bool {
        // TODO: This is a naive implementation. We can optimize this heavily.
        // Use bboxes.

        for point in &self.points {
            if !other.has_point(*point) {
                return false;
            }
        }

        true
    }
}
