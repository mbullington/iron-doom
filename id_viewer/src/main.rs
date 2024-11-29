extern crate pollster;
extern crate sdl2;

use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

use id_core::world::World;
use id_map_format::Wad;

use id_core::renderer::{debug_window, overlay_window, WindowRunner};
use id_core::renderer::{egui_window, main_user_context, main_window};
use id_core::Stopwatch;

mod sdl2_system;

use sdl2::event::Event;
use ultraviolet::UVec2;

use sdl2_system::ToSystemEventExt;

fn main() -> Result<(), String> {
    let sdl = sdl2::init()?;
    let sdl_window = sdl
        .video()?
        .window("DOOM", 800, 600)
        .position_centered()
        .resizable()
        .metal_view()
        .build()
        .map_err(|e| e.to_string())?;

    // Read IWAD.
    let wad = {
        let bytes = include_bytes!("../../freedoom1.wad").to_vec();
        Wad::new(bytes).expect("Failed to parse IWAD")
    };

    let world = World::new(wad, vec![], "E1M1").expect("Failed to create world");

    // Create our high level window that will handle events, thinking, and drawing.

    let user_context = main_user_context(Rc::new(RefCell::new(world)));
    let window = egui_window(overlay_window((main_window(), debug_window())));

    let drawable_size = sdl_window.drawable_size();
    let mut runner = match pollster::block_on(WindowRunner::from_system_window(
        &sdl_window,
        UVec2 {
            x: drawable_size.0,
            y: drawable_size.1,
        },
        user_context,
        window,
    )) {
        Ok(runner) => runner,
        Err(e) => panic!("Failed to create window runner! Reason: {:?}", e),
    };

    // For right now, we tick at 60fps; but its likely to be off slightly due to VSync.
    let tick_rate_ms = 1000 / 60;
    let mut stopwatch = Stopwatch::new();

    let mut event_pump = sdl.event_pump()?;
    'running: loop {
        // Step 1: Handle events.
        for event in event_pump.poll_iter() {
            if let Event::Quit { .. } = event {
                break 'running;
            }

            if let Some(system_event) = event.to_system_event(&sdl_window) {
                match runner.handle_event(system_event) {
                    Ok(_) => {}
                    Err(e) => panic!("Failed to handle event! Reason: {:?}", e),
                }
            }
        }

        // Step 2: Think.
        let delta = stopwatch.lap();
        let delta_ticks = delta.as_millis() / tick_rate_ms;
        stopwatch.rewind(Duration::from_millis(
            (delta.as_millis() % tick_rate_ms) as u64,
        ));

        if delta_ticks > 0 {
            match runner.think(Duration::from_millis((delta_ticks * tick_rate_ms) as u64)) {
                Ok(_) => {}
                Err(e) => panic!("Failed to think! Reason: {:?}", e),
            }
        }

        // Step 3: Draw.
        match runner.draw(delta) {
            Ok(_) => {}
            Err(e) => panic!("Failed to draw! Reason: {:?}", e),
        }
    }

    Ok(())
}
