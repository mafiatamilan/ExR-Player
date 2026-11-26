# 05 — Keymapper Spec (with worked examples)

Behavior is taken from MEmu's documented modules (doc 01 §3). Visuals are ExR's own.

## 1. Modes
- **Play mode** (default): keymap is live, overlay hidden (or faint hints if "Show keys" is on: key caps at 35 % α).
- **Edit mode** (toolbar ⌨ or Ctrl+Shift+K): scrim `KEYMAP_SCRIM` over the viewport, control panel slides in from the right, key caps drawn at full opacity, input to the game is **suspended**.
- Leaving Edit with unsaved changes → confirm (Save / Discard / Continue editing).

## 2. Edit-mode layout
```
┌──────────────────────────────────────────────┬───────────────────────┐
│ top strip (36): Profile ▾ [Game name]  Undo Redo  Grid▢ Snap▢  Save  ✕│
├──────────────────────────────────────────────┼───────────────────────┤
│                                              │ CONTROLS (280 w panel)│
│   game frame, dimmed, key caps on top        │  Basic                │
│                                              │   [Click][Slide]      │
│        ⌨ A   ⌨ SPACE      ◎ WASD            │   [Joystick][Aim]     │
│                                              │   [Fire][Skill]       │
│                                              │  Advanced             │
│                                              │   [Free-Look][Repeat] │
│                                              │   [Cooperate][Record] │
│                                              │   [Macro][Zoom] ...   │
│                                              │ ───────────────────── │
│                                              │ INSPECTOR (selected)  │
└──────────────────────────────────────────────┴───────────────────────┘
```
- Panel 280 w, `BG_PANEL`, left hairline. Palette tiles **72 × 64** (**3 per row**, gap 8): icon 24, label 11.5 under. Tiles are **drag sources** (`ui.dnd_drag_source`), drop onto the viewport creates the control at the pointer.
- Inspector (below palette, 12 pad): shows only fields of the selected control (see §4). Empty state: "Select a control or drag one onto the game."
- Top strip: `Profile` ComboBox (per-game profiles), Undo/Redo (cmd stack ≥ 100), Grid/Snap toggles (grid 8 px), Save (primary), Close ✕.

## 3. Key cap visual (the atom — must look right)
- Default **36 × 36**, radius 8, fill `BG_ELEVATED` @ 85 % α, 1.5 px border `ACCENT_CYAN` @ 70 % α, label 12/600 `TEXT` centered (≤ 3 chars) — longer names use 10 pt and widen to fit (max 72 w): `Space`, `Ctrl`, `LMB`, `RMB`, `Wheel↑`, `F5`.
- **Hover**: border 100 %, tooltip with module name + key. **Selected**: 2 px border + outer glow (cyan α 0.35, blur 10) + 4 corner handles (6 × 6) for resize (where resizable).
- **Awaiting key** state (after double-click or after drop): border pulses between cyan and white once, label "…", next key press binds; **Esc cancels, Backspace unbinds**. Binding a key already used → previous owner turns `DANGER` border for 800 ms and is unbound (or confirm).
- Dragging a cap moves it; **center point** of the cap is the touch coordinate. Store coordinates **normalized 0..1 of the Android frame**, never pixels (resolution independent).
- A thin 1 px connector line from cap to its true touch point is drawn only if the cap is dragged away from the point (Slide/Skill: the path).

## 4. Modules — shape on screen, inspector fields, runtime semantics

| Module | On-screen shape | Inspector | Runtime |
|---|---|---|---|
| **Click** | 1 cap (36²) | key, hold-time ms (default 30), note | key down → touch down at p; key up → touch up |
| **Slide** | polyline trail (stroke 3, `ACCENT_CYAN`, arrow head at end) + cap at start | key, duration ms (default 180), 2–32 sampled points | key down → replay path |
| **Joystick (WASD)** | ring Ø **160** + inner knob Ø 48 + 4 caps (W top, A left, S bottom, D right) at radius 56 | 4 keys, radius px (norm), **sprint/walk toggle** (9.0.7), sprint key | held keys → vector → move knob; diagonals normalized |
| **MOBA move** | same ring, caps hidden, RMB glyph | button (RMB default) | RMB click/hold → move |
| **Crosshair (Aim)** | cap + crosshair glyph Ø 56 | toggle key (**default Ctrl**), sensitivity X/Y 0.1–5.0, invert Y | toggle shoot-mode: hide+lock cursor, mouse delta → camera drag |
| **Fire / Attack** | cap with bullet glyph (Ø 44) | key (default LMB), mode hold/toggle | active only in shoot mode |
| **Skill attack** | cap + range ring Ø 140 | key, range, quick-cast on/off, accuracy | down: show aim; mouse offset → finger offset; **up: cast** |
| **Free-Look** | cap with eye glyph | key, sensitivity | hold+mouse = camera; release restores |
| **Repeat Touches** | cap + stepper chip `[−] 12/s [+]` | rate 1–50 /s (typed or ± ), **mode**: hold / toggle | while active → taps at p at rate |
| **Cooperate** | cap + two small badges ▼ (press) ▲ (lift) each placeable at its own point | key, p_down, p_up | key down → tap p_down; key up → tap p_up |
| **Operation Recorder** | cap + ● record button | key, **script list** (name, duration, delete), ▶ record / ■ stop | replay stored touch events with timing |
| **Macro** | cap + ✎ pencil (position irrelevant) | key, **Edit Macro** drawer | run script (§7) |
| **Zoom** | cap pair `+` `−` | two keys, step | two-finger pinch around viewport center |
| **Expand Sight** | cap with ⇱ glyph | key, max offset | hold + mouse = pan |
| **Gravity sensor** | 4 caps in a cross | 4 keys W/A/S/D default, tilt magnitude° | emulate accelerometer tilt |
| **3D view** | cap | key, step | camera yaw/pitch step |
| **Fake GPS** | cap cross + step bar | 4 keys, step meters (drag bar) | move virtual location |

Hit-testing order: topmost (last created) first. Right-click a control → context menu: *Duplicate · Bring to front · Change key · Delete*. `Del` deletes. Arrow keys nudge 1 px (Shift = 8). `Ctrl+Z / Ctrl+Y`.

## 5. Data model (serde JSON, versioned)
```json
{
  "version": 1,
  "package": "com.example.shooter",
  "name": "Example Shooter — Default",
  "frame_aspect": [16, 9],
  "controls": [
    { "id": "c1", "kind": "joystick", "center": [0.14, 0.72], "radius": 0.11,
      "keys": { "up": "KeyW", "left": "KeyA", "down": "KeyS", "right": "KeyD" },
      "sprint": { "enabled": true, "key": "ShiftLeft", "multiplier": 1.0 } },
    { "id": "c2", "kind": "crosshair", "center": [0.50, 0.46], "toggle_key": "ControlLeft",
      "sens": [1.0, 1.0], "invert_y": false },
    { "id": "c3", "kind": "fire", "center": [0.88, 0.70], "key": "MouseLeft", "mode": "hold" },
    { "id": "c4", "kind": "click", "center": [0.92, 0.45], "key": "Space", "hold_ms": 30, "note": "Jump" },
    { "id": "c5", "kind": "repeat", "center": [0.80, 0.30], "key": "KeyF", "rate_hz": 12, "mode": "hold" },
    { "id": "c6", "kind": "slide", "path": [[0.30,0.50],[0.45,0.50],[0.60,0.50]], "key": "KeyQ", "duration_ms": 180 },
    { "id": "c7", "kind": "macro", "key": "Enter", "script": "touch 0.9 0.4\ndelay 100\ntouch 0.9 0.5" }
  ]
}
```
Rust: `enum Control` with `#[serde(tag = "kind", rename_all = "snake_case")]`. Keys stored as **egui `Key` names / physical codes**, never localized labels. Profiles live in `%APPDATA%/ExR Player/keymaps/<package>.json`; atomic write (temp + rename).

## 6. Worked examples (what the agent must be able to reproduce)

**Ex. A — Click key (MEmu doc example "Alt")**
1. Open editor → drag **Click** tile onto a game button (or left-click the screen). 2. Cap appears awaiting key; press `Alt`. 3. Close editor → pressing Alt taps that button.

**Ex. B — Slide (MEmu doc example "A")**
1. Choose **Slide**, hold LMB on screen and drag left→right: trail with arrow follows the mouse. 2. Release → cap awaiting key; press `A`. 3. Play mode: `A` replays the swipe left→right over 180 ms.

**Ex. C — WASD**
1. Drag **Joystick** onto the left on-screen stick. 2. Click the "up" quadrant → it highlights → press `I` → rebinds up to I. 3. Save. Holding the keys moves the knob; W+D = diagonal at 0.707 length.

**Ex. D — Aim & shoot**
1. Drag **Crosshair** onto the game's crosshair (default `Ctrl`). 2. Drag **Fire** onto the fire button. 3. Play: Ctrl enters shoot mode (cursor hidden/locked, mouse moves camera); LMB fires; Ctrl again exits.

**Ex. E — Repeat touches**
Drag **Repeat** onto a button; tile chip shows `12/s`; click `+` → 13/s (min 1, max 50); choose mode *toggle*; pressing F starts auto-tap, F again stops.

**Ex. F — Cooperate**
After drop, ▼ and ▲ badges appear (initially same point). Drag ▲ to a second button. Key down taps ▼-point, key up taps ▲-point.

**Ex. G — Macro**
1. Drag **Macro** anywhere; bind `Enter`. 2. Click ✎ → right drawer **Edit Macro** (320 w): top **Edit area** (monospace 12, line numbers, 8-line min), bottom **Command area** (list of commands, each with **+** that appends its template line to the Edit area), header **Guide** button opens a read-only command reference. 3. Lines can be typed freely. 4. Save validates syntax; errors underline the line in `DANGER` with message.

## 7. ExR macro grammar (our own; MEmu's table is an unreadable image)
One command per line, `#` starts a comment (MEmu added a `Comment` command in 9.0.6 → we map it to `#`).
```
touch X Y [hold_ms]       # tap at normalized point
swipe X1 Y1 X2 Y2 [ms]
delay MS
keydown KEY / keyup KEY / keypress KEY
keymove DX DY             # relative mouse/finger move
input "text"              # type text
mapkey KEY                # temporarily trigger another mapped key
loop [N] ... loopstop     # N omitted = until key released/toggled
onkeydown ... / onkeyup ...   # blocks: run on press / on release
entershoot / exitshoot
```
Parser: hand-written, line-based, returns `Vec<Op>` or `Vec<LineError{line, col, msg}>`. Executed on a **separate thread/async task** with cancellation token; never block the UI thread.

## 8. Input capture rules
- In Edit mode we read `ctx.input(|i| i.events)` for `Event::Key` to bind; mouse buttons bind via `PointerButton`; wheel via `MouseWheel`.
- Modifier-only binds (Ctrl alone) are valid (Crosshair default).
- Reserved (cannot bind): `Esc` (cancel), `F11` (fullscreen), `Ctrl+Shift+K`.
