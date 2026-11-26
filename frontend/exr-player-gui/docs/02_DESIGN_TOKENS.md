# 02 — Design Tokens (ExR Player)

All sizes are **egui points at 100 % scale** (1 pt = 1 logical px). **[SPEC]** — see doc 01 note.
Colors marked ◆ were **sampled from the supplied logo PNG**; the rest are derived to sit on top of it.

## 1. Color

| Token | Hex | Use |
|---|---|---|
| `BG_BASE` ◆ | `#0D111A` | window background (matches logo canvas exactly) |
| `BG_PANEL` | `#121826` | title bar, toolbar, dialog body |
| `BG_ELEVATED` | `#181F31` | inputs, cards, popups |
| `BG_HOVER` | `#1F2840` | hovered buttons / rows |
| `BG_PRESSED` | `#0A0E16` | pressed |
| `BORDER` | `#262F45` | 1 px hairlines, dividers |
| `TEXT` ◆ | `#F7F9FC` | primary text (logo wordmark is #FDFEFF) |
| `TEXT_DIM` | `#8B93A1` | secondary / hints / subtitle |
| `TEXT_DISABLED` | `#4A5365` | disabled |
| `ACCENT_CYAN` ◆ | `#01E7FF` | primary accent, focus rings, selected tab bar |
| `ACCENT_BLUE` ◆ | `#16A6F5` | gradient end, links |
| `ACCENT_GREEN` ◆ | `#01E8A0` | success, "recording", toggles ON end of gradient |
| `DANGER` | `#FF4D5E` | close-hover, destructive, rec dot |
| `WARN` | `#FFB020` | "restart required" banner |
| `SCRIM` | `#000000` @ 55 % | modal backdrop |
| `KEYMAP_SCRIM` | `#000000` @ 35 % | keymapper overlay over the game |

Brand gradient (only for: logo, primary button fill, active toggle track, progress): left-top `#01E7FF` → right-bottom `#01E8A0`
(logo goes cyan→blue on the E and cyan→green on the R). Never use the gradient on body text or large panels.

## 2. Geometry

| Element | Size |
|---|---|
| Title bar height | **32** |
| Window-control buttons (min/max/close) | **46 × 32**, glyph 10 × 10, stroke 1.0 |
| Title-bar logo glyph | **20 h × 33 w** (aspect 452:270 = 1.674) — file `titlebar_glyph_20h.png`; render from a 40 h @2x texture for HiDPI |
| Right toolbar width | **44** |
| Toolbar button | hit area **36 × 36**, glyph **20 × 20**, gap **6**, outer padding 4, radius **8** |
| Toolbar group divider | 1 px × 20 w, `BORDER`, margin 6 |
| Nav strip height | **36**; buttons 40 × 28, glyph 16 |
| Window min size | 960 × 600; default 1280 × 760 (incl. chrome) |
| Corner radius | window 10 (if undecorated+transparent), dialog 12, button 8, input 6, chip 999 |
| Hairline | 1.0 |
| Spacing scale | 2 · 4 · 6 · 8 · 12 · 16 · 20 · 24 (nothing else) |
| Tooltip delay | 400 ms; tooltip radius 6, bg `BG_ELEVATED` |

## 3. Typography

- Family: **Inter** (SIL OFL) — Regular 400 + SemiBold 600 embedded via `include_bytes!` (the wordmark in the logo is a geometric grotesque; Inter is the closest free match). Fallback: egui default.
- Sizes (pt): body **13**, small **11.5**, button **13 / 600**, dialog title **16 / 600**, section header **11 / 600 UPPERCASE letter-spacing +6 %** (`TEXT_DIM`), key-cap label **12 / 600**, key-cap label for long names **10**.
- Wordmark "ExR PLAYER" in the title bar: **13 / 600**, `TEXT`, 8 px right of glyph, ALL CAPS "PLAYER".

## 4. App icon set (generated in `assets/logo/`)

| File | Pixels | Where used |
|---|---|---|
| `icon_16.png` | 16² | taskbar small, tray, tab favicon |
| `icon_24.png` | 24² | title-bar fallback |
| `icon_32.png` | 32² | taskbar, Alt-Tab |
| `icon_48.png` | 48² | desktop shortcut |
| `icon_64.png` / `icon_128.png` | 64² / 128² | About dialog, multi-instance cards |
| `icon_256.png` / `icon_512.png` | 256² / 512² | installer, store |
| `exr.ico` | 16–256 multi-res | Windows exe resource (`winres`) + `ViewportBuilder::with_icon` (use 256 RGBA) |
| `titlebar_glyph_20h.png` | 33 × 20 | title bar (load @2x from `exr_glyph.png` for sharpness) |
| `exr_glyph.png` | 452 × 270 | master raster glyph |

> The glyph PNGs were **keyed out of the supplied raster** (rough alpha). Replace with a hand-traced **SVG** when available; render
> SVG→texture at needed sizes with `resvg` at startup and cache. Glyph needs ≥ 10 % padding inside square icons (already applied).

## 5. Toolbar / UI icons — draw our own

MEmu's icon art is theirs. ExR draws a **single consistent icon set**: 24-unit grid, 1.75 stroke, round caps/joins, no fills except
the record dot. Options (pick ONE, do not mix): (a) embed **Lucide** SVGs (ISC license) and rasterize with `resvg` at 20 px @1x/@2x/@3x
into one atlas at startup; (b) paint with `egui::Painter` line segments (zero textures, perfectly crisp, a bit more code).
Default color `TEXT_DIM`, hover `TEXT`, active `ACCENT_CYAN`.

Suggested Lucide names: `package-plus` (APK), `keyboard`/`gamepad-2` (keymap), `layers` (multi-instance), `folder-sync` (shared folder),
`camera` (screenshot), `video` (record), `map-pin` (GPS), `volume-2`/`volume-1` (vol), `rotate-ccw` (rotate), `vibrate` (shake),
`maximize` (fullscreen), `settings` (gear), `menu` (hamburger), `arrow-left`/`circle`/`square` (back/home/recents),
`minus`/`square`/`x` (window controls — paint these manually).

## 6. Motion
- Hover color fade 120 ms (`ctx.animate_bool_with_time`). Dialog open: 140 ms fade + 6 px rise. Toggle knob 120 ms.
- Nothing animates continuously. Idle = zero repaints (doc 06).
