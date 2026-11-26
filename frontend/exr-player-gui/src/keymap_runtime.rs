use crate::backend::EmulatorBackend;
use eframe::egui::{self, Rect};
use exr_aim::{FpsShootingController, FreeLookController, MobaSkillController};
use exr_android_bridge::{CallbackInputBridge, GuestInputEvent, InputBridge};
use exr_gamepad::GamepadEngine;
use exr_gestures::{ActivePinch, ActiveSwipe, DpadState, GestureEngine, RepeatedTapTask};
use exr_input::{
    HostInputEvent, InputStateManager, InputTrigger, KeyCombo, Modifiers, MouseButton,
    StateTransition,
};
use exr_keymap::{
    ControlKind, KeymapProfile, MappingResolver, NormVec2, ProfileStore, RepeatMode, ViewportMapper,
};
use exr_keymap_ui::{egui_key_to_exr, KeymapEditorState};
use exr_macro::MacroPlayer;
use exr_smart_controls::SmartContextEngine;
use std::path::PathBuf;
use std::time::Instant;

pub struct KeymapRuntime {
    pub store: ProfileStore,
    pub editor: KeymapEditorState,
    pub resolver: MappingResolver,
    pub input_state: InputStateManager,
    pub gestures: GestureEngine,
    pub fps_aim: FpsShootingController,
    pub free_look: FreeLookController,
    pub moba: MobaSkillController,
    pub gamepad: GamepadEngine,
    pub macros: MacroPlayer,
    pub smart: SmartContextEngine,
    pub keymap_enabled: bool,
    pub active_package: Option<String>,
    last_pointer_px: Option<egui::Pos2>,
    was_focused: bool,
}

impl KeymapRuntime {
    pub fn new() -> Self {
        let profile_dir = directories::ProjectDirs::from("com", "ExR", "ExRPlayer")
            .map(|d| d.config_dir().join("profiles"))
            .unwrap_or_else(|| PathBuf::from("profiles"));
        let store = ProfileStore::new(profile_dir);
        let profile = store.load_or_recover("fps-default");
        let resolver = MappingResolver::from_profile(&profile);
        let mut rt = Self {
            store,
            editor: KeymapEditorState::new(profile),
            resolver,
            input_state: InputStateManager::new(),
            gestures: GestureEngine::new(),
            fps_aim: FpsShootingController::default(),
            free_look: FreeLookController::default(),
            moba: MobaSkillController::default(),
            gamepad: GamepadEngine::new(),
            macros: MacroPlayer::new(),
            smart: SmartContextEngine::new(),
            keymap_enabled: true,
            active_package: None,
            last_pointer_px: None,
            was_focused: true,
        };
        rt.rebuild_indices();
        rt
    }

    /// Automatically switches to `<package>.json` (creating it under `app_name` if new)
    /// whenever the foreground Android app changes.
    pub fn sync_foreground_app(
        &mut self,
        fg: Option<(String, String)>,
        backend: &mut dyn EmulatorBackend,
    ) {
        let next_pkg = fg.as_ref().map(|(p, _)| p.clone());
        if next_pkg == self.active_package {
            return;
        }
        self.release_all_virtual(backend);
        self.active_package = next_pkg;
        if let Some((pkg, app_name)) = fg {
            let profile = self.store.load_or_create_for_app(&pkg, &app_name);
            self.load_profile(profile);
        }
    }

    pub fn rebuild_indices(&mut self) {
        self.resolver = MappingResolver::from_profile(&self.editor.profile);
        self.gestures.dpads.clear();
        for ctrl in &self.editor.profile.controls {
            if !ctrl.enabled {
                continue;
            }
            match &ctrl.kind {
                ControlKind::Dpad {
                    radius,
                    deadzone,
                    sensitivity,
                    return_to_center,
                    ..
                } => {
                    let mut st = DpadState::new(ctrl.pos, *radius);
                    st.deadzone = *deadzone;
                    st.sensitivity = *sensitivity;
                    st.return_to_center = *return_to_center;
                    self.gestures.dpads.insert(ctrl.id.clone(), st);
                }
                ControlKind::AnalogJoystick {
                    radius,
                    deadzone,
                    curve,
                    invert_x,
                    invert_y,
                    ..
                } => {
                    let mut st = DpadState::new(ctrl.pos, *radius);
                    st.deadzone = *deadzone;
                    st.curve = *curve;
                    st.invert_x = *invert_x;
                    st.invert_y = *invert_y;
                    self.gestures.dpads.insert(ctrl.id.clone(), st);
                }
                ControlKind::FpsAim {
                    sensitivity_x,
                    sensitivity_y,
                    invert_y,
                    smoothing,
                    pitch_min,
                    pitch_max,
                    ..
                } => {
                    self.fps_aim.origin = ctrl.pos;
                    self.fps_aim.sensitivity_x = *sensitivity_x;
                    self.fps_aim.sensitivity_y = *sensitivity_y;
                    self.fps_aim.invert_y = *invert_y;
                    self.fps_aim.smoothing = *smoothing;
                    self.fps_aim.pitch_min = *pitch_min;
                    self.fps_aim.pitch_max = *pitch_max;
                }
                _ => {}
            }
        }
    }

    pub fn release_all_virtual(&mut self, backend: &mut dyn EmulatorBackend) {
        let mut bridge = make_backend_bridge(backend);
        self.gestures.release_all(&mut bridge);
        self.fps_aim.release(&mut self.gestures.slots, &mut bridge);
        self.free_look.stop(&mut self.gestures.slots, &mut bridge);
        self.moba.cancel_all(&mut self.gestures.slots, &mut bridge);
        self.gamepad.release_all(&mut self.gestures.slots, &mut bridge);
        self.macros.stop_all();
        bridge.release_all();
        self.last_pointer_px = None;
    }

    pub fn process_egui_frame(
        &mut self,
        ctx: &egui::Context,
        viewport_rect: Rect,
        backend: &mut dyn EmulatorBackend,
    ) {
        let now = Instant::now();
        let focused = ctx.input(|i| i.focused);
        if self.was_focused && !focused {
            self.input_state.process_event(&HostInputEvent::FocusLost);
            self.release_all_virtual(backend);
            self.was_focused = false;
            return;
        }
        if !self.was_focused && focused {
            self.input_state.process_event(&HostInputEvent::FocusGained);
            self.was_focused = true;
        }

        if !self.keymap_enabled || !focused {
            return;
        }

        let mapper = ViewportMapper::new(
            viewport_rect.left(),
            viewport_rect.top(),
            viewport_rect.width(),
            viewport_rect.height(),
        );

        let mut host_events = Vec::new();
        ctx.input(|i| {
            let mods = Modifiers {
                ctrl: i.modifiers.ctrl,
                shift: i.modifiers.shift,
                alt: i.modifiers.alt,
                super_key: i.modifiers.mac_cmd || i.modifiers.command,
            };
            for ev in &i.events {
                match ev {
                    egui::Event::Key { key, pressed, .. } => {
                        if let Some(kc) = egui_key_to_exr(*key) {
                            if *pressed {
                                host_events.push(HostInputEvent::KeyDown { key: kc, modifiers: mods });
                            } else {
                                host_events.push(HostInputEvent::KeyUp { key: kc });
                            }
                        }
                    }
                    egui::Event::PointerMoved(pos) => {
                        let norm = mapper.host_to_norm(pos.x, pos.y);
                        host_events.push(HostInputEvent::MouseMove { x: norm.x, y: norm.y });
                        if let Some(prev) = self.last_pointer_px {
                            let dx = pos.x - prev.x;
                            let dy = pos.y - prev.y;
                            if dx.abs() > 0.01 || dy.abs() > 0.01 {
                                host_events.push(HostInputEvent::MouseDelta { dx, dy });
                            }
                        }
                        self.last_pointer_px = Some(*pos);
                    }
                    egui::Event::PointerButton { pos, button, pressed, .. } => {
                        let norm = mapper.host_to_norm(pos.x, pos.y);
                        let mb = match button {
                            egui::PointerButton::Primary => MouseButton::Left,
                            egui::PointerButton::Secondary => MouseButton::Right,
                            egui::PointerButton::Middle => MouseButton::Middle,
                            egui::PointerButton::Extra1 => MouseButton::Back,
                            egui::PointerButton::Extra2 => MouseButton::Forward,
                        };
                        if *pressed {
                            host_events.push(HostInputEvent::MouseButtonDown {
                                button: mb,
                                x: norm.x,
                                y: norm.y,
                            });
                        } else {
                            host_events.push(HostInputEvent::MouseButtonUp {
                                button: mb,
                                x: norm.x,
                                y: norm.y,
                            });
                        }
                    }
                    egui::Event::MouseWheel { delta, .. } => {
                        if delta.y.abs() > 0.01 {
                            host_events.push(HostInputEvent::MouseWheel { delta_y: delta.y });
                        }
                    }
                    _ => {}
                }
            }
        });

        let mut bridge = make_backend_bridge(backend);
        for ev in host_events {
            let tr = self.input_state.process_event(&ev);
            if matches!(tr, StateTransition::Ignored) {
                continue;
            }
            if matches!(tr, StateTransition::ReleaseAll { .. }) {
                self.gestures.release_all(&mut bridge);
                self.fps_aim.release(&mut self.gestures.slots, &mut bridge);
                continue;
            }

            match ev {
                HostInputEvent::KeyDown { key, modifiers } => {
                    // Check profile toggle hotkey
                    let toggle = self.editor.profile.toggle_hotkey;
                    if toggle.key == key && toggle.modifiers.matches_exact(&modifiers) {
                        self.keymap_enabled = !self.keymap_enabled;
                        if !self.keymap_enabled {
                            self.gestures.release_all(&mut bridge);
                            self.fps_aim.release(&mut self.gestures.slots, &mut bridge);
                        }
                        continue;
                    }
                    // Escape always unlocks FPS capture safely
                    if key == exr_input::KeyCode::Escape && self.fps_aim.captured {
                        self.fps_aim.set_captured(false, &mut self.gestures.slots, &mut bridge);
                        continue;
                    }
                    self.dispatch_trigger_down(
                        InputTrigger::Key(KeyCombo::with_mods(key, modifiers)),
                        modifiers,
                        now,
                        &mut bridge,
                    );
                }
                HostInputEvent::KeyUp { key } => {
                    self.dispatch_trigger_up(
                        InputTrigger::key(key),
                        self.input_state.modifiers(),
                        &mut bridge,
                    );
                }
                HostInputEvent::MouseButtonDown { button, .. } => {
                    self.dispatch_trigger_down(
                        InputTrigger::Mouse(button),
                        self.input_state.modifiers(),
                        now,
                        &mut bridge,
                    );
                    if button == MouseButton::Left {
                        self.moba.confirm_manual(&mut self.gestures.slots, &mut bridge);
                    }
                }
                HostInputEvent::MouseButtonUp { button, .. } => {
                    self.dispatch_trigger_up(
                        InputTrigger::Mouse(button),
                        self.input_state.modifiers(),
                        &mut bridge,
                    );
                }
                HostInputEvent::MouseDelta { dx, dy } => {
                    self.fps_aim
                        .handle_mouse_delta(dx, dy, &mut self.gestures.slots, &mut bridge);
                    self.free_look.handle_delta(dx, dy, &mut bridge);
                }
                HostInputEvent::MouseMove { x, y } => {
                    let cur = NormVec2::new(x, y);
                    self.moba.update_cursor(cur, &mut bridge);
                    if let Some((_, center)) = self.gestures.right_walk_slot {
                        self.gestures.update_right_click_walk(
                            true,
                            center,
                            cur,
                            true,
                            &mut bridge,
                        );
                    }
                }
                HostInputEvent::MouseWheel { delta_y } => {
                    let trig = if delta_y > 0.0 {
                        InputTrigger::MouseWheelUp
                    } else {
                        InputTrigger::MouseWheelDown
                    };
                    self.dispatch_trigger_down(trig, self.input_state.modifiers(), now, &mut bridge);
                }
                _ => {}
            }
        }

        self.gestures.tick(now, &mut bridge);
        self.macros.tick(now, &mut bridge);

        if self.fps_aim.captured {
            ctx.set_cursor_icon(egui::CursorIcon::None);
        }
    }

    fn dispatch_trigger_down(
        &mut self,
        trigger: InputTrigger,
        mods: Modifiers,
        now: Instant,
        bridge: &mut dyn InputBridge,
    ) {
        let profile = &self.editor.profile;
        let matches = self.resolver.resolve(profile, trigger, mods);
        let cursor = NormVec2::new(
            self.input_state.cursor_norm().0,
            self.input_state.cursor_norm().1,
        );

        for ctrl in matches {
            if !self.smart.is_control_allowed(&ctrl.id) {
                continue;
            }
            match &ctrl.kind {
                ControlKind::TapSpot { .. } => {
                    self.gestures.tap_down(&ctrl.id, ctrl.pos, bridge);
                }
                ControlKind::RepeatedTap {
                    interval_ms,
                    initial_delay_ms,
                    mode,
                    ..
                } => {
                    if *mode == RepeatMode::Toggle && self.gestures.repeated_taps.contains_key(&ctrl.id) {
                        self.gestures.repeated_taps.remove(&ctrl.id);
                    } else {
                        let task = RepeatedTapTask::start(
                            ctrl.id.clone(),
                            ctrl.pos,
                            *interval_ms,
                            *initial_delay_ms,
                            *mode,
                            now,
                        );
                        self.gestures.repeated_taps.insert(ctrl.id.clone(), task);
                    }
                }
                ControlKind::Dpad { up, down, left, right, .. }
                | ControlKind::AnalogJoystick { up, down, left, right, .. } => {
                    if let InputTrigger::Key(combo) = trigger {
                        if let Some(dpad) = self.gestures.dpads.get_mut(&ctrl.id) {
                            if combo.key == *up { dpad.up = true; }
                            if combo.key == *down { dpad.down = true; }
                            if combo.key == *left { dpad.left = true; }
                            if combo.key == *right { dpad.right = true; }
                            dpad.sync(&mut self.gestures.slots, bridge);
                        }
                    }
                }
                ControlKind::FpsAim { toggle_key, .. } => {
                    if trigger == *toggle_key {
                        self.fps_aim.toggle(&mut self.gestures.slots, bridge);
                    }
                }
                ControlKind::Swipe { end_pos, duration_ms, .. } => {
                    if let Some(sw) = ActiveSwipe::start(
                        ctrl.pos,
                        *end_pos,
                        *duration_ms,
                        now,
                        &mut self.gestures.slots,
                        bridge,
                    ) {
                        self.gestures.swipes.push(sw);
                    }
                }
                ControlKind::MobaSkill {
                    trigger: skill_trig,
                    cancel_trigger,
                    cast_mode,
                    max_radius,
                    confirm_on_release,
                } => {
                    if cancel_trigger.as_ref() == Some(&trigger) {
                        self.moba.cancel_all(&mut self.gestures.slots, bridge);
                    } else if trigger == *skill_trig {
                        self.moba.press(
                            &ctrl.id,
                            ctrl.pos,
                            cursor,
                            *max_radius,
                            *cast_mode,
                            *confirm_on_release,
                            &mut self.gestures.slots,
                            bridge,
                        );
                    }
                }
                ControlKind::FreeLook {
                    sensitivity_x,
                    sensitivity_y,
                    radius,
                    ..
                } => {
                    self.free_look.start(
                        &ctrl.id,
                        ctrl.pos,
                        *sensitivity_x,
                        *sensitivity_y,
                        *radius,
                        &mut self.gestures.slots,
                        bridge,
                    );
                }
                ControlKind::ZoomPinch {
                    zoom_in_trigger,
                    spread_radius,
                    duration_ms,
                    speed,
                    ..
                } => {
                    let zoom_in = trigger == *zoom_in_trigger;
                    if let Some(p) = ActivePinch::start(
                        ctrl.pos,
                        zoom_in,
                        *spread_radius,
                        *duration_ms,
                        *speed,
                        now,
                        &mut self.gestures.slots,
                        bridge,
                    ) {
                        self.gestures.pinches.push(p);
                    }
                }
                ControlKind::RightClickWalk { reduce_inertia, .. } => {
                    self.gestures.update_right_click_walk(
                        true,
                        ctrl.pos,
                        cursor,
                        *reduce_inertia,
                        bridge,
                    );
                }
                ControlKind::MacroTrigger {
                    macro_id,
                    loop_count,
                    speed,
                    ..
                } => {
                    self.macros.start(macro_id, *loop_count, *speed, now);
                }
                _ => {}
            }
        }
    }

    fn dispatch_trigger_up(
        &mut self,
        trigger: InputTrigger,
        mods: Modifiers,
        bridge: &mut dyn InputBridge,
    ) {
        let profile = &self.editor.profile;
        let matches = self.resolver.resolve(profile, trigger, mods);
        for ctrl in matches {
            match &ctrl.kind {
                ControlKind::TapSpot { .. } => {
                    self.gestures.tap_up(&ctrl.id, bridge);
                }
                ControlKind::RepeatedTap { mode, .. } => {
                    if *mode == RepeatMode::Hold {
                        self.gestures.repeated_taps.remove(&ctrl.id);
                    }
                }
                ControlKind::Dpad { up, down, left, right, .. }
                | ControlKind::AnalogJoystick { up, down, left, right, .. } => {
                    if let InputTrigger::Key(combo) = trigger {
                        if let Some(dpad) = self.gestures.dpads.get_mut(&ctrl.id) {
                            if combo.key == *up { dpad.up = false; }
                            if combo.key == *down { dpad.down = false; }
                            if combo.key == *left { dpad.left = false; }
                            if combo.key == *right { dpad.right = false; }
                            dpad.sync(&mut self.gestures.slots, bridge);
                        }
                    }
                }
                ControlKind::MobaSkill { .. } => {
                    self.moba.release_key(&ctrl.id, &mut self.gestures.slots, bridge);
                }
                ControlKind::FreeLook { .. } => {
                    if self.free_look.is_active_for(&ctrl.id) {
                        self.free_look.stop(&mut self.gestures.slots, bridge);
                    }
                }
                ControlKind::RightClickWalk { reduce_inertia, .. } => {
                    self.gestures.update_right_click_walk(
                        false,
                        ctrl.pos,
                        ctrl.pos,
                        *reduce_inertia,
                        bridge,
                    );
                }
                _ => {}
            }
        }
    }

    pub fn load_profile(&mut self, profile: KeymapProfile) {
        self.editor = KeymapEditorState::new(profile);
        self.rebuild_indices();
    }
}

fn make_backend_bridge<'a>(backend: &'a mut dyn EmulatorBackend) -> CallbackInputBridge<impl FnMut(GuestInputEvent) + Send + 'a> {
    CallbackInputBridge::new(move |ev| match ev {
        GuestInputEvent::TouchDown { slot, x, y, .. } => backend.touch_down(slot, x, y),
        GuestInputEvent::TouchMove { slot, x, y, .. } => backend.touch_move(slot, x, y),
        GuestInputEvent::TouchUp { slot } => backend.touch_up(slot),
        _ => {}
    })
}
