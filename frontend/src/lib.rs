use crate::app::App;
use wasm_bindgen::{JsValue, UnwrapThrowExt, prelude::wasm_bindgen};
use winit::event_loop::EventLoop;

mod app;
mod request;
mod state;

#[wasm_bindgen(start)]
pub fn run() -> Result<(), JsValue> {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
    #[cfg(feature = "console_log")]
    console_log::init().expect_throw("Init Failed: Console");

    let event_loop = EventLoop::with_user_event()
        .build()
        .expect_throw("Init Failed: EventLoop");
    let mut app = App::new(&event_loop);
    event_loop.run_app(&mut app).map_err(|e| e.to_string())?;

    Ok(())
}
