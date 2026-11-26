//! Custom-painted atoms. Spec: docs/04 §4, docs/03 §3. No magic numbers: all from theme.
use super::icons::{self, Icon};
use crate::theme as t;
use eframe::egui::{self, vec2, Align2, FontId, Rect, Response, Sense, Stroke, Ui};

/// Toolbar button 36x36, glyph 20. `active` shows the 2x16 left indicator bar.
pub fn toolbar_button(ui: &mut Ui, icon: Icon, tip: &str, active: bool, enabled: bool) -> Response {
    let (rect, resp) = ui.allocate_exact_size(vec2(t::TOOLBAR_BTN, t::TOOLBAR_BTN), if enabled { Sense::click() } else { Sense::hover() });
    if ui.is_rect_visible(rect) {
        let p = ui.painter();
        let bg = if !enabled { None } else if resp.is_pointer_button_down_on() { Some(t::BG_PRESSED) } else if resp.hovered() { Some(t::BG_HOVER) } else { None };
        if let Some(c) = bg { p.rect_filled(rect, t::R_BTN, c); }
        let col = if !enabled { t::TEXT_DISABLED } else if active { t::ACCENT_CYAN } else if resp.hovered() { t::TEXT } else { t::TEXT_DIM };
        let g = Rect::from_center_size(rect.center(), vec2(t::TOOLBAR_GLYPH, t::TOOLBAR_GLYPH));
        icons::paint(p, icon, g, col);
        if active { p.rect_filled(Rect::from_center_size(rect.left_center() + vec2(1.0, 0.0), vec2(2.0, 16.0)), 1.0, t::ACCENT_CYAN); }
        if resp.has_focus() { p.rect_stroke(rect, t::R_BTN, Stroke::new(1.5, t::ACCENT_CYAN), egui::StrokeKind::Inside); }
    }
    resp.on_hover_text_at_pointer(tip) // egui tooltip delay is set via Style (set to 0.4 s in app init)
}

/// 36x20 toggle. Returns true if changed.
pub fn toggle(ui: &mut Ui, on: &mut bool) -> bool {
    let (rect, mut resp) = ui.allocate_exact_size(vec2(36.0, 20.0), Sense::click());
    if resp.clicked() { *on = !*on; resp.mark_changed(); }
    let k = ui.ctx().animate_bool_with_time(resp.id, *on, 0.12);
    let p = ui.painter();
    let track = if *on { t::ACCENT_CYAN.gamma_multiply(0.85) } else { t::BG_ELEVATED };
    p.rect_filled(rect, 10.0, track);
    if !*on { p.rect_stroke(rect, 10.0, Stroke::new(t::HAIRLINE, t::BORDER), egui::StrokeKind::Inside); }
    let x = egui::lerp((rect.left() + 10.0)..=(rect.right() - 10.0), k);
    p.circle_filled(egui::pos2(x, rect.center().y), 8.0, t::TEXT);
    resp.changed()
}

/// One settings row: label column 200 w (+optional hint), controls fill the rest. Doc 04 §2.
pub fn setting_row(ui: &mut Ui, label: &str, hint: Option<&str>, add: impl FnOnce(&mut Ui)) {
    let h = if hint.is_some() { 56.0 } else { t::SETTINGS_ROW_H };
    ui.horizontal(|ui| {
        ui.set_min_height(h);
        ui.allocate_ui_with_layout(vec2(t::SETTINGS_LABEL_W, h), egui::Layout::top_down(egui::Align::Min), |ui| {
            ui.add_space(if hint.is_some() { 8.0 } else { 12.0 });
            ui.label(egui::RichText::new(label).size(t::FS_BODY));
            if let Some(hn) = hint { ui.label(egui::RichText::new(hn).size(t::FS_SMALL).color(t::TEXT_DIM)); }
        });
        ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), add);
    });
    let r = ui.available_rect_before_wrap();
    ui.painter().hline(r.x_range(), ui.cursor().top(), Stroke::new(t::HAIRLINE, t::BORDER.gamma_multiply(0.4)));
}

pub fn section_header(ui: &mut Ui, text: &str) {
    ui.add_space(24.0);
    ui.label(egui::RichText::new(text.to_uppercase()).size(t::FS_SECTION).color(t::TEXT_DIM).family(t::semibold()));
    ui.add_space(8.0);
}

/// Segmented control, height 28, radius 8.
pub fn segmented(ui: &mut Ui, idx: &mut usize, labels: &[&str]) -> bool {
    let mut changed = false;
    ui.spacing_mut().item_spacing.x = 2.0;
    for (i, l) in labels.iter().enumerate() {
        let w = 16.0 + l.len() as f32 * 7.0;
        let (rect, resp) = ui.allocate_exact_size(vec2(w, 28.0), Sense::click());
        let sel = *idx == i;
        let bg = if sel { t::BG_HOVER } else if resp.hovered() { t::BG_ELEVATED } else { t::BG_PANEL };
        ui.painter().rect_filled(rect, t::R_BTN, bg);
        if sel { ui.painter().rect_stroke(rect, t::R_BTN, Stroke::new(1.0, t::ACCENT_CYAN), egui::StrokeKind::Inside); }
        ui.painter().text(rect.center(), Align2::CENTER_CENTER, l, FontId::new(t::FS_BODY, t::semibold()), if sel { t::TEXT } else { t::TEXT_DIM });
        if resp.clicked() && !sel { *idx = i; changed = true; }
    }
    changed
}

pub fn primary_button(ui: &mut Ui, text: &str, w: f32, enabled: bool) -> Response {
    let (rect, resp) = ui.allocate_exact_size(vec2(w, 32.0), if enabled { Sense::click() } else { Sense::hover() });
    let fill = if !enabled { t::BG_ELEVATED } else if resp.hovered() { t::ACCENT_CYAN } else { t::ACCENT_BLUE };
    ui.painter().rect_filled(rect, t::R_BTN, fill);
    ui.painter().text(rect.center(), Align2::CENTER_CENTER, text, FontId::new(t::FS_BODY, t::semibold()), if enabled { t::BG_BASE } else { t::TEXT_DISABLED });
    resp
}

pub fn ghost_button(ui: &mut Ui, text: &str, w: f32, enabled: bool) -> Response {
    let (rect, resp) = ui.allocate_exact_size(vec2(w, 32.0), if enabled { Sense::click() } else { Sense::hover() });
    if resp.hovered() && enabled { ui.painter().rect_filled(rect, t::R_BTN, t::BG_HOVER); }
    ui.painter().rect_stroke(rect, t::R_BTN, Stroke::new(t::HAIRLINE, t::BORDER), egui::StrokeKind::Inside);
    ui.painter().text(rect.center(), Align2::CENTER_CENTER, text, FontId::new(t::FS_BODY, t::semibold()), if enabled { t::TEXT } else { t::TEXT_DISABLED });
    resp
}
