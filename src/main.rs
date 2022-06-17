// #![allow(unused)]

use mouse_rs::{types::keys::Keys, Mouse};
use saved_config::{load_config, save_config, SavedState};
use std::sync::Mutex;
use std::time::Duration;
use std::{sync::Arc, thread::sleep};

use eframe::egui;

use mki::{remove_key_bind, Keyboard};

mod saved_config;

fn main() {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "Click",
        native_options,
        Box::new(|_cc| Box::new(Click::new())),
    );
}

enum Status {
    Ready,
    Running,
}

struct Click {
    key_bind: Keyboard,
    freq: Arc<u64>,
    mouse: Arc<Mouse>,
    status: Status,
    job: Arc<Mutex<Status>>,
    dirty: bool,
}

impl Click {
    fn new() -> Self {
        let fall_back = Self {
            key_bind: Keyboard::F5,
            freq: Arc::new(10),
            mouse: Arc::new(Mouse::new()),
            status: Status::Ready,
            job: Arc::new(Mutex::new(Status::Ready)),
            dirty: false,
        };
        if let Ok(Some(config)) = load_config() {
            Self {
                key_bind: config.key_bind,
                freq: Arc::new(config.freq),
                ..fall_back
            }
        } else {
            fall_back
        }
    }
}

impl eframe::App for Click {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            if self.dirty {
                if let Ok(_) = save_config(SavedState {
                    key_bind: self.key_bind,
                    freq: *self.freq,
                }) {
                    self.dirty = false;
                }
            }

            if let Some(freq) = Arc::get_mut(&mut self.freq) {
                let response = ui.add(egui::Slider::new(freq, 1..=100).vertical());
                if response.drag_released() {
                    self.dirty = true;
                }
            }

            match self.status {
                Status::Ready => {
                    if ui.button("Click to run...").clicked() {
                        let mouse = self.mouse.clone();
                        let freq = self.freq.clone();
                        self.key_bind
                            .bind(move |_| click(mouse.clone(), freq.clone()));
                        self.status = Status::Running;
                    }
                }
                Status::Running => {
                    if ui.button("Running").clicked() {
                        remove_key_bind(self.key_bind);
                        self.status = Status::Ready;
                    }
                }
            }
        });
    }
}

fn click(mouse: Arc<Mouse>, freq: Arc<u64>) {
    (0..*freq).for_each(|_| {
        mouse.press(&Keys::LEFT).expect("Unable to press button");
        mouse
            .release(&Keys::LEFT)
            .expect("Unable to release button");
        sleep(Duration::from_millis(1000 / *freq));
    });
}
