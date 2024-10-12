use std::time::Duration;

use anyhow::Result;
use ultraviolet::UVec2;
use wgpu::{Device, Queue, Texture, TextureFormat};

mod window_runner;
mod window_sequence;

mod egui_user_context;
mod egui_window;

mod overlay_window;

pub use egui_user_context::*;
pub use egui_window::*;
pub use overlay_window::*;

pub use window_runner::*;
pub use window_sequence::*;

use super::system::SystemEvent;

/// This is passed to the window to give it common information.
pub struct WindowContext<'a, UC: UserContext> {
    pub device: &'a Device,
    pub queue: &'a Queue,
    pub surface_format: &'a TextureFormat,
    pub size: UVec2,

    pub user_context: &'a mut UC,
}

/// This is passed to the top-level window to give it common information.
pub struct UserContextContext<'a> {
    pub device: &'a Device,
    pub queue: &'a Queue,
    pub surface_format: &'a TextureFormat,
    pub size: UVec2,
}

/// This trait sets up a window before construction.
pub trait WindowSetup<UC: UserContext> =
    FnOnce(&WindowContext<UC>, UVec2) -> Result<Box<dyn Window<UC>>>;

/// Windows are interactive, nestable, primitives that represent a
/// rectangular area of the screen.
///
/// The root window will always get the swapchain for render, subsequent windows
/// will be rendered to a texture and then composited into the root window.
pub trait Window<UC: UserContext> {
    /// Handle the system event. If true is returned, the event is consumed.
    fn handle_event(
        &mut self,
        _context: &mut WindowContext<UC>,
        _event: &SystemEvent,
    ) -> Result<bool> {
        Ok(false)
    }

    /// Think is called once per tick, and is used to update the state of the window.
    fn think(&mut self, _context: &mut WindowContext<UC>, _delta: Duration) -> Result<()> {
        Ok(())
    }

    fn draw(
        &mut self,
        _context: &mut WindowContext<UC>,
        texture: &Texture,
        delta: Duration,
    ) -> Result<()>;
}

/// This trait sets up a top-level window before construction.
pub trait UserContextSetup<UC: UserContext> = FnOnce(&UserContextContext, UVec2) -> Result<Box<UC>>;

/// UserContext is passed to the window runner at initialization.
///
/// User context can be used for:
/// - Shared buffers (like UBOs, or texture atlases), that are reused across windows
///   and may otherwise be expensive.
/// - Shared state (like a world, or a renderer).
/// - Shared tooling (such as egui::Context)
pub trait UserContext {
    /// Think is called once per tick, and is used to update the state of the window.
    ///
    /// Think is also the main place where [UserContext] should be updating its internal state.
    fn think(&mut self, _context: &UserContextContext, _delta: Duration) -> Result<()> {
        Ok(())
    }
}
