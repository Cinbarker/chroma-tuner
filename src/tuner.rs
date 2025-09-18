//! Main tuner application and GUI
//!
//! Contains the primary TunerApp struct with pitch detection logic, signal filtering,
//! and the complete user interface including the tuning display and device selector.

use eframe::egui;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use crate::audio::{get_default_input_device_name, get_input_devices, AudioCapture, AudioData};
use crate::pitch::{frequency_to_note, Note, PitchDetector};

pub struct TunerApp {
    audio_data: Arc<Mutex<AudioData>>,
    pitch_detector: PitchDetector,
    current_note: Option<Note>,
    last_update: Instant,
    frequency_history: Vec<f32>,
    magnitude_history: Vec<f32>,
    max_history: usize,
    stability_threshold: f32,
    min_magnitude_threshold: f32,
    available_devices: Vec<(String, cpal::Device)>,
    current_device_name: String,
    audio_capture: Option<AudioCapture>,
    smoothed_cents: f32,
    cents_history: Vec<f32>,
    max_cents_history: usize,
    last_device_refresh: std::time::Instant,
    device_refresh_interval: std::time::Duration,
}

impl TunerApp {
    pub fn new(audio_data: Arc<Mutex<AudioData>>) -> Self {
        let buffer_size = 8192;

        let sample_rate = if let Ok(audio_data) = audio_data.lock() {
            audio_data.sample_rate
        } else {
            44100.0
        };

        let available_devices = get_input_devices().unwrap_or_default();
        let current_device_name =
            get_default_input_device_name().unwrap_or_else(|_| "Default".to_string());

        Self {
            audio_data,
            pitch_detector: PitchDetector::new(buffer_size, sample_rate),
            current_note: None,
            last_update: Instant::now(),
            frequency_history: Vec::new(),
            magnitude_history: Vec::new(),
            max_history: 8,
            stability_threshold: 3.0,
            min_magnitude_threshold: 0.08,
            available_devices,
            current_device_name,
            audio_capture: None,
            smoothed_cents: 0.0,
            cents_history: Vec::new(),
            max_cents_history: 8,
            last_device_refresh: std::time::Instant::now(),
            device_refresh_interval: std::time::Duration::from_secs(2),
        }
    }

    pub fn set_audio_capture(&mut self, audio_capture: AudioCapture) {
        self.audio_capture = Some(audio_capture);
    }

    pub fn switch_device(&mut self, device_name: String, device: cpal::Device) {
        self.current_device_name = device_name;
        if let Ok(new_capture) = AudioCapture::new_with_device(self.audio_data.clone(), device) {
            self.audio_capture = Some(new_capture);
            self.frequency_history.clear();
            self.magnitude_history.clear();
            self.cents_history.clear();
            self.current_note = None;
            self.smoothed_cents = 0.0;
        }
    }

    fn refresh_audio_devices(&mut self) {
        if self.last_device_refresh.elapsed() >= self.device_refresh_interval {
            if let Ok(devices) = get_input_devices() {
                if devices.len() != self.available_devices.len() || 
                   !devices.iter().all(|(name, _)| self.available_devices.iter().any(|(existing_name, _)| existing_name == name)) {
                    println!("Audio device list changed - refreshing");
                    self.available_devices = devices;
                    
                    if !self.available_devices.iter().any(|(name, _)| name == &self.current_device_name) {
                        if let Ok(default_name) = get_default_input_device_name() {
                            self.current_device_name = default_name;
                            println!("Current device no longer available, switched to default");
                        }
                    }
                }
            }
            self.last_device_refresh = std::time::Instant::now();
        }
    }

    fn update_pitch_detection(&mut self) {
        if let Ok(mut audio_data) = self.audio_data.try_lock() {
            if audio_data.has_new_data() {
                let samples = audio_data.get_samples();

                if let Some((frequency, magnitude)) = self.pitch_detector.detect_pitch(&samples) {
                    if magnitude < self.min_magnitude_threshold {
                        if self.last_update.elapsed().as_millis() > 400 {
                            self.current_note = None;
                            self.frequency_history.clear();
                            self.magnitude_history.clear();
                            self.cents_history.clear();
                            self.smoothed_cents = 0.0;
                        }
                        return;
                    }

                    self.frequency_history.push(frequency);
                    self.magnitude_history.push(magnitude);

                    if self.frequency_history.len() > self.max_history {
                        self.frequency_history.remove(0);
                        self.magnitude_history.remove(0);
                    }

                    if self.frequency_history.len() >= self.max_history {
                        let max_freq = self
                            .frequency_history
                            .iter()
                            .fold(f32::NEG_INFINITY, |a, &b| a.max(b));
                        let min_freq = self
                            .frequency_history
                            .iter()
                            .fold(f32::INFINITY, |a, &b| a.min(b));

                        let avg_magnitude = self.magnitude_history.iter().sum::<f32>()
                            / self.magnitude_history.len() as f32;
                        let magnitude_stable = self
                            .magnitude_history
                            .iter()
                            .all(|&m| (m - avg_magnitude).abs() < avg_magnitude * 0.5);

                        if (max_freq - min_freq) < self.stability_threshold
                            && magnitude_stable
                            && avg_magnitude > self.min_magnitude_threshold * 2.0
                        {
                            let mut sorted_freq = self.frequency_history.clone();
                            sorted_freq.sort_by(|a, b| a.partial_cmp(b).unwrap());
                            let median_freq = sorted_freq[sorted_freq.len() / 2];

                            let note = frequency_to_note(median_freq);

                            self.cents_history.push(note.cents_off);
                            if self.cents_history.len() > self.max_cents_history {
                                self.cents_history.remove(0);
                            }

                            if self.cents_history.len() >= self.max_cents_history {
                                let cents_max = self
                                    .cents_history
                                    .iter()
                                    .fold(f32::NEG_INFINITY, |a, &b| a.max(b));
                                let cents_min = self
                                    .cents_history
                                    .iter()
                                    .fold(f32::INFINITY, |a, &b| a.min(b));

                                if (cents_max - cents_min) < 20.0 {
                                    let target_cents = self.cents_history.iter().sum::<f32>()
                                        / self.cents_history.len() as f32;
                                    self.smoothed_cents =
                                        self.smoothed_cents * 0.8 + target_cents * 0.2;

                                    let mut smoothed_note = note.clone();
                                    smoothed_note.cents_off = self.smoothed_cents;

                                    self.current_note = Some(smoothed_note);
                                    self.last_update = Instant::now();
                                } else {
                                    self.current_note = None;
                                    self.cents_history.clear();
                                    self.smoothed_cents = 0.0;
                                }
                            }
                        }
                    }
                } else if self.last_update.elapsed().as_millis() > 500 {
                    self.current_note = None;
                    self.frequency_history.clear();
                    self.magnitude_history.clear();
                    self.cents_history.clear();
                    self.smoothed_cents = 0.0;
                }
            }
        }
    }

    fn draw_tuner_display(&self, ui: &mut egui::Ui) {
        let available_size = ui.available_size();
        let center = available_size / 2.0;

        ui.scope_builder(
            egui::UiBuilder::new().max_rect(egui::Rect::from_center_size(
                egui::pos2(center.x, center.y - 30.0),
                egui::vec2(200.0, 40.0),
            )),
            |ui| {
                ui.vertical_centered(|ui| {
                    if let Some(note) = &self.current_note {
                        ui.label(
                            egui::RichText::new(&note.name)
                                .size(36.0)
                                .color(egui::Color32::WHITE)
                                .strong(),
                        );
                        ui.label(
                            egui::RichText::new(format!("{:.1} Hz", note.frequency))
                                .size(12.0)
                                .color(egui::Color32::from_rgb(142, 142, 147)),
                        );
                    } else {
                        ui.label(
                            egui::RichText::new("♪ Play a note...")
                                .size(18.0)
                                .color(egui::Color32::from_rgb(142, 142, 147)),
                        );
                        ui.add_space(16.0);
                    }
                });
            },
        );

        let cents_off = if let Some(note) = &self.current_note {
            note.cents_off
        } else {
            0.0
        };
        self.draw_tuner_needle(ui, cents_off, center);

        ui.scope_builder(
            egui::UiBuilder::new().max_rect(egui::Rect::from_center_size(
                egui::pos2(center.x, center.y + 38.0),
                egui::vec2(120.0, 20.0),
            )),
            |ui| {
                ui.vertical_centered(|ui| {
                    if let Some(note) = &self.current_note {
                        let sign = if note.cents_off > 0.0 { "+" } else { "" };
                        let cents_text =
                            egui::RichText::new(format!("{}{:.0} cents", sign, note.cents_off))
                                .size(11.0)
                                .color(if note.cents_off.abs() < 5.0 {
                                    egui::Color32::from_rgb(48, 209, 88)
                                } else if note.cents_off.abs() < 20.0 {
                                    egui::Color32::from_rgb(255, 159, 10)
                                } else {
                                    egui::Color32::from_rgb(255, 69, 58)
                                });
                        ui.label(cents_text);
                    } else {
                        ui.label(egui::RichText::new("").size(11.0));
                    }
                });
            },
        );
    }

    fn draw_tuner_needle(&self, ui: &mut egui::Ui, cents_off: f32, center: egui::Vec2) {
        let painter = ui.painter();
        let needle_area = egui::Rect::from_center_size(
            egui::pos2(center.x, center.y + 5.0),
            egui::vec2(220.0, 20.0),
        );

        painter.rect_filled(needle_area, 10.0, egui::Color32::from_rgb(59, 59, 59));

        let center_x = needle_area.center().x;
        painter.line_segment(
            [
                egui::pos2(center_x, needle_area.top() + 3.0),
                egui::pos2(center_x, needle_area.bottom() - 3.0),
            ],
            egui::Stroke::new(1.5, egui::Color32::from_rgb(99, 99, 102)),
        );

        let max_cents = 50.0;
        let normalized_cents = (cents_off / max_cents).clamp(-1.0, 1.0);
        let needle_x = center_x + normalized_cents * (needle_area.width() / 2.0 - 10.0);

        let needle_color = if cents_off.abs() < 5.0 {
            egui::Color32::from_rgb(48, 209, 88)
        } else if cents_off.abs() < 20.0 {
            egui::Color32::from_rgb(255, 159, 10)
        } else {
            egui::Color32::from_rgb(255, 69, 58)
        };

        if cents_off != 0.0 || self.current_note.is_some() {
            painter.circle_filled(
                egui::pos2(needle_x, needle_area.center().y),
                6.0,
                needle_color,
            );
        }

        for i in [-4i32, -2, 2, 4] {
            let mark_cents = i as f32 * 12.5;
            let mark_x = center_x + (mark_cents / max_cents) * (needle_area.width() / 2.0 - 10.0);
            painter.line_segment(
                [
                    egui::pos2(mark_x, needle_area.center().y - 3.0),
                    egui::pos2(mark_x, needle_area.center().y + 3.0),
                ],
                egui::Stroke::new(1.0, egui::Color32::from_rgb(99, 99, 102)),
            );
        }
    }
}

impl eframe::App for TunerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.refresh_audio_devices();
        self.update_pitch_detection();

        ctx.request_repaint();

        egui::CentralPanel::default()
            .frame(egui::Frame {
                fill: egui::Color32::from_rgba_premultiplied(31, 31, 31, 240),
                corner_radius: 8.0.into(),
                shadow: eframe::epaint::Shadow::NONE,
                outer_margin: egui::Margin::ZERO,
                inner_margin: egui::Margin::symmetric(0, 16),
                ..Default::default()
            })
            .show(ctx, |ui| {
                ui.add_space(8.0);

                ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                    ui.add_space(10.0);
                    self.draw_tuner_display(ui);
                    ui.add_space(8.0);

                    ui.add_space(4.0);
                    ui.vertical_centered(|ui| {
                        ui.add_space(20.0);

                        let available_rect = ui.available_rect_before_wrap();
                        let combo_rect = egui::Rect::from_center_size(
                            available_rect.center(),
                            egui::vec2(200.0, 25.0),
                        );

                        ui.scope_builder(egui::UiBuilder::new().max_rect(combo_rect), |ui| {
                            let mut style = (*ui.ctx().style()).clone();

                            style.visuals.widgets.inactive.bg_fill =
                                egui::Color32::from_rgb(59, 59, 59);
                            style.visuals.widgets.inactive.weak_bg_fill =
                                egui::Color32::from_rgb(59, 59, 59);
                            style.visuals.widgets.inactive.fg_stroke.color = egui::Color32::WHITE;

                            style.visuals.widgets.hovered.bg_fill =
                                egui::Color32::from_rgb(75, 75, 75);
                            style.visuals.widgets.hovered.weak_bg_fill =
                                egui::Color32::from_rgb(75, 75, 75);
                            style.visuals.widgets.hovered.fg_stroke.color = egui::Color32::WHITE;

                            style.visuals.widgets.active.bg_fill =
                                egui::Color32::from_rgb(59, 59, 59);
                            style.visuals.widgets.active.weak_bg_fill =
                                egui::Color32::from_rgb(59, 59, 59);
                            style.visuals.widgets.active.fg_stroke.color = egui::Color32::WHITE;

                            style.visuals.widgets.open.bg_fill =
                                egui::Color32::from_rgb(59, 59, 59);
                            style.visuals.widgets.open.weak_bg_fill =
                                egui::Color32::from_rgb(59, 59, 59);
                            style.visuals.widgets.open.fg_stroke.color = egui::Color32::WHITE;

                            style.visuals.window_fill = egui::Color32::from_rgb(59, 59, 59);
                            style.visuals.panel_fill = egui::Color32::from_rgb(59, 59, 59);
                            style.visuals.extreme_bg_color = egui::Color32::from_rgb(59, 59, 59);

                            style.visuals.selection.bg_fill = egui::Color32::from_rgb(75, 75, 75);
                            style.visuals.selection.stroke.color = egui::Color32::WHITE;

                            style.visuals.override_text_color = Some(egui::Color32::WHITE);

                            style.visuals.popup_shadow = eframe::epaint::Shadow::NONE;
                            ui.ctx().set_style(style);

                            egui::ComboBox::from_id_salt("device_selector")
                                .selected_text(&self.current_device_name)
                                .width(200.0)
                                .height(25.0)
                                .show_ui(ui, |ui| {
                                    let devices = self.available_devices.clone();
                                    for (device_name, device) in devices {
                                        if ui
                                            .selectable_value(
                                                &mut self.current_device_name,
                                                device_name.clone(),
                                                &device_name,
                                            )
                                            .clicked()
                                        {
                                            self.switch_device(device_name, device);
                                        }
                                    }
                                });
                        });
                    });
                });
            });
    }
}
