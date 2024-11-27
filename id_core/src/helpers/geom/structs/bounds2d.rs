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
}
