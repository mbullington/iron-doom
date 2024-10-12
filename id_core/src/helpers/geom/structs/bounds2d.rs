#![allow(unused)]

use ultraviolet::Vec2;

#[derive(Debug, Clone)]
pub struct Bounds2d {
    pub min: Vec2,
    pub max: Vec2,
}
