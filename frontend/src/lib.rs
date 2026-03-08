use crate::app::App;
use std::sync::Arc;
use wasm_bindgen::{JsValue, UnwrapThrowExt, prelude::wasm_bindgen};
use winit::event_loop::EventLoop;

mod app;
mod boilerplate;
mod request;

#[wasm_bindgen(start)]
pub fn run() -> Result<(), JsValue> {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
    #[cfg(feature = "console_log")]
    console_log::init().expect_throw("Init Failed: Console");

    let event_loop = EventLoop::with_user_event()
        .build()
        .expect_throw("Init Failed: EventLoop");
    let mut appwindow = boilerplate::window::AppWindow::new(&event_loop, Arc::new(App::new()));
    event_loop
        .run_app(&mut appwindow)
        .map_err(|e| e.to_string())?;

    Ok(())
}
