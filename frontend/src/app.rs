use crate::{boilerplate::ui, request::request};
use std::sync::{Arc, RwLock};
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;
use web_sys::js_sys::{Array, JsString};

pub struct App {
    background: ui::Element,
    trackedit: ui::Element,
    tracks: Arc<RwLock<Vec<String>>>,
}

impl App {
    pub fn new() -> Self {
        let this = Self {
            // all images must be 1920 x 1080, .resize_exact(nwidth, nheight, filter)
            background: ui::Element {
                shape: ui::Trapezoid::default(),
                image: image::load_from_memory_with_format(
                    include_bytes!("assets/background.png"),
                    image::ImageFormat::Png,
                )
                .unwrap()
                .into_rgba8(),
            },
            trackedit: ui::Element {
                shape: ui::Trapezoid::default(),
                image: image::load_from_memory_with_format(
                    include_bytes!("assets/track.png"),
                    image::ImageFormat::Png,
                )
                .unwrap()
                .into_rgba8(),
            },
            tracks: Arc::new(RwLock::new(Vec::new())),
        };
        this.get_tracks();
        this
    }

    fn get_tracks(&self) {
        let tracks = self.tracks.clone();
        spawn_local(async move {
            match request("/tracks", "GET", None).await {
                Ok(resp) => {
                    let mut tracks = tracks.write().unwrap();
                    let resp: Array<JsString> = resp.unchecked_into();
                    for track in resp {
                        tracks.push(track.into());
                    }
                }
                Err(_) => {}
            }
        });
    }
}

impl ui::Scene for App {
    fn update(&self) {}

    fn elements(&self) -> Vec<&ui::Element> {
        vec![&self.background, &self.trackedit]
    }
}
