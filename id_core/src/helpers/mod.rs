pub mod geom;

mod stopwatch;

pub use stopwatch::Stopwatch;

mod camera;
pub use camera::Camera;
use ultraviolet::{Rotor3, Vec3};

pub trait Movable {
    fn get_pos(&self) -> Vec3;
    fn translate(&mut self, delta: Vec3);
    fn translate_xz(&mut self, delta: Vec3);

    fn get_yaw(&self) -> f32;
    fn get_pitch(&self) -> f32;

    fn get_rotor(&self) -> Rotor3 {
        let mut rotor = Rotor3::from_euler_angles(
            0.0,
            self.get_pitch().to_radians(),
            self.get_yaw().to_radians(),
        );
        rotor.normalize();
        rotor
    }

    fn move_premul(&mut self, delta: Vec3) {
        // Split apart the XY and Z components of the delta vector.
        let delta_xy = Vec3 {
            x: delta.x,
            y: delta.y,
            z: 0.,
        };
        let delta_z = Vec3 {
            x: 0.,
            y: 0.,
            z: delta.z,
        };

        self.translate_xz(delta_xy);
        self.translate(delta_z);
    }

    fn get_look_at_vector(&self) -> Vec3 {
        // By default, we look down the positive Z axis (forward).
        let look_at_vector = Vec3 {
            x: 0.0,
            y: 0.0,
            z: 1.0,
        };

        self.get_rotor() * look_at_vector
    }

    fn rotate_pitch_yaw(&mut self, pitch: f32, yaw: f32);
}

pub trait UnwrapOrFn<T> {
    fn unwrap_or_fn<F>(self, f: F) -> T
    where
        F: FnOnce() -> T;
}

impl<T> UnwrapOrFn<T> for Option<T> {
    fn unwrap_or_fn<F>(self, f: F) -> T
    where
        F: FnOnce() -> T,
    {
        match self {
            Some(val) => val,
            None => f(),
        }
    }
}
