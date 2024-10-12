// Since we don't want to use SDL2 across the board (namely: so we can have viewer/editor on Web),
// we define a subset of system abstractions here.
//
// This is the SDL2 implementation, which tracks system.rs in "id_core".

use id_core::renderer::system::{SystemEvent, SystemKeycode, SystemMod, SystemMouseButton};
use sdl2::{
    event::{Event, WindowEvent},
    keyboard::Keycode,
    mouse::MouseButton,
    video::Window,
};

pub trait ToSystemKeycodeExt {
    fn to_system_keycode(&self) -> Option<SystemKeycode>;
}

impl ToSystemKeycodeExt for Keycode {
    fn to_system_keycode(&self) -> Option<SystemKeycode> {
        match *self {
            Keycode::LSHIFT => Some(SystemKeycode::LShift),
            Keycode::RSHIFT => Some(SystemKeycode::RShift),
            Keycode::UNDERSCORE => Some(SystemKeycode::Underscore),
            Keycode::MINUS => Some(SystemKeycode::Minus),
            Keycode::Left => Some(SystemKeycode::Left),
            Keycode::Up => Some(SystemKeycode::Up),
            Keycode::Right => Some(SystemKeycode::Right),
            Keycode::Down => Some(SystemKeycode::Down),
            Keycode::Escape => Some(SystemKeycode::Escape),
            Keycode::Tab => Some(SystemKeycode::Tab),
            Keycode::Backspace => Some(SystemKeycode::Backspace),
            Keycode::Space => Some(SystemKeycode::Space),
            Keycode::Return => Some(SystemKeycode::Return),
            Keycode::Insert => Some(SystemKeycode::Insert),
            Keycode::Home => Some(SystemKeycode::Home),
            Keycode::Delete => Some(SystemKeycode::Delete),
            Keycode::End => Some(SystemKeycode::End),
            Keycode::PageDown => Some(SystemKeycode::PageDown),
            Keycode::PageUp => Some(SystemKeycode::PageUp),
            Keycode::Kp0 => Some(SystemKeycode::Kp0),
            Keycode::Num0 => Some(SystemKeycode::Num0),
            Keycode::Kp1 => Some(SystemKeycode::Kp1),
            Keycode::Num1 => Some(SystemKeycode::Num1),
            Keycode::Kp2 => Some(SystemKeycode::Kp2),
            Keycode::Num2 => Some(SystemKeycode::Num2),
            Keycode::Kp3 => Some(SystemKeycode::Kp3),
            Keycode::Num3 => Some(SystemKeycode::Num3),
            Keycode::Kp4 => Some(SystemKeycode::Kp4),
            Keycode::Num4 => Some(SystemKeycode::Num4),
            Keycode::Kp5 => Some(SystemKeycode::Kp5),
            Keycode::Num5 => Some(SystemKeycode::Num5),
            Keycode::Kp6 => Some(SystemKeycode::Kp6),
            Keycode::Num6 => Some(SystemKeycode::Num6),
            Keycode::Kp7 => Some(SystemKeycode::Kp7),
            Keycode::Num7 => Some(SystemKeycode::Num7),
            Keycode::Kp8 => Some(SystemKeycode::Kp8),
            Keycode::Num8 => Some(SystemKeycode::Num8),
            Keycode::Kp9 => Some(SystemKeycode::Kp9),
            Keycode::Num9 => Some(SystemKeycode::Num9),
            Keycode::PERIOD => Some(SystemKeycode::Period),
            Keycode::A => Some(SystemKeycode::A),
            Keycode::B => Some(SystemKeycode::B),
            Keycode::C => Some(SystemKeycode::C),
            Keycode::D => Some(SystemKeycode::D),
            Keycode::E => Some(SystemKeycode::E),
            Keycode::F => Some(SystemKeycode::F),
            Keycode::G => Some(SystemKeycode::G),
            Keycode::H => Some(SystemKeycode::H),
            Keycode::I => Some(SystemKeycode::I),
            Keycode::J => Some(SystemKeycode::J),
            Keycode::K => Some(SystemKeycode::K),
            Keycode::L => Some(SystemKeycode::L),
            Keycode::M => Some(SystemKeycode::M),
            Keycode::N => Some(SystemKeycode::N),
            Keycode::O => Some(SystemKeycode::O),
            Keycode::P => Some(SystemKeycode::P),
            Keycode::Q => Some(SystemKeycode::Q),
            Keycode::R => Some(SystemKeycode::R),
            Keycode::S => Some(SystemKeycode::S),
            Keycode::T => Some(SystemKeycode::T),
            Keycode::U => Some(SystemKeycode::U),
            Keycode::V => Some(SystemKeycode::V),
            Keycode::W => Some(SystemKeycode::W),
            Keycode::X => Some(SystemKeycode::X),
            Keycode::Y => Some(SystemKeycode::Y),
            Keycode::Z => Some(SystemKeycode::Z),
            _ => None,
        }
    }
}

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
            Event::KeyDown {
                keycode, keymod, ..
            } => {
                if let Some(keycode) = keycode {
                    if let Some(keycode) = keycode.to_system_keycode() {
                        // Check the modifiers
                        use sdl2::keyboard::Mod;
                        let alt = (*keymod & Mod::LALTMOD == Mod::LALTMOD)
                            || (*keymod & Mod::RALTMOD == Mod::RALTMOD);
                        let ctrl = (*keymod & Mod::LCTRLMOD == Mod::LCTRLMOD)
                            || (*keymod & Mod::RCTRLMOD == Mod::RCTRLMOD);
                        let shift = (*keymod & Mod::LSHIFTMOD == Mod::LSHIFTMOD)
                            || (*keymod & Mod::RSHIFTMOD == Mod::RSHIFTMOD);
                        let mac_cmd = *keymod & Mod::LGUIMOD == Mod::LGUIMOD;
                        let command = (*keymod & Mod::LCTRLMOD == Mod::LCTRLMOD)
                            || (*keymod & Mod::LGUIMOD == Mod::LGUIMOD);

                        let mods = SystemMod {
                            alt,
                            ctrl,
                            shift,
                            mac_cmd,
                            command,
                        };

                        return Some(SystemEvent::KeyDown { keycode, mods });
                    }
                }
            }
            Event::KeyUp {
                keycode, keymod, ..
            } => {
                if let Some(keycode) = keycode {
                    if let Some(keycode) = keycode.to_system_keycode() {
                        // Check the modifiers
                        use sdl2::keyboard::Mod;
                        let alt = (*keymod & Mod::LALTMOD == Mod::LALTMOD)
                            || (*keymod & Mod::RALTMOD == Mod::RALTMOD);
                        let ctrl = (*keymod & Mod::LCTRLMOD == Mod::LCTRLMOD)
                            || (*keymod & Mod::RCTRLMOD == Mod::RCTRLMOD);
                        let shift = (*keymod & Mod::LSHIFTMOD == Mod::LSHIFTMOD)
                            || (*keymod & Mod::RSHIFTMOD == Mod::RSHIFTMOD);
                        let mac_cmd = *keymod & Mod::LGUIMOD == Mod::LGUIMOD;
                        let command = (*keymod & Mod::LCTRLMOD == Mod::LCTRLMOD)
                            || (*keymod & Mod::LGUIMOD == Mod::LGUIMOD);

                        let mods = SystemMod {
                            alt,
                            ctrl,
                            shift,
                            mac_cmd,
                            command,
                        };

                        return Some(SystemEvent::KeyUp { keycode, mods });
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
