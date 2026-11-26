use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::io::Write;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GuestSensorKind {
    Accelerometer,
    Gyroscope,
    RotationVector,
    Orientation,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum GuestInputEvent {
    TouchDown {
        slot: u8,
        x: f32,
        y: f32,
        pressure: f32,
    },
    TouchMove {
        slot: u8,
        x: f32,
        y: f32,
        pressure: f32,
    },
    TouchUp {
        slot: u8,
    },
    KeyEvent {
        key_code: u32,
        pressed: bool,
    },
    SensorEvent {
        kind: GuestSensorKind,
        values: [f32; 3],
        timestamp_ns: u64,
    },
}

/// Transport-agnostic bridge delivering virtual touch, key, and sensor events to the Android guest.
pub trait InputBridge: Send {
    fn send(&mut self, event: GuestInputEvent);

    /// Releases every currently active touch slot (used on focus loss or mode switch).
    fn release_all(&mut self);
}

/// Callback-based adapter bridging `InputBridge` to `exr-player`'s `EmulatorBackend`
/// (`touch_down`, `touch_move`, `touch_up`) without circular crate dependencies.
pub struct CallbackInputBridge<F>
where
    F: FnMut(GuestInputEvent) + Send,
{
    active_slots: BTreeSet<u8>,
    callback: F,
}

impl<F> CallbackInputBridge<F>
where
    F: FnMut(GuestInputEvent) + Send,
{
    pub fn new(callback: F) -> Self {
        Self {
            active_slots: BTreeSet::new(),
            callback,
        }
    }
}

impl<F> InputBridge for CallbackInputBridge<F>
where
    F: FnMut(GuestInputEvent) + Send,
{
    fn send(&mut self, event: GuestInputEvent) {
        match &event {
            GuestInputEvent::TouchDown { slot, .. } => {
                self.active_slots.insert(*slot);
            }
            GuestInputEvent::TouchUp { slot } => {
                if !self.active_slots.remove(slot) {
                    return;
                }
            }
            _ => {}
        }
        (self.callback)(event);
    }

    fn release_all(&mut self) {
        let slots: Vec<u8> = self.active_slots.iter().copied().collect();
        for slot in slots {
            self.active_slots.remove(&slot);
            (self.callback)(GuestInputEvent::TouchUp { slot });
        }
    }
}

/// Streaming bridge writing newline-delimited JSON packets to an Android guest input daemon
/// (virtio-serial / Unix socket / pipe) supporting 16-slot `ABS_MT_SLOT` + SensorHAL injection.
pub struct GuestDaemonBridge<W: Write + Send> {
    writer: W,
    active_slots: BTreeSet<u8>,
    fallback_on_error: bool,
}

impl<W: Write + Send> GuestDaemonBridge<W> {
    pub fn new(writer: W) -> Self {
        Self {
            writer,
            active_slots: BTreeSet::new(),
            fallback_on_error: true,
        }
    }

    pub fn active_slot_count(&self) -> usize {
        self.active_slots.len()
    }
}

impl<W: Write + Send> InputBridge for GuestDaemonBridge<W> {
    fn send(&mut self, event: GuestInputEvent) {
        match &event {
            GuestInputEvent::TouchDown { slot, .. } => {
                self.active_slots.insert(*slot);
            }
            GuestInputEvent::TouchUp { slot } => {
                if !self.active_slots.remove(slot) {
                    return;
                }
            }
            _ => {}
        }
        if let Ok(line) = serde_json::to_string(&event) {
            let res = writeln!(self.writer, "{line}");
            if res.is_err() && !self.fallback_on_error {
                // Ignored in non-fatal mode
            }
        }
    }

    fn release_all(&mut self) {
        let slots: Vec<u8> = self.active_slots.iter().copied().collect();
        for slot in slots {
            self.send(GuestInputEvent::TouchUp { slot });
        }
    }
}

/// Deterministic in-memory bridge for unit tests, integration tests, and latency benchmarks.
#[derive(Debug, Default)]
pub struct MockInputBridge {
    pub events: Vec<GuestInputEvent>,
    pub active_slots: BTreeSet<u8>,
}

impl MockInputBridge {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn clear(&mut self) {
        self.events.clear();
    }
}

impl InputBridge for MockInputBridge {
    fn send(&mut self, event: GuestInputEvent) {
        match &event {
            GuestInputEvent::TouchDown { slot, .. } => {
                self.active_slots.insert(*slot);
            }
            GuestInputEvent::TouchUp { slot } => {
                if !self.active_slots.remove(slot) {
                    return;
                }
            }
            _ => {}
        }
        self.events.push(event);
    }

    fn release_all(&mut self) {
        let slots: Vec<u8> = self.active_slots.iter().copied().collect();
        for slot in slots {
            self.send(GuestInputEvent::TouchUp { slot });
        }
    }
}

/// Measures single-event dispatch latency over an `InputBridge`.
pub fn benchmark_bridge_dispatch(bridge: &mut dyn InputBridge, iterations: u32) -> Duration {
    let start = Instant::now();
    for i in 0..iterations {
        let t = (i % 100) as f32 / 100.0;
        bridge.send(GuestInputEvent::TouchDown {
            slot: 0,
            x: t,
            y: t,
            pressure: 1.0,
        });
        bridge.send(GuestInputEvent::TouchMove {
            slot: 0,
            x: (t + 0.05).min(1.0),
            y: (t + 0.05).min(1.0),
            pressure: 1.0,
        });
        bridge.send(GuestInputEvent::TouchUp { slot: 0 });
    }
    start.elapsed()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mock_bridge_tracks_slots_and_releases_all() {
        let mut bridge = MockInputBridge::new();
        bridge.send(GuestInputEvent::TouchDown {
            slot: 0,
            x: 0.2,
            y: 0.8,
            pressure: 1.0,
        });
        bridge.send(GuestInputEvent::TouchDown {
            slot: 1,
            x: 0.7,
            y: 0.4,
            pressure: 1.0,
        });
        assert_eq!(bridge.active_slots.len(), 2);

        bridge.release_all();
        assert!(bridge.active_slots.is_empty());
        assert_eq!(bridge.events.len(), 4);
    }

    #[test]
    fn guest_daemon_bridge_serializes_ndjson_packets() {
        let mut buf: Vec<u8> = Vec::new();
        {
            let mut bridge = GuestDaemonBridge::new(&mut buf);
            bridge.send(GuestInputEvent::SensorEvent {
                kind: GuestSensorKind::Gyroscope,
                values: [0.1, -0.2, 0.0],
                timestamp_ns: 12345,
            });
        }
        let text = String::from_utf8(buf).unwrap();
        assert!(text.contains("\"type\":\"sensor_event\""));
        assert!(text.contains("\"kind\":\"gyroscope\""));
    }
}
