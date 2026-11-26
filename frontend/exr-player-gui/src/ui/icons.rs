//! Painter-drawn line icons (1.75 stroke @24-grid, round caps). Doc 02 §5 option (b).
//! Only the icons needed by the shell are drawn here; add the rest in the same style
//! (or switch to a Lucide/resvg atlas) — do NOT mix two styles.
use eframe::egui::{vec2, Color32, Painter, Rect, Stroke};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Icon {
    Menu, Min, Max, Restore, Close,
    Back, Home, Recents,
    Gear, Apk, Keyboard, Camera, Record, Pin, Volume,
    Fullscreen, Multi, VolUp, VolDown, Location, Folder, More, Collapse,
}

pub fn paint(p: &Painter, icon: Icon, rect: Rect, color: Color32) {
    let s = rect.width() / 24.0; // 24-unit grid
    let st = Stroke::new(2.35 * s.max(0.8), color);
    let pt = |x: f32, y: f32| rect.min + vec2(x, y) * s;
    let line = |a: (f32, f32), b: (f32, f32)| p.line_segment([pt(a.0, a.1), pt(b.0, b.1)], st);
    match icon {
        Icon::Menu => { line((4., 7.), (20., 7.)); line((4., 12.), (20., 12.)); line((4., 17.), (20., 17.)); }
        Icon::Min => { line((5., 12.), (19., 12.)); }
        Icon::Max => { p.rect_stroke(Rect::from_min_max(pt(5., 5.), pt(19., 19.)), 2.0 * s, st, eframe::egui::StrokeKind::Inside); }
        Icon::Restore => {
            p.rect_stroke(Rect::from_min_max(pt(4., 8.), pt(16., 20.)), 2.0 * s, st, eframe::egui::StrokeKind::Inside);
            line((8., 4.), (20., 4.)); line((20., 4.), (20., 16.));
        }
        Icon::Close => { line((5., 5.), (19., 19.)); line((19., 5.), (5., 19.)); }
        Icon::Back => {
            line((15.5, 5.5), (8.0, 12.0));
            line((8.0, 12.0), (15.5, 18.5));
        }
        Icon::Home => {
            line((12.0, 4.5), (4.0, 11.5));
            line((12.0, 4.5), (20.0, 11.5));
            line((6.5, 10.0), (6.5, 19.5));
            line((6.5, 19.5), (17.5, 19.5));
            line((17.5, 19.5), (17.5, 10.0));
        }
        Icon::Recents => {
            p.rect_stroke(Rect::from_min_max(pt(4.5, 7.5), pt(16.5, 19.5)), 2.5 * s, st, eframe::egui::StrokeKind::Inside);
            line((8.5, 4.5), (19.5, 4.5));
            line((19.5, 4.5), (19.5, 15.5));
        }
        Icon::Gear => {
            p.circle_stroke(pt(12., 12.), 3.0 * s, st);
            p.circle_stroke(pt(12., 12.), 6.8 * s, st);
            for i in 0..8 {
                let a = (i as f32) * std::f32::consts::FRAC_PI_4;
                let (sn, cs) = a.sin_cos();
                p.line_segment(
                    [pt(12. + cs * 6.8, 12. + sn * 6.8), pt(12. + cs * 9.8, 12. + sn * 9.8)],
                    st,
                );
            }
        }
        Icon::Apk => {
            // Open tray + clean downward install arrow
            line((4., 14.), (4., 19.5));
            line((4., 19.5), (20., 19.5));
            line((20., 19.5), (20., 14.));
            line((12., 3.5), (12., 14.5));
            line((8., 10.5), (12., 14.5));
            line((16., 10.5), (12., 14.5));
        }
        Icon::Keyboard => {
            p.rect_stroke(Rect::from_min_max(pt(2., 5.5), pt(22., 18.5)), 2.5 * s, st, eframe::egui::StrokeKind::Inside);
            for x in [6., 10., 14., 18.] {
                p.circle_filled(pt(x, 9.5), 1.05 * s, color);
            }
            for x in [6., 18.] {
                p.circle_filled(pt(x, 14.), 1.05 * s, color);
            }
            line((9., 14.), (15., 14.));
        }
        Icon::Camera => { p.rect_stroke(Rect::from_min_max(pt(3., 7.), pt(21., 20.)), 2.5 * s, st, eframe::egui::StrokeKind::Inside); p.circle_stroke(pt(12., 13.5), 3.5 * s, st); line((8., 4.), (16., 4.)); }
        Icon::Record => { p.circle_filled(pt(12., 12.), 5.0 * s, color); p.circle_stroke(pt(12., 12.), 9.0 * s, st); }
        Icon::Pin => { p.circle_stroke(pt(12., 10.), 4.0 * s, st); line((12., 14.), (12., 22.)); }
        Icon::Volume | Icon::VolUp => {
            line((3., 9.), (3., 15.)); line((3., 9.), (7., 9.)); line((7., 9.), (12., 5.)); line((12., 5.), (12., 19.)); line((12., 19.), (7., 15.)); line((7., 15.), (3., 15.));
            line((16., 12.), (21., 12.)); line((18.5, 9.5), (18.5, 14.5));
        }
        Icon::VolDown => {
            line((3., 9.), (3., 15.)); line((3., 9.), (7., 9.)); line((7., 9.), (12., 5.)); line((12., 5.), (12., 19.)); line((12., 19.), (7., 15.)); line((7., 15.), (3., 15.));
            line((16., 12.), (21., 12.));
        }
        Icon::Fullscreen => {
            line((4., 9.), (4., 4.)); line((4., 4.), (9., 4.));
            line((15., 4.), (20., 4.)); line((20., 4.), (20., 9.));
            line((4., 15.), (4., 20.)); line((4., 20.), (9., 20.));
            line((15., 20.), (20., 20.)); line((20., 20.), (20., 15.));
        }
        Icon::Multi => {
            p.rect_stroke(Rect::from_min_max(pt(4., 8.), pt(16., 20.)), 2.0 * s, st, eframe::egui::StrokeKind::Inside);
            line((8., 4.), (20., 4.)); line((20., 4.), (20., 16.)); line((8., 4.), (8., 8.)); line((16., 16.), (20., 16.));
        }
        Icon::Location => {
            p.circle_stroke(pt(12., 9.5), 5.2 * s, st);
            p.circle_filled(pt(12., 9.5), 1.6 * s, color);
            line((7.6, 12.2), (12., 21.));
            line((16.4, 12.2), (12., 21.));
        }
        Icon::Folder => {
            p.rect_stroke(Rect::from_min_max(pt(3., 7.), pt(21., 19.)), 2.0 * s, st, eframe::egui::StrokeKind::Inside);
            line((3., 7.), (3., 5.)); line((3., 5.), (10., 5.)); line((10., 5.), (12., 7.));
        }
        Icon::More => {
            p.circle_filled(pt(6., 12.), 1.8 * s, color);
            p.circle_filled(pt(12., 12.), 1.8 * s, color);
            p.circle_filled(pt(18., 12.), 1.8 * s, color);
        }
        Icon::Collapse => {
            p.rect_stroke(Rect::from_min_max(pt(3., 5.), pt(21., 19.)), 2.0 * s, st, eframe::egui::StrokeKind::Inside);
            line((15., 5.), (15., 19.));
            line((9., 9.), (12., 12.));
            line((12., 12.), (9., 15.));
        }
    }
}
