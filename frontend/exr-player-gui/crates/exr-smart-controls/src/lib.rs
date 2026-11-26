use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GameContext {
    OnFoot,
    Driving,
    Swimming,
    Parachuting,
    InventoryOpen,
    MapOpen,
    Combat,
    Menus,
    Custom(String),
}

impl GameContext {
    pub fn display_name(&self) -> &str {
        match self {
            Self::OnFoot => "On Foot",
            Self::Driving => "Driving",
            Self::Swimming => "Swimming",
            Self::Parachuting => "Parachuting",
            Self::InventoryOpen => "Inventory Open",
            Self::MapOpen => "Map Open",
            Self::Combat => "Combat",
            Self::Menus => "Menus",
            Self::Custom(s) => s.as_str(),
        }
    }

    pub fn should_release_mouse_capture(&self) -> bool {
        matches!(self, Self::InventoryOpen | Self::MapOpen | Self::Menus)
    }
}

/// Optional region color-signature hint evaluated off the hot input path.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RegionSignature {
    pub context: GameContext,
    pub norm_x: f32,
    pub norm_y: f32,
    pub target_rgb: [u8; 3],
    pub tolerance: u8,
}

/// Modular context-aware state machine with manual override and optional frame signature recognition.
#[derive(Debug, Clone)]
pub struct SmartContextEngine {
    pub enabled: bool,
    pub current: GameContext,
    pub manual_override: Option<GameContext>,
    pub signatures: Vec<RegionSignature>,
    pub context_active_controls: HashMap<GameContext, Vec<String>>,
}

impl Default for SmartContextEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl SmartContextEngine {
    pub fn new() -> Self {
        Self {
            enabled: false,
            current: GameContext::OnFoot,
            manual_override: None,
            signatures: Vec::new(),
            context_active_controls: HashMap::new(),
        }
    }

    pub fn active_context(&self) -> &GameContext {
        self.manual_override.as_ref().unwrap_or(&self.current)
    }

    pub fn set_manual_override(&mut self, ctx: Option<GameContext>) {
        self.manual_override = ctx;
    }

    pub fn is_control_allowed(&self, control_id: &str) -> bool {
        if !self.enabled {
            return true;
        }
        let ctx = self.active_context();
        match self.context_active_controls.get(ctx) {
            Some(list) => list.iter().any(|id| id == control_id),
            None => true,
        }
    }

    /// Evaluates RGBA framebuffer off the hot input path and updates `current` context if matched.
    pub fn inspect_frame_rgba(&mut self, width: u32, height: u32, rgba: &[u8]) {
        if !self.enabled || self.manual_override.is_some() || width == 0 || height == 0 {
            return;
        }
        for sig in &self.signatures {
            let px = ((sig.norm_x.clamp(0.0, 1.0) * (width - 1) as f32).round()) as usize;
            let py = ((sig.norm_y.clamp(0.0, 1.0) * (height - 1) as f32).round()) as usize;
            let idx = (py * width as usize + px) * 4;
            if idx + 2 < rgba.len() {
                let dr = rgba[idx].abs_diff(sig.target_rgb[0]);
                let dg = rgba[idx + 1].abs_diff(sig.target_rgb[1]);
                let db = rgba[idx + 2].abs_diff(sig.target_rgb[2]);
                if dr <= sig.tolerance && dg <= sig.tolerance && db <= sig.tolerance {
                    self.current = sig.context.clone();
                    return;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn context_releases_mouse_on_inventory_and_respects_manual_override() {
        let mut eng = SmartContextEngine::new();
        eng.enabled = true;
        eng.signatures.push(RegionSignature {
            context: GameContext::InventoryOpen,
            norm_x: 0.5,
            norm_y: 0.5,
            target_rgb: [255, 200, 0],
            tolerance: 10,
        });

        let mut rgba = vec![0u8; 4 * 4 * 4];
        // Pixel (2, 2)
        let idx = (2 * 4 + 2) * 4;
        rgba[idx] = 250;
        rgba[idx + 1] = 200;
        rgba[idx + 2] = 2;
        rgba[idx + 3] = 255;

        eng.inspect_frame_rgba(4, 4, &rgba);
        assert_eq!(eng.active_context(), &GameContext::InventoryOpen);
        assert!(eng.active_context().should_release_mouse_capture());

        eng.set_manual_override(Some(GameContext::Driving));
        assert_eq!(eng.active_context(), &GameContext::Driving);
        assert!(!eng.active_context().should_release_mouse_capture());
    }
}
