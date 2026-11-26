# 07 — Anti-Slop Checklist (reject the build if ANY item fails)

## A. "Fake" UI (the main failure mode)
- [ ] Every button is **wired**: click → real state change or backend call (MockBackend ok). No decorative buttons.
- [ ] No empty rectangles standing in for icons. Every icon is from ONE set (doc 02 §5) at the exact size (20 px toolbar / 16 px nav & nav-items).
- [ ] No placeholder text: no "Lorem", "TODO", "Button", "Label", "Item 1", "Coming soon" shown to the user.
- [ ] No giant `Frame::group` boxes around every row. Settings rows are hairline-separated (doc 04 §2).
- [ ] No boxes drawn to *pretend* to be the Android screen. The viewport shows the MockBackend test pattern through the real texture path.
- [ ] No emoji used as icons.

## B. Pixel fidelity
- [ ] All numbers come from `theme.rs` constants; grep for magic numbers in `ui/` → none except 0/1/2.
- [ ] Title bar = 32, toolbar = 44, button = 36, glyph = 20, nav strip = 36, dialog = 760×540, key cap = 36 (verify by `egui` debug overlay or screenshot at 100 %).
- [ ] Colors only from the token table; no `Color32::from_rgb(...)` inline in widgets.
- [ ] Text crisp at 100 %, 125 %, 150 %, 200 % DPI (test with `WINIT_X11_SCALE_FACTOR` / Windows scaling).
- [ ] Logo displays the real glyph (assets/logo), never a text "ExR".

## C. Behavior
- [ ] Window: custom title bar drags, double-click maximizes, edge-resize works, min/max/close work, Alt+F4 works.
- [ ] Settings: Save disabled until dirty; Cancel discards; restart banner appears for restart-fields; validation blocks Save.
- [ ] Shortcuts page rejects plain letters (F1–F12 or Ctrl/Shift/Alt combos only).
- [ ] Keymapper: all 17 modules placeable by drag from palette; keys bindable; Undo/Redo; JSON save/load round-trip; play-mode forwards mapped input to backend.
- [ ] Macro editor: Edit area + Command area + Guide; parse errors show line/col.
- [ ] Esc/Enter semantics as specified; keyboard focus ring visible.

## D. Performance
- [ ] Idle (no frames, mouse still): **0 repaints/sec** (log `ctx.frame_nr()` deltas to prove it).
- [ ] No `request_repaint()` inside `update()` unless an animation is active.
- [ ] Release binary < 12 MB (glow, stripped); startup < 300 ms.
- [ ] No per-frame `String`/`Vec` allocation in toolbar/titlebar (spot-check with `dhat` or `cargo flamegraph`).

## E. Code quality
- [ ] `cargo fmt`, `cargo clippy -- -D warnings`, `cargo test` all pass.
- [ ] No `unwrap()` on IO/parse paths; errors surface as toasts.
- [ ] No `unsafe` except inside the optional GL callback, with a comment.

## F. How to report
At the end print a table: *requirement → file:line → verified how*. Anything not implemented is listed under **NOT DONE**, never hidden.
