//! Top-level state + dispatch. Spec: docs/03.
use crate::backend::{create_backend, EmulatorBackend, NavKey};
use crate::keymap_runtime::KeymapRuntime;
use crate::{theme, ui};
use eframe::egui;
use exr_keymap_ui::EditorAction;
use std::sync::Arc;

pub struct ExrApp {
    pub backend: Box<dyn EmulatorBackend>,
    pub frame_tex: Option<egui::TextureHandle>,
    pub last_frame_id: u64,
    pub frame_aspect: f32,
    pub logo: egui::TextureHandle,
    pub keymap_open: bool,
    pub keymap_rt: KeymapRuntime,
    pub settings_open: bool,
    pub settings: ui::settings::SettingsState,
    pub menu_open: bool,
    pub toolbar_collapsed: bool,
    pub toasts: ui::toast::Toasts,
    pub recording_since: Option<std::time::Instant>,
}

impl ExrApp {
    pub fn new(cc: &eframe::CreationContext<'_>, config_override: Option<std::path::PathBuf>) -> Self {
        theme::install(&cc.egui_ctx);
        let img = image::load_from_memory(include_bytes!("../assets/logo/exr_glyph.png"))
            .expect("logo")
            .resize_exact(72, 44, image::imageops::FilterType::Lanczos3)
            .into_rgba8();
        let (w, h) = img.dimensions();
        let ci = egui::ColorImage::from_rgba_unmultiplied([w as usize, h as usize], img.as_raw());
        let tex_opts = egui::TextureOptions {
            magnification: egui::TextureFilter::Linear,
            minification: egui::TextureFilter::Linear,
            mipmap_mode: Some(egui::TextureFilter::Linear),
            ..Default::default()
        };
        let logo = cc.egui_ctx.load_texture("exr_logo", ci, tex_opts);
        let settings = ui::settings::SettingsState::default();
        let mut backend = create_backend(&settings.saved, config_override);
        let ctx = cc.egui_ctx.clone();
        backend.set_repaint_hook(Arc::new(move || ctx.request_repaint()));
        Self { backend, frame_tex: None, last_frame_id: 0, frame_aspect: 16.0 / 9.0, logo,
               keymap_open: false, keymap_rt: KeymapRuntime::new(), settings_open: false, settings, menu_open: false, toolbar_collapsed: false,
               toasts: Default::default(), recording_since: None }
    }
    pub fn nav(&mut self, k: NavKey) { self.backend.nav(k); }
}

impl eframe::App for ExrApp {
    fn ui(&mut self, ui: &mut egui::Ui, _f: &mut eframe::Frame) {
        // order matters: chrome first, then viewport fills the rest
        egui::Panel::top("titlebar").exact_size(theme::TITLEBAR_H)
            .frame(egui::Frame::new().fill(theme::BG_PANEL)).show_inside(ui, |ui| ui::titlebar::show(ui, self));
        if !self.toolbar_collapsed {
            egui::Panel::right("toolbar").exact_size(theme::TOOLBAR_W).resizable(false)
                .frame(egui::Frame::new().fill(theme::BG_PANEL)).show_inside(ui, |ui| ui::toolbar::show(ui, self));
        }
        if self.keymap_open {
            if self.keymap_rt.editor.try_capture_egui_input(ui.ctx()) {
                self.keymap_rt.rebuild_indices();
            }
            let mut action = EditorAction::None;
            egui::Panel::top("keymap_topbar").exact_size(36.0)
                .frame(egui::Frame::new().fill(theme::BG_PANEL)).show_inside(ui, |ui| {
                    action = exr_keymap_ui::show_top_bar(
                        ui,
                        &mut self.keymap_rt.editor,
                        &self.keymap_rt.store,
                    );
                });
            let mut side_changed = false;
            egui::Panel::right("keymap_panel").exact_size(theme::KEYMAP_PANEL_W).resizable(false)
                .frame(egui::Frame::new().fill(theme::BG_PANEL)).show_inside(ui, |ui| {
                    side_changed = exr_keymap_ui::show_side_panel(ui, &mut self.keymap_rt.editor);
                });
            if side_changed {
                self.keymap_rt.rebuild_indices();
            }
            match action {
                EditorAction::ProfileChanged => self.keymap_rt.rebuild_indices(),
                EditorAction::Saved(id) => self.toasts.push(format!("Saved keymap '{id}'")),
                EditorAction::CloseRequested => {
                    self.keymap_rt.editor.test_mode = false;
                    self.keymap_rt.release_all_virtual(self.backend.as_mut());
                    self.keymap_open = false;
                }
                EditorAction::ToggleTestMode(on) => {
                    if !on {
                        self.keymap_rt.release_all_virtual(self.backend.as_mut());
                    }
                }
                EditorAction::ExportRequested => {
                    if let Some(path) = rfd::FileDialog::new()
                        .set_file_name(format!("{}.json", self.keymap_rt.editor.profile.id))
                        .save_file()
                    {
                        if let Err(e) = self.keymap_rt.store.export_to_path(&self.keymap_rt.editor.profile, &path) {
                            self.toasts.push(format!("Export failed: {e}"));
                        } else {
                            self.toasts.push("Keymap exported");
                        }
                    }
                }
                EditorAction::ImportRequested => {
                    if let Some(path) = rfd::FileDialog::new().add_filter("JSON", &["json"]).pick_file() {
                        match self.keymap_rt.store.import_from_path(&path) {
                            Ok(p) => {
                                self.keymap_rt.load_profile(p);
                                self.toasts.push("Keymap imported");
                            }
                            Err(e) => self.toasts.push(format!("Import failed: {e}")),
                        }
                    }
                }
                EditorAction::None => {}
            }
        }
        egui::CentralPanel::default().frame(egui::Frame::new().fill(egui::Color32::BLACK))
            .show(ui, |ui| ui::viewport::show(ui, self));
        let ctx = ui.ctx().clone();
        ui::toast::show(&ctx, &mut self.toasts);
        if self.settings_open { ui::settings::show(&ctx, self); }
    }
}
