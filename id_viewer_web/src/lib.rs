mod winit_system;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
use winit::platform::web::WindowBuilderExtWebSys;
use winit::dpi::PhysicalSize;
use winit::{event::Event, event::WindowEvent, event_loop::{ControlFlow, EventLoop}, window::WindowBuilder};

use id_core::world::World;
use id_map_format::Wad;
use id_core::renderer::{debug_window, overlay_window, WindowRunner};
use id_core::renderer::{egui_window, main_user_context, main_window};
use id_core::Stopwatch;
use ultraviolet::UVec2;
use std::cell::RefCell;
use std::rc::Rc;

fn create_runner(window: &winit::window::Window, wad_bytes: Vec<u8>) -> WindowRunner<'static, impl id_core::renderer::helpers::window::UserContext> {
    let wad = Wad::new(wad_bytes).expect("Failed to parse IWAD");
    let world = World::new(wad, vec![], "E1M1").expect("Failed to create world");
    let user_context = main_user_context(Rc::new(RefCell::new(world)));
    let window_setup = egui_window(overlay_window((main_window(), debug_window())));
    let size = window.inner_size();
    pollster::block_on(WindowRunner::from_system_window(
        window,
        UVec2 { x: size.width, y: size.height },
        user_context,
        window_setup,
    )).expect("Failed to create WindowRunner")
}

#[wasm_bindgen]
pub fn run(canvas_id: &str) {
    console_error_panic_hook::set_once();

    let window = web_sys::window().expect("no global `window`");
    let document = window.document().expect("no document on window");
    let canvas = document
        .get_element_by_id(canvas_id)
        .expect("Canvas not found")
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .unwrap();

    let event_loop = EventLoop::new();
    let winit_window = WindowBuilder::new()
        .with_canvas(Some(canvas))
        .with_inner_size(PhysicalSize::new(800u32, 600u32))
        .build(&event_loop)
        .unwrap();

    use wasm_bindgen::JsCast;
    let body = document.body().unwrap();
    let drop_zone = document.create_element("div").unwrap();
    drop_zone.set_attribute("style", "position:absolute;left:0;top:0;right:0;bottom:0;border:2px dashed #888;").unwrap();
    body.append_child(&drop_zone).unwrap();
    drop_zone.set_inner_html("Drop WAD file here");

    let wad_bytes = Rc::new(RefCell::new(None));
    {
        let wad_bytes_clone = wad_bytes.clone();
        let closure = Closure::<dyn FnMut(web_sys::DragEvent)>::wrap(Box::new(move |e| {
            e.prevent_default();
            if let Some(items) = e.data_transfer().and_then(|dt| dt.files()) {
                if let Some(file) = items.get(0) {
                    let fr = web_sys::FileReader::new().unwrap();
                    let fr_c = fr.clone();
                    let wad_bytes_c = wad_bytes_clone.clone();
                    let onload = Closure::<dyn FnMut(_)>::wrap(Box::new(move |_| {
                        let array = js_sys::Uint8Array::new(&fr_c.result().unwrap());
                        let mut buf = vec![0u8; array.length() as usize];
                        array.copy_to(&mut buf[..]);
                        *wad_bytes_c.borrow_mut() = Some(buf);
                    }));
                    fr.set_onload(Some(onload.as_ref().unchecked_ref()));
                    onload.forget();
                    fr.read_as_array_buffer(&file).unwrap();
                }
            }
        }));
        drop_zone.add_event_listener_with_callback("drop", closure.as_ref().unchecked_ref()).unwrap();
        closure.forget();
        let closure = Closure::<dyn FnMut(web_sys::DragEvent)>::wrap(Box::new(|e| e.prevent_default()));
        drop_zone.add_event_listener_with_callback("dragover", closure.as_ref().unchecked_ref()).unwrap();
        closure.forget();
    }

    spawn_local(async move {
        loop {
            if let Some(bytes) = wad_bytes.borrow_mut().take() {
                let mut runner = create_runner(&winit_window, bytes);
                let mut stopwatch = Stopwatch::new();
                let tick_rate_ms = 1000/60;
                event_loop.set_control_flow(ControlFlow::Poll);
                event_loop.run(move |event, elwt| {
                    elwt.set_control_flow(ControlFlow::Poll);
                    match event {
                        Event::AboutToWait => {},
                        Event::WindowEvent { event, .. } => match event {
                            WindowEvent::Resized(size) => {
                                runner.handle_event(winit_system::to_system_event(WindowEvent::Resized(size))).unwrap();
                            }
                            _ => {
                                if let Some(sys) = winit_system::to_system_event(event) {
                                    runner.handle_event(sys).unwrap();
                                }
                            }
                        },
                        Event::RedrawEventsCleared => {
                            let delta = stopwatch.lap();
                            let delta_ticks = delta.as_millis()/tick_rate_ms;
                            stopwatch.rewind(std::time::Duration::from_millis((delta.as_millis()%tick_rate_ms) as u64));
                            if delta_ticks>0 {
                                runner.think(std::time::Duration::from_millis((delta_ticks*tick_rate_ms) as u64)).unwrap();
                            }
                            runner.draw(delta).unwrap();
                        }
                        Event::LoopExiting => { elwt.set_control_flow(ControlFlow::Exit); },
                        _ => {}
                    }
                });
                break;
}
            gloo_timers::future::TimeoutFuture::new(100).await;
        }
    });
}

