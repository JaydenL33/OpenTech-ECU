# Rust / simulation workstream — round 1

Date: 2026-11-13 Australia/Sydney. Base commit: `d73d5481b7b01ffc39ae176cf80e7c2fcfa1751e`.
Scope: host Rust libraries, simulator CLI, Rust verification and evidence. No KiCad, BOM, UI or upstream Speeduino source changes.

| ID | State | Concrete task / verification | Owner |
|---|---|---|---|
| RUST-SIM-001 | Complete: 170 workspace tests pass; see `FIRMWARE_TEST_EVIDENCE.md` | Correct output crossing timestamps, chronological wrap traversal and cycle attribution; expose seeded, bounded JSON command traces with loss counts. `python3 ecu/tools/verify-output-timing.py AUDIT/rust-round-1` | Rust / simulation |
| RUST-ENV-001 | Resolved locally | Host has no `cc`; initial Cargo build exited 101 before compilation. Isolated downloaded GCC driver + development libraries with Rust's LLD enabled host tests. Exact setup in `FIRMWARE_TEST_EVIDENCE.md`, versions/hashes in `rust-round-1/toolchain.txt`. No system compiler installation. | Rust / simulation |
| RUST-SIM-002 | Open; autonomous next task | `advance_trigger` skips only `tooth == teeth - missing`, so multiple-missing configurations still emit later missing teeth. Cam generation is tied to a crank tooth exactly at 690°, and the 320–340° window assumes zero sync offset. Add an independent crank/cam edge generator and 36-2 / 60-2 / offset vectors before claiming general missing-wheel simulation. Verify no forbidden missing-tooth edges, independent cam edges, and expected decoder phase. | Rust / simulation |
| RUST-TRG-001 | Open; autonomous | Decoder latches cam validity without loss timeout; fixed 250 ms crank timeout and short-edge rejection need bounded synthetic fault tests. Implement tooth/cycle-based loss handling with recovery vectors, retaining explicit differences from Speeduino. | Rust / simulation |
| RUST-TIM-001 | Open; autonomous model work | Commands still use plant-angle crossings after whole-step decoder/protection evaluation. Schedule replacement at cycle wrap, pre-wrap mixture accounting and same-step fault causality need discrete-event regression vectors. Requested durations are not pulse-end execution; dwell cancellation is not modelled. | Rust / simulation |
| RUST-REF-001 | Open; cross-workstream dependency | Full upstream binary differential testing needs the selected Speeduino firmware revision, reproducible native/target build and captured reference outputs. Existing two fuel formula/vector comparisons remain narrow source-derived checks. Host Rust success is not evidence of Arduino Mega compatibility. | Rust + firmware baseline owner |
| RUST-HIL-001 | Blocked on physical fixture and approved hardware | Need actual timer capture/compare implementation, signal-generator integration and scope/logic-analyser captures before timing claims about hardware. JSON traces are offline client input, not hardware waveforms. | Firmware / bench |

RUST-SIM-002, RUST-TRG-001 and RUST-TIM-001 can proceed without vehicle data or fabrication approval. The general-waveform defect does not invalidate this round's default 36-1, zero-offset seeded tests; it limits other configurations.

## H0–H9 disposition for this software round

| Gate | Disposition |
|---|---|
| H0 — Verified upstream reference | No gate advancement. Existing Speeduino source preserved; exact selected PCB/firmware baseline and build evidence remain with the baseline workstream. This round does not establish that selection. |
| H1 — Complete PCB design package | Not assessed in this workstream; no hardware changes. |
| H2 — Engineering checks completed | Host Rust checks supply software evidence only; no PCB/ERC/DRC gate claim. |
| H3 — Independent fabrication review candidate | Software review material supplied; no hardware release-gate claim. |
| H4 — Fabrication approved | Not approved here; requires independent engineer review and explicit human approval. |
| H5 — Assembled hardware | No new physical evidence. |
| H6 — Powered bench testing | No new physical evidence. |
| H7 — Engine-system validation candidate | Blocked pending independent review and bench evidence. |
| H8 — Vehicle-specific integration | Blocked pending verified Evo VIII signals, pinout and interface review. |
| H9 — Qualified vehicle-operation candidate | Blocked pending relevant independent engineering and regulatory evidence. |
