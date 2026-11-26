//! Persisted settings. Spec: docs/04 §3. Field set = documented MEmu fields (+ our extras).
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub struct Settings {
    pub language: String,
    pub width: u32, pub height: u32, pub dpi: u32,
    pub cpu_cores: u32, pub memory_mb: u32,
    pub nav_position: NavPosition, pub app_tabs: bool, pub start_with_windows: bool,
    pub preset: Preset, pub fps_limit: u32,
    pub gpu_mem_opt: bool, pub anti_flicker: bool, pub screen_capture: bool, pub microphone: bool, pub independent_disk: bool,
    pub phone_brand: String, pub phone_model: String, pub phone_number: String, pub imei: String,
    pub sensors: bool, pub root_mode: bool,
    pub shortcuts: Vec<(String, String)>, // (action, "Ctrl+Shift+S")
}
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Debug)] pub enum NavPosition { Bottom, Toolbar, Hidden }
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Debug)] pub enum Preset { Low, Medium, High, Top }

impl Default for Settings {
    fn default() -> Self {
        Self { language: "English".into(), width: 1280, height: 720, dpi: 240, cpu_cores: 2, memory_mb: 2048,
            nav_position: NavPosition::Bottom, app_tabs: true, start_with_windows: false, preset: Preset::Medium, fps_limit: 60,
            gpu_mem_opt: false, anti_flicker: true, screen_capture: true, microphone: false, independent_disk: false,
            phone_brand: "Samsung".into(), phone_model: "SM-G975F".into(), phone_number: String::new(), imei: String::new(),
            sensors: true, root_mode: false,
            shortcuts: vec![("Screenshot".into(), "Ctrl+Shift+S".into()), ("Record".into(), "Ctrl+Shift+R".into()), ("Full screen".into(), "F11".into()), ("Open keymapper".into(), "Ctrl+Shift+K".into())] }
    }
}
