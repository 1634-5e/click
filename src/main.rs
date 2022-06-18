// #![allow(unused)]
#![feature(mutex_unlock)]

use eframe::epaint::Color32;
use egui_hotkey::Hotkey;
use mouse_rs::{types::keys::Keys, Mouse};
use saved_config::{load_config, save_config, SavedState};
use std::sync::Mutex;
use std::time::Duration;
use std::{sync::Arc, thread::sleep};

use eframe::egui::{self, Button, Key, RichText};

use mki::{remove_key_bind, Keyboard};

mod saved_config;

fn main() {
    let native_options = eframe::NativeOptions {
        min_window_size: Some([1., 1.].into()),
        initial_window_size: Some([200., 80.].into()),
        ..eframe::NativeOptions::default()
    };
    eframe::run_native(
        "Click",
        native_options,
        Box::new(|_cc| Box::new(Click::new())),
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

    fn get_job_once(&self) -> Status {
        let job_guard = self.job.lock().unwrap();
        let job_status = *job_guard;
        Mutex::unlock(job_guard);
        job_status
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
                    }) {
                        self.dirty = false;
                    }
                }

                if let Some(freq) = Arc::get_mut(&mut self.freq) {
                    let response = ui.add(egui::Slider::new(freq, 1..=100));
                    if response.drag_released() {
                        self.dirty = true;
                    }
                }

                // ui.horizontal_centered(|ui| {
                //     if let Some(freq) = Arc::get_mut(&mut self.freq) {
                //         let response = ui.add(egui::Slider::new(freq, 1..=100));
                //         if response.drag_released() {
                //             self.dirty = true;
                //         }
                //         // if Hotkey::new(&mut self.key_bind).ui(ui).changed() {
                //         //     println!("Rebinded!");
                //         // }
                //     }
                // });

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

        (i + 1) % (*freq / 10) == 0 && job.lock().unwrap().should_stop()
    });
}

fn text(content: &str) -> RichText {
    RichText::new(content).size(20.)
}

//...
// fn convert_key(key: Keyboard) -> Option<Key> {
//     match key {
//         Keyboard::A => Some(Key::A),
//         Keyboard::B => Some(Key::B),
//         Keyboard::C => Some(Key::C),
//         Keyboard::D => Some(Key::D),
//         Keyboard::E => Some(Key::E),
//         Keyboard::F => Some(Key::F),
//         Keyboard::G => Some(Key::G),
//         Keyboard::H => Some(Key::H),
//         Keyboard::I => Some(Key::I),
//         Keyboard::J => Some(Key::J),
//         Keyboard::K => Some(Key::K),
//         Keyboard::L => Some(Key::L),
//         Keyboard::M => Some(Key::M),
//         Keyboard::N => Some(Key::N),
//         Keyboard::O => Some(Key::O),
//         Keyboard::P => Some(Key::P),
//         Keyboard::Q => Some(Key::Q),
//         Keyboard::R => Some(Key::R),
//         Keyboard::S => Some(Key::S),
//         Keyboard::T => Some(Key::T),
//         Keyboard::U => Some(Key::U),
//         Keyboard::V => Some(Key::V),
//         Keyboard::W => Some(Key::W),
//         Keyboard::X => Some(Key::X),
//         Keyboard::Y => Some(Key::Y),
//         Keyboard::Z => Some(Key::Z),

//         Keyboard::Number0 | Keyboard::Numpad0 => Some(Key::Num0),
//         Keyboard::Number1 | Keyboard::Numpad1 => Some(Key::Num1),
//         Keyboard::Number2 | Keyboard::Numpad2 => Some(Key::Num2),
//         Keyboard::Number3 | Keyboard::Numpad3 => Some(Key::Num3),
//         Keyboard::Number4 | Keyboard::Numpad4 => Some(Key::Num4),
//         Keyboard::Number5 | Keyboard::Numpad5 => Some(Key::Num5),
//         Keyboard::Number6 | Keyboard::Numpad6 => Some(Key::Num6),
//         Keyboard::Number7 | Keyboard::Numpad7 => Some(Key::Num7),
//         Keyboard::Number8 | Keyboard::Numpad8 => Some(Key::Num8),
//         Keyboard::Number9 | Keyboard::Numpad9 => Some(Key::Num9),

//         Keyboard::Left => Some(Key::ArrowLeft),
//         Keyboard::Up => Some(Key::ArrowUp),
//         Keyboard::Right => Some(Key::ArrowRight),
//         Keyboard::Down => Some(Key::ArrowDown),

//         Keyboard::Escape => Some(Key::Escape),
//         Keyboard::Tab => Some(Key::Tab),
//         Keyboard::BackSpace => Some(Key::Backspace),
//         Keyboard::Enter => Some(Key::Enter),
//         Keyboard::Space => Some(Key::Space),

//         Keyboard::Insert => Some(Key::Insert),
//         Keyboard::Delete => Some(Key::Delete),
//         Keyboard::Home => Some(Key::Home),
//         Keyboard::PageUp => Some(Key::PageUp),
//         Keyboard::PageDown => Some(Key::PageDown),
//         _ => None,
//     }
// }
