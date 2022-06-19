// #![allow(unused)]
#![windows_subsystem = "windows"]

use std::sync::Mutex;
use std::time::Duration;
use std::{sync::Arc, thread::sleep};

use eframe::egui::{self, Button, RichText};
use eframe::epaint::Color32;

// use mki_fork::{bind_key, Action, InhibitEvent};
use mki_fork::{remove_key_bind, Keyboard};
use mouse_rs::{types::keys::Keys, Mouse};

mod saved_config;

use saved_config::{load_config, save_config, SavedState};

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
    status: Status,
    job: Arc<Mutex<Status>>,
    dirty: bool,
    always_on_top: bool,
}

impl Click {
    const UPPER_BOUND: u64 = 200;

    fn new(key_bind: Keyboard, freq: Arc<u64>, always_on_top: bool) -> Self {
        Self {
            key_bind,
            freq,
            always_on_top,
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
                    match save_config(SavedState {
                        key_bind: self.key_bind,
                        freq: *self.freq,
                        always_on_top: self.always_on_top,
                    }) {
                        Ok(_) => {
                            self.dirty = false;
                        }
                        Err(e) => eprint!("{:?}", e),
                    }
                }

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
                    }
                });

                match self.status {
                    Status::Ready => {
                        if ui.button(text("Click to start service...")).clicked() {
                            let freq = self.freq.clone();
                            let job = self.job.clone();
                            let job_status = self.get_job_once();
                            //这个会在按下的时候重复输入导致“键盘连点”
                            // match job_status {
                            //     Status::Ready => {
                            //         bind_key(
                            //             self.key_bind,
                            //             Action {
                            //                 callback: Box::new(move |_event, _state| {
                            //                     job.lock().unwrap().switch();
                            //                     click(freq.to_owned(), job.to_owned());
                            //                 }),
                            //                 inhibit: InhibitEvent::Yes,
                            //                 sequencer: false,
                            //                 defer: true,
                            //             },
                            //         );
                            //     }
                            //     Status::Running => bind_key(
                            //         self.key_bind,
                            //         Action {
                            //             callback: Box::new(move |_event, _state| {
                            //                 job.lock().unwrap().switch()
                            //             }),
                            //             inhibit: InhibitEvent::Yes,
                            //             sequencer: false,
                            //             defer: true,
                            //         },
                            //     ),
                            // }
                            self.key_bind.bind(move |_| match job_status {
                                Status::Ready => {
                                    job.lock().unwrap().switch();
                                    click(freq.to_owned(), job.to_owned());
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

fn click(freq: Arc<u64>, job: Arc<Mutex<Status>>) {
    let mouse = Mouse::new();
    // let time = std::time::Instant::now();
    (0..Click::UPPER_BOUND).find(|i| {
        mouse.click(&Keys::LEFT).expect("Unable to click");
        sleep(Duration::from_millis(1000 / *freq));

        (i + 1) % (*freq / 10 + 1) == 0 && job.lock().unwrap().should_stop()
    });
}

fn text(content: &str) -> RichText {
    RichText::new(content).size(20.)
}
