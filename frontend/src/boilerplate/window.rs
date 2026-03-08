use crate::boilerplate::{render::RenderState, ui};
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

pub struct AppWindow {
    proxy: winit::event_loop::EventLoopProxy<RenderState>,
    state: Option<RenderState>,
    scene: Arc<dyn ui::Scene>,
}

impl AppWindow {
    pub fn new(event_loop: &EventLoop<RenderState>, scene: Arc<dyn ui::Scene>) -> Self {
        Self {
            state: None,
            proxy: event_loop.create_proxy(),
            scene,
        }
    }
}

impl ApplicationHandler<RenderState> for AppWindow {
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
        let scene = self.scene.clone();
        wasm_bindgen_futures::spawn_local(async move {
            proxy
                .send_event(RenderState::initialize(window, scene).await)
                .ok()
                .expect("Init Failed: Event Loop Closed");
        });
    }

    fn user_event(&mut self, _event_loop: &ActiveEventLoop, mut event: RenderState) {
        event.resize();
        self.state = Some(event);
    }

    fn window_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        let Some(state) = &mut self.state else { return };
        match event {
            WindowEvent::RedrawRequested => match state.render() {
                Err(SurfaceError::Lost | SurfaceError::Outdated) => {
                    state.resize();
                }
                Err(e) => {
                    panic!("Unable to render {}", e);
                }
                Ok(_) => {}
            },
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
