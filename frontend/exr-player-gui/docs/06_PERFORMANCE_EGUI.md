# 06 — Performance & egui Architecture

Target: **cold start < 300 ms to first frame, idle CPU ≈ 0 %, idle RAM < 60 MB (UI only), < 1 ms UI CPU time per frame while streaming.**

## 1. Cargo & build
```toml
[profile.release]
opt-level = 3
lto = "fat"
codegen-units = 1
panic = "abort"
strip = true
[profile.dev.package."*"]
opt-level = 2            # fast debug UI
```
- `eframe` with `default-features = false`, features `["glow", "default_fonts"?, "persistence"]`. Prefer **glow** (OpenGL) for startup time and low overhead on emulator machines; offer `wgpu` as a cargo feature only if Vulkan/DX12 is needed.
- Pin versions: **eframe/egui 0.37.0** (current as of 2026-11; MSRV Rust 1.95). **Run `cargo check` first** — egui APIs change between minor versions (e.g. `Rounding`→`CornerRadius`, `Frame::none()`→`Frame::new()`); fix against docs.rs for the pinned version rather than guessing.
- Windows: `#![windows_subsystem = "windows"]` in release; `winres` for icon (`assets/logo/exr.ico`) + manifest (per-monitor DPI v2).

## 2. Repaint discipline (the #1 optimisation)
- egui is immediate-mode: **repaints only on input or when asked**. Never call `ctx.request_repaint()` unconditionally.
- New emulator frame arrives → backend thread calls `ctx.request_repaint()` (cheap, thread-safe). Frame cap = backend FPS.
- Animations: `ctx.request_repaint_after(Duration::from_millis(16))` only while an animation is running.
- Recording timer text updates once per second via `request_repaint_after(1 s)`.
- Window hidden/minimized → stop requesting repaints.

## 3. Emulator frame path
```rust
pub trait FrameSource: Send {
    fn latest(&mut self) -> Option<FrameRef<'_>>;   // RGBA8 w,h,stride, or a GL texture id for zero-copy
}
```
- Keep **one** `TextureHandle`; update with `tex.set(ColorImage, TextureOptions::LINEAR)` only when a *new* frame id arrived. Do not recreate the texture per frame; do not clone the `Vec<u8>` more than once.
- Zero-copy upgrade path: `egui::PaintCallback` with `egui_glow::CallbackFn` drawing a backend-owned GL texture into the viewport rect (skips CPU upload entirely).
- Draw with `painter.image(tex_id, rect, UV_FULL, Color32::WHITE)` using an aspect-fitted `rect`.

## 4. UI-side rules
1. **Cache everything static**: icon textures loaded once into `Icons { apk: TextureHandle, … }`; fonts set once; `Visuals`/`Style` set once in `new()` (not per frame).
2. **No per-frame allocations in hot paths**: no `format!`/`String` building for static labels (use `&'static str`); reuse `Vec` buffers; `Galley`s for key-cap labels cached in a `HashMap<(KeyId, size), Arc<Galley>>`, invalidated on change.
3. **Custom-paint the atoms** (toolbar buttons, key caps, toggles) with `Painter` + `ui.allocate_exact_size` + `ui.interact` — avoids heavy widget layout; one `Shape::rect_filled` + one icon `Shape::image`.
4. Don't nest more than ~4 layout levels in the toolbar; use fixed-size `allocate_exact_size` columns.
5. `ScrollArea` only where content can overflow; set `auto_shrink([false;2])`.
6. Keymap overlay: iterate controls once, push shapes into `ui.painter()`; hit-testing on a `Vec<Control>` with AABB pre-check; thousands are not expected — don't over-engineer a quadtree.
7. Heavy work (APK install, file IO, macro execution, screenshots PNG encode) → worker thread / `std::thread::spawn` + channel; UI polls `try_recv()` each frame.
8. Persist with debounce: write JSON at most once per 500 ms after last change, atomic rename.
9. HiDPI: use `ctx.pixels_per_point()`; rasterize SVG icons at `ceil(20 * ppp)`; reload atlas when ppp changes.
10. Undecorated window shadow: draw own 1 px border; skip big blurred shadows on the main window (expensive); only dialogs/popups get `Shadow`.

## 5. Module layout
```
src/
  main.rs            window options, icon, run_native
  app.rs             ExrApp: update() dispatch, state machine
  theme.rs           ALL constants + Visuals + fonts   <-- single source of numbers
  icons.rs           icon atlas load/paint
  backend.rs         trait EmulatorBackend + MockBackend
  ui/
    mod.rs, titlebar.rs, toolbar.rs, navstrip.rs, viewport.rs, toast.rs, widgets.rs
    settings.rs      dialog + pages
  keymap/
    mod.rs, model.rs (serde), editor.rs (egui), runtime.rs (input→touch), macro_lang.rs (parser)
  settings/ model.rs, validate.rs
```
The UI talks to the emulator **only** through `trait EmulatorBackend` (+ `MockBackend` that renders an animated gradient test pattern) so the GUI is testable with no VM.

## 6. Tests the agent must add
`validate.rs` (IMEI, resolution, shortcut rule), `macro_lang.rs` (parse/err positions), `model.rs` (serde round-trip of the §5 JSON), `runtime.rs` (WASD vector normalisation, shoot-mode toggle).
