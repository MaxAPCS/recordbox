use crate::boilerplate::{render::RenderState, ui};
use std::sync::Arc;
use wasm_bindgen::{JsCast, UnwrapThrowExt};
use wgpu::SurfaceError;
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalPosition,
    event::*,
    event_loop::{ActiveEventLoop, EventLoop},
    keyboard::PhysicalKey,
    window::Window,
};

pub struct AppWindow {
    proxy: winit::event_loop::EventLoopProxy<RenderState>,
    state: Option<RenderState>,
    scene: Arc<dyn ui::Scene>,
    window: Option<Arc<Window>>,
    mouse: Option<PhysicalPosition<f64>>,
}

impl AppWindow {
    pub fn new(event_loop: &EventLoop<RenderState>, scene: Arc<dyn ui::Scene>) -> Self {
        Self {
            state: None,
            proxy: event_loop.create_proxy(),
            scene,
            window: None,
            mouse: None,
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
        self.window = Some(window.clone());
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
            WindowEvent::CursorMoved { position, .. } => self.mouse = Some(position),
            WindowEvent::MouseInput {
                state: button_state,
                button,
                ..
            } => {
                if let Some(mouse) = self.mouse
                    && let Some(window) = &self.window
                {
                    let size = window.inner_size();
                    let pos = (
                        2. * (mouse.x as f32) / (size.width as f32) - 1.,
                        1. - 2. * (mouse.y as f32) / (size.height as f32),
                    );
                    self.scene
                        .handle_mouse(button, pos, button_state.is_pressed())
                }
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(code),
                        state: key_state,
                        ..
                    },
                ..
            } => self.scene.handle_key(code, key_state.is_pressed()),
            _ => {}
        }
    }
}
