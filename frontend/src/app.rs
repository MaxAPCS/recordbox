use crate::{boilerplate::ui, request::request};
use std::{
    borrow::Cow,
    sync::{
        Arc, RwLock,
        atomic::{AtomicBool, Ordering},
    },
};
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;
use web_sys::{
    console::{info_1, info_2},
    js_sys::{Array, JsString},
};

pub struct App {
    background: ui::Element,
    showtrackedit: AtomicBool,
    trackedit: ui::Element,
    tracks: Arc<RwLock<Vec<Track>>>,
}

#[derive(Clone)]
struct Track {
    id: String,
    element: ui::Element,
}

impl App {
    pub fn new() -> Self {
        let mut this = Self {
            // all images must be 1920 x 1080
            background: ui::Element {
                shape: ui::Trapezoid::default(),
                image: image::load_from_memory_with_format(
                    include_bytes!("assets/background.png"),
                    image::ImageFormat::Png,
                )
                .unwrap()
                .into_rgba8(),
            },
            showtrackedit: AtomicBool::new(false),
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

    fn get_tracks(&mut self) {
        let tracks = self.tracks.clone();
        spawn_local(async move {
            match request("/tracks", "GET", None).await {
                Ok(resp) => {
                    let mut tracks = tracks.write().unwrap();

                    let resp: Array<JsString> = resp.unchecked_into();

                    let image = image::load_from_memory_with_format(
                        include_bytes!("assets/album.png"),
                        image::ImageFormat::Png,
                    )
                    .unwrap()
                    .resize_exact(1920, 1080, image::imageops::FilterType::Triangle)
                    .into_rgba8();

                    tracks.clear();
                    for (i, id) in resp.into_iter().enumerate() {
                        tracks.push(Track {
                            id: String::from(id),
                            element: ui::Element {
                                shape: ui::Trapezoid::from_square(
                                    -0.84375 + 0.15 * (i as f32),
                                    0.20370,
                                    0.078125,
                                ),
                                image: image.clone(),
                            },
                        });
                    }
                }
                Err(_) => {}
            }
        });
    }
}

impl ui::Scene for App {
    fn elements(&self) -> Vec<Cow<'_, ui::Element>> {
        let tracks = self.tracks.try_read();
        let tracklen = if let Ok(tracks) = &tracks {
            tracks.len()
        } else {
            0
        };

        let mut out = Vec::with_capacity(2 + tracklen);
        out.push(Cow::Borrowed(&self.background));
        if let Ok(tracks) = tracks {
            for Track { element, .. } in tracks.iter() {
                out.push(Cow::Owned(element.clone()));
            }
        }

        if self.showtrackedit.load(Ordering::Relaxed) {
            out.push(Cow::Borrowed(&self.trackedit));
        }

        out
    }

    fn update(&self) {}

    #[allow(unused)]
    fn handle_key(&self, key: winit::keyboard::KeyCode, pressed: bool) {}

    fn handle_mouse(&self, button: winit::event::MouseButton, (mx, my): (f32, f32), pressed: bool) {
        if !pressed || button != winit::event::MouseButton::Left {
            return;
        }

        if self.showtrackedit.load(Ordering::Acquire) && (mx.abs() > 0.5 || my.abs() > 0.5) {
            self.showtrackedit.store(false, Ordering::Relaxed);
            return;
        }

        let Ok(tracks) = self.tracks.try_read() else {
            return;
        };
        for track in tracks.iter().rev() {
            if track.element.shape.hit_test(mx, my) {
                info_2(&"Editing: ".into(), &(track.id.clone().into()));
                self.showtrackedit.store(true, Ordering::Relaxed);
                return;
            }
        }
    }
}
