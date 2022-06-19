// #![allow(unused)]
#![windows_subsystem = "windows"]

use eframe::epaint::Color32;
use mouse_rs::{types::keys::Keys, Mouse};
use saved_config::{load_config, save_config, SavedState};
use std::sync::Mutex;
use std::time::Duration;
use std::{sync::Arc, thread::sleep};

use eframe::egui::{self, Button, RichText};

use mki_fork::{remove_key_bind, Keyboard};

mod saved_config;

fn main() {
    let config = if let Ok(Some(config)) = load_config() {
        config
    } else {
        SavedState::default()
    };

    let native_options = eframe::NativeOptions {
        always_on_top: config.always_on_top,
        min_window_size: Some([1., 1.].into()),
        initial_window_size: Some([200., 80.].into()),
        initial_window_pos: Some([0., 0.].into()),
        resizable: false,
        ..eframe::NativeOptions::default()
    };
    eframe::run_native(
        "Click",
        native_options,
        Box::new(move |_cc| {
            Box::new(Click::new(
                config.key_bind,
                Arc::new(config.freq),
                config.always_on_top,
            ))
        }),
    );
}

#[derive(Clone, Copy)]
enum Status {
    Ready,
    Running,
}

impl Status {
    fn should_stop(&self) -> bool {
        match self {
            Status::Ready => true,
            _ => false,
        }
    }

    fn switch(&mut self) {
        match self {
            Status::Ready => *self = Status::Running,
            Status::Running => *self = Status::Ready,
        }
    }
}

struct Click {
    key_bind: Keyboard,
    freq: Arc<u64>,
    mouse: Arc<Mouse>,
    status: Status,
    job: Arc<Mutex<Status>>,
    dirty: bool,
    always_on_top: bool,
}

impl Click {
    fn new(key_bind: Keyboard, freq: Arc<u64>, always_on_top: bool) -> Self {
        Self {
            key_bind,
            freq,
            always_on_top,
            mouse: Arc::new(Mouse::new()),
            status: Status::Ready,
            job: Arc::new(Mutex::new(Status::Ready)),
            dirty: false,
        }
    }

    fn get_job_once(&self) -> Status {
        *self.job.lock().unwrap()
    }
}

impl eframe::App for Click {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered_justified(|ui| {
                ui.spacing_mut().item_spacing = [5., 10.].into();
                if self.dirty {
                    if let Ok(_) = save_config(SavedState {
                        key_bind: self.key_bind,
                        freq: *self.freq,
                        always_on_top: self.always_on_top,
                    }) {
                        self.dirty = false;
                    }
                }

                // if let Some(freq) = Arc::get_mut(&mut self.freq) {
                //     let response = ui.add(egui::Slider::new(freq, 1..=100));
                //     if response.drag_released() {
                //         self.dirty = true;
                //     }
                // }

                ui.horizontal(|ui| {
                    if let Some(freq) = Arc::get_mut(&mut self.freq) {
                        let response = ui.add(egui::Slider::new(freq, 1..=100));
                        if response.drag_released() {
                            self.dirty = true;
                        }

                        if self.always_on_top {
                            let res = ui.add(Button::new("TOP").fill(Color32::GRAY));
                            if res.clicked() {
                                self.always_on_top = !self.always_on_top;
                                self.dirty = true;
                            }
                        } else {
                            if ui.button("TOP").clicked() {
                                self.always_on_top = !self.always_on_top;
                                self.dirty = true;
                            }
                        }
                        // if Hotkey::new(&mut self.key_bind).ui(ui).changed() {
                        //     println!("Rebinded!");
                        // }
                    }
                });

                match self.status {
                    Status::Ready => {
                        if ui.button(text("Click to start service...")).clicked() {
                            let mouse = self.mouse.clone();
                            let freq = self.freq.clone();
                            let job = self.job.clone();
                            let job_status = self.get_job_once();
                            self.key_bind.bind(move |_| match job_status {
                                Status::Ready => {
                                    job.lock().unwrap().switch();
                                    click(mouse.to_owned(), freq.to_owned(), job.to_owned());
                                }
                                Status::Running => job.lock().unwrap().switch(),
                            });
                            self.status.switch();
                        }
                    }
                    Status::Running => {
                        let response = match self.get_job_once() {
                            Status::Ready => ui.add(Button::new(text("Inactive"))),
                            Status::Running => {
                                ui.add(Button::new(text("Active")).fill(Color32::LIGHT_GREEN))
                            }
                        };

                        if response.clicked() {
                            remove_key_bind(self.key_bind);
                            *self.job.lock().unwrap() = Status::Ready;
                            self.status.switch();
                        }
                    }
                }
            });
        });
    }
}

fn click(mouse: Arc<Mouse>, freq: Arc<u64>, job: Arc<Mutex<Status>>) {
    // let time = std::time::Instant::now();
    (0..).find(|i| {
        mouse.click(&Keys::LEFT).expect("Unable to click");
        sleep(Duration::from_millis(1000 / *freq));

        (i + 1) % (*freq / 10 + 1) == 0 && job.lock().unwrap().should_stop()
    });
}

fn text(content: &str) -> RichText {
    RichText::new(content).size(20.)
}
