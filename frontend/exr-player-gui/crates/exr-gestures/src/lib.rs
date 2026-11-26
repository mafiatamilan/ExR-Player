use exr_android_bridge::{GuestInputEvent, InputBridge};
use exr_keymap::{AxisCurve, NormVec2, RepeatMode};
use std::collections::HashMap;
use std::time::{Duration, Instant};

pub const MAX_TOUCH_SLOTS: usize = 16;

/// Allocates and tracks up to 16 simultaneous Android virtual touch slots (`0..=15`).
#[derive(Debug, Clone)]
pub struct TouchSlotAllocator {
    in_use: [bool; MAX_TOUCH_SLOTS],
}

impl Default for TouchSlotAllocator {
    fn default() -> Self {
        Self::new()
    }
}

impl TouchSlotAllocator {
    pub fn new() -> Self {
        Self {
            in_use: [false; MAX_TOUCH_SLOTS],
        }
    }

    pub fn allocate(&mut self) -> Option<u8> {
        for (i, used) in self.in_use.iter_mut().enumerate() {
            if !*used {
                *used = true;
                return Some(i as u8);
            }
        }
        None
    }

    pub fn release(&mut self, slot: u8) {
        if let Some(used) = self.in_use.get_mut(slot as usize) {
            *used = false;
        }
    }

    pub fn release_all(&mut self) {
        self.in_use = [false; MAX_TOUCH_SLOTS];
    }

    pub fn active_count(&self) -> usize {
        self.in_use.iter().filter(|&&u| u).count()
    }
}

/// Applies an `AxisCurve` to a normalized `[-1.0, 1.0]` axis value with deadzone.
pub fn apply_axis_curve(raw: f32, deadzone: f32, curve: AxisCurve) -> f32 {
    let abs = raw.abs();
    if abs <= deadzone || deadzone >= 0.99 {
        return 0.0;
    }
    let norm = ((abs - deadzone) / (1.0 - deadzone)).clamp(0.0, 1.0);
    let curved = match curve {
        AxisCurve::Linear => norm,
        AxisCurve::Quadratic => norm * norm,
        AxisCurve::SCurve => norm * norm * (3.0 - 2.0 * norm),
    };
    curved * raw.signum()
}

/// 8-way / analog WASD D-Pad virtual joystick state.
#[derive(Debug, Clone)]
pub struct DpadState {
    pub up: bool,
    pub down: bool,
    pub left: bool,
    pub right: bool,
    pub slot: Option<u8>,
    pub center: NormVec2,
    pub radius: f32,
    pub deadzone: f32,
    pub sensitivity: f32,
    pub return_to_center: bool,
    pub curve: AxisCurve,
    pub invert_x: bool,
    pub invert_y: bool,
}

impl DpadState {
    pub fn new(center: NormVec2, radius: f32) -> Self {
        Self {
            up: false,
            down: false,
            left: false,
            right: false,
            slot: None,
            center,
            radius: radius.clamp(0.01, 0.45),
            deadzone: 0.05,
            sensitivity: 1.0,
            return_to_center: true,
            curve: AxisCurve::Linear,
            invert_x: false,
            invert_y: false,
        }
    }

    /// Computes normalized direction vector `(dx, dy)` in `[-1.0, 1.0]` with diagonal normalization.
    pub fn direction_vector(&self) -> (f32, f32) {
        let mut dx = (self.right as i8 - self.left as i8) as f32;
        let mut dy = (self.down as i8 - self.up as i8) as f32;
        if self.invert_x {
            dx = -dx;
        }
        if self.invert_y {
            dy = -dy;
        }
        let mag = (dx * dx + dy * dy).sqrt();
        if mag <= self.deadzone || mag < 1e-5 {
            return (0.0, 0.0);
        }
        let unit_x = dx / mag;
        let unit_y = dy / mag;
        let radial = apply_axis_curve(mag.min(1.0), self.deadzone, self.curve);
        (unit_x * radial, unit_y * radial)
    }

    pub fn sync(
        &mut self,
        slots: &mut TouchSlotAllocator,
        bridge: &mut dyn InputBridge,
    ) {
        let (dx, dy) = self.direction_vector();
        let active = dx.abs() > 1e-4 || dy.abs() > 1e-4;

        if active {
            let tx = (self.center.x + dx * self.radius * self.sensitivity).clamp(0.0, 1.0);
            let ty = (self.center.y + dy * self.radius * self.sensitivity).clamp(0.0, 1.0);
            if let Some(slot) = self.slot {
                bridge.send(GuestInputEvent::TouchMove {
                    slot,
                    x: tx,
                    y: ty,
                    pressure: 1.0,
                });
            } else if let Some(slot) = slots.allocate() {
                self.slot = Some(slot);
                bridge.send(GuestInputEvent::TouchDown {
                    slot,
                    x: self.center.x,
                    y: self.center.y,
                    pressure: 1.0,
                });
                bridge.send(GuestInputEvent::TouchMove {
                    slot,
                    x: tx,
                    y: ty,
                    pressure: 1.0,
                });
            }
        } else if let Some(slot) = self.slot.take() {
            if self.return_to_center {
                bridge.send(GuestInputEvent::TouchMove {
                    slot,
                    x: self.center.x,
                    y: self.center.y,
                    pressure: 1.0,
                });
            }
            bridge.send(GuestInputEvent::TouchUp { slot });
            slots.release(slot);
        }
    }

    pub fn release(&mut self, slots: &mut TouchSlotAllocator, bridge: &mut dyn InputBridge) {
        self.up = false;
        self.down = false;
        self.left = false;
        self.right = false;
        if let Some(slot) = self.slot.take() {
            bridge.send(GuestInputEvent::TouchUp { slot });
            slots.release(slot);
        }
    }
}

/// Non-blocking monotonic-clock Repeated Tap task.
#[derive(Debug, Clone)]
pub struct RepeatedTapTask {
    pub control_id: String,
    pub pos: NormVec2,
    pub interval: Duration,
    pub mode: RepeatMode,
    pub active: bool,
    pub next_fire: Instant,
    pub remaining_bursts: Option<u32>,
}

impl RepeatedTapTask {
    pub fn start(
        control_id: String,
        pos: NormVec2,
        interval_ms: u32,
        initial_delay_ms: u32,
        mode: RepeatMode,
        now: Instant,
    ) -> Self {
        let remaining_bursts = match mode {
            RepeatMode::Burst(n) => Some(n),
            _ => None,
        };
        Self {
            control_id,
            pos,
            interval: Duration::from_millis(interval_ms.max(10) as u64),
            mode,
            active: true,
            next_fire: now + Duration::from_millis(initial_delay_ms as u64),
            remaining_bursts,
        }
    }

    pub fn tick(
        &mut self,
        now: Instant,
        slots: &mut TouchSlotAllocator,
        bridge: &mut dyn InputBridge,
    ) {
        if !self.active || now < self.next_fire {
            return;
        }
        if let Some(slot) = slots.allocate() {
            bridge.send(GuestInputEvent::TouchDown {
                slot,
                x: self.pos.x,
                y: self.pos.y,
                pressure: 1.0,
            });
            bridge.send(GuestInputEvent::TouchUp { slot });
            slots.release(slot);
        }
        self.next_fire = now + self.interval;
        if let Some(rem) = &mut self.remaining_bursts {
            *rem = rem.saturating_sub(1);
            if *rem == 0 {
                self.active = false;
            }
        }
    }
}

/// Smooth interpolated swipe gesture over monotonic time.
#[derive(Debug, Clone)]
pub struct ActiveSwipe {
    pub slot: u8,
    pub start: NormVec2,
    pub end: NormVec2,
    pub started_at: Instant,
    pub duration: Duration,
}

impl ActiveSwipe {
    pub fn start(
        start: NormVec2,
        end: NormVec2,
        duration_ms: u32,
        now: Instant,
        slots: &mut TouchSlotAllocator,
        bridge: &mut dyn InputBridge,
    ) -> Option<Self> {
        let slot = slots.allocate()?;
        bridge.send(GuestInputEvent::TouchDown {
            slot,
            x: start.x,
            y: start.y,
            pressure: 1.0,
        });
        Some(Self {
            slot,
            start,
            end,
            started_at: now,
            duration: Duration::from_millis(duration_ms.max(16) as u64),
        })
    }

    /// Returns `true` when the swipe has completed and released its slot.
    pub fn tick(
        &self,
        now: Instant,
        slots: &mut TouchSlotAllocator,
        bridge: &mut dyn InputBridge,
    ) -> bool {
        let t = (now.saturating_duration_since(self.started_at).as_secs_f32()
            / self.duration.as_secs_f32())
        .clamp(0.0, 1.0);
        // Smoothstep easing
        let e = t * t * (3.0 - 2.0 * t);
        let x = self.start.x + (self.end.x - self.start.x) * e;
        let y = self.start.y + (self.end.y - self.start.y) * e;
        bridge.send(GuestInputEvent::TouchMove {
            slot: self.slot,
            x: x.clamp(0.0, 1.0),
            y: y.clamp(0.0, 1.0),
            pressure: 1.0,
        });
        if t >= 1.0 {
            bridge.send(GuestInputEvent::TouchUp { slot: self.slot });
            slots.release(self.slot);
            return true;
        }
        false
    }
}

/// Simultaneous two-finger pinch-in / pinch-out gesture.
#[derive(Debug, Clone)]
pub struct ActivePinch {
    pub slot_a: u8,
    pub slot_b: u8,
    pub center: NormVec2,
    pub start_half_spread: f32,
    pub end_half_spread: f32,
    pub started_at: Instant,
    pub duration: Duration,
}

impl ActivePinch {
    pub fn start(
        center: NormVec2,
        zoom_in: bool,
        spread_radius: f32,
        duration_ms: u32,
        speed: f32,
        now: Instant,
        slots: &mut TouchSlotAllocator,
        bridge: &mut dyn InputBridge,
    ) -> Option<Self> {
        let slot_a = slots.allocate()?;
        let Some(slot_b) = slots.allocate() else {
            slots.release(slot_a);
            return None;
        };
        let min_r = (spread_radius * 0.25).clamp(0.01, 0.2);
        let max_r = spread_radius.clamp(min_r + 0.02, 0.4);
        let (start_half_spread, end_half_spread) = if zoom_in {
            (min_r, max_r)
        } else {
            (max_r, min_r)
        };
        let eff_ms = ((duration_ms as f32) / speed.max(0.1)).round().max(20.0) as u64;

        bridge.send(GuestInputEvent::TouchDown {
            slot: slot_a,
            x: (center.x - start_half_spread).clamp(0.0, 1.0),
            y: center.y,
            pressure: 1.0,
        });
        bridge.send(GuestInputEvent::TouchDown {
            slot: slot_b,
            x: (center.x + start_half_spread).clamp(0.0, 1.0),
            y: center.y,
            pressure: 1.0,
        });

        Some(Self {
            slot_a,
            slot_b,
            center,
            start_half_spread,
            end_half_spread,
            started_at: now,
            duration: Duration::from_millis(eff_ms),
        })
    }

    pub fn tick(
        &self,
        now: Instant,
        slots: &mut TouchSlotAllocator,
        bridge: &mut dyn InputBridge,
    ) -> bool {
        let t = (now.saturating_duration_since(self.started_at).as_secs_f32()
            / self.duration.as_secs_f32())
        .clamp(0.0, 1.0);
        let r = self.start_half_spread + (self.end_half_spread - self.start_half_spread) * t;
        bridge.send(GuestInputEvent::TouchMove {
            slot: self.slot_a,
            x: (self.center.x - r).clamp(0.0, 1.0),
            y: self.center.y,
            pressure: 1.0,
        });
        bridge.send(GuestInputEvent::TouchMove {
            slot: self.slot_b,
            x: (self.center.x + r).clamp(0.0, 1.0),
            y: self.center.y,
            pressure: 1.0,
        });
        if t >= 1.0 {
            bridge.send(GuestInputEvent::TouchUp { slot: self.slot_a });
            bridge.send(GuestInputEvent::TouchUp { slot: self.slot_b });
            slots.release(self.slot_a);
            slots.release(self.slot_b);
            return true;
        }
        false
    }
}

/// Manages active tap spots, D-pads, repeated taps, swipes, pinches, and right-click walking.
#[derive(Debug, Default)]
pub struct GestureEngine {
    pub slots: TouchSlotAllocator,
    pub tap_slots: HashMap<String, u8>,
    pub dpads: HashMap<String, DpadState>,
    pub repeated_taps: HashMap<String, RepeatedTapTask>,
    pub swipes: Vec<ActiveSwipe>,
    pub pinches: Vec<ActivePinch>,
    pub right_walk_slot: Option<(u8, NormVec2)>,
}

impl GestureEngine {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn tap_down(&mut self, id: &str, pos: NormVec2, bridge: &mut dyn InputBridge) {
        if self.tap_slots.contains_key(id) {
            return;
        }
        if let Some(slot) = self.slots.allocate() {
            self.tap_slots.insert(id.to_string(), slot);
            bridge.send(GuestInputEvent::TouchDown {
                slot,
                x: pos.x.clamp(0.0, 1.0),
                y: pos.y.clamp(0.0, 1.0),
                pressure: 1.0,
            });
        }
    }

    pub fn tap_up(&mut self, id: &str, bridge: &mut dyn InputBridge) {
        if let Some(slot) = self.tap_slots.remove(id) {
            bridge.send(GuestInputEvent::TouchUp { slot });
            self.slots.release(slot);
        }
    }

    pub fn update_right_click_walk(
        &mut self,
        active: bool,
        center: NormVec2,
        cursor: NormVec2,
        reduce_inertia: bool,
        bridge: &mut dyn InputBridge,
    ) {
        if active {
            let radius = if reduce_inertia { 0.10 } else { 0.14 };
            let dx = cursor.x - center.x;
            let dy = cursor.y - center.y;
            let len = (dx * dx + dy * dy).sqrt().max(1e-4);
            let scale = (len.min(radius)) / len;
            let tx = (center.x + dx * scale).clamp(0.0, 1.0);
            let ty = (center.y + dy * scale).clamp(0.0, 1.0);
            if let Some((slot, _)) = self.right_walk_slot {
                bridge.send(GuestInputEvent::TouchMove {
                    slot,
                    x: tx,
                    y: ty,
                    pressure: 1.0,
                });
            } else if let Some(slot) = self.slots.allocate() {
                self.right_walk_slot = Some((slot, center));
                bridge.send(GuestInputEvent::TouchDown {
                    slot,
                    x: center.x,
                    y: center.y,
                    pressure: 1.0,
                });
                bridge.send(GuestInputEvent::TouchMove {
                    slot,
                    x: tx,
                    y: ty,
                    pressure: 1.0,
                });
            }
        } else if let Some((slot, _)) = self.right_walk_slot.take() {
            bridge.send(GuestInputEvent::TouchUp { slot });
            self.slots.release(slot);
        }
    }

    pub fn tick(&mut self, now: Instant, bridge: &mut dyn InputBridge) {
        let mut finished_taps = Vec::new();
        for (id, task) in &mut self.repeated_taps {
            task.tick(now, &mut self.slots, bridge);
            if !task.active {
                finished_taps.push(id.clone());
            }
        }
        for id in finished_taps {
            self.repeated_taps.remove(&id);
        }

        let mut keep_swipes = Vec::with_capacity(self.swipes.len());
        for s in self.swipes.drain(..) {
            if !s.tick(now, &mut self.slots, bridge) {
                keep_swipes.push(s);
            }
        }
        self.swipes = keep_swipes;

        let mut keep_pinches = Vec::with_capacity(self.pinches.len());
        for p in self.pinches.drain(..) {
            if !p.tick(now, &mut self.slots, bridge) {
                keep_pinches.push(p);
            }
        }
        self.pinches = keep_pinches;
    }

    pub fn release_all(&mut self, bridge: &mut dyn InputBridge) {
        for (_, slot) in self.tap_slots.drain() {
            bridge.send(GuestInputEvent::TouchUp { slot });
            self.slots.release(slot);
        }
        for (_, dpad) in &mut self.dpads {
            dpad.release(&mut self.slots, bridge);
        }
        self.repeated_taps.clear();
        for s in self.swipes.drain(..) {
            bridge.send(GuestInputEvent::TouchUp { slot: s.slot });
            self.slots.release(s.slot);
        }
        for p in self.pinches.drain(..) {
            bridge.send(GuestInputEvent::TouchUp { slot: p.slot_a });
            bridge.send(GuestInputEvent::TouchUp { slot: p.slot_b });
            self.slots.release(p.slot_a);
            self.slots.release(p.slot_b);
        }
        if let Some((slot, _)) = self.right_walk_slot.take() {
            bridge.send(GuestInputEvent::TouchUp { slot });
            self.slots.release(slot);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use exr_android_bridge::MockInputBridge;

    #[test]
    fn dpad_diagonal_normalization_is_unit_length() {
        let mut dpad = DpadState::new(NormVec2::new(0.2, 0.8), 0.1);
        dpad.up = true;
        dpad.right = true;
        let (dx, dy) = dpad.direction_vector();
        let len = (dx * dx + dy * dy).sqrt();
        assert!((len - 1.0).abs() < 1e-4, "expected unit length, got {len}");
    }

    #[test]
    fn repeated_tap_burst_completes_and_stops() {
        let mut engine = GestureEngine::new();
        let mut bridge = MockInputBridge::new();
        let t0 = Instant::now();
        engine.repeated_taps.insert(
            "burst".into(),
            RepeatedTapTask::start(
                "burst".into(),
                NormVec2::new(0.5, 0.5),
                50,
                0,
                RepeatMode::Burst(3),
                t0,
            ),
        );
        for i in 0..5 {
            engine.tick(t0 + Duration::from_millis(i * 55), &mut bridge);
        }
        assert!(!engine.repeated_taps.contains_key("burst"));
        // 3 taps = 3 TouchDown + 3 TouchUp = 6 events
        assert_eq!(bridge.events.len(), 6);
    }

    #[test]
    fn pinch_uses_two_slots_and_releases_both() {
        let mut engine = GestureEngine::new();
        let mut bridge = MockInputBridge::new();
        let t0 = Instant::now();
        let pinch = ActivePinch::start(
            NormVec2::new(0.5, 0.5),
            true,
            0.2,
            100,
            1.0,
            t0,
            &mut engine.slots,
            &mut bridge,
        )
        .unwrap();
        assert_ne!(pinch.slot_a, pinch.slot_b);
        assert_eq!(engine.slots.active_count(), 2);
        engine.pinches.push(pinch);
        engine.tick(t0 + Duration::from_millis(120), &mut bridge);
        assert_eq!(engine.slots.active_count(), 0);
        assert!(bridge.active_slots.is_empty());
    }
}
