# BlueStacks 5 — Controls Editor Research

Reference screenshot: `uploaded_media_1_1795685047044.jpg` (BlueStacks 5.5.100.1040 N32).

## Editor Layout & Workflow
- **Top header**: Centered `Keyboard and mouse` / `Gamepad` tab switcher.
- **Right palette (`Controls editor`)**:
  - `Control scheme` dropdown (`Smart`, custom schemes) with cloud sync, import, export, and folder icons.
  - `Add controls` instruction: *"Drag the actions on the screen to bind keys. Right click to fine-tune settings."*
  - 3-column icon grid of 14 control widgets:
    1. `Tap spot`
    2. `Repeated tap`
    3. `D-pad`
    4. `Aim, pan and shoot`
    5. `Free look`
    6. `Script`
    7. `Swipe`
    8. `Zoom`
    9. `Tilt`
    10. `MOBA D-Pad`
    11. `MOBA Skill pad`
    12. `Rotate`
    13. `Edge scroll`
    14. `Scroll`
  - Bottom action bar: `Reset` and `Save`.
- **Canvas & Controls**:
  - White pill/circle badges on canvas (`W/A/S/D` D-pad with outer blue ring and inner directional pads, `Ctrl+1..8` tap spots, `MouseWheelUp`/`MouseWheelDown` zoom/scroll, `Right click`, `Middle click`).
  - Aim/pan/shoot node shows inline `X` / `Y` sensitivity readouts directly on the canvas widget (`X 1.00`, `Y 0.40`).
  - Right-clicking any placed control opens a fine-tuning properties popover (coordinates, sensitivity, tap count, hold/toggle mode, etc.).

## Key Takeaways for ExR Player
- Clear separation of `Keyboard & Mouse` vs `Gamepad` tabs.
- Drag-from-palette onto normalized viewport coordinates + right-click or side-panel fine-tuning.
- Inline sensitivity readout on FPS Aim widget (`X` / `Y`).
- Explicit `Control scheme` selector with Import / Export / Reset / Save.
