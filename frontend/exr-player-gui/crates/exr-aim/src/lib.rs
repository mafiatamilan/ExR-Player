use exr_android_bridge::{GuestInputEvent, InputBridge};
use exr_gestures::TouchSlotAllocator;
use exr_keymap::{MobaCastMode, NormVec2};
use std::collections::HashMap;

/// Manages FPS Shooting Mode (relative mouse capture, smoothing, edge recentering, pitch clamp).
#[derive(Debug, Clone)]
pub struct FpsShootingController {
    pub captured: bool,
    pub origin: NormVec2,
    pub sensitivity_x: f32,
    pub sensitivity_y: f32,
    pub invert_y: bool,
    pub smoothing: f32,
    pub pitch_min: f32,
    pub pitch_max: f32,
    slot: Option<u8>,
    virtual_pos: NormVec2,
    ema_dx: f32,
    ema_dy: f32,
    accumulated_pitch_deg: f32,
}

impl Default for FpsShootingController {
    fn default() -> Self {
        Self::new(NormVec2::new(0.62, 0.45))
    }
}

impl FpsShootingController {
    pub fn new(origin: NormVec2) -> Self {
        Self {
            captured: false,
            origin,
            sensitivity_x: 1.0,
            sensitivity_y: 0.8,
            invert_y: false,
            smoothing: 0.15,
            pitch_min: -80.0,
            pitch_max: 80.0,
            slot: None,
            virtual_pos: origin,
            ema_dx: 0.0,
            ema_dy: 0.0,
            accumulated_pitch_deg: 0.0,
        }
    }

    pub fn set_captured(
        &mut self,
        capture: bool,
        slots: &mut TouchSlotAllocator,
        bridge: &mut dyn InputBridge,
    ) {
        if self.captured == capture {
            return;
        }
        self.captured = capture;
        if capture {
            self.virtual_pos = self.origin;
            self.ema_dx = 0.0;
            self.ema_dy = 0.0;
            if self.slot.is_none() {
                if let Some(s) = slots.allocate() {
                    self.slot = Some(s);
                    bridge.send(GuestInputEvent::TouchDown {
                        slot: s,
                        x: self.origin.x,
                        y: self.origin.y,
                        pressure: 1.0,
                    });
                }
            }
        } else {
            self.release(slots, bridge);
        }
    }

    pub fn toggle(&mut self, slots: &mut TouchSlotAllocator, bridge: &mut dyn InputBridge) -> bool {
        let next = !self.captured;
        self.set_captured(next, slots, bridge);
        self.captured
    }

    /// Processes raw relative mouse movement in logical pixels.
    pub fn handle_mouse_delta(
        &mut self,
        dx: f32,
        dy: f32,
        slots: &mut TouchSlotAllocator,
        bridge: &mut dyn InputBridge,
    ) {
        if !self.captured {
            return;
        }
        let alpha = (1.0 - self.smoothing.clamp(0.0, 0.95)).clamp(0.05, 1.0);
        self.ema_dx = self.ema_dx * (1.0 - alpha) + dx * alpha;
        let raw_dy = if self.invert_y { -dy } else { dy };
        self.ema_dy = self.ema_dy * (1.0 - alpha) + raw_dy * alpha;

        // Convert pixel delta to normalized guest pan delta
        const BASE_SCALE: f32 = 0.0012;
        let step_x = self.ema_dx * BASE_SCALE * self.sensitivity_x;
        let mut step_y = self.ema_dy * BASE_SCALE * self.sensitivity_y;

        // Pitch clamp check (~180 deg per full screen height)
        let pitch_delta = step_y * 180.0;
        let next_pitch = (self.accumulated_pitch_deg + pitch_delta).clamp(self.pitch_min, self.pitch_max);
        step_y = (next_pitch - self.accumulated_pitch_deg) / 180.0;
        self.accumulated_pitch_deg = next_pitch;

        self.virtual_pos.x += step_x;
        self.virtual_pos.y += step_y;

        // Seamless edge recentering so camera rotation never hits screen bounds
        if self.virtual_pos.x < 0.15
            || self.virtual_pos.x > 0.85
            || self.virtual_pos.y < 0.15
            || self.virtual_pos.y > 0.85
        {
            if let Some(s) = self.slot.take() {
                bridge.send(GuestInputEvent::TouchUp { slot: s });
                slots.release(s);
            }
            self.virtual_pos = self.origin;
            if let Some(s) = slots.allocate() {
                self.slot = Some(s);
                bridge.send(GuestInputEvent::TouchDown {
                    slot: s,
                    x: self.origin.x,
                    y: self.origin.y,
                    pressure: 1.0,
                });
            }
        }

        if let Some(s) = self.slot {
            bridge.send(GuestInputEvent::TouchMove {
                slot: s,
                x: self.virtual_pos.x.clamp(0.0, 1.0),
                y: self.virtual_pos.y.clamp(0.0, 1.0),
                pressure: 1.0,
            });
        }
    }

    pub fn release(&mut self, slots: &mut TouchSlotAllocator, bridge: &mut dyn InputBridge) {
        self.captured = false;
        if let Some(s) = self.slot.take() {
            bridge.send(GuestInputEvent::TouchUp { slot: s });
            slots.release(s);
        }
    }
}

/// Independent Free Look camera drag controller.
#[derive(Debug, Default)]
pub struct FreeLookController {
    active_id: Option<String>,
    slot: Option<u8>,
    origin: NormVec2,
    pos: NormVec2,
    sensitivity_x: f32,
    sensitivity_y: f32,
    radius: f32,
}

impl FreeLookController {
    pub fn start(
        &mut self,
        id: &str,
        origin: NormVec2,
        sensitivity_x: f32,
        sensitivity_y: f32,
        radius: f32,
        slots: &mut TouchSlotAllocator,
        bridge: &mut dyn InputBridge,
    ) {
        if self.active_id.is_some() {
            return;
        }
        if let Some(s) = slots.allocate() {
            self.active_id = Some(id.to_string());
            self.slot = Some(s);
            self.origin = origin;
            self.pos = origin;
            self.sensitivity_x = sensitivity_x;
            self.sensitivity_y = sensitivity_y;
            self.radius = radius.clamp(0.05, 0.45);
            bridge.send(GuestInputEvent::TouchDown {
                slot: s,
                x: origin.x,
                y: origin.y,
                pressure: 1.0,
            });
        }
    }

    pub fn handle_delta(&mut self, dx: f32, dy: f32, bridge: &mut dyn InputBridge) {
        let Some(s) = self.slot else {
            return;
        };
        const SCALE: f32 = 0.0012;
        self.pos.x = (self.pos.x + dx * SCALE * self.sensitivity_x)
            .clamp(self.origin.x - self.radius, self.origin.x + self.radius)
            .clamp(0.0, 1.0);
        self.pos.y = (self.pos.y + dy * SCALE * self.sensitivity_y)
            .clamp(self.origin.y - self.radius, self.origin.y + self.radius)
            .clamp(0.0, 1.0);
        bridge.send(GuestInputEvent::TouchMove {
            slot: s,
            x: self.pos.x,
            y: self.pos.y,
            pressure: 1.0,
        });
    }

    pub fn stop(&mut self, slots: &mut TouchSlotAllocator, bridge: &mut dyn InputBridge) {
        self.active_id = None;
        if let Some(s) = self.slot.take() {
            bridge.send(GuestInputEvent::TouchUp { slot: s });
            slots.release(s);
        }
    }

    pub fn is_active_for(&self, id: &str) -> bool {
        self.active_id.as_deref() == Some(id)
    }
}

/// Active MOBA skill casting state.
#[derive(Debug, Clone)]
struct ActiveMobaCast {
    slot: u8,
    origin: NormVec2,
    max_radius: f32,
    cast_mode: MobaCastMode,
    confirm_on_release: bool,
}

/// Handles Quick, Auto, and Manual MOBA directional skill casting + cancellation.
#[derive(Debug, Default)]
pub struct MobaSkillController {
    active: HashMap<String, ActiveMobaCast>,
}

impl MobaSkillController {
    pub fn press(
        &mut self,
        id: &str,
        origin: NormVec2,
        cursor: NormVec2,
        max_radius: f32,
        cast_mode: MobaCastMode,
        confirm_on_release: bool,
        slots: &mut TouchSlotAllocator,
        bridge: &mut dyn InputBridge,
    ) {
        if self.active.contains_key(id) {
            return;
        }
        let Some(slot) = slots.allocate() else {
            return;
        };
        bridge.send(GuestInputEvent::TouchDown {
            slot,
            x: origin.x,
            y: origin.y,
            pressure: 1.0,
        });

        match cast_mode {
            MobaCastMode::Auto => {
                bridge.send(GuestInputEvent::TouchUp { slot });
                slots.release(slot);
            }
            MobaCastMode::Quick | MobaCastMode::Manual => {
                let target = Self::project_target(origin, cursor, max_radius);
                bridge.send(GuestInputEvent::TouchMove {
                    slot,
                    x: target.x,
                    y: target.y,
                    pressure: 1.0,
                });
                self.active.insert(
                    id.to_string(),
                    ActiveMobaCast {
                        slot,
                        origin,
                        max_radius,
                        cast_mode,
                        confirm_on_release,
                    },
                );
            }
        }
    }

    pub fn update_cursor(&self, cursor: NormVec2, bridge: &mut dyn InputBridge) {
        for cast in self.active.values() {
            let target = Self::project_target(cast.origin, cursor, cast.max_radius);
            bridge.send(GuestInputEvent::TouchMove {
                slot: cast.slot,
                x: target.x,
                y: target.y,
                pressure: 1.0,
            });
        }
    }

    pub fn release_key(&mut self, id: &str, slots: &mut TouchSlotAllocator, bridge: &mut dyn InputBridge) {
        let should_fire = self
            .active
            .get(id)
            .map(|c| c.cast_mode == MobaCastMode::Quick || c.confirm_on_release)
            .unwrap_or(false);
        if should_fire {
            if let Some(cast) = self.active.remove(id) {
                bridge.send(GuestInputEvent::TouchUp { slot: cast.slot });
                slots.release(cast.slot);
            }
        }
    }

    pub fn confirm_manual(&mut self, slots: &mut TouchSlotAllocator, bridge: &mut dyn InputBridge) {
        let manual_ids: Vec<String> = self
            .active
            .iter()
            .filter(|(_, c)| c.cast_mode == MobaCastMode::Manual)
            .map(|(k, _)| k.clone())
            .collect();
        for id in manual_ids {
            if let Some(cast) = self.active.remove(&id) {
                bridge.send(GuestInputEvent::TouchUp { slot: cast.slot });
                slots.release(cast.slot);
            }
        }
    }

    pub fn cancel_all(&mut self, slots: &mut TouchSlotAllocator, bridge: &mut dyn InputBridge) {
        for (_, cast) in self.active.drain() {
            // Drag back to neutral/cancel region before releasing
            bridge.send(GuestInputEvent::TouchMove {
                slot: cast.slot,
                x: 0.95,
                y: 0.10,
                pressure: 1.0,
            });
            bridge.send(GuestInputEvent::TouchUp { slot: cast.slot });
            slots.release(cast.slot);
        }
    }

    fn project_target(origin: NormVec2, cursor: NormVec2, max_radius: f32) -> NormVec2 {
        let dx = cursor.x - 0.5;
        let dy = cursor.y - 0.5;
        let len = (dx * dx + dy * dy).sqrt();
        if len < 1e-4 {
            return origin;
        }
        let norm_r = (len * 2.0).min(1.0) * max_radius;
        NormVec2::new(
            (origin.x + (dx / len) * norm_r).clamp(0.0, 1.0),
            (origin.y + (dy / len) * norm_r).clamp(0.0, 1.0),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use exr_android_bridge::MockInputBridge;

    #[test]
    fn fps_shooting_recentering_and_focus_release() {
        let mut slots = TouchSlotAllocator::new();
        let mut bridge = MockInputBridge::new();
        let mut fps = FpsShootingController::new(NormVec2::new(0.5, 0.5));
        fps.set_captured(true, &mut slots, &mut bridge);
        assert!(fps.captured);
        assert_eq!(slots.active_count(), 1);

        // Large delta triggers seamless recentering without dropping capture
        for _ in 0..10 {
            fps.handle_mouse_delta(120.0, 0.0, &mut slots, &mut bridge);
        }
        assert!(fps.captured);
        assert_eq!(slots.active_count(), 1);

        fps.release(&mut slots, &mut bridge);
        assert!(!fps.captured);
        assert_eq!(slots.active_count(), 0);
        assert!(bridge.active_slots.is_empty());
    }

    #[test]
    fn moba_quick_cast_projects_and_fires_on_release() {
        let mut slots = TouchSlotAllocator::new();
        let mut bridge = MockInputBridge::new();
        let mut moba = MobaSkillController::default();

        moba.press(
            "q",
            NormVec2::new(0.8, 0.8),
            NormVec2::new(0.7, 0.3),
            0.12,
            MobaCastMode::Quick,
            true,
            &mut slots,
            &mut bridge,
        );
        assert_eq!(slots.active_count(), 1);
        moba.release_key("q", &mut slots, &mut bridge);
        assert_eq!(slots.active_count(), 0);
        assert!(bridge.active_slots.is_empty());
    }
}
