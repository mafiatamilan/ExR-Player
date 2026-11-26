# ExR Android Guest Input Bridge

## Purpose
`exr-android-bridge` decouples the keymapping engine from the transport used to deliver touch, key, and sensor events into the Android guest running inside `exr-x86`.

## `InputBridge` Trait
All mapping subsystems emit `GuestInputEvent` variants:
- `TouchDown { slot: u8, x: f32, y: f32, pressure: f32 }`
- `TouchMove { slot: u8, x: f32, y: f32, pressure: f32 }`
- `TouchUp { slot: u8 }`
- `KeyEvent { key_code: u32, pressed: bool }`
- `SensorEvent { kind: SensorKind, values: [f32; 3], timestamp_ns: u64 }`

Coordinates `x` and `y` are normalized `[0.0, 1.0]` in guest display space.

## Transport Adapters

### 1. `ExrEngineBridge` (Direct `libexr-x86.so` C ABI)
- Uses `exr_machine_send_pointer_abs(machine, (x * 32767.0) as i32, (y * 32767.0) as i32)` and `exr_machine_send_pointer_btn(machine, 1, down)` for primary touch slot `0`, plus `exr_machine_send_key` for keyboard pass-through.
- Tracks active slot `0` state so `TouchUp` is never dropped or duplicated.

### 2. `GuestDaemonBridge` (Multi-Touch & Sensor Socket Protocol)
- Binary/JSON-line framed protocol targeting a lightweight Android guest input daemon (`uinput` / `EV_ABS` `ABS_MT_SLOT` `0..15` + `SensorHAL` pipe over virtio-serial / TCP).
- Enables true 16-finger simultaneous multi-touch (e.g., WASD in slot 0 + FPS aim in slot 1 + 2-finger pinch in slots 2-3 + MOBA skill in slot 4) and real accelerometer/gyroscope/orientation injection.
- Falls back cleanly to `ExrEngineBridge` single-pointer mode when the guest daemon is not connected.

### 3. `MockInputBridge` (Testing & Benchmarks)
- Records all emitted `GuestInputEvent`s with timestamps for deterministic unit tests, integration tests, and latency benchmarks.
