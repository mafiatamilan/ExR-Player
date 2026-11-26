# ExR Player — GUI spec pack + egui skeleton

**What this is:** a research-backed spec (MEmu-Play-style UI, built for the supplied ExR logo) plus a Rust/egui starter, packaged so a coding agent
builds the real thing instead of "slop" (fake boxes, placeholder text, invented layouts).

## Start here
1. Open `docs/00_AGENT_PROMPT.md` → paste into your coding agent (Claude Code / Cursor / etc.), and let it read the rest of `docs/`.
2. Put **Inter-Regular.ttf** and **Inter-SemiBold.ttf** (SIL OFL) in `assets/fonts/` and uncomment the lines in `src/theme.rs`.
3. `cargo check` — see the caveat below, then follow the agent's deliverable order.

## Contents
| Path | What |
|---|---|
| `docs/01_RESEARCH_MEMU_UI.md` | Sourced findings; every fact tagged SRC / COMMUNITY / SPEC |
| `docs/02_DESIGN_TOKENS.md` | Colors (sampled from your logo), sizes, fonts, icon sizes |
| `docs/03_MAIN_WINDOW_SPEC.md` | Title bar, toolbar, viewport, nav strip |
| `docs/04_SETTINGS_DIALOG_SPEC.md` | Settings popup: size, nav, rows, every field/validation |
| `docs/05_KEYMAPPER_SPEC.md` | 17 documented modules, key-cap atom, JSON schema, worked examples, macro grammar |
| `docs/06_PERFORMANCE_EGUI.md` | Repaint discipline, texture path, build profile, module layout |
| `docs/07_ANTI_SLOP_CHECKLIST.md` | Acceptance gate |
| `assets/logo/` | Reference image, keyed glyph, `icon_16…512.png`, `exr.ico`, title-bar glyph |
| `src/`, `Cargo.toml`, `build.rs` | Starter: theme, backend trait + mock, title bar, toolbar, nav, viewport, toasts, settings shell/General page, keymap model, macro parser, validators (+ unit tests) |

## Honest caveats
- **The Rust skeleton was NOT compiled** (no Rust toolchain in the authoring sandbox). It targets egui/eframe **0.37.0**; expect small API fixes
  (`cargo check`). The prompt tells the agent to do this first.
- **MEmu publishes no pixel specs.** All pixel sizes are chosen design values (tagged SPEC). Colors marked ◆ were sampled from your logo.
  If you have real MEmu screenshots, measure them and edit only `src/theme.rs`.
- Macro grammar is **our own** (MEmu's command table is an image); the macro *concept* and editor layout follow MEmu.
- Logo glyph PNGs were keyed from your raster (rough alpha). Replace with a traced SVG for production.
- Icons are painter-drawn placeholders for the shell; toolbar items still to add are marked `TODO(agent)` and listed in doc 03.
- MEmu artwork/trademarks are not reused; this is an independent UI inspired by its documented layout and features.
