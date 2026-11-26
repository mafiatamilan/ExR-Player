# ExR Keymapping Engine — Implementation Status

## Workspace Crates (`frontend/exr-player-gui/crates/`)
1. `exr-input`: Normalized `KeyCode`, `MouseButton`, `Modifiers`, `KeyCombo`, `GamepadButton`, `GamepadAxis`, `InputTrigger`, `HostInputEvent`, and `InputStateManager` (auto-repeat deduplication, focus-loss `ReleaseAll` recovery).
2. `exr-keymap`: Versioned `KeymapProfile` (`schema_version = 1`), all 14 `ControlKind` variants, `ViewportMapper` (`[0.0, 1.0]` normalized coordinates + grid snap), `MappingResolver` (exact modifier + unmodified fallback), `ProfileStore` (save/load/import/export/delete + `.json.corrupt` recovery), `validate_profile()`, and `UndoRedoStack`.
3. `exr-android-bridge`: `InputBridge` trait, `GuestInputEvent` (`TouchDown`, `TouchMove`, `TouchUp`, `KeyEvent`, `SensorEvent`), `CallbackInputBridge` (connects directly to `ExrEngineBackend` / `libexr-x86.so`), `GuestDaemonBridge` (NDJSON 16-slot `ABS_MT_SLOT` + SensorHAL stream), and `MockInputBridge`.
4. `exr-gestures`: `TouchSlotAllocator` (16 slots `0..=15`), radial-deadzone 8-way normalized `DpadState`, `apply_axis_curve` (`Linear`, `Quadratic`, `SCurve`), monotonic `RepeatedTapTask` (`Hold`, `Toggle`, `Burst`), smoothstep `ActiveSwipe`, dual-slot `ActivePinch` zoom, `RightClickWalk`, and `GestureEngine`.
5. `exr-aim`: `FpsShootingController` (relative mouse delta, independent X/Y sensitivity, invert Y, EMA smoothing, pitch clamp, seamless edge recentering, focus-loss unlock), `FreeLookController`, and `MobaSkillController` (`Quick`, `Auto`, `Manual` + cancel).
6. `exr-gamepad`: `GamepadEngine` with hot-plug tracking, deadzone + axis curves, left-stick D-pad, and continuous right-stick camera pan.
7. `exr-macro`: `MacroRecorder` (`start`, `pause`, `resume`, `stop`), `MacroDefinition` (JSON import/export, step reorder/delete), and non-blocking monotonic `MacroPlayer` (loop count, speed multiplier).
8. `exr-smart-controls`: `SmartContextEngine` (`OnFoot`, `Driving`, `Swimming`, `Parachuting`, `InventoryOpen`, `MapOpen`, `Combat`, `Menus`, `Custom`), manual override, and off-hot-path RGBA region signature matching.
9. `exr-keymap-ui`: egui Keymapping Editor (`show_top_bar`, `show_side_panel` with 14-control palette + dynamic properties + opacity/validation, `paint_and_handle_overlay` with drag/snap/rebind/delete, `paint_play_overlay`).

## `exr-player` Integration
- `frontend/exr-player-gui/src/keymap_runtime.rs`: `KeymapRuntime` bridging `egui` events + focus state into `exr-input` -> `exr-keymap` -> `exr-gestures`/`exr-aim`/`exr-macro` -> `EmulatorBackend`.
- `frontend/exr-player-gui/src/app.rs` & `src/ui/viewport.rs`: Toggles Keymapping Editor via `app.keymap_open` (right toolbar Keyboard button), supports live `Test Mode` inside the editor, and runs Play Mode input interception + HUD overlay when closed.
