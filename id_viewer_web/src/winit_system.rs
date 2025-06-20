use winit::event::{KeyboardInput, ElementState, MouseButton, MouseScrollDelta, WindowEvent};
use id_core::renderer::system::{SystemEvent, SystemMouseButton, SystemMod};
use id_core::renderer::system::parse_keymap_from_usb;

pub fn to_system_event(event: WindowEvent) -> Option<SystemEvent> {
    match event {
        WindowEvent::KeyboardInput { input: KeyboardInput { scancode, state, .. }, .. } => {
            if let Ok(keymap) = parse_keymap_from_usb(scancode as u16) {
                let keycode = keymap.code?;
                let mods = keymap.modifier.unwrap_or(SystemMod::empty());
                match state {
                    ElementState::Pressed => Some(SystemEvent::KeyDown { keycode, mods }),
                    ElementState::Released => Some(SystemEvent::KeyUp { keycode, mods }),
                }
            } else { None }
        }
        WindowEvent::ReceivedCharacter(c) => Some(SystemEvent::Text { text: c.to_string() }),
        WindowEvent::CursorMoved { position, .. } => {
            Some(SystemEvent::MouseMotion { x: position.x as i32, y: position.y as i32, xrel: 0, yrel: 0 })
        }
        WindowEvent::MouseInput { state, button, .. } => {
            let btn = match button {
                MouseButton::Left => SystemMouseButton::Left,
                MouseButton::Middle => SystemMouseButton::Middle,
                MouseButton::Right => SystemMouseButton::Right,
                _ => return None,
            };
            match state {
                ElementState::Pressed => Some(SystemEvent::MouseButtonDown { mouse_btn: btn }),
                ElementState::Released => Some(SystemEvent::MouseButtonUp { mouse_btn: btn }),
            }
        }
        WindowEvent::MouseWheel { delta, .. } => {
            let (x, y) = match delta {
                MouseScrollDelta::LineDelta(x, y) => (x as i32, y as i32),
                MouseScrollDelta::PixelDelta(p) => (p.x as i32, p.y as i32),
            };
            Some(SystemEvent::MouseWheel { x, y })
        }
        WindowEvent::Resized(size) => Some(SystemEvent::SizeChanged { width: size.width, height: size.height }),
        _ => None,
    }
}
