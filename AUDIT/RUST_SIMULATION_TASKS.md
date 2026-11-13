# Rust / simulation workstream — round 2

Date: 2026-11-13 Australia/Sydney. Round 2 base commit: `82f466a021e4b98af8931b5b3da9a3ba237a069b`. Round 1 evidence remains preserved separately.
Scope: host Rust libraries, simulator CLI, Rust verification and evidence. No KiCad, BOM, UI or upstream Speeduino source changes.

| ID | State | Concrete task / verification | Owner |
|---|---|---|---|
| RUST-SIM-001 | Complete in round 1: 170 workspace tests passed; see `RUST_OUTPUT_TIMING_EVIDENCE.md` | Correct output crossing timestamps, chronological wrap traversal and cycle attribution; expose seeded, bounded JSON command traces with loss counts. `python3 ecu/tools/verify-output-timing.py AUDIT/rust-round-1` | Rust / simulation |
| RUST-ENV-001 | Resolved locally | Host has no `cc`; initial Cargo build exited 101 before compilation. Isolated downloaded GCC driver + development libraries with Rust's LLD enabled host tests. Exact setup in `RUST_OUTPUT_TIMING_EVIDENCE.md`, versions/hashes in `rust-round-1/toolchain.txt`. No system compiler installation. | Rust / simulation |
| RUST-SIM-002 | Complete for tested ideal input model; round 2 | Independent `TriggerWaveform` omits all consecutive missing teeth, emits cam independently and applies tooth-zero offset. Decoder cam window is gap-relative; cycle phase retains offset and extrapolates across wrap. 6,432 f64 edge comparisons (36-1 / 36-2 / 60-2 / 7-2); independent decoder vectors; seeded 36-2/60-2 simulator integration. Bounded `triggers.json` exposes delivered edges and decoder status. See `RUST_TRIGGER_WAVEFORM_EVIDENCE.md` and `rust-round-2/verification.txt`. | Rust / simulation |
| RUST-TRG-001 | Open; autonomous | Decoder latches cam validity without loss timeout; fixed 250 ms crank timeout and short-edge rejection need bounded synthetic fault tests. Implement tooth/cycle-based loss handling with recovery vectors, retaining explicit differences from Speeduino. | Rust / simulation |
| RUST-TIM-001 | Open; autonomous model work | Commands still use plant-angle crossings after whole-step decoder/protection evaluation. Schedule replacement at cycle wrap, pre-wrap mixture accounting and same-step fault causality need discrete-event regression vectors. Requested durations are not pulse-end execution; dwell cancellation is not modelled. | Rust / simulation |
| RUST-REF-001 | Open; cross-workstream dependency | Full upstream binary differential testing needs the selected Speeduino firmware revision, reproducible native/target build and captured reference outputs. Existing two fuel formula/vector comparisons remain narrow source-derived checks. Host Rust success is not evidence of Arduino Mega compatibility. | Rust + firmware baseline owner |
| RUST-HIL-001 | Blocked on physical fixture and approved hardware | Need actual timer capture/compare implementation, signal-generator integration and scope/logic-analyser captures before timing claims about hardware. JSON traces are offline client input, not hardware waveforms. | Firmware / bench |

RUST-TRG-001 is the next independent unblocked task; RUST-TIM-001 can also proceed without vehicle data or fabrication approval. RUST-SIM-002 closes the identified generator defects for the tested ideal model, not arbitrary-wheel decoder compatibility or electrical signal generation. `SimEngine` retains the synthetic cam at 690° relative to tooth zero; the standalone generator accepts other cam phases. Independent crank/cam dropout and stochastic electrical jitter remain unimplemented. No gate disposition below changes in round 2.

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


### RUST-TRG-001 implementation checkpoint (2026-11-13)
Incomplete, unverified host-only change. Decoder adds `set_loss_bounds(teeth,
cam_revolutions)` without changing serialized MissingToothConfig fields. Defaults:
crank deadline four normal intervals for 36-1, capped at timeout_ms; cam phase
expires after six revolutions of accepted edges or timeout_ms since valid cam.
Historical cam_seen remains historical; cycle phase is revoked on expiration.
Short edges below half the estimated normal interval are ignored without updating
capture history. Counters saturate. Valid window cam re-establishes phase.
These are explicit Rust policies, not Speeduino binary-equivalent behavior.

Evidence: AUDIT/rust-round-3/check-{1,2,3,4}.log contains exact commands and
exit codes. Format check 0; clippy, workspace tests, trigger nocapture each 101:
missing cc linker prevents compilation. No passing test count measured in this
checkout. Recovery vectors are still outstanding; baseline compatibility and
noise/startup behavior remain unverified. No MCU or physical timing claim.
H0–H9 hardware gate status unchanged. Task remains open; next action is supply
a host linker, add deterministic recovery vectors, then repeat all checks.
