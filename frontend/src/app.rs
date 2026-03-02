use crate::state::State;
use std::sync::Arc;
use wasm_bindgen::{JsCast, UnwrapThrowExt};
use wgpu::SurfaceError;
use winit::{
    application::ApplicationHandler,
    event::*,
    event_loop::{ActiveEventLoop, EventLoop},
    keyboard::PhysicalKey,
    window::Window,
};

pub struct App {
    proxy: winit::event_loop::EventLoopProxy<State>,
    state: Option<State>,
}

impl App {
    pub fn new(event_loop: &EventLoop<State>) -> Self {
        Self {
            state: None,
            proxy: event_loop.create_proxy(),
        }
    }
}

impl ApplicationHandler<State> for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        use winit::platform::web::WindowAttributesExtWebSys;

        let canvas = wgpu::web_sys::window()
            .expect_throw("Init Failed: Window")
            .document()
            .expect_throw("Init Failed: Document")
            .get_element_by_id("canvas")
            .expect_throw("Init Failed: Canvas")
            .unchecked_into();
        let window_attributes = Window::default_attributes().with_canvas(Some(canvas));

        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());
        let proxy = self.proxy.clone();
        wasm_bindgen_futures::spawn_local(async move {
            proxy
                .send_event(State::new(window).await.expect_throw("Init Failed: State"))
                .ok()
                .expect("Init Failed: Event Loop Closed");
        });
    }

    fn user_event(&mut self, _event_loop: &ActiveEventLoop, mut event: State) {
        event.resize();
        self.state = Some(event);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        let Some(state) = &mut self.state else { return };
        match event {
            WindowEvent::RedrawRequested => {
                state.update();
                match state.render() {
                    Err(SurfaceError::Lost | SurfaceError::Outdated) => {
                        state.resize();
                    }
                    Err(e) => {
                        panic!("Unable to render {}", e);
                    }
                    Ok(_) => {}
                }
            }
            WindowEvent::MouseInput {
                state: button_state,
                button,
                ..
            } => state.handle_mouse(button, button_state.is_pressed()),
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(code),
                        state: key_state,
                        ..
                    },
                ..
            } => state.handle_key(code, key_state.is_pressed()),
            _ => {}
        }
    }
}
