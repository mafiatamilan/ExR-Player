//! Android nav strip. Spec: docs/03 §4.
use super::icons::{self, Icon};
use crate::{app::ExrApp, backend::NavKey, theme as t};
use eframe::egui::{self, vec2, Rect, Sense, Ui};

pub fn show(ui: &mut Ui, app: &mut ExrApp) {
    let bar = ui.max_rect();
    let items = [(Icon::Back, NavKey::Back, "Back"), (Icon::Home, NavKey::Home, "Home"), (Icon::Recents, NavKey::Recents, "Recent apps")];
    let gap = 24.0;
    let total = items.len() as f32 * t::NAV_BTN.x + (items.len() as f32 - 1.0) * gap;
    let mut x = bar.center().x - total / 2.0;
    for (icon, key, tip) in items {
        let r = Rect::from_min_size(egui::pos2(x, bar.center().y - t::NAV_BTN.y / 2.0), t::NAV_BTN);
        let resp = ui.interact(r, ui.id().with(tip), Sense::click());
        if resp.hovered() { ui.painter().rect_filled(r, t::R_BTN, t::BG_HOVER); }
        icons::paint(ui.painter(), icon, Rect::from_center_size(r.center(), vec2(t::NAV_GLYPH, t::NAV_GLYPH)), if resp.hovered() { t::TEXT } else { t::TEXT_DIM });
        if resp.on_hover_text(tip).clicked() { app.nav(key); }
        x += t::NAV_BTN.x + gap;
    }
}
