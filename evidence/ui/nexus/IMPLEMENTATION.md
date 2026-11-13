UI-NEXUS-01 partial implementation; base 1594ebac.
Implemented shell, back history, draggable tabs, dark/contrast preferences,
local dashboard persistence, disabled device commands, surface reset/empty guard.
Backend inspected; no backend changes. Existing approval/pause APIs preserved.
Verification: npm ci exit 228 ENOSPC; initial tsc exit 1 missing dependencies;
Vite build not reached. cargo test --workspace --locked exit 101 ENOSPC;
cargo clippy --workspace --all-targets --locked -- -D warnings exit 101 ENOSPC.
Rust commands used PATH=/home/jlee/tools/bin:$PATH from ecu.
Removed only generated ecu/target after disk exhaustion.
No browser verification or screenshots.
Remaining: full table keyboard/range selection; surface live tracking and selection;
dash component settings/types/arrangement; F6/F7/F8 logging; forward history,
full menu/shortcut coverage. No multi-log API or device API exists.
PDM/scope/flash/reboot/update disabled with reason. No hardware gate changed.

## Open-page tabs follow-up

Merged checkpoint from `t3/nexus-tuning-interface` onto isolated master
1594ebac worktree. Replaced all-page button duplication with initially Main-only
open tabs. Navigation opens tabs; tabs expose tablist/tab/aria-selected, roving
focus, arrows/Home/End navigation, close with adjacent activation/focus, and drag
reorder. Last tab remains open. Backend routes inspected; no backend changes.

Validation: npm ci exit 0; npx tsc --noEmit exit 0; npx vite build exit 0.
Initial npx tsc failed before dependency installation. Rust outputs are recorded
alongside this file. Browser verification not performed: Playwright package was
unavailable in the workspace. No screenshots or zero-console-error claim.

Remaining acceptance work: advanced table keyboard workflow; telemetry tracking;
3D interaction/readouts; full dash designer; logging shortcuts/manager; complete
menu command wiring; persistent tab state; browser coverage and screenshots.
This follow-up completes the concrete tab defect only, not the full specification.
Hardware readiness gates unchanged.

Final checks: `PATH=/home/jlee/tools/bin:$PATH cargo test --workspace --locked`
and `PATH=/home/jlee/tools/bin:$PATH cargo clippy --workspace --all-targets --locked -- -D warnings`
from ecu both exited 0; raw output: rust-test.log and clippy.log.
