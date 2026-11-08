#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::egui::IconData;
use karnaugh_studio::app::KarnaughStudioApp;

fn load_app_icon() -> Option<IconData> {
    let png_bytes = include_bytes!("../assets/icon-256.png");
    if let Ok(img) = image::load_from_memory(png_bytes) {
        let rgba = img.to_rgba8();
        let (width, height) = rgba.dimensions();
        Some(IconData {
            rgba: rgba.into_raw(),
            width,
            height,
        })
    } else {
        None
    }
}

fn main() -> eframe::Result {
    let mut viewport = egui::ViewportBuilder::default()
        .with_title("Karnaugh Studio")
        .with_inner_size([1280.0, 780.0])
        .with_min_inner_size([1024.0, 680.0]);

    if let Some(icon) = load_app_icon() {
        viewport = viewport.with_icon(icon);
    }

    let native_options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };

    eframe::run_native(
        "Karnaugh Studio",
        native_options,
        Box::new(|_cc| Ok(Box::new(KarnaughStudioApp::default()))),
    )
}
