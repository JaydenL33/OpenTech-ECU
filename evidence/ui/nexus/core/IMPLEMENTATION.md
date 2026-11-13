Partial implementation: VE live indices are now used only on the VE table.
Ignition and target tables no longer display an incorrectly indexed live marker.
No backend changes. Core selection/keyboard/3D workflow remains outstanding.

Verification: npm ci exit 0; npx tsc --noEmit exit 0; npx vite build exit 0.
cargo test --workspace --locked exited 0.
cargo clippy --workspace --all-targets --locked -- -D warnings exited 0.
Both Rust commands used PATH=/home/jlee/tools/bin:$PATH from ecu/.
No Playwright run, screenshots, or console-error assertion completed.
