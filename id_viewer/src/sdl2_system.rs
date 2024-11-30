// Since we don't want to use SDL2 across the board (namely: so we can have viewer/editor on Web),
// we define a subset of system abstractions here.
//
// This is the SDL2 implementation, which tracks system.rs in "id_core".

use std::mem::transmute;

use id_core::renderer::system::{parse_keymap_from_usb, SystemEvent, SystemMod, SystemMouseButton};

use sdl2::{
    event::{Event, WindowEvent},
    keyboard::Scancode,
    mouse::MouseButton,
    video::Window,
};

pub trait ToSystemMouseButtonExt {
    fn to_system_mouse_button(&self) -> Option<SystemMouseButton>;
}

impl ToSystemMouseButtonExt for MouseButton {
    fn to_system_mouse_button(&self) -> Option<SystemMouseButton> {
        match self {
            MouseButton::Left => Some(SystemMouseButton::Left),
            MouseButton::Middle => Some(SystemMouseButton::Middle),
            MouseButton::Right => Some(SystemMouseButton::Right),
            _ => None,
        }
    }
}

pub trait ToSystemEventExt {
    fn to_system_event(&self, sdl_window: &Window) -> Option<SystemEvent>;
}

impl ToSystemEventExt for Event {
    fn to_system_event(&self, sdl_window: &Window) -> Option<SystemEvent> {
        match self {
            Event::TextInput { text, .. } => {
                return Some(SystemEvent::Text { text: text.clone() });
            }
            Event::KeyDown {
                scancode: Some(scancode),
                ..
            } => {
                let scancode = unsafe { transmute::<Scancode, u32>(*scancode) as u16 };
                let keymap = parse_keymap_from_usb(scancode);

                if let Ok(keymap) = keymap {
                    let keycode = keymap.code;
                    let mods = keymap.modifier;

                    if let Some(keycode) = keycode {
                        return Some(SystemEvent::KeyDown {
                            keycode,
                            mods: mods.unwrap_or(SystemMod::empty()),
                        });
                    }
                }
            }
            Event::KeyUp {
                scancode: Some(scancode),
                ..
            } => {
                let scancode = unsafe { transmute::<Scancode, u32>(*scancode) as u16 };
                let keymap = parse_keymap_from_usb(scancode);

                if let Ok(keymap) = keymap {
                    let keycode = keymap.code;
                    let mods = keymap.modifier;

                    if let Some(keycode) = keycode {
                        return Some(SystemEvent::KeyUp {
                            keycode,
                            mods: mods.unwrap_or(SystemMod::empty()),
                        });
                    }
                }
            }
            Event::MouseMotion {
                x, y, xrel, yrel, ..
            } => {
                return Some(SystemEvent::MouseMotion {
                    x: *x,
                    y: *y,
                    xrel: *xrel,
                    yrel: *yrel,
                });
            }
            Event::MouseWheel { x, y, .. } => {
                return Some(SystemEvent::MouseWheel { x: *x, y: *y });
            }
            Event::MouseButtonDown { mouse_btn, .. } => {
                if let Some(mouse_btn) = mouse_btn.to_system_mouse_button() {
                    return Some(SystemEvent::MouseButtonDown { mouse_btn });
                }
            }
            Event::MouseButtonUp { mouse_btn, .. } => {
                if let Some(mouse_btn) = mouse_btn.to_system_mouse_button() {
                    return Some(SystemEvent::MouseButtonUp { mouse_btn });
                }
            }
            Event::Window {
                window_id,
                win_event: WindowEvent::SizeChanged(width, height),
                ..
            } if *window_id == sdl_window.id() => {
                if *width <= 0 || *height <= 0 {
                    return None;
                }
                let width = *width as u32;
                let height = *height as u32;

                return Some(SystemEvent::SizeChanged { width, height });
            }
            _ => {}
        }
        None
    }
}
