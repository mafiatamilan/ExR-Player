# 03 — Main Window Spec

```
┌──────────────────────────────────────────────────────────────────┐ y=0
│ [glyph 33x20] ExR PLAYER   (app tabs, optional)   ≡  –  ☐  ✕     │ 32  title bar (draggable)
├───────────────────────────────────────────────────┬──────────────┤
│                                                   │ ▣ APK        │
│                                                   │ ⌨ Keymap     │ 44 wide
│            ANDROID VIEWPORT                       │ ❏ Multi      │ right toolbar
│       (aspect-locked, letterboxed #000)           │ 📁 Shared    │ (vertical,
│                                                   │ ─────        │  scrolls if
│                                                   │ 📷 Shot      │  window short)
│                                                   │ ● Record     │
│                                                   │ ⌖ GPS        │
│                                                   │ 🔊 Vol+ Vol− │
│                                                   │ ⟳ Rotate     │
│                                                   │ ⤢ Fullscreen │
│                                                   │ ⚙ Settings   │ pinned bottom
├───────────────────────────────────────────────────┴──────────────┤
│                      ◁ Back     ○ Home     □ Recents             │ 36  nav strip
└──────────────────────────────────────────────────────────────────┘
```

## 1. Title bar (custom, `with_decorations(false)`)
- Left→right: 10 px pad · logo glyph (20 h) · 8 px · "ExR **PLAYER**" 13/600 · 16 px · **app tabs** (optional, §1.1) · flexible spacer = **drag region**
  (`ViewportCommand::StartDrag` on `drag_started`; double-click toggles maximize) · **Menu ≡** (46×32) · Min · Max/Restore · Close.
- Close hover: bg `DANGER`, glyph white. Others hover `BG_HOVER`.
- Menu ≡ opens a popup (width 220, radius 10, `BG_ELEVATED`, shadow blur 24 α 0.45): *New instance… · Multi-Instance Manager · Install APK… ·
  Screenshot · Settings · Check updates · About ExR Player · Exit*. Row height 32, shortcut text right-aligned `TEXT_DIM`.
- Resizing an undecorated window: 6 px invisible edge/corner grips → `ViewportCommand::BeginResize(dir)` (verify enum in pinned egui version).
- 1.1 App tabs: height 28, tab min 96 / max 180 wide, radius 6 top, active = `BG_ELEVATED` + 2 px `ACCENT_CYAN` underline, close × appears on hover. Hidden when setting "App tabs" is off.

## 2. Viewport
- Renders the emulator frame via `FrameSource` trait (doc 06 §3). Maintain Android aspect ratio; letterbox with `#000000`.
- Pointer/keyboard events over the viewport are forwarded to the backend as touch/keys **only when keymap editor is closed**.
- States: *Booting* (centered 128 px logo + 3-dot loader + status text from backend), *Running*, *Crashed* (message + Restart button), *No instance*.
- Cursor: normal arrow; crosshair/shoot mode (keymapper) hides OS cursor and locks it to the viewport center (`ViewportCommand::CursorGrab`).

## 3. Right toolbar (order top→bottom, groups separated by dividers)

| Group | Item | Tooltip (400 ms) | Action | Shortcut default |
|---|---|---|---|---|
| A | APK | "Install APK  Ctrl+Shift+I" | file dialog (`rfd`) → backend.install_apk | Ctrl+Shift+I |
| A | Keymapping | "Key mapping  Ctrl+Shift+K" | toggle keymap editor | Ctrl+Shift+K |
| A | Multi-instance | "Multi-Instance Manager" | opens manager window | Ctrl+Shift+M |
| A | Shared folder | "Shared folder" | opens host folder | — |
| B | Screenshot | "Screenshot  Ctrl+Shift+S" | save PNG to Pictures/ExR | Ctrl+Shift+S |
| B | Record | "Record screen" | toggle; shows red dot + mm:ss while on | Ctrl+Shift+R |
| B | Location (GPS) | "Set location" | popup w/ lat/lon fields + presets | — |
| C | Vol + / Vol − | "Volume up/down" | backend.key(VolumeUp/Down) | F-keys optional |
| C | Rotate | "Rotate screen" | backend.rotate | — |
| C | Shake | "Shake" | backend.sensor_shake | — |
| C | Fullscreen | "Full screen  F11" | toggle | F11 |
| pinned bottom | Settings | "Settings" | opens Settings dialog (doc 04) | Ctrl+, |

(Items in A, B, record/GPS, multi-instance, shared folder, settings are **[SRC]**; vol/rotate/shake/fullscreen are common-knowledge extras.)

Toolbar button states: idle glyph `TEXT_DIM` · hover bg `BG_HOVER` + glyph `TEXT` · pressed `BG_PRESSED` · active (keymap open, recording)
glyph `ACCENT_CYAN` (record = `DANGER`) with a 2 × 16 px left indicator bar. Focus ring 1.5 px `ACCENT_CYAN` for keyboard nav. Disabled when no instance is running (50 % α).
Short window: toolbar scrolls vertically (`ScrollArea::vertical().auto_shrink`) with Settings pinned.

## 4. Nav strip
Three centered buttons 40 × 28, 24 gap: Back (◁), Home (○), Recents (□). Position is configurable (Settings → General → Nav bar position: *Bottom / Right toolbar / Hidden*).

## 5. Toasts
Bottom-center, above nav strip, 280 × 40, radius 10, `BG_ELEVATED`, 3 s, max 3 stacked. e.g. "Screenshot saved — Open folder".

## 6. Window persistence
Save position/size/maximized + toolbar state on exit to `%APPDATA%/ExR Player/ui.json` (use `eframe` persistence or `directories` + `serde_json`).
