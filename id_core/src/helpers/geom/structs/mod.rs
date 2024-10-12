mod bounds2d;
mod graph2d;
mod polygon2d;
mod polygon_shape2d;
mod triangles2d;

pub use bounds2d::*;
pub use graph2d::*;
pub use polygon2d::*;
pub use polygon_shape2d::*;
pub use triangles2d::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Winding {
    Clockwise,
    CounterClockwise,
}
