use std::{collections::HashMap, time::Duration};
use ultraviolet::Vec3;

use crate::helpers::Movable;

use super::system::{SystemEvent, SystemKeycode};

#[derive(Default)]
pub struct MovementController {
    // These are keyboard movements, which should be denormalized with the render.
    // This is a bit of a hack, but it works for now.
    key_presses: HashMap<SystemKeycode, bool>,
}

impl MovementController {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    pub fn handle_event(&mut self, movable: &mut impl Movable, event: &SystemEvent) {
        match event {
            SystemEvent::KeyDown { keycode, .. } => {
                self.key_presses.insert(*keycode, true);
            }
            SystemEvent::KeyUp { keycode, .. } => {
                self.key_presses.insert(*keycode, false);
            }
            SystemEvent::MouseMotion { xrel, yrel, .. } => {
                movable.rotate_pitch_yaw((*yrel as f32) * 2.2, (*xrel as f32) * 2.2);
            }
            _ => {}
        }
    }

    pub fn think(&self, movable: &mut impl Movable, delta: Duration) {
        let mut movement = Vec3::default();

        if *self.key_presses.get(&SystemKeycode::KeyW).unwrap_or(&false) {
            movement.z += 1.;
        }
        if *self.key_presses.get(&SystemKeycode::KeyS).unwrap_or(&false) {
            movement.z -= 1.;
        }
        if *self.key_presses.get(&SystemKeycode::KeyA).unwrap_or(&false) {
            movement.x -= 1.;
        }
        if *self.key_presses.get(&SystemKeycode::KeyD).unwrap_or(&false) {
            movement.x += 1.;
        }

        // If either Shift is pressed, make the movement faster.
        if *self
            .key_presses
            .get(&SystemKeycode::ShiftLeft)
            .unwrap_or(&false)
            || *self
                .key_presses
                .get(&SystemKeycode::ShiftRight)
                .unwrap_or(&false)
        {
            movement *= 3.;
        }

        if movement.mag() != 0.0 {
            let norm = movement * (delta.as_millis() as f32) * 291.66 / 1000.;
            movable.move_premul(Vec3 {
                x: -norm.x,
                y: 0.,
                z: -norm.z,
            });
        }
    }
}
