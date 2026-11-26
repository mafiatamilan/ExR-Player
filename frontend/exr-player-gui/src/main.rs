#![cfg_attr(all(not(debug_assertions), windows), windows_subsystem = "windows")]
mod app;
mod backend;
mod keymap;
mod keymap_runtime;
mod settings;
mod theme;
mod ui;

use eframe::egui;

fn load_icon() -> Option<egui::IconData> {
    let bytes = include_bytes!("../assets/logo/icon_256.png");
    let img = image::load_from_memory(bytes).ok()?.into_rgba8();
    let (w, h) = img.dimensions();
    Some(egui::IconData { rgba: img.into_raw(), width: w, height: h })
}

fn main() -> eframe::Result<()> {
    let mut config_override = None;
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        if arg == "--config" {
            config_override = args.next().map(std::path::PathBuf::from);
        }
    }
    let mut vp = egui::ViewportBuilder::default()
        .with_title("ExR Player")
        .with_decorations(false)
        .with_inner_size(theme::WIN_DEFAULT)
        .with_min_inner_size(theme::WIN_MIN);
    if let Some(i) = load_icon() { vp = vp.with_icon(i); }
    let opts = eframe::NativeOptions { viewport: vp, ..Default::default() };
    eframe::run_native("ExR Player", opts, Box::new(move |cc| Ok(Box::new(app::ExrApp::new(cc, config_override)))))
}
