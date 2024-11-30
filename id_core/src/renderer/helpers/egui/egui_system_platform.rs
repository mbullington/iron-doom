use egui::{Key, Modifiers, Pos2};

use super::super::system::{
    SystemEvent,
    SystemKeycode::{self, *},
    SystemMouseButton,
};

/// A trait that adds a method to convert to an egui key
pub trait ToEguiKey {
    /// Convert the struct to an egui key
    fn to_egui_key(&self) -> Option<egui::Key>;
}

impl ToEguiKey for SystemKeycode {
    fn to_egui_key(&self) -> Option<egui::Key> {
        Some(match *self {
            Left => Key::ArrowLeft,
            Up => Key::ArrowUp,
            Right => Key::ArrowRight,
            Down => Key::ArrowDown,
            Escape => Key::Escape,
            Tab => Key::Tab,
            Backspace => Key::Backspace,
            Space => Key::Space,
            Return => Key::Enter,
            Insert => Key::Insert,
            Home => Key::Home,
            Delete => Key::Delete,
            End => Key::End,
            PageDown => Key::PageDown,
            PageUp => Key::PageUp,
            Kp0 | Num0 => Key::Num0,
            Kp1 | Num1 => Key::Num1,
            Kp2 | Num2 => Key::Num2,
            Kp3 | Num3 => Key::Num3,
            Kp4 | Num4 => Key::Num4,
            Kp5 | Num5 => Key::Num5,
            Kp6 | Num6 => Key::Num6,
            Kp7 | Num7 => Key::Num7,
            Kp8 | Num8 => Key::Num8,
            Kp9 | Num9 => Key::Num9,
            A => Key::A,
            B => Key::B,
            C => Key::C,
            D => Key::D,
            E => Key::E,
            F => Key::F,
            G => Key::G,
            H => Key::H,
            I => Key::I,
            J => Key::J,
            K => Key::K,
            L => Key::L,
            M => Key::M,
            N => Key::N,
            O => Key::O,
            P => Key::P,
            Q => Key::Q,
            R => Key::R,
            S => Key::S,
            T => Key::T,
            U => Key::U,
            V => Key::V,
            W => Key::W,
            X => Key::X,
            Y => Key::Y,
            Z => Key::Z,
            _ => {
                return None;
            }
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

            // Handle a key being pressed
            SystemEvent::KeyDown { keycode, mods, .. } => {
                // Convert the keycode to text.
                if self.egui_ctx.wants_keyboard_input() {
                    if let Some(text) = keycode.to_text() {
                        let mut text = text.to_string();
                        if mods.shift {
                            // Set the text to lowercase if shift is not pressed.
                            text = text.to_uppercase();
                        }

                        self.egui_input.events.push(egui::Event::Text(text));
                    }
                }

                // Convert the keycode to an egui key
                if let Some(key) = keycode.to_egui_key() {
                    // Update the modifiers
                    self.modifiers = Modifiers {
                        alt: mods.alt,
                        ctrl: mods.ctrl,
                        shift: mods.shift,
                        mac_cmd: mods.mac_cmd,
                        command: mods.command,
                    };

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
                // Convert the keycode to an egui key
                if let Some(key) = keycode.to_egui_key() {
                    // Update the modifiers
                    self.modifiers = Modifiers {
                        alt: mods.alt,
                        ctrl: mods.ctrl,
                        shift: mods.shift,
                        mac_cmd: mods.mac_cmd,
                        command: mods.command,
                    };
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
