//! Toasts. Spec: docs/03 §5 (280x40, radius 10, 3 s, max 3).
use crate::theme as t;
use eframe::egui::{self, Align2, Color32};
use std::time::{Duration, Instant};

#[derive(Default)]
pub struct Toasts(Vec<(String, Instant)>);
impl Toasts {
    pub fn push(&mut self, s: impl Into<String>) { self.0.push((s.into(), Instant::now())); if self.0.len() > 3 { self.0.remove(0); } }
}

pub fn show(ctx: &egui::Context, toasts: &mut Toasts) {
    toasts.0.retain(|(_, at)| at.elapsed() < Duration::from_secs(3));
    if toasts.0.is_empty() { return; }
    ctx.request_repaint_after(Duration::from_millis(250)); // only while toasts are visible
    for (i, (msg, _)) in toasts.0.iter().enumerate() {
        egui::Area::new(egui::Id::new(("toast", i))).anchor(Align2::CENTER_BOTTOM, egui::vec2(0.0, -(t::NAV_H + 12.0 + i as f32 * 48.0))).show(ctx, |ui| {
            egui::Frame::new().fill(t::BG_ELEVATED).stroke(egui::Stroke::new(1.0, t::BORDER)).corner_radius(10).inner_margin(egui::Margin::symmetric(16, 10)).show(ui, |ui| {
                ui.set_min_width(248.0);
                ui.label(egui::RichText::new(msg).size(t::FS_BODY).color(Color32::from_rgb(0xF7, 0xF9, 0xFC)));
            });
        });
    }
}
