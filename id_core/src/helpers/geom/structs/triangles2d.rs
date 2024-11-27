#![allow(unused)]

use ultraviolet::Vec2;

use super::{Bounds2d, Graph2d, GraphVertIndex};

/// [Triangles2d] is a set of tesselated triangles in 2D space.
///
/// This data type makes no other assumptions about the triangles. It is likely complex.
#[derive(Debug, Clone)]
pub struct Triangles2d {
    pub points: Vec<Vec2>,
    /// Indices into the points array.
    /// These are in groups of three, as they represent triangles.
    pub indices: Vec<u32>,
}

impl Triangles2d {
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
        // TODO: This is embarassingly parallel.
        // We can try using SIMD to speed this up.

        for chunk in self.indices.chunks(3) {
            let a = self.points[chunk[0] as usize];
            let b = self.points[chunk[1] as usize];
            let c = self.points[chunk[2] as usize];

            if _point_in_triangle(point, a, b, c) {
                return true;
            }
        }

        false
    }
}

fn _point_in_triangle(point: Vec2, a: Vec2, b: Vec2, c: Vec2) -> bool {
    let v0 = Vec2 {
        x: c.x - a.x,
        y: c.y - a.y,
    };
    let v1 = Vec2 {
        x: b.x - a.x,
        y: b.y - a.y,
    };
    let v2 = Vec2 {
        x: point.x - a.x,
        y: point.y - a.y,
    };

    let dot00 = v0.x * v0.x + v0.y * v0.y;
    let dot01 = v0.x * v1.x + v0.y * v1.y;
    let dot02 = v0.x * v2.x + v0.y * v2.y;
    let dot11 = v1.x * v1.x + v1.y * v1.y;
    let dot12 = v1.x * v2.x + v1.y * v2.y;

    let inv_denom = 1.0 / (dot00 * dot11 - dot01 * dot01);
    let u = (dot11 * dot02 - dot01 * dot12) * inv_denom;
    let v = (dot00 * dot12 - dot01 * dot02) * inv_denom;

    // Check if point is in triangle
    u >= 0.0 && v >= 0.0 && (u + v) <= 1.0
}
