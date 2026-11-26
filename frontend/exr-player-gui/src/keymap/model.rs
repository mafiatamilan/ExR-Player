//! Keymap profile schema (serde, versioned). Spec: docs/05 §5. Coordinates normalized 0..1.
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Profile { pub version: u32, pub package: String, pub name: String, pub frame_aspect: [u32; 2], pub controls: Vec<Control> }

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct JoyKeys { pub up: String, pub left: String, pub down: String, pub right: String }
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Sprint { pub enabled: bool, pub key: String, pub multiplier: f32 }

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Control {
    Click { id: String, center: [f32; 2], key: String, hold_ms: u32, #[serde(default)] note: String },
    Slide { id: String, path: Vec<[f32; 2]>, key: String, duration_ms: u32 },
    Joystick { id: String, center: [f32; 2], radius: f32, keys: JoyKeys, sprint: Sprint },
    MobaMove { id: String, center: [f32; 2], radius: f32, button: String },
    Crosshair { id: String, center: [f32; 2], toggle_key: String, sens: [f32; 2], invert_y: bool },
    Fire { id: String, center: [f32; 2], key: String, mode: HoldMode },
    Skill { id: String, center: [f32; 2], key: String, range: f32, quick_cast: bool },
    FreeLook { id: String, center: [f32; 2], key: String, sens: f32 },
    Repeat { id: String, center: [f32; 2], key: String, rate_hz: u32, mode: HoldMode },
    Cooperate { id: String, down: [f32; 2], up: [f32; 2], key: String },
    Recorder { id: String, center: [f32; 2], key: String, script: Option<String> },
    Macro { id: String, key: String, script: String },
    Zoom { id: String, plus: String, minus: String },
    ExpandSight { id: String, key: String, max_offset: f32 },
    Gravity { id: String, keys: JoyKeys, degrees: f32 },
    View3d { id: String, key: String, step: f32 },
    FakeGps { id: String, keys: JoyKeys, step_m: f32 },
}
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)] #[serde(rename_all = "snake_case")] pub enum HoldMode { Hold, Toggle }

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn roundtrip() {
        let p = Profile { version: 1, package: "com.x".into(), name: "t".into(), frame_aspect: [16, 9], controls: vec![
            Control::Click { id: "c1".into(), center: [0.5, 0.5], key: "Space".into(), hold_ms: 30, note: String::new() },
            Control::Repeat { id: "c2".into(), center: [0.8, 0.3], key: "KeyF".into(), rate_hz: 12, mode: HoldMode::Hold } ] };
        let s = serde_json::to_string_pretty(&p).unwrap();
        assert!(s.contains("\"kind\": \"click\""));
        assert_eq!(serde_json::from_str::<Profile>(&s).unwrap(), p);
    }
}
