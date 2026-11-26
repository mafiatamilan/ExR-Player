use exr_android_bridge::{GuestInputEvent, InputBridge};
use exr_input::KeyCode;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum MacroStep {
    Wait { delay_ms: u32 },
    TouchDown { slot: u8, x: f32, y: f32 },
    TouchMove { slot: u8, x: f32, y: f32 },
    TouchUp { slot: u8 },
    KeyPress { key: KeyCode, down: bool },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MacroDefinition {
    pub id: String,
    pub name: String,
    pub steps: Vec<MacroStep>,
    #[serde(default = "default_speed")]
    pub default_speed: f32,
    #[serde(default = "default_loops")]
    pub default_loop_count: u32,
}

fn default_speed() -> f32 {
    1.0
}

fn default_loops() -> u32 {
    1
}

impl MacroDefinition {
    pub fn to_json(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self).map_err(|e| format!("serialize macro: {e}"))
    }

    pub fn from_json(json: &str) -> Result<Self, String> {
        serde_json::from_str(json).map_err(|e| format!("parse macro: {e}"))
    }

    pub fn move_step(&mut self, from: usize, to: usize) {
        if from < self.steps.len() && to < self.steps.len() && from != to {
            let item = self.steps.remove(from);
            self.steps.insert(to, item);
        }
    }

    pub fn delete_step(&mut self, idx: usize) {
        if idx < self.steps.len() {
            self.steps.remove(idx);
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecorderState {
    Idle,
    Recording,
    Paused,
}

#[derive(Debug)]
pub struct MacroRecorder {
    pub state: RecorderState,
    id: String,
    name: String,
    steps: Vec<MacroStep>,
    last_event_at: Option<Instant>,
}

impl MacroRecorder {
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            state: RecorderState::Idle,
            id: id.into(),
            name: name.into(),
            steps: Vec::new(),
            last_event_at: None,
        }
    }

    pub fn start(&mut self, now: Instant) {
        self.steps.clear();
        self.state = RecorderState::Recording;
        self.last_event_at = Some(now);
    }

    pub fn pause(&mut self) {
        if self.state == RecorderState::Recording {
            self.state = RecorderState::Paused;
        }
    }

    pub fn resume(&mut self, now: Instant) {
        if self.state == RecorderState::Paused {
            self.state = RecorderState::Recording;
            self.last_event_at = Some(now);
        }
    }

    pub fn record_step(&mut self, step: MacroStep, now: Instant) {
        if self.state != RecorderState::Recording {
            return;
        }
        if let Some(prev) = self.last_event_at {
            let ms = now.saturating_duration_since(prev).as_millis() as u32;
            if ms >= 5 {
                self.steps.push(MacroStep::Wait { delay_ms: ms });
            }
        }
        self.last_event_at = Some(now);
        self.steps.push(step);
    }

    pub fn stop(&mut self) -> MacroDefinition {
        self.state = RecorderState::Idle;
        self.last_event_at = None;
        MacroDefinition {
            id: self.id.clone(),
            name: self.name.clone(),
            steps: std::mem::take(&mut self.steps),
            default_speed: 1.0,
            default_loop_count: 1,
        }
    }
}

#[derive(Debug)]
struct ActivePlayback {
    def: MacroDefinition,
    cursor: usize,
    loops_remaining: Option<u32>,
    speed: f32,
    next_step_at: Instant,
}

/// Non-blocking monotonic-clock macro playback engine.
#[derive(Debug, Default)]
pub struct MacroPlayer {
    library: HashMap<String, MacroDefinition>,
    active: HashMap<String, ActivePlayback>,
}

impl MacroPlayer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, def: MacroDefinition) {
        self.library.insert(def.id.clone(), def);
    }

    pub fn get(&self, id: &str) -> Option<&MacroDefinition> {
        self.library.get(id)
    }

    pub fn start(&mut self, id: &str, loop_count: u32, speed: f32, now: Instant) -> bool {
        let Some(def) = self.library.get(id).cloned() else {
            return false;
        };
        if def.steps.is_empty() {
            return false;
        }
        let loops_remaining = if loop_count == 0 { None } else { Some(loop_count) };
        self.active.insert(
            id.to_string(),
            ActivePlayback {
                def,
                cursor: 0,
                loops_remaining,
                speed: speed.clamp(0.1, 10.0),
                next_step_at: now,
            },
        );
        true
    }

    pub fn stop(&mut self, id: &str) {
        self.active.remove(id);
    }

    pub fn stop_all(&mut self) {
        self.active.clear();
    }

    pub fn is_playing(&self, id: &str) -> bool {
        self.active.contains_key(id)
    }

    pub fn tick(&mut self, now: Instant, bridge: &mut dyn InputBridge) {
        let mut finished = Vec::new();
        for (id, pb) in &mut self.active {
            while now >= pb.next_step_at {
                if pb.cursor >= pb.def.steps.len() {
                    if let Some(rem) = &mut pb.loops_remaining {
                        *rem = rem.saturating_sub(1);
                        if *rem == 0 {
                            finished.push(id.clone());
                            break;
                        }
                    }
                    pb.cursor = 0;
                }
                let step = &pb.def.steps[pb.cursor];
                pb.cursor += 1;
                match step {
                    MacroStep::Wait { delay_ms } => {
                        let scaled = ((*delay_ms as f32) / pb.speed).round().max(1.0) as u64;
                        pb.next_step_at = now + Duration::from_millis(scaled);
                        break;
                    }
                    MacroStep::TouchDown { slot, x, y } => {
                        bridge.send(GuestInputEvent::TouchDown {
                            slot: *slot,
                            x: x.clamp(0.0, 1.0),
                            y: y.clamp(0.0, 1.0),
                            pressure: 1.0,
                        });
                    }
                    MacroStep::TouchMove { slot, x, y } => {
                        bridge.send(GuestInputEvent::TouchMove {
                            slot: *slot,
                            x: x.clamp(0.0, 1.0),
                            y: y.clamp(0.0, 1.0),
                            pressure: 1.0,
                        });
                    }
                    MacroStep::TouchUp { slot } => {
                        bridge.send(GuestInputEvent::TouchUp { slot: *slot });
                    }
                    MacroStep::KeyPress { .. } => {}
                }
            }
        }
        for id in finished {
            self.active.remove(&id);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use exr_android_bridge::MockInputBridge;

    #[test]
    fn macro_record_edit_export_and_playback() {
        let t0 = Instant::now();
        let mut rec = MacroRecorder::new("combo-1", "Quick Combo");
        rec.start(t0);
        rec.record_step(MacroStep::TouchDown { slot: 5, x: 0.5, y: 0.5 }, t0);
        rec.record_step(
            MacroStep::TouchUp { slot: 5 },
            t0 + Duration::from_millis(40),
        );
        let mut def = rec.stop();
        assert_eq!(def.steps.len(), 3);

        let json = def.to_json().unwrap();
        let parsed = MacroDefinition::from_json(&json).unwrap();
        assert_eq!(def, parsed);

        def.move_step(0, 1);
        def.delete_step(0);
        assert_eq!(def.steps.len(), 2);

        let mut player = MacroPlayer::new();
        player.register(parsed);
        assert!(player.start("combo-1", 1, 2.0, t0));
        let mut bridge = MockInputBridge::new();
        player.tick(t0, &mut bridge);
        player.tick(t0 + Duration::from_millis(25), &mut bridge);
        player.tick(t0 + Duration::from_millis(30), &mut bridge);
        assert!(!player.is_playing("combo-1"));
        assert_eq!(bridge.events.len(), 2);
    }
}
