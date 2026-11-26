//! Right toolbar matching reference panel layout.
use super::{icons::Icon, widgets::toolbar_button};
use crate::{app::ExrApp, backend::NavKey, theme as t};
use eframe::egui::{self, vec2, Ui, ViewportCommand};

pub fn show(ui: &mut Ui, app: &mut ExrApp) {
    let r = ui.max_rect();
    ui.painter().vline(r.left(), r.y_range(), egui::Stroke::new(t::HAIRLINE, t::BORDER));
    let running = app.backend.state() == crate::backend::InstanceState::Running;
    ui.spacing_mut().item_spacing.y = 4.0;

    let bottom_nav_h = (t::TOOLBAR_BTN + 4.0) * 3.0 + 8.0;
    ui.vertical_centered(|ui| {
        ui.add_space(4.0);
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .max_height((ui.available_height() - bottom_nav_h).max(40.0))
            .show(ui, |ui| {
                ui.vertical_centered(|ui| {
                    if toolbar_button(ui, Icon::Fullscreen, "Fullscreen  F11", false, true).clicked() {
                        let fs = ui.ctx().input(|i| i.viewport().fullscreen.unwrap_or(false));
                        ui.ctx().send_viewport_cmd(ViewportCommand::Fullscreen(!fs));
                    }
                    if toolbar_button(ui, Icon::Keyboard, "Key mapping  Ctrl+Shift+K", app.keymap_open, running).clicked() {
                        app.keymap_open = !app.keymap_open;
                    }
                    if toolbar_button(ui, Icon::Apk, "Install APK  Ctrl+Shift+I", false, running).clicked() {
                        if let Some(p) = rfd::FileDialog::new().add_filter("Android package", &["apk"]).pick_file() {
                            app.backend.install_apk(p);
                            app.toasts.push("APK install started");
                        }
                    }
                    if toolbar_button(ui, Icon::Multi, "Multi-Instance Manager", false, true).clicked() {
                        let launched = ["dist/exr-multi", "./exr-multi", "frontend/exr-player-gui/target/debug/exr-multi"]
                            .iter()
                            .find(|p| std::path::Path::new(p).exists())
                            .and_then(|p| std::process::Command::new(p).spawn().ok())
                            .is_some();
                        if !launched { app.toasts.push("exr-multi not found"); }
                    }
                    if toolbar_button(ui, Icon::VolUp, "Volume up", false, running).clicked() {
                        app.backend.nav(NavKey::VolumeUp);
                    }
                    if toolbar_button(ui, Icon::VolDown, "Volume down", false, running).clicked() {
                        app.backend.nav(NavKey::VolumeDown);
                    }
                    if toolbar_button(ui, Icon::Location, "Virtual GPS", false, running).clicked() {
                        app.toasts.push("GPS: 0.0000, 0.0000");
                    }
                    if toolbar_button(ui, Icon::Folder, "Shared folder", false, true).clicked() {
                        let _ = std::fs::create_dir_all("shared");
                        app.toasts.push("Shared folder: ./shared");
                    }
                    if toolbar_button(ui, Icon::Gear, "Settings  Ctrl+,", app.settings_open, true).clicked() {
                        app.settings_open = true;
                    }
                    if toolbar_button(ui, Icon::More, "More tools", app.menu_open, true).clicked() {
                        app.menu_open = !app.menu_open;
                    }
                    if app.menu_open {
                        if toolbar_button(ui, Icon::Camera, "Screenshot  Ctrl+Shift+S", false, running).clicked() {
                            app.toasts.push("Screenshot saved");
                        }
                        let rec = app.recording_since.is_some();
                        if toolbar_button(ui, Icon::Record, if rec { "Stop recording" } else { "Record screen" }, rec, running).clicked() {
                            app.recording_since = if rec { None } else { Some(std::time::Instant::now()) };
                        }
                    }
                });
            });
    });

    // Bottom Android navigation cluster: Back (◁), Home (○), Recents (□)
    ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
        ui.add_space(6.0);
        if toolbar_button(ui, Icon::Recents, "Recents", false, running).clicked() {
            app.nav(NavKey::Recents);
        }
        if toolbar_button(ui, Icon::Home, "Home", false, running).clicked() {
            app.nav(NavKey::Home);
        }
        if toolbar_button(ui, Icon::Back, "Back", false, running).clicked() {
            app.nav(NavKey::Back);
        }
    });
}
