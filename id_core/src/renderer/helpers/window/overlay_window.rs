use std::time::Duration;

use anyhow::Result;
use ultraviolet::UVec2;

use crate::renderer::system::SystemEvent;

use super::{UserContext, Window, WindowContext, WindowSequence, WindowSetup};

pub fn overlay_window<UC: UserContext + 'static>(
    children: impl Into<WindowSequence<UC>>,
) -> impl WindowSetup<UC> {
    move |_context: &WindowContext<UC>, _size: UVec2| {
        Ok(Box::new(OverlayWindow {
            children: children
                .into()
                .sequence
                .into_iter()
                .map(|child| child(_context, _size))
                .collect::<Result<Vec<_>>>()?,
        }))
    }
}

pub struct OverlayWindow<UC: UserContext + 'static> {
    pub children: Vec<Box<dyn Window<UC>>>,
}

impl<UC: UserContext + 'static> Window<UC> for OverlayWindow<UC> {
    fn handle_event(
        &mut self,
        context: &mut WindowContext<UC>,
        event: &SystemEvent,
    ) -> Result<bool> {
        // Run all children in order. If any child consumes the event, return true.
        for child in &mut self.children {
            if child.handle_event(context, event)? {
                return Ok(true);
            }
        }

        Ok(false)
    }

    fn think(&mut self, context: &mut WindowContext<UC>, delta: Duration) -> Result<()> {
        // Run all children in order.
        for child in &mut self.children {
            child.think(context, delta)?;
        }

        Ok(())
    }

    fn draw(
        &mut self,
        context: &mut WindowContext<UC>,
        texture: &wgpu::Texture,
        _delta: Duration,
    ) -> Result<()> {
        // Run all children in order.
        // This is guaranteed by the order of the queue, of which there are only one in WGPU.
        for child in &mut self.children {
            child.draw(context, texture, _delta)?;
        }

        Ok(())
    }
}
