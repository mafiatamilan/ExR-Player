use eframe::egui::{self, Color32, CornerRadius, Pos2, Rect, Sense, Stroke, StrokeKind, Vec2};
use exr_input::{GamepadButton, InputTrigger, KeyCode, KeyCombo, Modifiers, MouseButton};
use exr_keymap::{
    validate_profile, AxisCurve, ControlKind, ControlMapping, GamepadControlTarget, KeymapProfile,
    MobaCastMode, NormVec2, ProfileMode, ProfileStore, RepeatMode, SensorKind, UndoRedoStack,
    ValidationIssue, ViewportMapper,
};
use std::collections::HashSet;

const CYAN: Color32 = Color32::from_rgb(0x22, 0xD3, 0xEE);
const TEAL_FILL: Color32 = Color32::from_rgba_premultiplied(0x0E, 0x3A, 0x45, 210);
const PANEL_BG: Color32 = Color32::from_rgb(0x14, 0x16, 0x1A);
const CARD_BG: Color32 = Color32::from_rgb(0x1E, 0x22, 0x29);
const WARN_AMBER: Color32 = Color32::from_rgb(0xF5, 0x9E, 0x0B);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditorTab {
    KeyboardMouse,
    Gamepad,
    Macros,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaletteTemplate {
    TapSpot,
    RepeatedTap,
    Dpad,
    AnalogJoystick,
    FpsAim,
    CrosshairAim,
    Swipe,
    MobaSkill,
    FreeLook,
    ZoomPinch,
    RightClickWalk,
    GamepadButton,
    MacroTrigger,
    SensorControl,
}

impl PaletteTemplate {
    pub const ALL: &'static [Self] = &[
        Self::TapSpot,
        Self::RepeatedTap,
        Self::Dpad,
        Self::AnalogJoystick,
        Self::FpsAim,
        Self::CrosshairAim,
        Self::Swipe,
        Self::MobaSkill,
        Self::FreeLook,
        Self::ZoomPinch,
        Self::RightClickWalk,
        Self::GamepadButton,
        Self::MacroTrigger,
        Self::SensorControl,
    ];

    pub fn label(&self) -> &'static str {
        match self {
            Self::TapSpot => "Tap Spot",
            Self::RepeatedTap => "Repeat Tap",
            Self::Dpad => "D-Pad",
            Self::AnalogJoystick => "Analog",
            Self::FpsAim => "FPS Shoot",
            Self::CrosshairAim => "Crosshair",
            Self::Swipe => "Swipe",
            Self::MobaSkill => "MOBA Skill",
            Self::FreeLook => "Free Look",
            Self::ZoomPinch => "Zoom Pinch",
            Self::RightClickWalk => "R-Walk",
            Self::GamepadButton => "Gamepad",
            Self::MacroTrigger => "Macro",
            Self::SensorControl => "Sensor",
        }
    }

    pub fn instantiate(&self, id: String, pos: NormVec2) -> ControlMapping {
        let kind = match self {
            Self::TapSpot => ControlKind::TapSpot {
                trigger: InputTrigger::key(KeyCode::F),
                extra_triggers: vec![],
                hold_duration_ms: 0,
            },
            Self::RepeatedTap => ControlKind::RepeatedTap {
                trigger: InputTrigger::key(KeyCode::V),
                interval_ms: 80,
                initial_delay_ms: 0,
                mode: RepeatMode::Hold,
            },
            Self::Dpad => ControlKind::Dpad {
                up: KeyCode::W,
                down: KeyCode::S,
                left: KeyCode::A,
                right: KeyCode::D,
                radius: 0.09,
                deadzone: 0.05,
                sensitivity: 1.0,
                return_to_center: true,
            },
            Self::AnalogJoystick => ControlKind::AnalogJoystick {
                up: KeyCode::Up,
                down: KeyCode::Down,
                left: KeyCode::Left,
                right: KeyCode::Right,
                radius: 0.09,
                deadzone: 0.05,
                curve: AxisCurve::SCurve,
                invert_x: false,
                invert_y: false,
            },
            Self::FpsAim => ControlKind::FpsAim {
                toggle_key: InputTrigger::key(KeyCode::Backtick),
                fire_key: InputTrigger::mouse(MouseButton::Left),
                secondary_fire_key: Some(InputTrigger::mouse(MouseButton::Right)),
                sensitivity_x: 1.0,
                sensitivity_y: 0.8,
                invert_y: false,
                smoothing: 0.15,
                pitch_min: -80.0,
                pitch_max: 80.0,
            },
            Self::CrosshairAim => ControlKind::CrosshairAim {
                trigger: InputTrigger::key(KeyCode::Tab),
                fire_trigger: InputTrigger::mouse(MouseButton::Left),
                region_radius: 0.15,
                sensitivity_x: 1.0,
                sensitivity_y: 1.0,
            },
            Self::Swipe => ControlKind::Swipe {
                trigger: InputTrigger::key(KeyCode::X),
                end_pos: NormVec2::new((pos.x + 0.12).min(0.95), pos.y),
                duration_ms: 180,
            },
            Self::MobaSkill => ControlKind::MobaSkill {
                trigger: InputTrigger::key(KeyCode::Q),
                cancel_trigger: Some(InputTrigger::key(KeyCode::Escape)),
                cast_mode: MobaCastMode::Quick,
                max_radius: 0.12,
                confirm_on_release: true,
            },
            Self::FreeLook => ControlKind::FreeLook {
                trigger: InputTrigger::key(KeyCode::LAlt),
                sensitivity_x: 1.0,
                sensitivity_y: 1.0,
                radius: 0.15,
            },
            Self::ZoomPinch => ControlKind::ZoomPinch {
                zoom_in_trigger: InputTrigger::key(KeyCode::Equals),
                zoom_out_trigger: InputTrigger::key(KeyCode::Minus),
                spread_radius: 0.16,
                duration_ms: 160,
                speed: 1.0,
            },
            Self::RightClickWalk => ControlKind::RightClickWalk {
                trigger: InputTrigger::mouse(MouseButton::Right),
                reduce_inertia: true,
                hold_to_walk: true,
            },
            Self::GamepadButton => ControlKind::GamepadControl {
                button: Some(GamepadButton::South),
                axis_x: None,
                axis_y: None,
                target: GamepadControlTarget::TapSpot,
                deadzone: 0.15,
                curve: AxisCurve::Linear,
                radius: 0.08,
            },
            Self::MacroTrigger => ControlKind::MacroTrigger {
                trigger: InputTrigger::key(KeyCode::G),
                macro_id: "macro-1".into(),
                loop_count: 1,
                speed: 1.0,
            },
            Self::SensorControl => ControlKind::SensorControl {
                sensor: SensorKind::Accelerometer,
                positive_trigger: InputTrigger::key(KeyCode::E),
                negative_trigger: Some(InputTrigger::key(KeyCode::Q)),
                axis_index: 0,
                magnitude: 4.5,
                smoothing: 0.2,
            },
        };
        ControlMapping {
            id,
            label: self.label().to_string(),
            pos: pos.clamped(),
            enabled: true,
            kind,
        }
    }
}

pub enum EditorAction {
    None,
    ProfileChanged,
    Saved(String),
    CloseRequested,
    ToggleTestMode(bool),
    ExportRequested,
    ImportRequested,
}

pub struct KeymapEditorState {
    pub profile: KeymapProfile,
    pub undo: UndoRedoStack,
    pub selected_id: Option<String>,
    pub capturing_trigger_for: Option<String>,
    pub pending_template: Option<PaletteTemplate>,
    pub snap_to_grid: bool,
    pub show_grid: bool,
    pub test_mode: bool,
    pub active_tab: EditorTab,
    pub issues: Vec<ValidationIssue>,
    pub status: Option<String>,
    next_seq: usize,
}

impl KeymapEditorState {
    pub fn new(profile: KeymapProfile) -> Self {
        let issues = validate_profile(&profile);
        Self {
            profile,
            undo: UndoRedoStack::new(40),
            selected_id: None,
            capturing_trigger_for: None,
            pending_template: None,
            snap_to_grid: false,
            show_grid: true,
            test_mode: false,
            active_tab: EditorTab::KeyboardMouse,
            issues,
            status: None,
            next_seq: 100,
        }
    }

    pub fn refresh_validation(&mut self) {
        self.issues = validate_profile(&self.profile);
    }

    fn push_undo(&mut self) {
        self.undo.push_snapshot(self.profile.clone());
    }

    fn alloc_id(&mut self, prefix: &str) -> String {
        self.next_seq += 1;
        format!("{prefix}-{}", self.next_seq)
    }

    pub fn add_control_at(&mut self, tpl: PaletteTemplate, pos: NormVec2) {
        self.push_undo();
        let id = self.alloc_id("ctrl");
        let snapped = if self.snap_to_grid {
            ViewportMapper::snap_to_grid(pos, 0.025)
        } else {
            pos.clamped()
        };
        let ctrl = tpl.instantiate(id.clone(), snapped);
        self.profile.controls.push(ctrl);
        self.selected_id = Some(id);
        self.refresh_validation();
    }

    pub fn remove_control(&mut self, id: &str) {
        self.push_undo();
        self.profile.controls.retain(|c| c.id != id);
        if self.selected_id.as_deref() == Some(id) {
            self.selected_id = None;
        }
        self.refresh_validation();
    }

    /// Captures a keyboard/mouse trigger when rebinding a selected control.
    pub fn try_capture_egui_input(&mut self, ctx: &egui::Context) -> bool {
        let Some(target_id) = self.capturing_trigger_for.clone() else {
            return false;
        };
        let mut captured: Option<InputTrigger> = None;
        ctx.input(|i| {
            let mods = Modifiers {
                ctrl: i.modifiers.ctrl,
                shift: i.modifiers.shift,
                alt: i.modifiers.alt,
                super_key: i.modifiers.mac_cmd || i.modifiers.command,
            };
            for ev in &i.events {
                match ev {
                    egui::Event::Key { key, pressed: true, .. } => {
                        if let Some(kc) = egui_key_to_exr(*key) {
                            if !kc.is_modifier() {
                                captured = Some(InputTrigger::Key(KeyCombo::with_mods(kc, mods)));
                                break;
                            }
                        }
                    }
                    egui::Event::PointerButton { button, pressed: true, .. } => {
                        let mb = match button {
                            egui::PointerButton::Primary => MouseButton::Left,
                            egui::PointerButton::Secondary => MouseButton::Right,
                            egui::PointerButton::Middle => MouseButton::Middle,
                            egui::PointerButton::Extra1 => MouseButton::Back,
                            egui::PointerButton::Extra2 => MouseButton::Forward,
                        };
                        captured = Some(InputTrigger::Mouse(mb));
                        break;
                    }
                    _ => {}
                }
            }
        });
        if let Some(trig) = captured {
            self.push_undo();
            if let Some(ctrl) = self.profile.controls.iter_mut().find(|c| c.id == target_id) {
                set_primary_trigger(&mut ctrl.kind, trig);
            }
            self.capturing_trigger_for = None;
            self.refresh_validation();
            return true;
        }
        false
    }
}

fn set_primary_trigger(kind: &mut ControlKind, trig: InputTrigger) {
    match kind {
        ControlKind::TapSpot { trigger, .. }
        | ControlKind::RepeatedTap { trigger, .. }
        | ControlKind::Swipe { trigger, .. }
        | ControlKind::MobaSkill { trigger, .. }
        | ControlKind::FreeLook { trigger, .. }
        | ControlKind::RightClickWalk { trigger, .. }
        | ControlKind::MacroTrigger { trigger, .. }
        | ControlKind::CrosshairAim { trigger, .. } => *trigger = trig,
        ControlKind::FpsAim { toggle_key, .. } => *toggle_key = trig,
        ControlKind::ZoomPinch { zoom_in_trigger, .. } => *zoom_in_trigger = trig,
        ControlKind::SensorControl { positive_trigger, .. } => *positive_trigger = trig,
        ControlKind::GamepadControl { button, .. } => {
            if let InputTrigger::GamepadButton(b) = trig {
                *button = Some(b);
            }
        }
        ControlKind::Dpad { .. } | ControlKind::AnalogJoystick { .. } => {}
    }
}

/// Top editor toolbar inside the viewport/window.
pub fn show_top_bar(
    ui: &mut egui::Ui,
    state: &mut KeymapEditorState,
    store: &ProfileStore,
) -> EditorAction {
    let mut action = EditorAction::None;
    egui::Frame::new()
        .fill(PANEL_BG)
        .inner_margin(egui::Margin::symmetric(10, 6))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new("KEYMAPPING EDITOR")
                        .strong()
                        .color(CYAN),
                );
                ui.separator();

                egui::ComboBox::from_id_salt("exr_profile_preset")
                    .selected_text(&state.profile.name)
                    .show_ui(ui, |ui| {
                        if ui.button("FPS Default (Smart)").clicked() {
                            state.push_undo();
                            state.profile = KeymapProfile::fps_default();
                            state.refresh_validation();
                            action = EditorAction::ProfileChanged;
                        }
                        if ui.button("MOBA Default").clicked() {
                            state.push_undo();
                            state.profile = KeymapProfile::moba_default();
                            state.refresh_validation();
                            action = EditorAction::ProfileChanged;
                        }
                        if ui.button("Blank Profile").clicked() {
                            state.push_undo();
                            state.profile = KeymapProfile::blank("custom", "Custom Profile");
                            state.refresh_validation();
                            action = EditorAction::ProfileChanged;
                        }
                        for id in store.list_ids() {
                            if ui.button(format!("Saved: {id}")).clicked() {
                                if let Ok(p) = store.load(&id) {
                                    state.push_undo();
                                    state.profile = p;
                                    state.refresh_validation();
                                    action = EditorAction::ProfileChanged;
                                }
                            }
                        }
                    });

                if ui
                    .add_enabled(state.undo.can_undo(), egui::Button::new("Undo"))
                    .clicked()
                {
                    if let Some(prev) = state.undo.undo(&state.profile) {
                        state.profile = prev;
                        state.refresh_validation();
                        action = EditorAction::ProfileChanged;
                    }
                }
                if ui
                    .add_enabled(state.undo.can_redo(), egui::Button::new("Redo"))
                    .clicked()
                {
                    if let Some(next) = state.undo.redo(&state.profile) {
                        state.profile = next;
                        state.refresh_validation();
                        action = EditorAction::ProfileChanged;
                    }
                }

                ui.checkbox(&mut state.snap_to_grid, "Snap");
                if ui
                    .toggle_value(&mut state.test_mode, "Test Mode")
                    .changed()
                {
                    action = EditorAction::ToggleTestMode(state.test_mode);
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("Done").clicked() {
                        action = EditorAction::CloseRequested;
                    }
                    if ui
                        .add(
                            egui::Button::new(egui::RichText::new("Save").strong().color(Color32::BLACK))
                                .fill(CYAN),
                        )
                        .clicked()
                    {
                        match store.save(&state.profile) {
                            Ok(_) => {
                                state.status = Some(format!("Saved '{}'", state.profile.name));
                                action = EditorAction::Saved(state.profile.id.clone());
                            }
                            Err(e) => state.status = Some(format!("Save error: {e}")),
                        }
                    }
                    if ui.button("Export").clicked() {
                        action = EditorAction::ExportRequested;
                    }
                    if ui.button("Import").clicked() {
                        action = EditorAction::ImportRequested;
                    }
                    if ui.button("Reset").clicked() {
                        state.push_undo();
                        state.profile = KeymapProfile::fps_default();
                        state.refresh_validation();
                        action = EditorAction::ProfileChanged;
                    }
                });
            });
        });
    action
}

/// Right-side Control Palette & Properties Panel.
pub fn show_side_panel(ui: &mut egui::Ui, state: &mut KeymapEditorState) -> bool {
    let mut changed = false;
    egui::Frame::new()
        .fill(PANEL_BG)
        .inner_margin(egui::Margin::same(10))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.selectable_value(&mut state.profile.mode, ProfileMode::Standard, "Standard");
                ui.selectable_value(&mut state.profile.mode, ProfileMode::Smart, "Smart Mode");
            });
            ui.separator();

            ui.horizontal(|ui| {
                ui.selectable_value(&mut state.active_tab, EditorTab::KeyboardMouse, "KBM");
                ui.selectable_value(&mut state.active_tab, EditorTab::Gamepad, "Gamepad");
                ui.selectable_value(&mut state.active_tab, EditorTab::Macros, "Macros");
            });
            ui.add_space(6.0);

            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("App:").small().color(Color32::GRAY));
                if ui
                    .add(egui::TextEdit::singleline(&mut state.profile.name).desired_width(110.0))
                    .changed()
                {
                    let slug: String = state
                        .profile
                        .name
                        .to_ascii_lowercase()
                        .chars()
                        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
                        .collect();
                    if !slug.trim_matches('-').is_empty() {
                        state.profile.id = slug.trim_matches('-').to_string();
                    }
                    changed = true;
                }
                let mut pkg = state.profile.game_package.clone().unwrap_or_default();
                if ui
                    .add(
                        egui::TextEdit::singleline(&mut pkg)
                            .hint_text("com.app.id")
                            .desired_width(95.0),
                    )
                    .changed()
                {
                    state.profile.game_package = if pkg.trim().is_empty() {
                        None
                    } else {
                        Some(pkg.trim().to_string())
                    };
                    changed = true;
                }
            });
            ui.add_space(4.0);

            ui.label(
                egui::RichText::new("Click a control, then click viewport (double-click adds)")
                    .small()
                    .color(Color32::GRAY),
            );
            ui.add_space(4.0);

            egui::Grid::new("exr_palette_grid")
                .num_columns(3)
                .spacing([6.0, 6.0])
                .show(ui, |ui| {
                    for (idx, tpl) in PaletteTemplate::ALL.iter().enumerate() {
                        let active = state.pending_template == Some(*tpl);
                        let resp = palette_tile(ui, *tpl, active);
                        if resp.clicked() {
                            state.pending_template = if active { None } else { Some(*tpl) };
                        }
                        if resp.double_clicked() {
                            state.add_control_at(*tpl, NormVec2::new(0.5, 0.5));
                            state.pending_template = None;
                            changed = true;
                        }
                        if idx % 3 == 2 {
                            ui.end_row();
                        }
                    }
                });

            ui.separator();
            ui.label(egui::RichText::new("Properties").strong().color(CYAN));

            if let Some(sel_id) = state.selected_id.clone() {
                if let Some(ctrl) = state.profile.controls.iter_mut().find(|c| c.id == sel_id) {
                    ui.horizontal(|ui| {
                        ui.label("Label:");
                        if ui.text_edit_singleline(&mut ctrl.label).changed() {
                            changed = true;
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label(format!("Pos: ({:.2}, {:.2})", ctrl.pos.x, ctrl.pos.y));
                        ui.checkbox(&mut ctrl.enabled, "Enabled");
                    });

                    let is_cap = state.capturing_trigger_for.as_deref() == Some(&sel_id);
                    let cap_label = if is_cap {
                        "Press any key / mouse..."
                    } else {
                        "Rebind Primary Key"
                    };
                    if ui.button(cap_label).clicked() {
                        state.capturing_trigger_for = if is_cap { None } else { Some(sel_id.clone()) };
                    }

                    match &mut ctrl.kind {
                        ControlKind::Dpad { radius, sensitivity, return_to_center, .. } => {
                            changed |= ui.add(egui::Slider::new(radius, 0.04..=0.25).text("Radius")).changed();
                            changed |= ui.add(egui::Slider::new(sensitivity, 0.2..=2.0).text("Sens")).changed();
                            changed |= ui.checkbox(return_to_center, "Return to center").changed();
                        }
                        ControlKind::FpsAim {
                            sensitivity_x,
                            sensitivity_y,
                            invert_y,
                            smoothing,
                            ..
                        } => {
                            changed |= ui.add(egui::Slider::new(sensitivity_x, 0.1..=4.0).text("X Sens")).changed();
                            changed |= ui.add(egui::Slider::new(sensitivity_y, 0.1..=4.0).text("Y Sens")).changed();
                            changed |= ui.add(egui::Slider::new(smoothing, 0.0..=0.9).text("Smooth")).changed();
                            changed |= ui.checkbox(invert_y, "Invert Y").changed();
                        }
                        ControlKind::RepeatedTap { interval_ms, .. } => {
                            let mut ms = *interval_ms as f32;
                            if ui.add(egui::Slider::new(&mut ms, 15.0..=1000.0).text("Interval ms")).changed() {
                                *interval_ms = ms as u32;
                                changed = true;
                            }
                        }
                        ControlKind::MobaSkill { max_radius, cast_mode, .. } => {
                            changed |= ui.add(egui::Slider::new(max_radius, 0.05..=0.30).text("Range")).changed();
                            ui.horizontal(|ui| {
                                changed |= ui.selectable_value(cast_mode, MobaCastMode::Quick, "Quick").changed();
                                changed |= ui.selectable_value(cast_mode, MobaCastMode::Auto, "Auto").changed();
                                changed |= ui.selectable_value(cast_mode, MobaCastMode::Manual, "Manual").changed();
                            });
                        }
                        ControlKind::Swipe { duration_ms, .. } => {
                            let mut ms = *duration_ms as f32;
                            if ui.add(egui::Slider::new(&mut ms, 30.0..=1000.0).text("Duration ms")).changed() {
                                *duration_ms = ms as u32;
                                changed = true;
                            }
                        }
                        _ => {}
                    }

                    if ui.button(egui::RichText::new("Delete Control").color(WARN_AMBER)).clicked() {
                        state.remove_control(&sel_id);
                        changed = true;
                    }
                }
            } else {
                ui.label(egui::RichText::new("Select a control on canvas to edit.").small().color(Color32::GRAY));
            }

            ui.separator();
            ui.add(egui::Slider::new(&mut state.profile.overlay_opacity, 0.1..=1.0).text("Overlay Opacity"));
            ui.checkbox(&mut state.profile.show_overlay_in_play, "Show overlay in Play Mode");

            if !state.issues.is_empty() {
                ui.add_space(4.0);
                ui.label(
                    egui::RichText::new(format!("{} binding warning(s)", state.issues.len()))
                        .small()
                        .color(WARN_AMBER),
                );
            }
            if let Some(msg) = &state.status {
                ui.label(egui::RichText::new(msg).small().color(CYAN));
            }
        });
    changed
}

/// Interactive Keymapping Canvas Overlay rendered directly over the guest viewport `Rect`.
pub fn paint_and_handle_overlay(
    ui: &mut egui::Ui,
    viewport_rect: Rect,
    state: &mut KeymapEditorState,
) -> bool {
    let mut changed = false;
    let mapper = ViewportMapper::new(
        viewport_rect.left(),
        viewport_rect.top(),
        viewport_rect.width(),
        viewport_rect.height(),
    );

    let painter = ui.painter_at(viewport_rect);
    painter.rect_filled(viewport_rect, 0.0, Color32::from_black_alpha(95));

    if state.show_grid {
        let step = 0.05;
        let grid_color = Color32::from_white_alpha(18);
        let mut g = step;
        while g < 1.0 {
            let (x, _) = mapper.norm_to_host(NormVec2::new(g, 0.0));
            let (_, y) = mapper.norm_to_host(NormVec2::new(0.0, g));
            painter.line_segment(
                [Pos2::new(x, viewport_rect.top()), Pos2::new(x, viewport_rect.bottom())],
                Stroke::new(1.0, grid_color),
            );
            painter.line_segment(
                [Pos2::new(viewport_rect.left(), y), Pos2::new(viewport_rect.right(), y)],
                Stroke::new(1.0, grid_color),
            );
            g += step;
        }
    }

    // Place pending palette control on canvas click
    if let Some(tpl) = state.pending_template {
        let bg_resp = ui.interact(viewport_rect, ui.scope_id().with("keymap_bg_drop"), Sense::click());
        if bg_resp.clicked() {
            if let Some(pos) = bg_resp.interact_pointer_pos() {
                let norm = mapper.host_to_norm(pos.x, pos.y);
                state.add_control_at(tpl, norm);
                state.pending_template = None;
                changed = true;
            }
        }
    }

    let conflicted_ids: HashSet<String> = state
        .issues
        .iter()
        .map(|i| i.control_id.clone())
        .collect();

    let mut delete_target: Option<String> = None;
    let controls_len = state.profile.controls.len();
    for idx in 0..controls_len {
        let (id, pos, kind, enabled) = {
            let c = &state.profile.controls[idx];
            (c.id.clone(), c.pos, c.kind.clone(), c.enabled)
        };
        let (hx, hy) = mapper.norm_to_host(pos);
        let center = Pos2::new(hx, hy);
        let is_selected = state.selected_id.as_deref() == Some(&id);
        let has_warn = conflicted_ids.contains(&id);

        let stroke_color = if !enabled {
            Color32::GRAY
        } else if has_warn {
            WARN_AMBER
        } else if is_selected {
            Color32::WHITE
        } else {
            CYAN
        };

        let hit_radius = match &kind {
            ControlKind::Dpad { radius, .. } | ControlKind::AnalogJoystick { radius, .. } => {
                (radius * viewport_rect.width()).clamp(28.0, 90.0)
            }
            ControlKind::FpsAim { .. } | ControlKind::CrosshairAim { .. } => 26.0,
            _ => 20.0,
        };

        let widget_rect = Rect::from_center_size(center, Vec2::splat(hit_radius * 2.0));
        let resp = ui.interact(
            widget_rect,
            ui.scope_id().with(("ctrl_node", &id)),
            Sense::click_and_drag(),
        );

        if resp.clicked() {
            state.selected_id = Some(id.clone());
        }
        if resp.double_clicked() {
            state.selected_id = Some(id.clone());
            state.capturing_trigger_for = Some(id.clone());
        }
        if resp.drag_started() {
            state.push_undo();
            state.selected_id = Some(id.clone());
        }
        if resp.dragged() {
            let new_host = center + resp.drag_delta();
            let mut norm = mapper.host_to_norm(new_host.x, new_host.y);
            if state.snap_to_grid {
                norm = ViewportMapper::snap_to_grid(norm, 0.025);
            }
            state.profile.controls[idx].pos = norm;
            changed = true;
        }

        // Render control badge
        match &kind {
            ControlKind::Dpad { up, down, left, right, .. }
            | ControlKind::AnalogJoystick { up, down, left, right, .. } => {
                painter.circle(
                    center,
                    hit_radius,
                    Color32::from_rgba_premultiplied(14, 58, 69, 150),
                    Stroke::new(2.0, stroke_color),
                );
                let off = hit_radius * 0.58;
                let font = egui::FontId::proportional(12.0);
                painter.text(center + Vec2::new(0.0, -off), egui::Align2::CENTER_CENTER, up.short_label(), font.clone(), Color32::WHITE);
                painter.text(center + Vec2::new(0.0, off), egui::Align2::CENTER_CENTER, down.short_label(), font.clone(), Color32::WHITE);
                painter.text(center + Vec2::new(-off, 0.0), egui::Align2::CENTER_CENTER, left.short_label(), font.clone(), Color32::WHITE);
                painter.text(center + Vec2::new(off, 0.0), egui::Align2::CENTER_CENTER, right.short_label(), font, Color32::WHITE);
            }
            ControlKind::FpsAim {
                toggle_key,
                sensitivity_x,
                sensitivity_y,
                ..
            } => {
                painter.circle(center, 22.0, TEAL_FILL, Stroke::new(2.0, stroke_color));
                painter.line_segment(
                    [center - Vec2::new(26.0, 0.0), center + Vec2::new(26.0, 0.0)],
                    Stroke::new(1.5, stroke_color),
                );
                painter.line_segment(
                    [center - Vec2::new(0.0, 26.0), center + Vec2::new(0.0, 26.0)],
                    Stroke::new(1.5, stroke_color),
                );
                painter.text(
                    center,
                    egui::Align2::CENTER_CENTER,
                    toggle_key.to_string(),
                    egui::FontId::proportional(13.0),
                    Color32::WHITE,
                );
                painter.text(
                    center + Vec2::new(0.0, 32.0),
                    egui::Align2::CENTER_CENTER,
                    format!("X:{:.1} Y:{:.1}", sensitivity_x, sensitivity_y),
                    egui::FontId::proportional(10.5),
                    CYAN,
                );
            }
            ControlKind::Swipe { end_pos, .. } => {
                let (ex, ey) = mapper.norm_to_host(*end_pos);
                let end = Pos2::new(ex, ey);
                painter.line_segment([center, end], Stroke::new(2.5, stroke_color));
                painter.circle_filled(end, 5.0, stroke_color);
                painter.circle(center, 18.0, TEAL_FILL, Stroke::new(2.0, stroke_color));
                painter.text(
                    center,
                    egui::Align2::CENTER_CENTER,
                    kind.primary_badge_text(),
                    egui::FontId::proportional(12.0),
                    Color32::WHITE,
                );
            }
            _ => {
                let label = kind.primary_badge_text();
                let pill_w = (label.len() as f32 * 8.0 + 18.0).max(34.0);
                let pill = Rect::from_center_size(center, Vec2::new(pill_w, 28.0));
                painter.rect(
                    pill,
                    CornerRadius::same(14),
                    TEAL_FILL,
                    Stroke::new(2.0, stroke_color),
                    StrokeKind::Outside,
                );
                painter.text(
                    center,
                    egui::Align2::CENTER_CENTER,
                    label,
                    egui::FontId::proportional(12.0),
                    Color32::WHITE,
                );
            }
        }

        // Corner delete button on selected control
        if is_selected {
            let del_center = center + Vec2::new(hit_radius * 0.8, -hit_radius * 0.8);
            let del_rect = Rect::from_center_size(del_center, Vec2::splat(16.0));
            painter.circle_filled(del_center, 8.0, Color32::from_rgb(0xDC, 0x26, 0x26));
            painter.text(
                del_center,
                egui::Align2::CENTER_CENTER,
                "x",
                egui::FontId::proportional(11.0),
                Color32::WHITE,
            );
            if ui
                .interact(del_rect, ui.scope_id().with(("ctrl_del", &id)), Sense::click())
                .clicked()
            {
                delete_target = Some(id);
            }
        }
    }

    if let Some(del_id) = delete_target {
        state.remove_control(&del_id);
        changed = true;
    }

    changed
}

/// Lightweight read-only keymap HUD overlay during Play Mode.
pub fn paint_play_overlay(
    ui: &mut egui::Ui,
    viewport_rect: Rect,
    profile: &KeymapProfile,
    fps_captured: bool,
) {
    if !profile.show_overlay_in_play {
        return;
    }
    let alpha = (profile.overlay_opacity.clamp(0.1, 1.0) * 200.0) as u8;
    let painter = ui.painter_at(viewport_rect);
    let mapper = ViewportMapper::new(
        viewport_rect.left(),
        viewport_rect.top(),
        viewport_rect.width(),
        viewport_rect.height(),
    );

    let bg = Color32::from_rgba_unmultiplied(14, 58, 69, alpha);
    let fg = Color32::from_rgba_unmultiplied(255, 255, 255, alpha.saturating_add(40));
    let border = Color32::from_rgba_unmultiplied(34, 211, 238, alpha);

    for ctrl in &profile.controls {
        if !ctrl.enabled {
            continue;
        }
        let (hx, hy) = mapper.norm_to_host(ctrl.pos);
        let center = Pos2::new(hx, hy);
        let label = ctrl.kind.primary_badge_text();
        let w = (label.len() as f32 * 7.5 + 14.0).max(26.0);
        let pill = Rect::from_center_size(center, Vec2::new(w, 22.0));
        painter.rect(
            pill,
            CornerRadius::same(11),
            bg,
            Stroke::new(1.2, border),
            StrokeKind::Outside,
        );
        painter.text(
            center,
            egui::Align2::CENTER_CENTER,
            label,
            egui::FontId::proportional(11.0),
            fg,
        );
    }

    if fps_captured {
        let banner_center = Pos2::new(viewport_rect.center().x, viewport_rect.top() + 18.0);
        let banner = Rect::from_center_size(banner_center, Vec2::new(260.0, 22.0));
        painter.rect_filled(banner, CornerRadius::same(6), Color32::from_black_alpha(170));
        painter.text(
            banner_center,
            egui::Align2::CENTER_CENTER,
            "FPS AIM LOCKED — Press ~ or Esc to unlock",
            egui::FontId::proportional(11.0),
            CYAN,
        );
    }
}

/// Converts an `egui::Key` into an `exr_input::KeyCode`.
pub fn egui_key_to_exr(key: egui::Key) -> Option<KeyCode> {
    use egui::Key as K;
    Some(match key {
        K::A => KeyCode::A, K::B => KeyCode::B, K::C => KeyCode::C, K::D => KeyCode::D,
        K::E => KeyCode::E, K::F => KeyCode::F, K::G => KeyCode::G, K::H => KeyCode::H,
        K::I => KeyCode::I, K::J => KeyCode::J, K::K => KeyCode::K, K::L => KeyCode::L,
        K::M => KeyCode::M, K::N => KeyCode::N, K::O => KeyCode::O, K::P => KeyCode::P,
        K::Q => KeyCode::Q, K::R => KeyCode::R, K::S => KeyCode::S, K::T => KeyCode::T,
        K::U => KeyCode::U, K::V => KeyCode::V, K::W => KeyCode::W, K::X => KeyCode::X,
        K::Y => KeyCode::Y, K::Z => KeyCode::Z,
        K::Num0 => KeyCode::Num0, K::Num1 => KeyCode::Num1, K::Num2 => KeyCode::Num2,
        K::Num3 => KeyCode::Num3, K::Num4 => KeyCode::Num4, K::Num5 => KeyCode::Num5,
        K::Num6 => KeyCode::Num6, K::Num7 => KeyCode::Num7, K::Num8 => KeyCode::Num8,
        K::Num9 => KeyCode::Num9,
        K::F1 => KeyCode::F1, K::F2 => KeyCode::F2, K::F3 => KeyCode::F3, K::F4 => KeyCode::F4,
        K::F5 => KeyCode::F5, K::F6 => KeyCode::F6, K::F7 => KeyCode::F7, K::F8 => KeyCode::F8,
        K::F9 => KeyCode::F9, K::F10 => KeyCode::F10, K::F11 => KeyCode::F11, K::F12 => KeyCode::F12,
        K::Space => KeyCode::Space,
        K::Enter => KeyCode::Enter,
        K::Escape => KeyCode::Escape,
        K::Tab => KeyCode::Tab,
        K::Backspace => KeyCode::Backspace,
        K::ArrowUp => KeyCode::Up,
        K::ArrowDown => KeyCode::Down,
        K::ArrowLeft => KeyCode::Left,
        K::ArrowRight => KeyCode::Right,
        K::Home => KeyCode::Home,
        K::End => KeyCode::End,
        K::PageUp => KeyCode::PageUp,
        K::PageDown => KeyCode::PageDown,
        K::Insert => KeyCode::Insert,
        K::Delete => KeyCode::Delete,
        K::Backtick => KeyCode::Backtick,
        K::Minus => KeyCode::Minus,
        K::Equals => KeyCode::Equals,
        K::OpenBracket => KeyCode::BracketLeft,
        K::CloseBracket => KeyCode::BracketRight,
        K::Backslash => KeyCode::Backslash,
        K::Semicolon => KeyCode::Semicolon,
        K::Quote => KeyCode::Quote,
        K::Comma => KeyCode::Comma,
        K::Period => KeyCode::Period,
        K::Slash => KeyCode::Slash,
        _ => return None,
    })
}

fn palette_tile(ui: &mut egui::Ui, tpl: PaletteTemplate, active: bool) -> egui::Response {
    let size = Vec2::new(82.0, 52.0);
    let (rect, resp) = ui.allocate_exact_size(size, Sense::click());
    let hovered = resp.hovered();
    let fill = if active {
        TEAL_FILL
    } else if hovered {
        Color32::from_rgb(0x27, 0x2C, 0x35)
    } else {
        CARD_BG
    };
    let border = if active {
        Stroke::new(1.5, CYAN)
    } else if hovered {
        Stroke::new(1.0, Color32::from_rgb(0x45, 0x50, 0x62))
    } else {
        Stroke::new(1.0, Color32::from_rgb(0x28, 0x2E, 0x38))
    };

    let p = ui.painter_at(rect);
    p.rect(rect, CornerRadius::same(6), fill, border, StrokeKind::Inside);

    let icon_color = if active || hovered { CYAN } else { Color32::from_rgb(0xD0, 0xD7, 0xE2) };
    let icon_center = Pos2::new(rect.center().x, rect.top() + 19.0);
    paint_palette_icon(&p, icon_center, tpl, icon_color);

    p.text(
        Pos2::new(rect.center().x, rect.bottom() - 10.0),
        egui::Align2::CENTER_CENTER,
        tpl.label(),
        egui::FontId::proportional(10.0),
        Color32::from_rgb(0xD8, 0xDF, 0xEA),
    );
    resp
}

fn paint_palette_icon(painter: &egui::Painter, c: Pos2, tpl: PaletteTemplate, color: Color32) {
    let st = Stroke::new(1.7, color);
    match tpl {
        PaletteTemplate::TapSpot => {
            painter.circle_stroke(c, 10.0, st);
            painter.text(c, egui::Align2::CENTER_CENTER, "A", egui::FontId::proportional(11.0), color);
        }
        PaletteTemplate::RepeatedTap => {
            painter.circle_stroke(c + Vec2::new(-3.0, 1.0), 8.0, st);
            painter.circle_stroke(c + Vec2::new(3.5, -1.5), 8.0, st);
            painter.text(c + Vec2::new(3.5, -1.5), egui::Align2::CENTER_CENTER, "A", egui::FontId::proportional(9.5), color);
        }
        PaletteTemplate::Dpad => {
            painter.circle_stroke(c, 10.5, st);
            painter.line_segment([c - Vec2::new(6.0, 0.0), c + Vec2::new(6.0, 0.0)], st);
            painter.line_segment([c - Vec2::new(0.0, 6.0), c + Vec2::new(0.0, 6.0)], st);
        }
        PaletteTemplate::AnalogJoystick => {
            painter.circle_stroke(c, 10.5, st);
            painter.circle_filled(c, 4.0, color);
            for (dx, dy) in [(0.0, -8.0), (0.0, 8.0), (-8.0, 0.0), (8.0, 0.0)] {
                painter.circle_filled(c + Vec2::new(dx, dy), 1.3, color);
            }
        }
        PaletteTemplate::FpsAim => {
            painter.circle_stroke(c, 8.5, st);
            painter.circle_filled(c, 2.0, color);
            painter.line_segment([c - Vec2::new(12.0, 0.0), c - Vec2::new(5.0, 0.0)], st);
            painter.line_segment([c + Vec2::new(5.0, 0.0), c + Vec2::new(12.0, 0.0)], st);
            painter.line_segment([c - Vec2::new(0.0, 12.0), c - Vec2::new(0.0, 5.0)], st);
            painter.line_segment([c + Vec2::new(0.0, 5.0), c + Vec2::new(0.0, 12.0)], st);
        }
        PaletteTemplate::CrosshairAim => {
            painter.circle_stroke(c, 6.0, st);
            painter.circle_filled(c, 1.8, color);
            let r = 10.0;
            painter.line_segment([c + Vec2::new(-r, -r), c + Vec2::new(-r + 4.0, -r)], st);
            painter.line_segment([c + Vec2::new(-r, -r), c + Vec2::new(-r, -r + 4.0)], st);
            painter.line_segment([c + Vec2::new(r, r), c + Vec2::new(r - 4.0, r)], st);
            painter.line_segment([c + Vec2::new(r, r), c + Vec2::new(r, r - 4.0)], st);
        }
        PaletteTemplate::Swipe => {
            let p0 = c + Vec2::new(-9.0, 5.0);
            let p1 = c + Vec2::new(9.0, -5.0);
            painter.circle_filled(p0, 2.5, color);
            painter.line_segment([p0, p1], st);
            painter.line_segment([p1, p1 + Vec2::new(-5.0, 0.5)], st);
            painter.line_segment([p1, p1 + Vec2::new(-1.5, 4.5)], st);
        }
        PaletteTemplate::MobaSkill => {
            painter.circle_stroke(c, 10.5, st);
            painter.circle_stroke(c, 5.0, st);
            painter.line_segment([c, c + Vec2::new(8.0, -7.0)], st);
        }
        PaletteTemplate::FreeLook => {
            let pts = [
                c + Vec2::new(-11.0, 0.0),
                c + Vec2::new(-5.0, -5.5),
                c + Vec2::new(5.0, -5.5),
                c + Vec2::new(11.0, 0.0),
                c + Vec2::new(5.0, 5.5),
                c + Vec2::new(-5.0, 5.5),
            ];
            painter.add(egui::Shape::closed_line(pts.to_vec(), st));
            painter.circle_filled(c, 3.0, color);
        }
        PaletteTemplate::ZoomPinch => {
            let a0 = c + Vec2::new(-8.0, 8.0);
            let a1 = c + Vec2::new(-2.0, 2.0);
            let b0 = c + Vec2::new(8.0, -8.0);
            let b1 = c + Vec2::new(2.0, -2.0);
            painter.line_segment([a0, a1], st);
            painter.line_segment([a0, a0 + Vec2::new(4.5, 0.0)], st);
            painter.line_segment([a0, a0 + Vec2::new(0.0, -4.5)], st);
            painter.line_segment([b0, b1], st);
            painter.line_segment([b0, b0 + Vec2::new(-4.5, 0.0)], st);
            painter.line_segment([b0, b0 + Vec2::new(0.0, 4.5)], st);
        }
        PaletteTemplate::RightClickWalk => {
            let m = Rect::from_center_size(c, Vec2::new(14.0, 20.0));
            painter.rect_stroke(m, CornerRadius::same(7), st, StrokeKind::Inside);
            painter.line_segment([Pos2::new(m.left(), m.center().y - 1.0), Pos2::new(m.right(), m.center().y - 1.0)], st);
            painter.line_segment([Pos2::new(m.center().x, m.top()), Pos2::new(m.center().x, m.center().y - 1.0)], st);
            let r_btn = Rect::from_min_max(Pos2::new(m.center().x + 1.0, m.top() + 2.0), Pos2::new(m.right() - 2.0, m.center().y - 2.0));
            painter.rect_filled(r_btn, CornerRadius::same(3), color);
        }
        PaletteTemplate::GamepadButton => {
            let body = Rect::from_center_size(c, Vec2::new(22.0, 13.0));
            painter.rect_stroke(body, CornerRadius::same(5), st, StrokeKind::Inside);
            let d = c + Vec2::new(-5.5, 0.0);
            painter.line_segment([d - Vec2::new(3.0, 0.0), d + Vec2::new(3.0, 0.0)], st);
            painter.line_segment([d - Vec2::new(0.0, 3.0), d + Vec2::new(0.0, 3.0)], st);
            painter.circle_filled(c + Vec2::new(5.5, -2.0), 1.5, color);
            painter.circle_filled(c + Vec2::new(5.5, 2.0), 1.5, color);
        }
        PaletteTemplate::MacroTrigger => {
            let bolt = [
                c + Vec2::new(2.0, -10.0),
                c + Vec2::new(-5.0, 1.0),
                c + Vec2::new(0.0, 1.0),
                c + Vec2::new(-2.0, 10.0),
                c + Vec2::new(5.5, -1.0),
                c + Vec2::new(0.5, -1.0),
            ];
            painter.add(egui::Shape::line(bolt.to_vec(), st));
        }
        PaletteTemplate::SensorControl => {
            let phone = Rect::from_center_size(c, Vec2::new(12.0, 18.0));
            painter.rect_stroke(phone, CornerRadius::same(2), st, StrokeKind::Inside);
            painter.line_segment([c + Vec2::new(-10.0, -4.0), c + Vec2::new(-10.0, 4.0)], st);
            painter.line_segment([c + Vec2::new(10.0, -4.0), c + Vec2::new(10.0, 4.0)], st);
        }
    }
}
