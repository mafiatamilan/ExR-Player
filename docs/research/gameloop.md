# GameLoop — Keymapping & Smart Controls Research

Sources: Publicly documented GameLoop keymapping & Smart Mode documentation (gameloop.com).

## Editor Layout & Workflow
- Triggered from the right-side toolbar Keyboard icon, rendering a semi-transparent overlay over the live game viewport.
- Mode selector between **Standard** layout and **Smart Keymapping / Smart Mode** for FPS/BR titles.

## Control Types & Features
- **Smart Mode / Shooting Mode**:
  - Dynamically locks/unlocks cursor (`Ctrl` toggle by default).
  - Relative mouse movement controls FPS camera/crosshair continuously without holding a mouse button.
  - `Left Click` = Fire, `Right Click` = Aim Down Sights (ADS), `W/A/S/D` = Virtual Joystick, `R/Space/C/Z/F/M/Tab` = standard FPS tap spots.
- **Independent X/Y Sensitivity**: Tunable horizontal and vertical camera sensitivity per profile.
- **Context-Aware Profiles**: Game-specific layouts that adapt controls to active gameplay contexts (e.g., on-foot vs driving vs inventory/map cursor release).

## Key Takeaways for ExR Player
- Seamless relative-mouse FPS capture + instant `Ctrl` / `~` cursor release.
- Context-aware profile switching (e.g., releasing mouse capture automatically when inventory/map is open).
- Clear focus-loss recovery so mouse lock never traps the desktop cursor.
