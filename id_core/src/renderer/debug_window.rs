use std::time::Duration;

use anyhow::Result;
use egui_console::{ConsoleBuilder, ConsoleEvent, ConsoleWindow};
use ultraviolet::UVec2;

use super::{
    helpers::window::{HasEguiUserContext, Window, WindowContext, WindowSetup},
    main_user_context::MainUserContext,
    system::{SystemEvent, SystemKeycode},
};

use MainUserContext as UC;

pub fn debug_window() -> impl WindowSetup<UC> {
    move |context: &WindowContext<UC>, _size: UVec2| {
        let mut console = ConsoleBuilder::new()
            .prompt(">> ")
            .history_size(20)
            .tab_quote_character('\"')
            .build();

        let command_table_mut = console.command_table_mut();
        let cvars = &context.user_context.world.borrow().cvars;

        for name in cvars.keys() {
            command_table_mut.push(name.to_string());
        }

        Ok(Box::new(DebugWindow {
            console_active: false,
            autofocus: false,
            console,
        }))
    }
}

pub struct DebugWindow {
    console_active: bool,
    autofocus: bool,

    console: ConsoleWindow,
}

impl Window<UC> for DebugWindow {
    fn handle_event(
        &mut self,
        _context: &mut WindowContext<UC>,
        event: &super::system::SystemEvent,
    ) -> Result<bool> {
        if let SystemEvent::KeyDown { keycode, mods, .. } = event {
            if *keycode == SystemKeycode::T && mods.ctrl {
                self.console_active = !self.console_active;
                self.autofocus = self.console_active;
                return Ok(true);
            }
        }

        return Ok(false);
    }

    fn draw(
        &mut self,
        context: &mut WindowContext<UC>,
        _texture: &wgpu::Texture,
        _delta: Duration,
    ) -> Result<()> {
        let ui = &context.user_context.ui();
        let setup_time = &context.user_context.setup_time;

        // Update camera.
        egui::Area::new("Setup".into()).show(ui, |ui| {
            ui.label(format!("Setup: {:?}ms", setup_time.as_millis()));
        });

        // Open console.
        if self.console_active {
            egui::Window::new("Console").show(ui, |ui| {
                let console_response = self.console.draw(ui);

                if let ConsoleEvent::Command(command) = console_response {
                    let mut error = || {
                        self.console
                            .write(&format!("Invalid command: '{}'", command));
                        self.console.prompt();
                    };

                    // Commands should have look like:
                    // [cvar_name] [value]
                    let parts = command.split_whitespace().collect::<Vec<_>>();
                    if parts.len() != 2 {
                        error();
                        return;
                    }

                    let cvar_name = parts[0];
                    let value = parts[1];

                    let mut world = context.user_context.world.borrow_mut();
                    if let Some(cvar) = world.cvars.get_mut(cvar_name) {
                        if let Err(e) = cvar.value.set_from_str(value) {
                            self.console.write(&format!("Error: {}", e));
                        } else {
                            self.console
                                .write(&format!("{} set to {}", cvar_name, value));
                        }

                        self.console.prompt();
                    } else {
                        error();
                    }
                }
            });

            if self.autofocus {
                self.autofocus = false;
                ui.memory_mut(|mem| mem.request_focus(self.console.id));
            }
        }

        Ok(())
    }
}
