//! Single source of truth for every number/color. Spec: docs/02_DESIGN_TOKENS.md
//! egui API names drift between versions — verify against docs.rs/egui/0.37.0 (`cargo check` first).
use eframe::egui::{self, Color32, CornerRadius, FontData, FontDefinitions, FontFamily, Stroke, Vec2};

// ---- colors (◆ = sampled from logo) ----
pub const BG_BASE: Color32 = Color32::from_rgb(0x0D, 0x11, 0x1A); // ◆
pub const BG_PANEL: Color32 = Color32::from_rgb(0x12, 0x18, 0x26);
pub const BG_ELEVATED: Color32 = Color32::from_rgb(0x18, 0x1F, 0x31);
pub const BG_HOVER: Color32 = Color32::from_rgb(0x1F, 0x28, 0x40);
pub const BG_PRESSED: Color32 = Color32::from_rgb(0x0A, 0x0E, 0x16);
pub const BORDER: Color32 = Color32::from_rgb(0x26, 0x2F, 0x45);
pub const TEXT: Color32 = Color32::from_rgb(0xF7, 0xF9, 0xFC); // ◆
pub const TEXT_DIM: Color32 = Color32::from_rgb(0x8B, 0x93, 0xA1);
pub const TEXT_DISABLED: Color32 = Color32::from_rgb(0x4A, 0x53, 0x65);
pub const ACCENT_CYAN: Color32 = Color32::from_rgb(0x01, 0xE7, 0xFF); // ◆
pub const ACCENT_BLUE: Color32 = Color32::from_rgb(0x16, 0xA6, 0xF5); // ◆
pub const ACCENT_GREEN: Color32 = Color32::from_rgb(0x01, 0xE8, 0xA0); // ◆
pub const DANGER: Color32 = Color32::from_rgb(0xFF, 0x4D, 0x5E);
pub const WARN: Color32 = Color32::from_rgb(0xFF, 0xB0, 0x20);

// ---- geometry (points @100%) ----
pub const TITLEBAR_H: f32 = 34.0;
pub const WINCTL_W: f32 = 44.0;
pub const WINCTL_GLYPH: f32 = 14.0;
pub const LOGO_H: f32 = 22.0;
pub const LOGO_W: f32 = 36.0; // 452:270 aspect
pub const TOOLBAR_W: f32 = 46.0;
pub const TOOLBAR_BTN: f32 = 38.0;
pub const TOOLBAR_GLYPH: f32 = 22.0;
pub const TOOLBAR_GAP: f32 = 6.0;
pub const NAV_H: f32 = 36.0;
pub const NAV_BTN: Vec2 = Vec2::new(40.0, 28.0);
pub const NAV_GLYPH: f32 = 16.0;
pub const WIN_DEFAULT: [f32; 2] = [1280.0, 760.0];
pub const WIN_MIN: [f32; 2] = [960.0, 600.0];
pub const DIALOG_SETTINGS: Vec2 = Vec2::new(760.0, 540.0);
pub const SETTINGS_NAV_W: f32 = 176.0;
pub const SETTINGS_ROW_H: f32 = 40.0;
pub const SETTINGS_LABEL_W: f32 = 200.0;
pub const KEYCAP: f32 = 36.0;
pub const KEYMAP_PANEL_W: f32 = 280.0;
pub const R_BTN: CornerRadius = CornerRadius::same(8);
pub const R_DIALOG: CornerRadius = CornerRadius::same(12);
pub const R_INPUT: CornerRadius = CornerRadius::same(6);
pub const HAIRLINE: f32 = 1.0;

// ---- fonts ----
pub const FS_BODY: f32 = 14.5;
pub const FS_SMALL: f32 = 12.5;
pub const FS_TITLE: f32 = 16.0;
pub const FS_SECTION: f32 = 11.0;
pub const FS_KEYCAP: f32 = 12.0;
pub fn semibold() -> FontFamily { FontFamily::Name("semibold".into()) }

/// Call ONCE at startup. Drop Inter-Regular.ttf / Inter-SemiBold.ttf into assets/fonts/ and
/// uncomment the include_bytes! lines (fonts are not bundled to keep the zip license-clean).
pub fn install(ctx: &egui::Context) {
    let mut fonts = FontDefinitions::default();
    // fonts.font_data.insert("inter".into(), std::sync::Arc::new(FontData::from_static(include_bytes!("../assets/fonts/Inter-Regular.ttf"))));
    // fonts.font_data.insert("inter-sb".into(), std::sync::Arc::new(FontData::from_static(include_bytes!("../assets/fonts/Inter-SemiBold.ttf"))));
    // fonts.families.entry(FontFamily::Proportional).or_default().insert(0, "inter".into());
    // fonts.families.insert(semibold(), vec!["inter-sb".into()]);
    let _ = FontData::from_static(&[]); // keeps import used until fonts are enabled
    fonts.families.insert(semibold(), fonts.families[&FontFamily::Proportional].clone());
    ctx.set_fonts(fonts);

    let mut v = egui::Visuals::dark();
    v.panel_fill = BG_BASE;
    v.window_fill = BG_PANEL;
    v.extreme_bg_color = BG_ELEVATED;
    v.faint_bg_color = BG_PANEL;
    v.override_text_color = Some(TEXT);
    v.window_stroke = Stroke::new(HAIRLINE, BORDER);
    v.window_corner_radius = R_DIALOG;
    v.selection.bg_fill = ACCENT_CYAN.gamma_multiply(0.25);
    v.selection.stroke = Stroke::new(1.0, ACCENT_CYAN);
    for w in [&mut v.widgets.inactive, &mut v.widgets.hovered, &mut v.widgets.active] {
        w.corner_radius = R_INPUT;
    }
    v.widgets.inactive.bg_fill = BG_ELEVATED;
    v.widgets.hovered.bg_fill = BG_HOVER;
    v.widgets.active.bg_fill = BG_PRESSED;
    ctx.set_visuals(v);
}
