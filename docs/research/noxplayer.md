# NoxPlayer — Keyset Editor Research

Sources: Publicly documented NoxPlayer Keyset / Keyboard Mapping specifications (bignox.com).

## Control Types & Features
- **D-Pad (Direction Control)**: 360-degree / 8-way directional movement mapped to `W/A/S/D` by default, dragged over the game's virtual joystick.
- **Single Click (Tap Spot)**: Maps a key press to a single touch coordinate.
- **Multistrike (Repeated Tap)**: Supports **Long Press Mode** (continuous rapid taps while held) and **Click Mode** (fixed burst count per press).
- **View Control (Free Look / Shooting Mode)**: Camera pan via mouse or key bounds, paired with cursor hide/lock toggle in FPS titles.
- **Targeted Skill (MOBA Skill Pad)**:
  - **Auto Cast**: Instant cast toward cursor on press.
  - **Quick Cast**: Hold key, aim with mouse, release to fire.
  - **Manual Cast**: Press key, aim with mouse, left-click to fire.
- **Two-Finger Pinch (Zoom In / Zoom Out)**: Simultaneous dual-pointer pinch gesture mapped to keys (e.g., `Z`/`X`) or scroll wheel.
- **Macro / Script Editor**: Record or script multi-step sequences with delays, bound to a single trigger key.

## Key Takeaways for ExR Player
- Explicit three-mode MOBA casting (`Auto`, `Quick`, `Manual`) with range scaling.
- Dual-mode Multistrike (`Hold` vs `Burst` count).
- Two-finger pinch with configurable center and speed.
