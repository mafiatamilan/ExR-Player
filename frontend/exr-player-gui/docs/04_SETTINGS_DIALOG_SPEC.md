# 04 — Settings Dialog Spec

MEmu's settings are organised in pages with **Save → restart** semantics (doc 01 §2). ExR reproduces that behaviour with its own look.

## 1. Shell
- **Modal** over the main window; `SCRIM` backdrop (click on scrim does NOT close if there are unsaved changes → shows "Discard changes?" confirm).
- Size **760 × 540** (min 640 × 460; if the window is smaller, shrink to window − 48). Centered. Radius 12, `BG_PANEL`, 1 px `BORDER`, shadow blur 40 α 0.55.
- Header 52 h: title "Settings" 16/600 left, close ✕ (36 × 36) right, bottom hairline.
- Left nav **176 w**, `BG_BASE`, items **36 h**, 10 px side pad, radius 8, icon 16 + 10 gap + label 13. Active: bg `BG_ELEVATED`, 3 × 16 px `ACCENT_CYAN` bar at left, label `TEXT`; idle label `TEXT_DIM`.
- Content area: padding 20, vertical `ScrollArea`, sections separated by 24.
- Footer 56 h, hairline on top: left = warning chip (when needed) "⟳ Restart required to apply"; right = **Cancel** (ghost, 88 w) · **Save** (secondary, 88 w) · **Save & Restart** (primary gradient, 136 w). Buttons 32 h, radius 8. Save disabled until dirty.
- Esc = Cancel, Enter = Save (when focus not in a text field).

## 2. Row anatomy (the thing agents get wrong)
Every setting is a **row 40 h min**: label column **200 w** (13/400 `TEXT`, optional 11.5 `TEXT_DIM` hint under it → row grows to 56) · control column fills the rest, left-aligned.
Rows are separated by 1 px `BORDER` at 40 % α, **not** boxed. Section header (11/600 uppercase `TEXT_DIM`) sits 24 above its rows. **Never wrap each row in a card/box.**

## 3. Pages and exact fields

### 3.1 General  [SRC]
| Field | Control | Details |
|---|---|---|
| Language | ComboBox 220 w | English, 简体中文, 繁體中文, Español, Português, Français, Deutsch, Русский, 日本語, 한국어, हिन्दी, தமிழ் (list is data-driven) — *restart* |
| Resolution | ComboBox 220 w | presets: `1280 × 720 · 240 dpi`, `1920 × 1080 · 320 dpi`, `2560 × 1440 · 320 dpi`, `Custom…` — *restart* |
| Custom resolution | two DragValue 88 w (W, H) + "×" + DPI DragValue 72 w | visible only when Custom; W 320–3840, H 240–2160, DPI 120–640; invalid → red border + hint |
| CPU cores | Slider + DragValue | 1 … host logical cores; if VT disabled: slider capped at 1 and hint "Enable virtualization (VT-x/AMD-V) in BIOS to use more cores" (WARN color) — *restart* |
| Memory (MB) | Slider + DragValue | 512 … min(host_free − 2048, 16384), step 256 — *restart* |
| Nav bar position | SegmentedControl | Bottom · Right toolbar · Hidden |
| App tabs | Toggle | on/off |
| Start with Windows | Toggle | — |

### 3.2 Engine & Display  [COMMUNITY + changelog]
| Field | Control |
|---|---|
| Performance preset | SegmentedControl: Low · Medium · High · Top (Top disabled w/ tooltip unless 64-bit image) |
| Graphics renderer | ComboBox: Auto · OpenGL · DirectX · Vulkan (list comes from backend capabilities) |
| FPS limit | ComboBox: 30 · 60 · 90 · 120 · Unlimited |
| GPU memory optimization | Toggle |
| Anti-flicker | Toggle |
| Enable screen capture | Toggle |
| Microphone | Toggle |
| Disk sharing | ComboBox: Shared system disk · **Independent system disk** (hint: required for root) |

### 3.3 Advanced  [SRC]
| Field | Control |
|---|---|
| Phone brand / model | two ComboBoxes (brand → model list), 180 w each |
| Phone number | TextEdit 180 w, regex `^\+?[0-9]{0,15}$` |
| IMEI | TextEdit 180 w + **Randomize** button; Luhn-validated 15 digits |
| Sensors | Toggle |
| Root mode | Toggle (+ WARN hint: "Some apps refuse to run on rooted devices") |
| Map location | lat/lon DragValues + "Pick on map…" disabled stub |
| Network | SegmentedControl: NAT · Bridge ; Proxy: Toggle + host/port fields |

### 3.4 Shortcuts  [SRC constraint]
Table: columns *Action (240)* | *Shortcut (160)* | *Reset (32)*. Row 36 h. Click shortcut cell → "Press keys…" capture state (cell border `ACCENT_CYAN`, Esc cancels, Backspace clears).
**Validation: only `F1–F12` or any combination containing Ctrl/Shift/Alt + a key is accepted** (MEmu 9.2.5 rule). Plain letters rejected with inline red message.
Duplicate detection: conflicting row flashes `DANGER` border; Save disabled while conflicts exist.
Actions: Screenshot, Record, Full screen, Open keymapper, Show/Hide keys overlay, Volume up/down, Boss key (hide window), Install APK, Rotate.

### 3.5 About
128² icon, "ExR Player", version string, "Check for updates" button, license/credits link.

## 4. Controls to implement (reusable, in `ui/widgets.rs`)
`toggle(ui, &mut bool)` 36 × 20 track, 16 knob, off `BG_ELEVATED`+`BORDER`, on gradient; `segmented(ui, &mut idx, &[labels])` height 28, radius 8;
`combo` styled through `ui.style_mut()`; `setting_row(ui, label, hint, |ui| {...})`; `primary_button`, `ghost_button`; `section_header`.

## 5. State model
```rust
struct SettingsState { draft: Settings, saved: Settings, page: Page, restart_needed: bool, errors: Vec<FieldError> }
```
`dirty = draft != saved`. Fields tagged *restart* flip `restart_needed` when changed. Save writes JSON + calls `backend.apply()`; Save & Restart additionally `backend.restart()`.
Validation is pure functions in `settings/validate.rs` with unit tests (IMEI Luhn, resolution bounds, shortcut rules).
