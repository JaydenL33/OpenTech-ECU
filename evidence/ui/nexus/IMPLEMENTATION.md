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
