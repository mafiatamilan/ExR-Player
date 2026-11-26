use exr_android_bridge::{GuestInputEvent, InputBridge};
use exr_gestures::{apply_axis_curve, TouchSlotAllocator};
use exr_input::GamepadAxis;
use exr_keymap::{AxisCurve, NormVec2};
use std::collections::HashMap;

/// Tracks active gamepad stick-to-touch mappings and hot-plugged devices.
#[derive(Debug, Default)]
pub struct GamepadEngine {
    pub active_devices: HashMap<u32, String>,
    left_stick_slot: Option<u8>,
    right_stick_slot: Option<u8>,
    right_stick_pos: NormVec2,
    left_xy: (f32, f32),
    right_xy: (f32, f32),
}

impl GamepadEngine {
    pub fn new() -> Self {
        Self {
            active_devices: HashMap::new(),
            left_stick_slot: None,
            right_stick_slot: None,
            right_stick_pos: NormVec2::new(0.65, 0.45),
            left_xy: (0.0, 0.0),
            right_xy: (0.0, 0.0),
        }
    }

    pub fn on_connected(&mut self, id: u32, name: String) {
        self.active_devices.insert(id, name);
    }

    pub fn on_disconnected(
        &mut self,
        id: u32,
        slots: &mut TouchSlotAllocator,
        bridge: &mut dyn InputBridge,
    ) {
        self.active_devices.remove(&id);
        if self.active_devices.is_empty() {
            self.release_all(slots, bridge);
        }
    }

    pub fn update_axis(&mut self, axis: GamepadAxis, value: f32) {
        match axis {
            GamepadAxis::LeftX => self.left_xy.0 = value.clamp(-1.0, 1.0),
            GamepadAxis::LeftY => self.left_xy.1 = value.clamp(-1.0, 1.0),
            GamepadAxis::RightX => self.right_xy.0 = value.clamp(-1.0, 1.0),
            GamepadAxis::RightY => self.right_xy.1 = value.clamp(-1.0, 1.0),
            _ => {}
        }
    }

    pub fn sync_left_stick(
        &mut self,
        center: NormVec2,
        radius: f32,
        deadzone: f32,
        curve: AxisCurve,
        slots: &mut TouchSlotAllocator,
        bridge: &mut dyn InputBridge,
    ) {
        let dx = apply_axis_curve(self.left_xy.0, deadzone, curve);
        let dy = apply_axis_curve(self.left_xy.1, deadzone, curve);
        if dx.abs() > 1e-4 || dy.abs() > 1e-4 {
            let tx = (center.x + dx * radius).clamp(0.0, 1.0);
            let ty = (center.y + dy * radius).clamp(0.0, 1.0);
            if let Some(s) = self.left_stick_slot {
                bridge.send(GuestInputEvent::TouchMove {
                    slot: s,
                    x: tx,
                    y: ty,
                    pressure: 1.0,
                });
            } else if let Some(s) = slots.allocate() {
                self.left_stick_slot = Some(s);
                bridge.send(GuestInputEvent::TouchDown {
                    slot: s,
                    x: center.x,
                    y: center.y,
                    pressure: 1.0,
                });
                bridge.send(GuestInputEvent::TouchMove {
                    slot: s,
                    x: tx,
                    y: ty,
                    pressure: 1.0,
                });
            }
        } else if let Some(s) = self.left_stick_slot.take() {
            bridge.send(GuestInputEvent::TouchUp { slot: s });
            slots.release(s);
        }
    }

    pub fn tick_right_stick_aim(
        &mut self,
        origin: NormVec2,
        deadzone: f32,
        curve: AxisCurve,
        sensitivity: f32,
        dt_secs: f32,
        slots: &mut TouchSlotAllocator,
        bridge: &mut dyn InputBridge,
    ) {
        let dx = apply_axis_curve(self.right_xy.0, deadzone, curve);
        let dy = apply_axis_curve(self.right_xy.1, deadzone, curve);
        if dx.abs() > 1e-4 || dy.abs() > 1e-4 {
            if self.right_stick_slot.is_none() {
                if let Some(s) = slots.allocate() {
                    self.right_stick_slot = Some(s);
                    self.right_stick_pos = origin;
                    bridge.send(GuestInputEvent::TouchDown {
                        slot: s,
                        x: origin.x,
                        y: origin.y,
                        pressure: 1.0,
                    });
                }
            }
            if let Some(s) = self.right_stick_slot {
                self.right_stick_pos.x = (self.right_stick_pos.x + dx * sensitivity * dt_secs).clamp(0.05, 0.95);
                self.right_stick_pos.y = (self.right_stick_pos.y + dy * sensitivity * dt_secs).clamp(0.05, 0.95);
                bridge.send(GuestInputEvent::TouchMove {
                    slot: s,
                    x: self.right_stick_pos.x,
                    y: self.right_stick_pos.y,
                    pressure: 1.0,
                });
            }
        } else if let Some(s) = self.right_stick_slot.take() {
            bridge.send(GuestInputEvent::TouchUp { slot: s });
            slots.release(s);
        }
    }

    pub fn release_all(&mut self, slots: &mut TouchSlotAllocator, bridge: &mut dyn InputBridge) {
        self.left_xy = (0.0, 0.0);
        self.right_xy = (0.0, 0.0);
        if let Some(s) = self.left_stick_slot.take() {
            bridge.send(GuestInputEvent::TouchUp { slot: s });
            slots.release(s);
        }
        if let Some(s) = self.right_stick_slot.take() {
            bridge.send(GuestInputEvent::TouchUp { slot: s });
            slots.release(s);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use exr_android_bridge::MockInputBridge;

    #[test]
    fn gamepad_left_stick_respects_deadzone_and_disconnect() {
        let mut gp = GamepadEngine::new();
        let mut slots = TouchSlotAllocator::new();
        let mut bridge = MockInputBridge::new();

        gp.on_connected(0, "Xbox Controller".into());
        gp.update_axis(GamepadAxis::LeftX, 0.08);
        gp.sync_left_stick(
            NormVec2::new(0.15, 0.75),
            0.1,
            0.15,
            AxisCurve::Linear,
            &mut slots,
            &mut bridge,
        );
        assert_eq!(slots.active_count(), 0);

        gp.update_axis(GamepadAxis::LeftX, 0.85);
        gp.sync_left_stick(
            NormVec2::new(0.15, 0.75),
            0.1,
            0.15,
            AxisCurve::Linear,
            &mut slots,
            &mut bridge,
        );
        assert_eq!(slots.active_count(), 1);

        gp.on_disconnected(0, &mut slots, &mut bridge);
        assert_eq!(slots.active_count(), 0);
    }
}
