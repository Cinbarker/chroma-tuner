//! Application entry point for Chroma Tuner
//!
//! Sets up the eframe window with native styling, initializes audio capture,
//! and creates the main TunerApp instance.

use eframe::egui;
use std::sync::{Arc, Mutex};
use egui::IconData;

mod audio;
mod pitch;
mod tuner;

use audio::AudioCapture;
use tuner::TunerApp;

fn load_app_icon() -> IconData {
    let icon_bytes = include_bytes!("../assets/icons/icon.png");
    match image::load_from_memory(icon_bytes) {
        Ok(img) => {
            let img = img.to_rgba8();
            let (width, height) = img.dimensions();
            IconData {
                rgba: img.into_raw(),
                width: width as u32,
                height: height as u32,
            }
        }
        Err(_) => {
            let size = 32;
            let mut rgba = Vec::with_capacity(size * size * 4);
            for _i in 0..(size * size) {
                rgba.extend_from_slice(&[63, 81, 181, 255]);
            }
            IconData {
                rgba,
                width: size as u32,
                height: size as u32,
            }
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([320.0, 180.0])
            .with_resizable(false)
            .with_always_on_top()
            .with_decorations(true)
            .with_title_shown(false)
            .with_titlebar_buttons_shown(true)
            .with_titlebar_shown(false)
            .with_fullsize_content_view(true)
            .with_transparent(true)
            .with_icon(load_app_icon()),
        ..Default::default()
    };

    let audio_data = Arc::new(Mutex::new(audio::AudioData::new()));
    let audio_capture = AudioCapture::new(audio_data.clone())?;

    eframe::run_native(
        "Chroma Tuner",
        options,
        Box::new(|_cc| {
            let mut app = TunerApp::new(audio_data);
            app.set_audio_capture(audio_capture);
            Ok(Box::new(app))
        }),
    )?;

    Ok(())
}
