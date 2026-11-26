# ExR Player

Cross-platform Android gaming emulator featuring a headless x86/x86_64 virtualization engine, an `eframe`/`egui` desktop player, a Multi-Instance Manager (`exr-multi`), and a modular 9-crate Rust keymapping engine.

## Repository Layout

- `engine/` — Headless x86/x86_64 execution engine (`exr-x86`, `exr-img`, `libexr-x86.so`, `include/exr/api.h`)
- `frontend/exr-player-gui/` — `exr-player` & `exr-multi` desktop UI (`eframe 0.37` / `egui`) + 9-crate keymapping workspace:
  - `exr-input` — Host keyboard, mouse, gamepad, and focus-state tracking
  - `exr-keymap` — Versioned `.json` per-app profiles, `[0.0, 1.0]` normalized coordinates, resolver, validation, undo/redo
  - `exr-android-bridge` — Multi-touch slot & SensorHAL guest bridge
  - `exr-gestures` — 16-slot touch allocator, WASD D-pad, analog stick, repeated tap, swipe, 2-finger pinch/zoom, right-click walk
  - `exr-aim` — FPS shooting mode (relative mouse capture + recentering), crosshair aim, free look, MOBA skill casting
  - `exr-gamepad` — Controller stick/button mapping, deadzones, response curves, hot-plugging
  - `exr-macro` — Macro recorder, JSON import/export, and non-blocking playback
  - `exr-smart-controls` — Context-aware mapping engine
  - `exr-keymap-ui` — Interactive keymapping editor, 14-control vector icon palette, and Play Mode HUD overlay
- `tools/` — Standalone ISO-to-`.exrimg` converter (`exr-convert.c`)
- `docs/` — Competitor research, architecture specs, implementation status, and test reports
- `examples/` — Reference `config.exr` VM profile

## Quick Build & Run

```sh
# 1. Build headless x86 engine + verify BIOS smoke test
./engine/scripts/build-headless.sh
./engine/tests/exr/smoke-bios.sh

# 2. Test & build frontend workspace (NixOS)
nix-shell -p cargo rustc pkg-config gtk3 glib libxkbcommon libGL --run \
  "cargo test --workspace --manifest-path frontend/exr-player-gui/Cargo.toml && \
   cargo build --release --manifest-path frontend/exr-player-gui/Cargo.toml"

# 3. Launch staged player or multi-instance manager
./dist/run-player.sh
./dist/run-multi.sh
```
