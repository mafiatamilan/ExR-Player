# ExR Keymap Profile Schema (`v1`)

Profiles are stored as versioned JSON files in the user's ExR config directory (`profiles/<profile_id>.json`).

## Top-Level Structure
- `schema_version`: `u32` (currently `1`)
- `id`: `String` (UUID or slug, e.g. `"fps-default"`)
- `name`: `String` (Display name)
- `game_package`: `Option<String>` (Optional Android package name for auto-matching)
- `mode`: `"standard" | "smart"`
- `toggle_hotkey`: `KeyCombo` (Default `Alt+F1` or `F1`)
- `overlay_opacity`: `f32` (`0.0..=1.0`)
- `show_overlay_in_play`: `bool`
- `controls`: `Vec<ControlMapping>`
- `gamepad_deadzone_left`: `f32`
- `gamepad_deadzone_right`: `f32`
- `macros`: `Vec<MacroDefinition>`
- `smart_contexts`: `Vec<SmartContextBinding>`

## `ControlMapping` Variants
Each control has `id: String`, `label: String`, `pos: NormVec2` (`[0.0, 1.0]`), `enabled: bool`, and `kind: ControlKind`:
1. `TapSpot` (`trigger`, `hold_duration_ms`, `multi_triggers`)
2. `RepeatedTap` (`trigger`, `interval_ms`, `initial_delay_ms`, `mode: Hold | Toggle | Burst(u32)`)
3. `Dpad` (`up`, `down`, `left`, `right`, `radius`, `deadzone`, `sensitivity`, `return_to_center`)
4. `AnalogJoystick` (`up`, `down`, `left`, `right`, `radius`, `deadzone`, `curve`, `invert_x`, `invert_y`)
5. `FpsAim` (`toggle_key`, `fire_key`, `secondary_fire_key`, `sensitivity_x`, `sensitivity_y`, `invert_y`, `smoothing`, `pitch_min`, `pitch_max`)
6. `CrosshairAim` (`trigger`, `region_radius`, `sensitivity_x`, `sensitivity_y`, `fire_trigger`)
7. `Swipe` (`trigger`, `end_pos: NormVec2`, `duration_ms`)
8. `MobaSkill` (`trigger`, `cancel_trigger`, `cast_mode: Quick | Auto | Manual`, `max_radius`, `confirm_on_release`)
9. `FreeLook` (`trigger`, `sensitivity_x`, `sensitivity_y`, `radius`)
10. `ZoomPinch` (`zoom_in_trigger`, `zoom_out_trigger`, `spread_radius`, `duration_ms`, `speed`)
11. `RightClickWalk` (`trigger`, `reduce_inertia`, `hold_to_walk`)
12. `GamepadControl` (`source: GamepadInput`, `target_kind`, `deadzone`, `curve`, `radius`)
13. `MacroTrigger` (`trigger`, `macro_id`, `loop_mode`, `speed`)
14. `SensorControl` (`sensor: Accelerometer | Gyroscope | RotationVector | Orientation`, `keys_or_axis`, `magnitude`, `smoothing`)

## Validation & Recovery
- Coordinates are clamped to `[0.0, 1.0]`.
- Duplicate triggers are flagged by `validate_profile()` as `BindingConflict` warnings without crashing.
- Corrupted JSON loads fall back safely to `ProfileStore::backup_and_reset_corrupt()`.
