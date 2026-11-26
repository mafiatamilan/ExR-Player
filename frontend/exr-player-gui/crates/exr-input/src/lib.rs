use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fmt;

/// Physical keyboard key codes independent of any UI toolkit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum KeyCode {
    A, B, C, D, E, F, G, H, I, J, K, L, M,
    N, O, P, Q, R, S, T, U, V, W, X, Y, Z,
    Num0, Num1, Num2, Num3, Num4, Num5, Num6, Num7, Num8, Num9,
    F1, F2, F3, F4, F5, F6, F7, F8, F9, F10, F11, F12,
    Space, Enter, Escape, Tab, Backspace,
    Up, Down, Left, Right,
    Home, End, PageUp, PageDown, Insert, Delete,
    LShift, RShift, LCtrl, RCtrl, LAlt, RAlt, LSuper, RSuper,
    CapsLock, Backtick, Minus, Equals,
    BracketLeft, BracketRight, Backslash, Semicolon, Quote, Comma, Period, Slash,
    Numpad0, Numpad1, Numpad2, Numpad3, Numpad4,
    Numpad5, Numpad6, Numpad7, Numpad8, Numpad9,
}

impl KeyCode {
    pub fn short_label(&self) -> &'static str {
        match self {
            Self::A => "A", Self::B => "B", Self::C => "C", Self::D => "D",
            Self::E => "E", Self::F => "F", Self::G => "G", Self::H => "H",
            Self::I => "I", Self::J => "J", Self::K => "K", Self::L => "L",
            Self::M => "M", Self::N => "N", Self::O => "O", Self::P => "P",
            Self::Q => "Q", Self::R => "R", Self::S => "S", Self::T => "T",
            Self::U => "U", Self::V => "V", Self::W => "W", Self::X => "X",
            Self::Y => "Y", Self::Z => "Z",
            Self::Num0 => "0", Self::Num1 => "1", Self::Num2 => "2", Self::Num3 => "3",
            Self::Num4 => "4", Self::Num5 => "5", Self::Num6 => "6", Self::Num7 => "7",
            Self::Num8 => "8", Self::Num9 => "9",
            Self::F1 => "F1", Self::F2 => "F2", Self::F3 => "F3", Self::F4 => "F4",
            Self::F5 => "F5", Self::F6 => "F6", Self::F7 => "F7", Self::F8 => "F8",
            Self::F9 => "F9", Self::F10 => "F10", Self::F11 => "F11", Self::F12 => "F12",
            Self::Space => "Space", Self::Enter => "Enter", Self::Escape => "Esc",
            Self::Tab => "Tab", Self::Backspace => "Bksp",
            Self::Up => "Up", Self::Down => "Down", Self::Left => "Left", Self::Right => "Right",
            Self::Home => "Home", Self::End => "End", Self::PageUp => "PgUp",
            Self::PageDown => "PgDn", Self::Insert => "Ins", Self::Delete => "Del",
            Self::LShift | Self::RShift => "Shift",
            Self::LCtrl | Self::RCtrl => "Ctrl",
            Self::LAlt | Self::RAlt => "Alt",
            Self::LSuper | Self::RSuper => "Super",
            Self::CapsLock => "Caps",
            Self::Backtick => "~",
            Self::Minus => "-",
            Self::Equals => "=",
            Self::BracketLeft => "[",
            Self::BracketRight => "]",
            Self::Backslash => "\\",
            Self::Semicolon => ";",
            Self::Quote => "'",
            Self::Comma => ",",
            Self::Period => ".",
            Self::Slash => "/",
            Self::Numpad0 => "Num0", Self::Numpad1 => "Num1", Self::Numpad2 => "Num2",
            Self::Numpad3 => "Num3", Self::Numpad4 => "Num4", Self::Numpad5 => "Num5",
            Self::Numpad6 => "Num6", Self::Numpad7 => "Num7", Self::Numpad8 => "Num8",
            Self::Numpad9 => "Num9",
        }
    }

    pub fn is_modifier(&self) -> bool {
        matches!(
            self,
            Self::LShift
                | Self::RShift
                | Self::LCtrl
                | Self::RCtrl
                | Self::LAlt
                | Self::RAlt
                | Self::LSuper
                | Self::RSuper
        )
    }
}

impl fmt::Display for KeyCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.short_label())
    }
}

/// Host mouse buttons.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    Back,
    Forward,
}

impl fmt::Display for MouseButton {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Left => f.write_str("LClick"),
            Self::Right => f.write_str("RClick"),
            Self::Middle => f.write_str("MClick"),
            Self::Back => f.write_str("Mouse4"),
            Self::Forward => f.write_str("Mouse5"),
        }
    }
}

/// Modifier key state.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Modifiers {
    pub ctrl: bool,
    pub shift: bool,
    pub alt: bool,
    pub super_key: bool,
}

impl Modifiers {
    pub const NONE: Self = Self {
        ctrl: false,
        shift: false,
        alt: false,
        super_key: false,
    };

    pub fn is_empty(&self) -> bool {
        !self.ctrl && !self.shift && !self.alt && !self.super_key
    }

    /// True if all modifiers required by `self` are present in `held`.
    /// When `self` is empty, matches regardless of held modifiers unless strict matching is requested.
    pub fn matches_exact(&self, held: &Self) -> bool {
        self == held
    }
}

/// Key + optional modifiers combo.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct KeyCombo {
    pub key: KeyCode,
    #[serde(default)]
    pub modifiers: Modifiers,
}

impl KeyCombo {
    pub const fn new(key: KeyCode) -> Self {
        Self {
            key,
            modifiers: Modifiers::NONE,
        }
    }

    pub const fn with_mods(key: KeyCode, modifiers: Modifiers) -> Self {
        Self { key, modifiers }
    }
}

impl fmt::Display for KeyCombo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.modifiers.ctrl {
            f.write_str("Ctrl+")?;
        }
        if self.modifiers.alt {
            f.write_str("Alt+")?;
        }
        if self.modifiers.shift {
            f.write_str("Shift+")?;
        }
        if self.modifiers.super_key {
            f.write_str("Super+")?;
        }
        write!(f, "{}", self.key)
    }
}

/// Standard gamepad digital buttons.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GamepadButton {
    South, // A / Cross
    East,  // B / Circle
    West,  // X / Square
    North, // Y / Triangle
    LB,
    RB,
    LT,
    RT,
    Select,
    Start,
    LStick,
    RStick,
    DpadUp,
    DpadDown,
    DpadLeft,
    DpadRight,
}

impl fmt::Display for GamepadButton {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::South => "GP:A",
            Self::East => "GP:B",
            Self::West => "GP:X",
            Self::North => "GP:Y",
            Self::LB => "GP:LB",
            Self::RB => "GP:RB",
            Self::LT => "GP:LT",
            Self::RT => "GP:RT",
            Self::Select => "GP:Back",
            Self::Start => "GP:Start",
            Self::LStick => "GP:L3",
            Self::RStick => "GP:R3",
            Self::DpadUp => "GP:Up",
            Self::DpadDown => "GP:Down",
            Self::DpadLeft => "GP:Left",
            Self::DpadRight => "GP:Right",
        };
        f.write_str(s)
    }
}

/// Standard gamepad analog axes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GamepadAxis {
    LeftX,
    LeftY,
    RightX,
    RightY,
    LeftTrigger,
    RightTrigger,
}

/// Any bindable host input trigger.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum InputTrigger {
    Key(KeyCombo),
    Mouse(MouseButton),
    MouseWheelUp,
    MouseWheelDown,
    GamepadButton(GamepadButton),
}

impl InputTrigger {
    pub const fn key(code: KeyCode) -> Self {
        Self::Key(KeyCombo::new(code))
    }

    pub const fn mouse(btn: MouseButton) -> Self {
        Self::Mouse(btn)
    }
}

impl fmt::Display for InputTrigger {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Key(c) => write!(f, "{c}"),
            Self::Mouse(b) => write!(f, "{b}"),
            Self::MouseWheelUp => f.write_str("WheelUp"),
            Self::MouseWheelDown => f.write_str("WheelDn"),
            Self::GamepadButton(b) => write!(f, "{b}"),
        }
    }
}

/// Normalized host input event fed into the keymapping pipeline.
#[derive(Debug, Clone, PartialEq)]
pub enum HostInputEvent {
    KeyDown { key: KeyCode, modifiers: Modifiers },
    KeyUp { key: KeyCode },
    /// Absolute pointer position normalized to `[0.0, 1.0]` inside the guest viewport.
    MouseMove { x: f32, y: f32 },
    /// Raw relative mouse movement in logical pixels (used in FPS Shooting Mode).
    MouseDelta { dx: f32, dy: f32 },
    MouseButtonDown { button: MouseButton, x: f32, y: f32 },
    MouseButtonUp { button: MouseButton, x: f32, y: f32 },
    MouseWheel { delta_y: f32 },
    GamepadConnected { id: u32, name: String },
    GamepadDisconnected { id: u32 },
    GamepadButton { id: u32, button: GamepadButton, pressed: bool },
    GamepadAxis { id: u32, axis: GamepadAxis, value: f32 },
    FocusLost,
    FocusGained,
}

/// Result of processing a `HostInputEvent` through `InputStateManager`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StateTransition {
    /// Event ignored (e.g. OS key auto-repeat when key is already held).
    Ignored,
    /// State changed normally.
    Applied,
    /// Focus was lost; caller must release all active virtual pointers and macros.
    ReleaseAll {
        released_keys: Vec<KeyCode>,
        released_mouse: Vec<MouseButton>,
    },
}

/// Tracks active host keys, mouse buttons, modifiers, gamepad states, and window focus.
/// Prevents stuck keys, duplicate press transitions on key auto-repeat, and invalid release sequences.
#[derive(Debug, Clone)]
pub struct InputStateManager {
    pressed_keys: HashSet<KeyCode>,
    pressed_mouse: HashSet<MouseButton>,
    modifiers: Modifiers,
    focused: bool,
    cursor_norm: (f32, f32),
    connected_gamepads: HashMap<u32, String>,
    gamepad_buttons: HashMap<(u32, GamepadButton), bool>,
    gamepad_axes: HashMap<(u32, GamepadAxis), f32>,
}

impl Default for InputStateManager {
    fn default() -> Self {
        Self::new()
    }
}

impl InputStateManager {
    pub fn new() -> Self {
        Self {
            pressed_keys: HashSet::with_capacity(16),
            pressed_mouse: HashSet::with_capacity(8),
            modifiers: Modifiers::NONE,
            focused: true,
            cursor_norm: (0.5, 0.5),
            connected_gamepads: HashMap::new(),
            gamepad_buttons: HashMap::new(),
            gamepad_axes: HashMap::new(),
        }
    }

    pub fn is_focused(&self) -> bool {
        self.focused
    }

    pub fn modifiers(&self) -> Modifiers {
        self.modifiers
    }

    pub fn cursor_norm(&self) -> (f32, f32) {
        self.cursor_norm
    }

    pub fn is_key_pressed(&self, key: KeyCode) -> bool {
        self.pressed_keys.contains(&key)
    }

    pub fn is_mouse_pressed(&self, btn: MouseButton) -> bool {
        self.pressed_mouse.contains(&btn)
    }

    pub fn pressed_keys(&self) -> &HashSet<KeyCode> {
        &self.pressed_keys
    }

    pub fn connected_gamepads(&self) -> &HashMap<u32, String> {
        &self.connected_gamepads
    }

    pub fn gamepad_axis(&self, id: u32, axis: GamepadAxis) -> f32 {
        self.gamepad_axes.get(&(id, axis)).copied().unwrap_or(0.0)
    }

    pub fn is_gamepad_button_pressed(&self, id: u32, btn: GamepadButton) -> bool {
        self.gamepad_buttons.get(&(id, btn)).copied().unwrap_or(false)
    }

    pub fn process_event(&mut self, event: &HostInputEvent) -> StateTransition {
        match event {
            HostInputEvent::FocusLost => {
                self.focused = false;
                let released_keys: Vec<KeyCode> = self.pressed_keys.drain().collect();
                let released_mouse: Vec<MouseButton> = self.pressed_mouse.drain().collect();
                self.modifiers = Modifiers::NONE;
                self.gamepad_buttons.clear();
                self.gamepad_axes.clear();
                StateTransition::ReleaseAll {
                    released_keys,
                    released_mouse,
                }
            }
            HostInputEvent::FocusGained => {
                self.focused = true;
                StateTransition::Applied
            }
            HostInputEvent::KeyDown { key, modifiers } => {
                self.modifiers = *modifiers;
                self.sync_modifier_from_key(*key, true);
                if !self.pressed_keys.insert(*key) {
                    return StateTransition::Ignored;
                }
                StateTransition::Applied
            }
            HostInputEvent::KeyUp { key } => {
                self.sync_modifier_from_key(*key, false);
                if !self.pressed_keys.remove(key) {
                    return StateTransition::Ignored;
                }
                StateTransition::Applied
            }
            HostInputEvent::MouseMove { x, y } => {
                self.cursor_norm = (x.clamp(0.0, 1.0), y.clamp(0.0, 1.0));
                StateTransition::Applied
            }
            HostInputEvent::MouseDelta { .. } => StateTransition::Applied,
            HostInputEvent::MouseButtonDown { button, x, y } => {
                self.cursor_norm = (x.clamp(0.0, 1.0), y.clamp(0.0, 1.0));
                if !self.pressed_mouse.insert(*button) {
                    return StateTransition::Ignored;
                }
                StateTransition::Applied
            }
            HostInputEvent::MouseButtonUp { button, x, y } => {
                self.cursor_norm = (x.clamp(0.0, 1.0), y.clamp(0.0, 1.0));
                if !self.pressed_mouse.remove(button) {
                    return StateTransition::Ignored;
                }
                StateTransition::Applied
            }
            HostInputEvent::MouseWheel { .. } => StateTransition::Applied,
            HostInputEvent::GamepadConnected { id, name } => {
                self.connected_gamepads.insert(*id, name.clone());
                StateTransition::Applied
            }
            HostInputEvent::GamepadDisconnected { id } => {
                self.connected_gamepads.remove(id);
                self.gamepad_buttons.retain(|(gid, _), _| gid != id);
                self.gamepad_axes.retain(|(gid, _), _| gid != id);
                StateTransition::Applied
            }
            HostInputEvent::GamepadButton { id, button, pressed } => {
                let prev = self.gamepad_buttons.insert((*id, *button), *pressed);
                if prev == Some(*pressed) {
                    return StateTransition::Ignored;
                }
                StateTransition::Applied
            }
            HostInputEvent::GamepadAxis { id, axis, value } => {
                self.gamepad_axes.insert((*id, *axis), value.clamp(-1.0, 1.0));
                StateTransition::Applied
            }
        }
    }

    fn sync_modifier_from_key(&mut self, key: KeyCode, down: bool) {
        match key {
            KeyCode::LShift | KeyCode::RShift => {
                self.modifiers.shift = down
                    || self.pressed_keys.contains(&KeyCode::LShift)
                    || self.pressed_keys.contains(&KeyCode::RShift);
            }
            KeyCode::LCtrl | KeyCode::RCtrl => {
                self.modifiers.ctrl = down
                    || self.pressed_keys.contains(&KeyCode::LCtrl)
                    || self.pressed_keys.contains(&KeyCode::RCtrl);
            }
            KeyCode::LAlt | KeyCode::RAlt => {
                self.modifiers.alt = down
                    || self.pressed_keys.contains(&KeyCode::LAlt)
                    || self.pressed_keys.contains(&KeyCode::RAlt);
            }
            KeyCode::LSuper | KeyCode::RSuper => {
                self.modifiers.super_key = down
                    || self.pressed_keys.contains(&KeyCode::LSuper)
                    || self.pressed_keys.contains(&KeyCode::RSuper);
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deduplicates_key_auto_repeat_and_invalid_release() {
        let mut mgr = InputStateManager::new();
        assert_eq!(
            mgr.process_event(&HostInputEvent::KeyUp { key: KeyCode::W }),
            StateTransition::Ignored
        );
        assert_eq!(
            mgr.process_event(&HostInputEvent::KeyDown {
                key: KeyCode::W,
                modifiers: Modifiers::NONE
            }),
            StateTransition::Applied
        );
        assert_eq!(
            mgr.process_event(&HostInputEvent::KeyDown {
                key: KeyCode::W,
                modifiers: Modifiers::NONE
            }),
            StateTransition::Ignored
        );
        assert!(mgr.is_key_pressed(KeyCode::W));
        assert_eq!(
            mgr.process_event(&HostInputEvent::KeyUp { key: KeyCode::W }),
            StateTransition::Applied
        );
        assert!(!mgr.is_key_pressed(KeyCode::W));
    }

    #[test]
    fn focus_lost_releases_all_held_inputs() {
        let mut mgr = InputStateManager::new();
        mgr.process_event(&HostInputEvent::KeyDown {
            key: KeyCode::W,
            modifiers: Modifiers::NONE,
        });
        mgr.process_event(&HostInputEvent::MouseButtonDown {
            button: MouseButton::Left,
            x: 0.4,
            y: 0.6,
        });

        let tr = mgr.process_event(&HostInputEvent::FocusLost);
        match tr {
            StateTransition::ReleaseAll {
                released_keys,
                released_mouse,
            } => {
                assert_eq!(released_keys, vec![KeyCode::W]);
                assert_eq!(released_mouse, vec![MouseButton::Left]);
            }
            other => panic!("expected ReleaseAll, got {other:?}"),
        }
        assert!(!mgr.is_focused());
        assert!(!mgr.is_key_pressed(KeyCode::W));
        assert!(!mgr.is_mouse_pressed(MouseButton::Left));
    }

    #[test]
    fn key_combo_display_formats_modifiers() {
        let combo = KeyCombo::with_mods(
            KeyCode::Num1,
            Modifiers {
                ctrl: true,
                shift: false,
                alt: true,
                super_key: false,
            },
        );
        assert_eq!(combo.to_string(), "Ctrl+Alt+1");
    }
}
