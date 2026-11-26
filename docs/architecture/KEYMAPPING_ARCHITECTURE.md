# ExR Keymapping Engine Architecture

## Design Goals
1. **Zero UI Coupling**: Input capture, binding resolution, gesture synthesis, and guest injection live in pure Rust crates independent of `eframe`/`egui`.
2. **Normalized Coordinates**: Every control position, radius, and swipe endpoint is stored in `[0.0, 1.0]` normalized guest viewport space, invariant to host window size or guest resolution.
3. **Sub-millisecond Hot Path**: Input events pass through `InputStateManager` -> `MappingResolver` -> `MultiTouchSlotAllocator` -> `InputBridge` with zero heap allocations on steady-state pointer moves.
4. **Safety & Focus Recovery**: Window focus loss or disabling Play Mode immediately releases all active virtual touch slots, cancels active timers/macros, and unlocks the host cursor.

## Crate Topology (`frontend/exr-player-gui/crates/`)

```
+-------------------------------------------------------------------+
|                        exr-player (eframe)                        |
|  app.rs / ui/viewport.rs / ui/toolbar.rs                          |
+---------------------------+---------------------------------------+
                            |
           +----------------+----------------+
           v                                 v
+---------------------+           +---------------------+
|   exr-keymap-ui     |           |   exr-keymap        |
| (egui Editor,       |---------->| (Schema, Profiles,  |
|  Overlay, Palette,  |           |  Resolver, Undo,    |
|  Properties Panel)  |           |  Validation)        |
+---------------------+           +----------+----------+
                                             |
       +-------------+-------------+---------+---------+-------------+
       v             v             v                   v             v
+------------+ +-----------+ +-----------+      +------------+ +-------------+
| exr-input  | |exr-gestures| |  exr-aim  |     |exr-gamepad | |  exr-macro  |
| (Host Evts,| |(16-slot MT,| |(FPS, Aim, |     |(SDL/XInput | | (Record,    |
|  Focus/Keys| | WASD, Tap, | | FreeLook, |     | Sticks,    | |  Edit, Loop,|
|  State Mgr)| | Swipe,Pinch| | MOBA Cast)|     | Curves)    | |  Playback)  |
+-----+------+ +-----+-----+ +-----+-----+      +-----+------+ +------+------+
      |              |             |                  |               |
      +--------------+-------------+------------------+---------------+
                                   |
                                   v
                      +-------------------------+
                      |  exr-smart-controls     |
                      | (Context Engine, Hints) |
                      +------------+------------+
                                   |
                                   v
                      +-------------------------+
                      |  exr-android-bridge     |
                      | (InputBridge trait,     |
                      |  ExrGuestBridge, MT/    |
                      |  Sensor Event Protocol) |
                      +-------------------------+
```

## Data Flow (Play Mode)
1. Host input events (`KeyDown`, `KeyUp`, `MouseMove`, `MouseDelta`, `MouseButton`, `MouseWheel`, `GamepadAxis`, `GamepadButton`, `FocusLost`) enter `exr-input::InputStateManager`.
2. `InputStateManager` tracks pressed keys/modifiers/buttons and deduplicates key auto-repeats.
3. `exr-keymap::MappingResolver` matches active `ControlBinding`s in the active `KeymapProfile` (and optional `SmartContextEngine` layer).
4. Specialized engines (`exr-gestures`, `exr-aim`, `exr-gamepad`, `exr-macro`) emit `GuestInputEvent`s using `TouchSlotAllocator` (slots `0..15`).
5. `exr-android-bridge::InputBridge` translates `GuestInputEvent`s into `libexr-x86.so` calls or guest input-daemon packets.
