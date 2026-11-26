//! Android viewport. Spec: docs/03 §2, docs/06 §3. One TextureHandle, updated only on new frame id.
use crate::{app::ExrApp, backend::InstanceState};
use eframe::egui::{self, Color32, Rect, Sense, TextureOptions, Ui};

pub fn show(ui: &mut Ui, app: &mut ExrApp) {
    app.keymap_rt
        .sync_foreground_app(app.backend.foreground_app(), app.backend.as_mut());
    if let Some(f) = app.backend.frame_if_new(app.last_frame_id) {
        let img = egui::ColorImage::from_rgba_unmultiplied([f.w, f.h], &f.rgba);
        match &mut app.frame_tex { Some(t) => t.set(img, TextureOptions::LINEAR), None => app.frame_tex = Some(ui.ctx().load_texture("frame", img, TextureOptions::LINEAR)) }
        app.last_frame_id = f.id;
        app.frame_aspect = f.w as f32 / f.h as f32;
    }
    let avail = ui.available_rect_before_wrap();
    let (w, h) = { let a = avail.width() / avail.height(); if a > app.frame_aspect { (avail.height() * app.frame_aspect, avail.height()) } else { (avail.width(), avail.width() / app.frame_aspect) } };
    let rect = Rect::from_center_size(avail.center(), egui::vec2(w, h));
    let resp = ui.interact(rect, ui.id().with("vp"), Sense::click_and_drag());
    match (app.backend.state(), &app.frame_tex) {
        (InstanceState::Running, Some(tex)) => { ui.painter().image(tex.id(), rect, Rect::from_min_max(egui::pos2(0., 0.), egui::pos2(1., 1.)), Color32::WHITE); }
        _ => { /* Booting / Crashed / NoInstance states: doc 03 §2 — agent implements */ }
    }
    // forward pointer as touch only when keymap editor closed and FPS capture is not active
    if !app.keymap_open {
        app.keymap_rt.process_egui_frame(ui.ctx(), rect, app.backend.as_mut());
        exr_keymap_ui::paint_play_overlay(
            ui,
            rect,
            &app.keymap_rt.editor.profile,
            app.keymap_rt.fps_aim.captured,
        );
        if !app.keymap_rt.fps_aim.captured {
            let norm = |p: egui::Pos2| ((p.x - rect.left()) / rect.width(), (p.y - rect.top()) / rect.height());
            if let Some(p) = resp.interact_pointer_pos() {
                let (nx, ny) = norm(p);
                if resp.drag_started() { app.backend.touch_down(0, nx, ny); }
                else if resp.dragged() { app.backend.touch_move(0, nx, ny); }
            }
            if resp.drag_stopped() { app.backend.touch_up(0); }
        }
    } else {
        if app.keymap_rt.editor.test_mode {
            app.keymap_rt.process_egui_frame(ui.ctx(), rect, app.backend.as_mut());
            exr_keymap_ui::paint_play_overlay(
                ui,
                rect,
                &app.keymap_rt.editor.profile,
                app.keymap_rt.fps_aim.captured,
            );
        } else if exr_keymap_ui::paint_and_handle_overlay(ui, rect, &mut app.keymap_rt.editor) {
            app.keymap_rt.rebuild_indices();
        }
    }
    // Mock backend has no repaint thread: tick at ~30 fps only while running (real backend uses the hook instead).
    ui.ctx().request_repaint_after(std::time::Duration::from_millis(33));
}
