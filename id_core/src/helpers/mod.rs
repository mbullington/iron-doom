pub mod geom;

mod camera;
mod stopwatch;

pub use camera::Camera;
pub use stopwatch::Stopwatch;

use ultraviolet::Vec3;

pub trait Movable {
    fn move_premul(&mut self, delta: Vec3);
}
