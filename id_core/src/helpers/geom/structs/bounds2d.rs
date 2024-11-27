#![allow(unused)]

use ultraviolet::Vec2;

#[derive(Debug, Clone)]
pub struct Bounds2d {
    pub min: Vec2,
    pub max: Vec2,
}

impl Bounds2d {
    pub fn has_point(&self, point: Vec2) -> bool {
        (point - self.min).component_min() >= 0. && (point - self.max).component_min() <= 0.
    }

    pub fn from_iter<I: IntoIterator<Item = Bounds2d>>(iter: I) -> Self {
        let mut min_x = std::f32::MAX;
        let mut min_y = std::f32::MAX;
        let mut max_x = std::f32::MIN;
        let mut max_y = std::f32::MIN;

        for bounds in iter {
            min_x = min_x.min(bounds.min.x);
            min_y = min_y.min(bounds.min.y);
            max_x = max_x.max(bounds.max.x);
            max_y = max_y.max(bounds.max.y);
        }

        Self {
            min: Vec2::new(min_x, min_y),
            max: Vec2::new(max_x, max_y),
        }
    }
}
