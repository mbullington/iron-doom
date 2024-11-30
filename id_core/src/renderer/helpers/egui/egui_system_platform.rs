use egui::{Key::*, Modifiers, Pos2};

use crate::renderer::system::{SystemKeycode, SystemMod};

use super::super::system::{SystemEvent, SystemMouseButton};

/// A trait that adds a method to convert to an egui key
pub trait ToEguiKey {
    /// Convert the struct to an egui key
    fn to_egui_key(&self, mods: &Modifiers) -> Option<egui::Key>;
}

type S = SystemKeycode;

impl ToEguiKey for SystemKeycode {
    fn to_egui_key(&self, mods: &Modifiers) -> Option<egui::Key> {
        Some(match *self {
            S::ArrowDown => ArrowDown,
            S::ArrowLeft => ArrowLeft,
            S::ArrowRight => ArrowRight,
            S::ArrowUp => ArrowUp,

            S::Escape => Escape,
            S::Tab => Tab,
            S::Backspace => Backspace,
            S::Enter => Enter,
            S::Space => Space,

            S::Insert => Insert,
            S::Delete => Delete,
            S::Home => Home,
            S::End => End,
            S::PageUp => PageUp,
            S::PageDown => PageDown,

            S::Copy => Copy,
            S::Cut => Cut,
            S::Paste => Paste,

            // ----------------------------------------------
            // Punctuation:
            S::Backquote => Backtick,
            // `,`
            S::Comma => Comma,

            // `[`
            S::BracketLeft => OpenBracket,

            // `]`
            S::BracketRight => CloseBracket,

            // `-`
            S::Minus => Minus,

            // `.`
            S::Period => Period,

            // `=/+`
            S::Equal => {
                if mods.shift {
                    Plus
                } else {
                    Equals
                }
            }

            // `;/:`
            S::Semicolon => {
                if mods.shift {
                    Colon
                } else {
                    Semicolon
                }
            }

            // `\|`
            S::Backslash => {
                if mods.shift {
                    Pipe
                } else {
                    Backslash
                }
            }

            // `/?`
            S::Slash => {
                if mods.shift {
                    Questionmark
                } else {
                    Slash
                }
            }

            // `'`
            S::Quote => Quote,

            // ----------------------------------------------
            // Digits:
            // `0` (from main row or numpad)
            S::Digit0 => Num0,

            // `1` (from main row or numpad)
            S::Digit1 => Num1,

            // `2` (from main row or numpad)
            S::Digit2 => Num2,

            // `3` (from main row or numpad)
            S::Digit3 => Num3,

            // `4` (from main row or numpad)
            S::Digit4 => Num4,

            // `5` (from main row or numpad)
            S::Digit5 => Num5,

            // `6` (from main row or numpad)
            S::Digit6 => Num6,

            // `7` (from main row or numpad)
            S::Digit7 => Num7,

            // `8` (from main row or numpad)
            S::Digit8 => Num8,

            // `9` (from main row or numpad)
            S::Digit9 => Num9,

            // ----------------------------------------------
            // Letters:
            S::KeyA => A, // Used for cmd+A (select All)
            S::KeyB => B,
            S::KeyC => C, // |CMD COPY|
            S::KeyD => D, // |CMD BOOKMARK|
            S::KeyE => E, // |CMD SEARCH|
            S::KeyF => F, // |CMD FIND firefox & chrome|
            S::KeyG => G, // |CMD FIND chrome|
            S::KeyH => H, // |CMD History|
            S::KeyI => I, // italics
            S::KeyJ => J, // |CMD SEARCH firefox/DOWNLOAD chrome|
            S::KeyK => K, // Used for ctrl+K (delete text after cursor)
            S::KeyL => L,
            S::KeyM => M,
            S::KeyN => N,
            S::KeyO => O, // |CMD OPEN|
            S::KeyP => P, // |CMD PRINT|
            S::KeyQ => Q,
            S::KeyR => R, // |CMD REFRESH|
            S::KeyS => S, // |CMD SAVE|
            S::KeyT => T, // |CMD TAB|
            S::KeyU => U, // Used for ctrl+U (delete text before cursor)
            S::KeyV => V, // |CMD PASTE|
            S::KeyW => W, // Used for ctrl+W (delete previous word)
            S::KeyX => X, // |CMD CUT|
            S::KeyY => Y,
            S::KeyZ => Z, // |CMD UNDO|

            // ----------------------------------------------
            // Function keys:
            S::F1 => F1,
            S::F2 => F2,
            S::F3 => F3,
            S::F4 => F4,
            S::F5 => F5,
            S::F6 => F6,
            S::F7 => F7,
            S::F8 => F8,
            S::F9 => F9,
            S::F10 => F10,
            S::F11 => F11,
            S::F12 => F12,
            S::F13 => F13,
            S::F14 => F14,
            S::F15 => F15,
            S::F16 => F16,
            S::F17 => F17,
            S::F18 => F18,
            S::F19 => F19,
            S::F20 => F20,
            S::F21 => F21,
            S::F22 => F22,
            S::F23 => F23,
            S::F24 => F24,

            // S::F25 => F25,
            // S::F26 => F26,
            // S::F27 => F27,
            // S::F28 => F28,
            // S::F29 => F29,
            // S::F30 => F30,
            // S::F31 => F31,
            // S::F32 => F32,
            // S::F33 => F33,
            // S::F34 => F34,
            // S::F35 => F35,
            _ => return None,
        })
    }
}

/// The sdl2 platform for egui
pub struct EguiPlatform {
    // The position of the mouse pointer
    pointer_pos: Pos2,
    // The egui modifiers
    modifiers: Modifiers,
    // The raw input
    egui_input: egui::RawInput,

    // The egui context
    egui_ctx: egui::Context,
}

impl EguiPlatform {
    /// Construct a new [`Platform`]
    pub fn new(screen_size: (u32, u32)) -> anyhow::Result<Self> {
        Ok(Self {
            pointer_pos: Pos2::ZERO,
            egui_input: egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::Vec2 {
                        x: screen_size.0 as f32,
                        y: screen_size.1 as f32,
                    },
                )),
                ..Default::default()
            },
            modifiers: Modifiers::default(),
            egui_ctx: egui::Context::default(),
        })
    }

    /// Handle a sdl2 event
    pub fn handle_event(&mut self, event: &SystemEvent) -> bool {
        match event {
            // Handle reizing
            SystemEvent::SizeChanged { width, height } => {
                self.egui_input.screen_rect = Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::Vec2 {
                        x: *width as f32,
                        y: *height as f32,
                    },
                ));
            }

            // Handle the mouse button being held down
            SystemEvent::MouseButtonDown { mouse_btn, .. } => {
                let btn = match mouse_btn {
                    SystemMouseButton::Left => Some(egui::PointerButton::Primary),
                    SystemMouseButton::Middle => Some(egui::PointerButton::Middle),
                    SystemMouseButton::Right => Some(egui::PointerButton::Secondary),
                };
                if let Some(btn) = btn {
                    self.egui_input.events.push(egui::Event::PointerButton {
                        pos: self.pointer_pos,
                        button: btn,
                        pressed: true,
                        modifiers: self.modifiers,
                    });
                }
            }

            // Handle the mouse button being released
            SystemEvent::MouseButtonUp { mouse_btn, .. } => {
                let btn = match mouse_btn {
                    SystemMouseButton::Left => Some(egui::PointerButton::Primary),
                    SystemMouseButton::Middle => Some(egui::PointerButton::Middle),
                    SystemMouseButton::Right => Some(egui::PointerButton::Secondary),
                };
                if let Some(btn) = btn {
                    self.egui_input.events.push(egui::Event::PointerButton {
                        pos: self.pointer_pos,
                        button: btn,
                        pressed: false,
                        modifiers: self.modifiers,
                    });
                }
            }

            // Handle mouse motion
            SystemEvent::MouseMotion { x, y, .. } => {
                // Update the pointer position
                self.pointer_pos = egui::Pos2::new(*x as f32, *y as f32);
                self.egui_input
                    .events
                    .push(egui::Event::PointerMoved(self.pointer_pos));

                return self.egui_ctx.wants_pointer_input();
            }

            // Handle text
            SystemEvent::Text { text } => {
                if self.egui_ctx.wants_keyboard_input() {
                    self.egui_input.events.push(egui::Event::Text(text.clone()));
                }
            }

            // Handle a key being pressed
            SystemEvent::KeyDown { keycode, mods, .. } => {
                let modifiers = Modifiers {
                    alt: mods.contains(SystemMod::AltLeft) || mods.contains(SystemMod::AltRight),
                    ctrl: mods.contains(SystemMod::ControlLeft)
                        || mods.contains(SystemMod::ControlRight),
                    shift: mods.contains(SystemMod::ShiftLeft)
                        || mods.contains(SystemMod::ShiftRight),
                    mac_cmd: mods.contains(SystemMod::MetaLeft)
                        || mods.contains(SystemMod::MetaRight),
                    // TODO: Check for macOS specifically.
                    command: mods.contains(SystemMod::MetaLeft)
                        || mods.contains(SystemMod::MetaRight),
                };

                // Convert the keycode to an egui key
                if let Some(key) = keycode.to_egui_key(&modifiers) {
                    // Update the modifiers
                    self.modifiers = modifiers;
                    self.egui_input.modifiers = self.modifiers;

                    // Push the event
                    self.egui_input.events.push(egui::Event::Key {
                        key,
                        physical_key: None,
                        pressed: true,
                        repeat: false,
                        modifiers: self.modifiers,
                    });

                    return self.egui_ctx.wants_keyboard_input();
                }
            }

            // Handle a key being released
            SystemEvent::KeyUp { keycode, mods, .. } => {
                let modifiers = Modifiers {
                    alt: mods.contains(SystemMod::AltLeft) || mods.contains(SystemMod::AltRight),
                    ctrl: mods.contains(SystemMod::ControlLeft)
                        || mods.contains(SystemMod::ControlRight),
                    shift: mods.contains(SystemMod::ShiftLeft)
                        || mods.contains(SystemMod::ShiftRight),
                    mac_cmd: mods.contains(SystemMod::MetaLeft)
                        || mods.contains(SystemMod::MetaRight),
                    // TODO: Check for macOS specifically.
                    command: mods.contains(SystemMod::MetaLeft)
                        || mods.contains(SystemMod::MetaRight),
                };

                // Convert the keycode to an egui key
                if let Some(key) = keycode.to_egui_key(&modifiers) {
                    // Update the modifiers
                    self.modifiers = modifiers;
                    self.egui_input.modifiers = self.modifiers;
                    // Push the event
                    self.egui_input.events.push(egui::Event::Key {
                        key,
                        physical_key: None,
                        pressed: false,
                        repeat: false,
                        modifiers: self.modifiers,
                    });
                }
            }

            _ => {}
        };

        false
    }

    /// Return the processed context
    pub fn context(&mut self) -> egui::Context {
        self.egui_ctx.clone()
    }

    /// Begin drawing the egui frame
    pub fn begin_frame(&mut self) {
        self.egui_ctx.begin_pass(self.egui_input.take());
    }

    /// Stop drawing the egui frame and return the full output
    pub fn end_frame(&mut self) -> anyhow::Result<egui::FullOutput> {
        let output = self.egui_ctx.end_pass();
        Ok(output)
    }

    /// Tessellate the egui frame
    pub fn tessellate(
        &self,
        full_output: &egui::FullOutput,
        pixels_per_point: f32,
    ) -> Vec<egui::ClippedPrimitive> {
        self.egui_ctx
            .tessellate(full_output.shapes.clone(), pixels_per_point)
    }
}
