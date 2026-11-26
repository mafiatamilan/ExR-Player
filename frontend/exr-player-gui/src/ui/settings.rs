//! Settings dialog shell + General page. Spec: docs/04. Other pages: implement per doc 04 §3.2–3.5.
use super::widgets::{ghost_button, primary_button, section_header, segmented, setting_row, toggle};
use crate::settings::{model::{NavPosition, Settings}, validate};
use crate::{app::ExrApp, theme as t};
use eframe::egui::{self, vec2, Align2, Color32, Stroke};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Page { General, Engine, Advanced, Shortcuts, About }
impl Page { const ALL: [(Page, &'static str); 5] = [(Page::General, "General"), (Page::Engine, "Engine & Display"), (Page::Advanced, "Advanced"), (Page::Shortcuts, "Shortcuts"), (Page::About, "About")]; }

pub struct SettingsState { pub draft: Settings, pub saved: Settings, pub page: Page, pub restart_needed: bool }
impl Default for SettingsState { fn default() -> Self { let s = Settings::default(); Self { draft: s.clone(), saved: s, page: Page::General, restart_needed: false } } }
impl SettingsState {
    fn dirty(&self) -> bool { self.draft != self.saved }
    fn valid(&self) -> bool { validate::resolution_ok(self.draft.width, self.draft.height, self.draft.dpi) }
    fn mark_restart_if_needed(&mut self) {
        let (a, b) = (&self.draft, &self.saved);
        self.restart_needed = (a.width, a.height, a.dpi, a.cpu_cores, a.memory_mb, a.language.as_str(), a.preset, a.root_mode)
            != (b.width, b.height, b.dpi, b.cpu_cores, b.memory_mb, b.language.as_str(), b.preset, b.root_mode);
    }
}

pub fn show(ctx: &egui::Context, app: &mut ExrApp) {
    // scrim
    let screen = ctx.content_rect(); // verify name in 0.37 (was screen_rect())
    egui::Area::new(egui::Id::new("scrim")).order(egui::Order::Foreground).fixed_pos(screen.min).show(ctx, |ui| {
        let r = ui.allocate_rect(screen, egui::Sense::click());
        ui.painter().rect_filled(screen, 0.0, Color32::from_black_alpha(140));
        let _ = r;
    });
    let mut close = false;
    egui::Window::new("settings").title_bar(false).collapsible(false).resizable(false).order(egui::Order::Tooltip)
        .anchor(Align2::CENTER_CENTER, vec2(0.0, 0.0)).fixed_size(t::DIALOG_SETTINGS)
        .frame(egui::Frame::new().fill(t::BG_PANEL).stroke(Stroke::new(1.0, t::BORDER)).corner_radius(t::R_DIALOG)
            .shadow(egui::Shadow { offset: [0, 8], blur: 40, spread: 0, color: Color32::from_black_alpha(140) }))
        .show(ctx, |ui| {
            let st = &mut app.settings;
            ui.set_min_size(t::DIALOG_SETTINGS);
            ui.vertical(|ui| {
                // header 52
                ui.allocate_ui_with_layout(
                    vec2(t::DIALOG_SETTINGS.x, 52.0),
                    egui::Layout::left_to_right(egui::Align::Center),
                    |ui| {
                        ui.add_space(20.0);
                        ui.label(egui::RichText::new("Settings").size(t::FS_TITLE).family(t::semibold()));
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.add_space(12.0);
                            if ghost_button(ui, "✕", 36.0, true).clicked() {
                                close = true;
                            }
                        });
                    },
                );
                let hdr_y = ui.cursor().top();
                ui.painter().hline(ui.min_rect().left()..=ui.min_rect().left() + t::DIALOG_SETTINGS.x, hdr_y, Stroke::new(1.0, t::BORDER));

                let body_h = t::DIALOG_SETTINGS.y - 52.0 - 56.0;
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 0.0;
                    // left nav 176
                    ui.allocate_ui_with_layout(
                        vec2(t::SETTINGS_NAV_W, body_h),
                        egui::Layout::top_down(egui::Align::Min),
                        |ui| {
                            ui.set_min_size(vec2(t::SETTINGS_NAV_W, body_h));
                            let nav_rect = egui::Rect::from_min_size(ui.cursor().min, vec2(t::SETTINGS_NAV_W, body_h));
                            ui.painter().rect_filled(nav_rect, 0.0, t::BG_BASE);
                            ui.add_space(8.0);
                            for (p, name) in Page::ALL {
                                let (r, resp) = ui.allocate_exact_size(vec2(t::SETTINGS_NAV_W - 20.0, 36.0), egui::Sense::click());
                                let r = r.translate(vec2(10.0, 0.0));
                                let sel = st.page == p;
                                if sel || resp.hovered() {
                                    ui.painter().rect_filled(r, t::R_BTN, if sel { t::BG_ELEVATED } else { t::BG_HOVER });
                                }
                                if sel {
                                    ui.painter().rect_filled(egui::Rect::from_center_size(r.left_center() + vec2(1.5, 0.0), vec2(3.0, 16.0)), 1.5, t::ACCENT_CYAN);
                                }
                                ui.painter().text(r.left_center() + vec2(16.0, 0.0), Align2::LEFT_CENTER, name, egui::FontId::proportional(t::FS_BODY), if sel { t::TEXT } else { t::TEXT_DIM });
                                if resp.clicked() {
                                    st.page = p;
                                }
                            }
                        },
                    );
                    // content
                    let content_w = t::DIALOG_SETTINGS.x - t::SETTINGS_NAV_W;
                    ui.allocate_ui_with_layout(
                        vec2(content_w, body_h),
                        egui::Layout::top_down(egui::Align::Min),
                        |ui| {
                            ui.set_min_size(vec2(content_w, body_h));
                            egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                                egui::Frame::new().inner_margin(egui::Margin::same(20)).show(ui, |ui| match st.page {
                                    Page::General => general(ui, &mut st.draft),
                                    _ => {
                                        ui.label(egui::RichText::new("Use the Keymapping Editor (Keyboard icon) for controls, or General for Display & Performance.").color(t::TEXT_DIM));
                                    }
                                });
                            });
                        },
                    );
                });

                // footer 56
                st.mark_restart_if_needed();
                let ftr_y = ui.cursor().top();
                ui.painter().hline(ui.min_rect().left()..=ui.min_rect().left() + t::DIALOG_SETTINGS.x, ftr_y, Stroke::new(1.0, t::BORDER));
                ui.allocate_ui_with_layout(
                    vec2(t::DIALOG_SETTINGS.x, 56.0),
                    egui::Layout::left_to_right(egui::Align::Center),
                    |ui| {
                        ui.add_space(20.0);
                        if st.restart_needed {
                            ui.label(egui::RichText::new("⟳ Restart required to apply").size(t::FS_SMALL).color(t::WARN));
                        }
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.add_space(20.0);
                            let can = st.dirty() && st.valid();
                            if primary_button(ui, "Save & Restart", 136.0, can).clicked() {
                                st.saved = st.draft.clone();
                                app.backend.apply_settings(&st.saved);
                                app.backend.restart();
                                close = true;
                            }
                            if ghost_button(ui, "Save", 88.0, can).clicked() {
                                st.saved = st.draft.clone();
                                app.backend.apply_settings(&st.saved);
                                app.toasts.push("Settings saved to config.exr");
                            }
                            if ghost_button(ui, "Cancel", 88.0, true).clicked() {
                                st.draft = st.saved.clone();
                                close = true;
                            }
                        });
                    },
                );
            });
        });
    if ctx.input(|i| i.key_pressed(egui::Key::Escape)) { app.settings.draft = app.settings.saved.clone(); close = true; }
    if close { app.settings_open = false; }
}

fn general(ui: &mut egui::Ui, s: &mut Settings) {
    section_header(ui, "Display");
    setting_row(ui, "Resolution", None, |ui| {
        ui.add(egui::DragValue::new(&mut s.width).range(320..=3840).suffix(" w")); ui.label("×");
        ui.add(egui::DragValue::new(&mut s.height).range(240..=2160).suffix(" h"));
        ui.add(egui::DragValue::new(&mut s.dpi).range(120..=640).suffix(" dpi"));
    });
    section_header(ui, "Performance");
    setting_row(ui, "CPU cores", Some("Needs hardware virtualization for >1"), |ui| { ui.add(egui::Slider::new(&mut s.cpu_cores, 1..=16)); });
    setting_row(ui, "Memory (MB)", None, |ui| { ui.add(egui::Slider::new(&mut s.memory_mb, 512..=16384).step_by(256.0)); });
    section_header(ui, "Interface");
    setting_row(ui, "Nav bar position", None, |ui| {
        let mut i = match s.nav_position { NavPosition::Bottom => 0, NavPosition::Toolbar => 1, NavPosition::Hidden => 2 };
        if segmented(ui, &mut i, &["Bottom", "Toolbar", "Hidden"]) { s.nav_position = [NavPosition::Bottom, NavPosition::Toolbar, NavPosition::Hidden][i]; }
    });
    setting_row(ui, "App tabs", None, |ui| { toggle(ui, &mut s.app_tabs); });
    setting_row(ui, "Start with Windows", None, |ui| { toggle(ui, &mut s.start_with_windows); });
}
