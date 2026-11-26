# ExR Keymapping Engine — Test Report

Command:
```bash
nix-shell -p cargo rustc pkg-config gtk3 glib libxkbcommon libGL --run \
  "cargo test --workspace --manifest-path /home/david/Projects/ExR/frontend/exr-player-gui/Cargo.toml"
```

## Results Summary
- `exr-aim`: 2 passed, 0 failed (`fps_shooting_recentering_and_focus_release`, `moba_quick_cast_projects_and_fires_on_release`)
- `exr-android-bridge`: 2 passed, 0 failed (`guest_daemon_bridge_serializes_ndjson_packets`, `mock_bridge_tracks_slots_and_releases_all`)
- `exr-gamepad`: 1 passed, 0 failed (`gamepad_left_stick_respects_deadzone_and_disconnect`)
- `exr-gestures`: 3 passed, 0 failed (`dpad_diagonal_normalization_is_unit_length`, `pinch_uses_two_slots_and_releases_both`, `repeated_tap_burst_completes_and_stops`)
- `exr-input`: 3 passed, 0 failed (`deduplicates_key_auto_repeat_and_invalid_release`, `focus_lost_releases_all_held_inputs`, `key_combo_display_formats_modifiers`)
- `exr-keymap`: 5 passed, 0 failed (`profile_roundtrip_and_validation`, `corrupt_profile_recovers_safely`, `resolver_handles_modifiers_and_unmodified_fallback`, `undo_redo_stack_restores_snapshots`, `viewport_mapper_converts_and_snaps`)
- `exr-keymap-ui`: 0 tests (compiled clean, integrated into `exr-player`)
- `exr-macro`: 1 passed, 0 failed (`macro_record_edit_export_and_playback`)
- `exr-smart-controls`: 1 passed, 0 failed (`context_releases_mouse_on_inventory_and_respects_manual_override`)
- `exr-player` (`src/main.rs`): 6 passed, 0 failed
- `exr-multi` (`src/multi.rs`): 1 passed, 0 failed
- **Total**: 25 passed, 0 failed, 0 ignored.

## Release Build & Staging
- `cargo build --release --manifest-path /home/david/Projects/ExR/frontend/exr-player-gui/Cargo.toml` succeeded.
- Staged binaries: `dist/exr-player`, `dist/exr-multi`.
