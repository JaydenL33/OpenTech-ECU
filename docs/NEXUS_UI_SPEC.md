# Nexus-style tuning UI — implementation specification

Status: design contract for the Tuning Studio. This describes the intended
interaction model, not a claim that every feature is already backed by the ECU
service.

## 0. Product boundary

The current product is a local ECU tuning studio: Rust service + React/TypeScript
Vite UI. It does **not** yet have PDM outputs, a dash display, an oscilloscope,
firmware flashing, or a vehicle connection. The UI must not pretend those exist.

Rules:

1. Implement the shell, theme, table editing, live-engine tracking, 3D surface,
   gauge/dash layout, and keyboard workflow against real existing data.
2. If a spec feature needs a backend capability that does not exist, either:
   - add a real, tested backend endpoint/state, or
   - render it disabled with an explicit "not available in this build" reason.
3. Never show a control that silently does nothing or writes fake data.
4. Preserve the existing approval, pause, interlock and no-physical-output rules.

## 1. Window shell

Tri-pane desktop layout:

- **Menu bar** (File, Edit, Setup, Tools, Logging, Preferences, Help) with the
  command groups listed below. Unsupported commands are disabled with a reason.
- **Toolbar strip** below the menu: navigation-tree toggle, connect/disconnect
  state, reboot-required indicator, new/open/save, import/upload, compare,
  copy/paste/undo/redo, table axis setup, IO report, oscilloscope, firmware
  upgrade, data-log manager. State-aware colours:
  - reboot required = red
  - firmware update available = green (only if a real update source exists)
  - connect state = colour by connection/playback mode
- **Left sidebar**: collapsible navigation tree with search, forward/back
  history, and a context-tip panel. The tree contains real pages only; disabled
  future nodes are visibly marked.
- **Main workspace**: draggable page tabs. A page is a user-arranged collection
  of tables, gauges and plots. A default Main tab exists.
- **Status strip**: connection, playback, logging, physical-output interlock,
  calibration checksum/revision, and unsaved-change state.

## 2. Theme and visual language

- Default **dark charcoal/near-black** surface, slightly lighter panels, thin
  low-contrast gray borders.
- Cyan/blue-green selection outline; blue engine-position marker; green live
  tracking when the cursor is on the live cell; amber/red warning thresholds;
  red reboot-required; green firmware-update-ready.
- High-contrast and night variants selectable from Preferences; custom day/night
  dash schemes.
- Font: system UI sans-serif, compact 11–13 px base, tabular numerals, bold axis
  headers, minimal italics. Global font scaling.
- Dense 2–4 px padding, 1 px dividers, no decorative shadows or large radii.

## 3. Tuning tables

- Grid with RPM/load (or configured) axes; up to 64 breakpoints per axis.
- Active cell cyan outline; multi-cell selection:
  - Shift+Arrow expands; Ctrl+Arrow shrinks; Ctrl+Space returns to one cell.
  - Ctrl+A selects all.
- Engine position: blue circle over the live cell; Space aligns the cursor to
  the live point. In 3D, a blue arrow marks the live position.
- Editing:
  - PgUp/PgDn increment/decrement; Ctrl+PgUp/Ctrl+PgDn small; Shift+PgUp/PgDn
    large.
  - 0–9 direct numeric entry; Enter commits; Escape cancels.
  - P = percentage change, L = linearise across selection, basic + - * /
    operations on a range.
  - Ctrl+C/Ctrl+V copy/paste; Ctrl+Shift+Arrow copy to adjacent cell.
  - Insert/Delete row/column where the table backend supports it.
  - Home/End move to row edges; Alt+Arrow changes map.
  - Q = quick tune current live cell (only when the existing lambda workflow is
    eligible and approval is present); W = current cell and higher RPM/load.
  - T = trace toggle/reset.
- Output readout and selection size shown in the table/3D view.
- Axis setup dialog: rename/rescale/insert/delete breakpoints, with validation
  and the existing calibration revision/approval flow.
- All writes go through the existing calibration API; a failed write must not
  mutate the displayed table.

## 4. 3D surface

- Rotatable shaded surface with visible mesh; selected area glows blue.
- Arrow keys move selection; PgUp/PgDn edit.
- Blue arrow for live engine position; selection size top-right; output value
  bottom-right; "return to default angle" button top-right.
- Left-drag rotates; reset restores the default camera.
- No fake surface when data is missing; show an explicit empty/disabled state.

## 5. Gauges and dash designer

- Component types: bar gauge, circular dial, numeric readout.
- Click selects; selected component gets a cyan outline and opens settings:
  channel, label, units, decimals, display min/max, warning min/max.
- Add/remove/arrange components; drag to reorder dash screens; day/night colour
  schemes; warning thresholds change colour.
- Layouts persist locally (and through the calibration store if appropriate).
- Only real telemetry channels are selectable; unsupported channels are shown
  disabled with a reason.

## 6. Datalogging and playback

- F6 start/stop PC logging; F7 enter/exit playback; F8 play/pause loaded log.
- Data Log Manager: list, select, load, export, delete (with confirmation).
- Live tracing during playback; allow edits only where the existing approval
  rules allow them, and clearly mark that playback is not a running engine.
- No claim of ECU onboard logging unless a real backend path exists.

## 7. Menus and commands

- **File:** New, Open, Save, Save As, Recent, Import/Upload, Restore Defaults,
  Compare, Exit.
- **Edit:** Undo, Redo, Copy, Paste.
- **Setup:** Table Axis Setup, IO Report, Password Protection (disabled unless
  implemented).
- **Tools:** Oscilloscope (disabled unless implemented), Upgrade Firmware
  (disabled; no flashing path), Reboot Device (disabled; no device path).
- **Logging:** Data Log Manager, Start/Stop PC Logging, Playback.
- **Preferences:** Theme, Units, Navigation Tree options, Table Text options.
- **Help:** Help, Keyboard Reference, Check for Updates (disabled unless a real
  update source exists), About.

Menu/toolbar items that are not implemented must say why, not open a mock.

## 8. Keyboard reference

Arrows move; Shift+Arrow expands; Ctrl+Arrow shrinks; Alt+Arrow changes map;
Tab/Shift+Tab next/previous page; Space align to live; Ctrl+Space single cell;
F1 help; F3 axes; F4 setup; F5 connect/disconnect; F6 logging; F7 playback;
F8 play/pause; F11 full screen; Ctrl+A select all; Ctrl+C/V copy/paste;
Ctrl+F12 reboot (disabled); P percentage; L linearise; T trace; Insert/Delete
row/column; Home/End row edges; Ctrl+Shift+Arrow copy adjacent; R all-ranges
toggle; A all-ranges-above; Q quick tune; W quick tune upward.

Keyboard handling must not fire while typing in an input, and must respect
platform modifiers.

## 9. Backend implications to assess

The UI agent must inspect the service and decide, with tests:

- calibration table edit/resample/undo/apply endpoints and revision/approval
  semantics;
- live telemetry stream for engine position, gauges and playback;
- log list/load/replay endpoints;
- persistence for page tabs, dash layouts, theme and preferences;
- axis metadata and bounded breakpoints;
- IO report/harness data;
- explicit disabled states for PDM, oscilloscope, firmware flashing and reboot.

Do not add a physical-output path. Do not weaken the approval or pause rules.

## 10. Acceptance

- `npx tsc --noEmit` and `npx vite build` pass.
- Existing Rust workspace tests pass; any backend change has tests.
- No placeholder control that appears functional; disabled features are labelled.
- Keyboard, table selection/editing, 3D tracking and dash editing work against
  the local virtual ECU.
- Browser smoke test/screenshots cover the shell, table, 3D, dash, logging and
  disabled-feature states.
- Host-only evidence; no vehicle, engine or physical-output claim.
