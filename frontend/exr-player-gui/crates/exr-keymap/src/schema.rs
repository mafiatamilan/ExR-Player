use exr_input::{GamepadAxis, GamepadButton, InputTrigger, KeyCode, KeyCombo, Modifiers, MouseButton};
use serde::{Deserialize, Serialize};

pub const PROFILE_SCHEMA_VERSION: u32 = 1;

/// Normalized coordinate in guest display space `[0.0, 1.0]`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
pub struct NormVec2 {
    pub x: f32,
    pub y: f32,
}

impl NormVec2 {
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn clamped(self) -> Self {
        Self {
            x: self.x.clamp(0.0, 1.0),
            y: self.y.clamp(0.0, 1.0),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ProfileMode {
    #[default]
    Standard,
    Smart,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RepeatMode {
    Hold,
    Toggle,
    Burst(u32),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum AxisCurve {
    #[default]
    Linear,
    Quadratic,
    SCurve,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum MobaCastMode {
    #[default]
    Quick,
    Auto,
    Manual,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SensorKind {
    Accelerometer,
    Gyroscope,
    RotationVector,
    Orientation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GamepadControlTarget {
    TapSpot,
    LeftStickDpad,
    RightStickAim,
}

/// All 14 independent keymapping control types supported by ExR Player.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ControlKind {
    TapSpot {
        trigger: InputTrigger,
        #[serde(default)]
        extra_triggers: Vec<InputTrigger>,
        #[serde(default)]
        hold_duration_ms: u32,
    },
    RepeatedTap {
        trigger: InputTrigger,
        interval_ms: u32,
        #[serde(default)]
        initial_delay_ms: u32,
        mode: RepeatMode,
    },
    Dpad {
        up: KeyCode,
        down: KeyCode,
        left: KeyCode,
        right: KeyCode,
        radius: f32,
        deadzone: f32,
        sensitivity: f32,
        return_to_center: bool,
    },
    AnalogJoystick {
        up: KeyCode,
        down: KeyCode,
        left: KeyCode,
        right: KeyCode,
        radius: f32,
        deadzone: f32,
        curve: AxisCurve,
        invert_x: bool,
        invert_y: bool,
    },
    FpsAim {
        toggle_key: InputTrigger,
        fire_key: InputTrigger,
        secondary_fire_key: Option<InputTrigger>,
        sensitivity_x: f32,
        sensitivity_y: f32,
        invert_y: bool,
        smoothing: f32,
        pitch_min: f32,
        pitch_max: f32,
    },
    CrosshairAim {
        trigger: InputTrigger,
        fire_trigger: InputTrigger,
        region_radius: f32,
        sensitivity_x: f32,
        sensitivity_y: f32,
    },
    Swipe {
        trigger: InputTrigger,
        end_pos: NormVec2,
        duration_ms: u32,
    },
    MobaSkill {
        trigger: InputTrigger,
        cancel_trigger: Option<InputTrigger>,
        cast_mode: MobaCastMode,
        max_radius: f32,
        confirm_on_release: bool,
    },
    FreeLook {
        trigger: InputTrigger,
        sensitivity_x: f32,
        sensitivity_y: f32,
        radius: f32,
    },
    ZoomPinch {
        zoom_in_trigger: InputTrigger,
        zoom_out_trigger: InputTrigger,
        spread_radius: f32,
        duration_ms: u32,
        speed: f32,
    },
    RightClickWalk {
        trigger: InputTrigger,
        reduce_inertia: bool,
        hold_to_walk: bool,
    },
    GamepadControl {
        button: Option<GamepadButton>,
        axis_x: Option<GamepadAxis>,
        axis_y: Option<GamepadAxis>,
        target: GamepadControlTarget,
        deadzone: f32,
        curve: AxisCurve,
        radius: f32,
    },
    MacroTrigger {
        trigger: InputTrigger,
        macro_id: String,
        loop_count: u32,
        speed: f32,
    },
    SensorControl {
        sensor: SensorKind,
        positive_trigger: InputTrigger,
        negative_trigger: Option<InputTrigger>,
        axis_index: u8,
        magnitude: f32,
        smoothing: f32,
    },
}

impl ControlKind {
    pub fn short_type_name(&self) -> &'static str {
        match self {
            Self::TapSpot { .. } => "Tap Spot",
            Self::RepeatedTap { .. } => "Repeated Tap",
            Self::Dpad { .. } => "D-Pad / WASD",
            Self::AnalogJoystick { .. } => "Analog Stick",
            Self::FpsAim { .. } => "FPS Shoot",
            Self::CrosshairAim { .. } => "Crosshair",
            Self::Swipe { .. } => "Swipe",
            Self::MobaSkill { .. } => "MOBA Skill",
            Self::FreeLook { .. } => "Free Look",
            Self::ZoomPinch { .. } => "Zoom / Pinch",
            Self::RightClickWalk { .. } => "R-Click Walk",
            Self::GamepadControl { .. } => "Gamepad",
            Self::MacroTrigger { .. } => "Macro",
            Self::SensorControl { .. } => "Sensor",
        }
    }

    pub fn primary_badge_text(&self) -> String {
        match self {
            Self::TapSpot { trigger, .. }
            | Self::RepeatedTap { trigger, .. }
            | Self::Swipe { trigger, .. }
            | Self::MobaSkill { trigger, .. }
            | Self::FreeLook { trigger, .. }
            | Self::RightClickWalk { trigger, .. }
            | Self::MacroTrigger { trigger, .. }
            | Self::CrosshairAim { trigger, .. } => trigger.to_string(),
            Self::Dpad { up, down, left, right, .. }
            | Self::AnalogJoystick { up, down, left, right, .. } => {
                format!("{up}{left}{down}{right}")
            }
            Self::FpsAim { toggle_key, .. } => toggle_key.to_string(),
            Self::ZoomPinch { zoom_in_trigger, zoom_out_trigger, .. } => {
                format!("{zoom_in_trigger}/{zoom_out_trigger}")
            }
            Self::GamepadControl { button, target, .. } => button
                .map(|b| b.to_string())
                .unwrap_or_else(|| format!("{target:?}")),
            Self::SensorControl { positive_trigger, .. } => positive_trigger.to_string(),
        }
    }

    /// Collect all `InputTrigger`s bound by this control for conflict detection and resolution.
    pub fn bound_triggers(&self) -> Vec<InputTrigger> {
        match self {
            Self::TapSpot { trigger, extra_triggers, .. } => {
                let mut v = vec![*trigger];
                v.extend(extra_triggers.iter().copied());
                v
            }
            Self::RepeatedTap { trigger, .. }
            | Self::Swipe { trigger, .. }
            | Self::FreeLook { trigger, .. }
            | Self::RightClickWalk { trigger, .. }
            | Self::MacroTrigger { trigger, .. } => vec![*trigger],
            Self::Dpad { up, down, left, right, .. }
            | Self::AnalogJoystick { up, down, left, right, .. } => vec![
                InputTrigger::key(*up),
                InputTrigger::key(*down),
                InputTrigger::key(*left),
                InputTrigger::key(*right),
            ],
            Self::FpsAim { toggle_key, fire_key, secondary_fire_key, .. } => {
                let mut v = vec![*toggle_key, *fire_key];
                if let Some(sec) = secondary_fire_key {
                    v.push(*sec);
                }
                v
            }
            Self::CrosshairAim { trigger, fire_trigger, .. } => vec![*trigger, *fire_trigger],
            Self::MobaSkill { trigger, cancel_trigger, .. } => {
                let mut v = vec![*trigger];
                if let Some(c) = cancel_trigger {
                    v.push(*c);
                }
                v
            }
            Self::ZoomPinch { zoom_in_trigger, zoom_out_trigger, .. } => {
                vec![*zoom_in_trigger, *zoom_out_trigger]
            }
            Self::GamepadControl { button, .. } => {
                button.map(|b| vec![InputTrigger::GamepadButton(b)]).unwrap_or_default()
            }
            Self::SensorControl { positive_trigger, negative_trigger, .. } => {
                let mut v = vec![*positive_trigger];
                if let Some(n) = negative_trigger {
                    v.push(*n);
                }
                v
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ControlMapping {
    pub id: String,
    pub label: String,
    pub pos: NormVec2,
    #[serde(default = "default_true")]
    pub enabled: bool,
    pub kind: ControlKind,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SmartContextRule {
    pub id: String,
    pub context_name: String,
    pub enabled: bool,
    pub control_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KeymapProfile {
    pub schema_version: u32,
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub game_package: Option<String>,
    #[serde(default)]
    pub mode: ProfileMode,
    pub toggle_hotkey: KeyCombo,
    #[serde(default = "default_opacity")]
    pub overlay_opacity: f32,
    #[serde(default = "default_true")]
    pub show_overlay_in_play: bool,
    pub controls: Vec<ControlMapping>,
    #[serde(default = "default_deadzone")]
    pub gamepad_deadzone_left: f32,
    #[serde(default = "default_deadzone")]
    pub gamepad_deadzone_right: f32,
    #[serde(default)]
    pub smart_contexts: Vec<SmartContextRule>,
}

fn default_opacity() -> f32 {
    0.75
}

fn default_deadzone() -> f32 {
    0.15
}

impl Default for KeymapProfile {
    fn default() -> Self {
        Self::fps_default()
    }
}

impl KeymapProfile {
    pub fn blank(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            schema_version: PROFILE_SCHEMA_VERSION,
            id: id.into(),
            name: name.into(),
            game_package: None,
            mode: ProfileMode::Standard,
            toggle_hotkey: KeyCombo::with_mods(
                KeyCode::F1,
                Modifiers {
                    alt: true,
                    ..Modifiers::NONE
                },
            ),
            overlay_opacity: 0.75,
            show_overlay_in_play: true,
            controls: Vec::new(),
            gamepad_deadzone_left: 0.15,
            gamepad_deadzone_right: 0.15,
            smart_contexts: Vec::new(),
        }
    }

    pub fn fps_default() -> Self {
        let mut p = Self::blank("fps-default", "FPS Default (Smart)");
        p.mode = ProfileMode::Smart;
        p.controls = vec![
            ControlMapping {
                id: "wasd".into(),
                label: "Move".into(),
                pos: NormVec2::new(0.16, 0.75),
                enabled: true,
                kind: ControlKind::Dpad {
                    up: KeyCode::W,
                    down: KeyCode::S,
                    left: KeyCode::A,
                    right: KeyCode::D,
                    radius: 0.09,
                    deadzone: 0.05,
                    sensitivity: 1.0,
                    return_to_center: true,
                },
            },
            ControlMapping {
                id: "fps-aim".into(),
                label: "Aim & Shoot".into(),
                pos: NormVec2::new(0.62, 0.45),
                enabled: true,
                kind: ControlKind::FpsAim {
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
            },
            ControlMapping {
                id: "fire".into(),
                label: "Fire".into(),
                pos: NormVec2::new(0.84, 0.70),
                enabled: true,
                kind: ControlKind::TapSpot {
                    trigger: InputTrigger::mouse(MouseButton::Left),
                    extra_triggers: vec![],
                    hold_duration_ms: 0,
                },
            },
            ControlMapping {
                id: "jump".into(),
                label: "Jump".into(),
                pos: NormVec2::new(0.90, 0.56),
                enabled: true,
                kind: ControlKind::TapSpot {
                    trigger: InputTrigger::key(KeyCode::Space),
                    extra_triggers: vec![],
                    hold_duration_ms: 0,
                },
            },
            ControlMapping {
                id: "reload".into(),
                label: "Reload".into(),
                pos: NormVec2::new(0.76, 0.85),
                enabled: true,
                kind: ControlKind::TapSpot {
                    trigger: InputTrigger::key(KeyCode::R),
                    extra_triggers: vec![],
                    hold_duration_ms: 0,
                },
            },
            ControlMapping {
                id: "crouch".into(),
                label: "Crouch".into(),
                pos: NormVec2::new(0.83, 0.85),
                enabled: true,
                kind: ControlKind::TapSpot {
                    trigger: InputTrigger::key(KeyCode::C),
                    extra_triggers: vec![],
                    hold_duration_ms: 0,
                },
            },
        ];
        p
    }

    pub fn moba_default() -> Self {
        let mut p = Self::blank("moba-default", "MOBA Default");
        p.controls = vec![
            ControlMapping {
                id: "moba-move".into(),
                label: "Move".into(),
                pos: NormVec2::new(0.15, 0.76),
                enabled: true,
                kind: ControlKind::Dpad {
                    up: KeyCode::W,
                    down: KeyCode::S,
                    left: KeyCode::A,
                    right: KeyCode::D,
                    radius: 0.09,
                    deadzone: 0.05,
                    sensitivity: 1.0,
                    return_to_center: true,
                },
            },
            ControlMapping {
                id: "skill-q".into(),
                label: "Skill 1".into(),
                pos: NormVec2::new(0.74, 0.78),
                enabled: true,
                kind: ControlKind::MobaSkill {
                    trigger: InputTrigger::key(KeyCode::Q),
                    cancel_trigger: Some(InputTrigger::key(KeyCode::Escape)),
                    cast_mode: MobaCastMode::Quick,
                    max_radius: 0.12,
                    confirm_on_release: true,
                },
            },
            ControlMapping {
                id: "skill-e".into(),
                label: "Skill 2".into(),
                pos: NormVec2::new(0.80, 0.64),
                enabled: true,
                kind: ControlKind::MobaSkill {
                    trigger: InputTrigger::key(KeyCode::E),
                    cancel_trigger: Some(InputTrigger::key(KeyCode::Escape)),
                    cast_mode: MobaCastMode::Quick,
                    max_radius: 0.12,
                    confirm_on_release: true,
                },
            },
            ControlMapping {
                id: "skill-r".into(),
                label: "Ultimate".into(),
                pos: NormVec2::new(0.88, 0.54),
                enabled: true,
                kind: ControlKind::MobaSkill {
                    trigger: InputTrigger::key(KeyCode::R),
                    cancel_trigger: Some(InputTrigger::key(KeyCode::Escape)),
                    cast_mode: MobaCastMode::Manual,
                    max_radius: 0.14,
                    confirm_on_release: false,
                },
            },
        ];
        p
    }
}
