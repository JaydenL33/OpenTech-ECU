Partial implementation: VE live indices are now used only on the VE table.
Ignition and target tables no longer display an incorrectly indexed live marker.
No backend changes. Core selection/keyboard/3D workflow remains outstanding.

Verification: npm ci exit 0; npx tsc --noEmit exit 0; npx vite build exit 0.
Rust verification was started; results must be checked before claiming success.
No Playwright run, screenshots, or console-error assertion completed.
