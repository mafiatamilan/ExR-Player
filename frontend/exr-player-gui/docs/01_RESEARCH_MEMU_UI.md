# 01 — MEmu Play UI Research (what is KNOWN vs. what is SPEC'D)

Every statement below is tagged:

- **[SRC]**  = stated by an official MEmu page / changelog (URL in section 9).
- **[COMMUNITY]** = from user guides / forum posts, consistent across several posters.
- **[SPEC]** = MEmu does not publish this; it is a *design decision made for ExR Player*. Do not claim it is pixel-measured from MEmu.

> Honest limitation: MEmu publishes **no pixel specs** (icon sizes, paddings). All pixel numbers in
> docs 02–05 are **[SPEC]** values chosen to feel like MEmu 9 on a 1080p / 100 % DPI display.
> If you have a real MEmu screenshot, measure it and override `src/theme.rs` constants — that file
> is the single source of truth for every number.

## 1. Window anatomy (MEmu 9.x)

| Region | Contents | Tag |
|---|---|---|
| Title / top bar | App logo + title, a **Menu** (hamburger) button, **Minimize / Maximize / Close** | [SRC] |
| Optional app tabs | Browser-like tabs for running apps; can be toggled on/off in settings (added 7.0) | [SRC] changelog 7.0.1 |
| Android viewport | The emulated phone/tablet screen. Default Android home = launcher with icons, folders, Play Store, Settings, File Manager | [SRC] |
| Right-side toolbar (vertical) | APK Install, Keymapping, Multi-Instance Manager, Shared Folder, Screen Capture, Screen Recording, Location (GPS) | [SRC] |
| Extra toolbar items | Volume up / down (shortcuts added 7.0), Rotate/Screen orientation, Shake, Full-screen, Settings (gear "on the side bar") | [COMMUNITY]/[SRC manual: "Settings button on the side bar"] |
| Android nav strip | **Back, Home, Recent Apps**. Nav-bar *location* is user-configurable (settings) | [SRC] |

Layout used by ExR Player (see doc 03): `title bar (top) → [viewport | right toolbar] → nav strip (bottom)`.

## 2. Settings (documented fields)

From the official manual + blog + 7.0.1 / 9.x changelogs:

**General page [SRC]** – language (English / Simplified / Traditional Chinese in the old manual; many more in 9.x),
resolution (drop-down with 3 common presets **plus** a custom width/height box), CPU core count, memory size.
CPU cores can only be raised if hardware virtualization (VT-x / AMD-V) is enabled. Values may not exceed the host.
Changing most values requires **Save + restart**.

**Advanced page [SRC]** – phone brand & model, phone number & IMEI, Sensor checkbox, **Root mode** checkbox, map location.

**Engine / Display / Network [COMMUNITY]** (Multi-MEmu instance settings) – Engine preset (Low / Medium / High / Top;
"Top" only meaningful on 64-bit Android), Custom resolution + **DPI** (e.g. 2560×1440 @ 320 dpi), microphone enable/disable,
disk sharing (shared vs *independent system disk* — needed for root), network bridge / proxy.

**Features added over versions [SRC changelog]**
- 7.0.1: GPU-memory optimization toggle, anti-flicker toggle, screen-capture toggle, app-tabs toggle, auto-start with Windows, volume up/down shortcuts, nav-bar location.
- 9.2.5: **in the settings Shortcuts page, shortcuts can only be F1–F12 or combos with Ctrl / Shift / Alt.** (important constraint for our Shortcuts tab)
- 9.0.x/9.1: keymapping "key description mode", user-editable key notes, WASD page gains **sprint/walk** toggle, macro "Comment" command, operation-recorder script list, controller (gamepad) mode optimizations.
- Beginner tip from MEmu: enable hardware virtualization (VT) and configure keymapping.

## 3. Keymapper (documented modules, 100 % [SRC] — "Key-Mapping teaching" + macro article)

Workflow: click **Keyboard/Keymapping** in the sidebar → editor mode opens over the viewport with a **panel of controls**.
Controls are **dragged from the panel onto the touch target** in the game (or created by clicking/dragging directly on the screen).
Each placed control shows a **key cap** (default or user-assigned key). Exit mapping mode → the keys become live.
Save is explicit.

| # | Module | Behaviour (exactly as documented) |
|---|---|---|
| 1 | **Click** | Most common. Key cap on a screen point; pressing the key taps that point. Created by left-click on screen or drag from panel. Custom key (example in docs: `Alt`). |
| 2 | **Slide** | Hold left mouse and drag → a **track/trail** is drawn; on release you assign a key. Pressing it replays the swipe (example: `A` slides left→right). Used for runner / Fruit-Ninja style games. |
| 3 | **WASD / Joystick** | Drag the Joystick control over the game's virtual stick (left half of screen). Click each of the **4 directions**; when highlighted, press a key to rebind (default W A S D). 9.0.7+: sprint/walk toggle. |
| 4 | **MOBA move** | Same joystick control; right mouse click / long-press moves the character. |
| 5 | **Crosshair (Aim)** | Drag onto the game's crosshair. **Default key `Ctrl`**. Press → enter *camera/shoot mode*, mouse moves the view; press again → leave. |
| 6 | **Attack / Fire** | Drag onto the game's fire button. After entering shoot mode, left-click / hold fires. |
| 7 | **Skill attack** | For MOBA skills. Press key + move mouse to aim the skill; **release key to cast**. Has accuracy / feature options. |
| 8 | **Free-Look** | Drag to the eye icon. Hold key + move mouse to look around; releasing restores the view. |
| 9 | **Repeat Touches** | Auto-clicker on a point. Left value = **taps per second** with `+` / `−` buttons; a selector on the right chooses the repeat *mode*. (9.0.9.3: number of triggers editable via keyboard.) |
| 10 | **Cooperate** | After drag, **"press" and "lift"** icons appear; one keyboard press produces **two** screen taps (on press and on release). |
| 11 | **Operation Recorder** | Drag anywhere → edit → click ▶ triangle to start recording on-screen actions → ■ stop (top-right) → Save. Triggering the key replays. 9.0.7 adds a **script list**. |
| 12 | **Macro** | Drag anywhere (position irrelevant). Assign a trigger key, click the **pencil** → **Edit Macro** panel on the right with an **Edit area** and a **Command area**; clicking **+** on a command appends its script line to the Edit area; lines can be hand-edited; a **Guide** button shows command docs. |
| 13 | **Zoom** | `+` = two-finger pinch-out, `−` = pinch-in on the game area. |
| 14 | **Expand Sight** | Hold key + move mouse to peek beyond the original view (MOBA). |
| 15 | **Gravity sensor** | Simulates 4-direction tilt: W/A/S/D = forward/left/back/right tilt. |
| 16 | **3D view** | Key quickly pans the in-game camera angle. |
| 17 | **Fake GPS** | W/A/S/D = walk N/W/S/E; drag the edge to choose the step distance (Pokémon-GO style). |

**Macro command vocabulary [SRC, names only]:** `loop` / `loopstop`, `onkeydown` / `onkeyup`, `input`,
`keydown` / `keyup` / `keymove`, `touch`, `swipe`, `keypress`, `mapkey`, `entershoot` / `exitshoot`, plus delay and (9.0.6+) `Comment`.
The exact grammar lives in two table *images* on MEmu's site that cannot be machine-read, so **ExR Player defines its own grammar
(doc 05 §7)**. Do not pretend it is byte-compatible with MEmu.

## 4. Other facts the UI must respect

- Operation is "like standard Windows applications" for min/max/close **[SRC]**.
- Android nav buttons behave like a real device **[SRC]**.
- Settings apply on **Save & restart** (instance restart) **[SRC]**; our dialog must show a "restart required" state, not silently apply.
- Multi-instance manager exists as a separate window ("Multi-MEmu") with per-instance settings, groups, randomize, batch settings **[SRC 7.0.1]**.

## 5. Things NOT verifiable (do not invent)

- Exact pixel dimensions, font, and colors of MEmu's chrome.
- Exact icon artwork (it is MEmu's trademarked art — ExR Player must draw **its own** icons, see doc 02 §5).
- Macro grammar details.
- Anything in the 2026 GitHub "patch pack" pages found in search — that content is unverified marketing text, ignored.

## 9. Sources
- https://www.memuplay.com/blog/memu-play-interface-overview-for-beginners.html (interface overview, 2026-03)
- https://www.memuplay.com/blog/key-mapping-teaching.html (all 17 keymapper modules)
- https://www.memuplay.com/blog/how-to-use-macro-in-key-mapping.html (macro editor layout + command names)
- https://www.neowin.net/news/memu-android-emulator-701 (7.0.1 changelog: toggles, auto-start, nav bar)
- https://www.softpedia.com/progChangelog/MEmu-Changelog-250243.html and https://www.filehorse.com/download-memu (9.0.x–9.2.5 changelogs)
- MEmu manual (Scribd mirror) for General/Advanced settings pages
- https://gist.github.com/TameemS/603686cec857ff1f91e68607e374b0d8 (community: Engine/Display/DPI/disk sharing)
