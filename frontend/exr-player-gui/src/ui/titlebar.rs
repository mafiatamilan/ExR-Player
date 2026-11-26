//! Custom title bar. Spec: docs/03 §1.
use super::icons::{self, Icon};
use crate::{app::ExrApp, theme as t};
use eframe::egui::{self, vec2, Align2, FontId, Rect, Sense, Stroke, Ui, ViewportCommand};

pub fn show(ui: &mut Ui, app: &mut ExrApp) {
    let bar = ui.max_rect();
    // drag region = whole bar; buttons are laid on top and consume their own clicks
    let drag = ui.interact(bar, ui.id().with("drag"), Sense::click_and_drag());
    if drag.drag_started() { ui.ctx().send_viewport_cmd(ViewportCommand::StartDrag); }
    if drag.double_clicked() { toggle_max(ui.ctx()); }

    // logo + wordmark ("ExR PLAY")
    let logo_rect = Rect::from_min_size(bar.min + vec2(10.0, (t::TITLEBAR_H - t::LOGO_H) / 2.0), vec2(t::LOGO_W, t::LOGO_H));
    ui.painter().image(app.logo.id(), logo_rect, Rect::from_min_max(egui::pos2(0., 0.), egui::pos2(1., 1.)), egui::Color32::WHITE);
    let mut cursor_x = logo_rect.right() + 8.0;
    let g1 = ui.painter().layout_no_wrap("ExR ".into(), FontId::new(t::FS_BODY, t::semibold()), t::TEXT);
    let w1 = g1.size().x;
    ui.painter().galley(egui::pos2(cursor_x, bar.center().y - g1.size().y / 2.0), g1, t::TEXT);
    cursor_x += w1;
    let g2 = ui.painter().layout_no_wrap("PLAY".into(), FontId::new(t::FS_BODY, t::semibold()), t::ACCENT_GREEN);
    let w2 = g2.size().x;
    ui.painter().galley(egui::pos2(cursor_x, bar.center().y - g2.size().y / 2.0), g2, t::ACCENT_GREEN);
    cursor_x += w2 + 16.0;

    // Home app tab matching screenshot
    if app.settings.saved.app_tabs {
        let tab_r = Rect::from_min_size(egui::pos2(cursor_x, bar.top() + 3.0), vec2(128.0, t::TITLEBAR_H - 3.0));
        let tab_resp = ui.interact(tab_r, ui.id().with("tab_home"), Sense::click());
        ui.painter().rect_filled(tab_r, egui::CornerRadius { nw: 6, ne: 6, sw: 0, se: 0 }, if tab_resp.hovered() { t::BG_HOVER } else { t::BG_ELEVATED });
        ui.painter().hline(tab_r.x_range(), tab_r.bottom() - 1.0, Stroke::new(2.0, t::ACCENT_GREEN));
        ui.painter().circle_filled(egui::pos2(tab_r.left() + 14.0, tab_r.center().y), 5.0, t::ACCENT_GREEN);
        ui.painter().text(egui::pos2(tab_r.left() + 26.0, tab_r.center().y), Align2::LEFT_CENTER, "Home", FontId::new(t::FS_SMALL, t::semibold()), t::TEXT);
        if tab_resp.clicked() { app.nav(crate::backend::NavKey::Home); }
    }
    ui.painter().hline(bar.x_range(), bar.bottom(), Stroke::new(t::HAIRLINE, t::BORDER));

    // right-aligned: [pin][menu][collapse][min][max][close]
    let maximized = ui.ctx().input(|i| i.viewport().maximized.unwrap_or(false));
    let mut x = bar.right();
    let mut ctl = |ui: &mut Ui, icon: Icon, danger: bool| -> bool {
        x -= t::WINCTL_W;
        let r = Rect::from_min_size(egui::pos2(x, bar.top()), vec2(t::WINCTL_W, t::TITLEBAR_H));
        let resp = ui.interact(r, ui.id().with(("wc", x as i32)), Sense::click());
        if resp.hovered() { ui.painter().rect_filled(r, 0.0, if danger { t::DANGER } else { t::BG_HOVER }); }
        let col = if danger && resp.hovered() { egui::Color32::WHITE } else if resp.hovered() { t::TEXT } else { t::TEXT_DIM };
        icons::paint(ui.painter(), icon, Rect::from_center_size(r.center(), vec2(t::WINCTL_GLYPH + 4.0, t::WINCTL_GLYPH + 4.0)), col);
        resp.clicked()
    };
    if ctl(ui, Icon::Close, true) { ui.ctx().send_viewport_cmd(ViewportCommand::Close); }
    if ctl(ui, if maximized { Icon::Restore } else { Icon::Max }, false) { toggle_max(ui.ctx()); }
    if ctl(ui, Icon::Min, false) { ui.ctx().send_viewport_cmd(ViewportCommand::Minimized(true)); }
    if ctl(ui, Icon::Collapse, false) { app.toolbar_collapsed = !app.toolbar_collapsed; }
    if ctl(ui, Icon::Menu, false) { app.menu_open = !app.menu_open; }
    if ctl(ui, Icon::Pin, false) {
        let id = egui::Id::new("exr_pinned");
        let top = ui.ctx().data(|d| d.get_temp::<bool>(id).unwrap_or(false));
        let next = !top;
        ui.ctx().data_mut(|d| d.insert_temp(id, next));
        ui.ctx().send_viewport_cmd(ViewportCommand::WindowLevel(if next { egui::WindowLevel::AlwaysOnTop } else { egui::WindowLevel::Normal }));
    }
}

fn toggle_max(ctx: &egui::Context) {
    let m = ctx.input(|i| i.viewport().maximized.unwrap_or(false));
    ctx.send_viewport_cmd(ViewportCommand::Maximized(!m));
}
