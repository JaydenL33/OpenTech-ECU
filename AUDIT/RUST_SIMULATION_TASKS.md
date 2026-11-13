# Rust / simulation workstream — round 2

Date: 2026-11-13 Australia/Sydney. Round 2 base commit: `82f466a021e4b98af8931b5b3da9a3ba237a069b`. Round 1 evidence remains preserved separately.
Scope: host Rust libraries, simulator CLI, Rust verification and evidence. No KiCad, BOM, UI or upstream Speeduino source changes.

| ID | State | Concrete task / verification | Owner |
|---|---|---|---|
| RUST-SIM-001 | Complete in round 1: 170 workspace tests passed; see `RUST_OUTPUT_TIMING_EVIDENCE.md` | Correct output crossing timestamps, chronological wrap traversal and cycle attribution; expose seeded, bounded JSON command traces with loss counts. `python3 ecu/tools/verify-output-timing.py AUDIT/rust-round-1` | Rust / simulation |
| RUST-ENV-001 | Resolved locally | Host has no `cc`; initial Cargo build exited 101 before compilation. Isolated downloaded GCC driver + development libraries with Rust's LLD enabled host tests. Exact setup in `RUST_OUTPUT_TIMING_EVIDENCE.md`, versions/hashes in `rust-round-1/toolchain.txt`. No system compiler installation. | Rust / simulation |
| RUST-SIM-002 | Complete for tested ideal input model; round 2 | Independent `TriggerWaveform` omits all consecutive missing teeth, emits cam independently and applies tooth-zero offset. Decoder cam window is gap-relative; cycle phase retains offset and extrapolates across wrap. 6,432 f64 edge comparisons (36-1 / 36-2 / 60-2 / 7-2); independent decoder vectors; seeded 36-2/60-2 simulator integration. Bounded `triggers.json` exposes delivered edges and decoder status. See `RUST_TRIGGER_WAVEFORM_EVIDENCE.md` and `rust-round-2/verification.txt`. | Rust / simulation |
| RUST-TRG-001 | **Complete (host simulation; independent review pending)** | Bounded crank-loss deadline (normal interval × teeth, capped by `timeout_ms`), bounded cam-phase expiry (six crank revolutions or `timeout_ms`), short-edge rejection, stall-recovery baseline, and deterministic dropout/recovery vectors. 25 `ecu-trigger` tests pass; full workspace 222 pass. Explicit Rust policies, not Speeduino binary equivalence. See completion verification below. | Rust / simulation |
| RUST-TIM-001 | Open; autonomous model work | Commands still use plant-angle crossings after whole-step decoder/protection evaluation. Schedule replacement at cycle wrap, pre-wrap mixture accounting and same-step fault causality need discrete-event regression vectors. Requested durations are not pulse-end execution; dwell cancellation is not modelled. | Rust / simulation |
| RUST-REF-001 | Open; cross-workstream dependency | Full upstream binary differential testing needs the selected Speeduino firmware revision, reproducible native/target build and captured reference outputs. Existing two fuel formula/vector comparisons remain narrow source-derived checks. Host Rust success is not evidence of Arduino Mega compatibility. | Rust + firmware baseline owner |
| RUST-HIL-001 | Blocked on physical fixture and approved hardware | Need actual timer capture/compare implementation, signal-generator integration and scope/logic-analyser captures before timing claims about hardware. JSON traces are offline client input, not hardware waveforms. | Firmware / bench |

RUST-TRG-001 is complete for the host decoder; RUST-TIM-001 is the next independent Rust task and can proceed without vehicle data or fabrication approval. RUST-SIM-002 closes the identified generator defects for the tested ideal model, not arbitrary-wheel decoder compatibility or electrical signal generation. `SimEngine` retains the synthetic cam at 690° relative to tooth zero; the standalone generator accepts other cam phases. Independent crank/cam dropout and stochastic electrical jitter remain unimplemented. No gate disposition below changes in round 2.

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


### RUST-TRG-001 initial checkpoint (superseded)

> The checkpoint below is retained as historical evidence. It was correct at
> the time: the worktree had no C linker, so no test compiled. The lead resolved
> the environment with the local GCC wrapper and completed the verification and
> recovery vectors. See the completion section after it.

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

### RUST-TRG-001 completion verification (2026-11-13)

**Status: complete for the host simulation model; independent review pending.**

Environment: Rust/Cargo 1.99.1, local GCC 13.3.0 wrapper at
`/home/jlee/tools/bin/cc` (extracted toolchain; system compiler unchanged).
Exact commands, full raw output and exit codes are in
`AUDIT/rust-round-3/check-5-environment.txt` through `check-9-trigger-test.log`.
The earlier 101-exit logs (`check-2..check-4`) are retained as historical
environment failures, not test results.

Implemented behavior:
- Crank loss deadline = `expected_interval_us × loss_teeth`, capped by
  `timeout_ms`. Default `loss_teeth = missing + 3`, so a 36-1 wheel allows four
  normal tooth intervals (the legitimate gap is two).
- Cam phase expires after `(teeth − missing) × cam_loss_revolutions` accepted
  tooth edges (default six crank revolutions) or `timeout_ms` since the last
  valid cam. Expiry revokes `Sequential`, clears `cycle_angle_deg`, and keeps
  `cam_seen` as historical evidence.
- A valid cam inside the configured window re-establishes `Sequential`.
- Edges shorter than 50 % of the expected interval are ignored without updating
  capture time, speed history or error counters.
- The first edge after a stall/loss is consumed as a fresh baseline; the next
  edge establishes the period. This prevents a long gap from poisoning
  `expected_interval_us` and starving the short-edge filter.
- Error and gap counters saturate instead of wrapping.

Verification results:
| Command | Exit | Result |
|---|---:|---|
| `cargo fmt --all -- --check` | 0 | pass |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | 0 | pass, no warnings |
| `cargo test --workspace --locked` | 0 | **222 passed, 0 failed** |
| `cargo test -p ecu-trigger --locked -- --nocapture` | 0 | **25 passed, 0 failed** |

New deterministic vectors in `ecu/crates/ecu-trigger/src/lib.rs`:
`cam_loss_expires_sequential_phase_and_reacquires`,
`crank_loss_times_out_and_recovers`,
`short_edge_is_ignored_without_poisoning_sync`,
`cam_outside_window_drops_sequential_but_recovers`, and
`loss_bounds_reject_impossible_values`. Existing dropout, extra-tooth,
missing-tooth, acceleration and window tests continue to pass.

Defects found and fixed during verification (both were real, not test-only):
1. The old cam fixture emitted one cam edge in eight crank revolutions. The
   bounded expiry correctly rejected that as a lost cam, exposing a test-fixture
   defect. The fixture now emits one cam per 720-degree cycle.
2. The stall-recovery path treated the first post-stall edge as an interval,
   producing an oversized expected period and causing `ToothDropped` during
   resynchronisation. The decoder now consumes that edge as a baseline and
   returns to `Unsynced`.

Remaining limits and review needs:
- The 50 % short-edge threshold and six-revolution/250 ms expiry bounds are
  engineering choices, not values validated on a real engine or Speeduino
  capture. Rapid acceleration, cranking and noisy electrical edges need
  bench/simulation evidence before any timing claim.
- `set_loss_bounds` is a runtime method; it is not part of the serialized wheel
  configuration. A caller that never invokes it gets the documented defaults.
- The model assumes a clean missing-tooth pattern with optional cam; arbitrary
  wheels, cam-phase patterns other than the tested window, and hardware capture
  jitter remain out of scope.
- These are explicit Rust policies, not Speeduino binary equivalence. Physical
  timer capture, MCU compatibility and engine timing are **not** demonstrated.
- H0–H9 hardware gate dispositions are unchanged; no physical or vehicle claim.


### RUST-TRG-001 independent ownership review (2026-11-13)

Candidate 14a14b0b/b2c7ac33 reviewed in the isolated rust-engineer checkout.
Decision: reject unchanged candidate; accept the corrected host implementation
subject to the verification recorded in check-10 through check-14.

Found a real state-transition defect: after external tick() timeout cleared the
timestamp, the first restart edge set a baseline but left state Lost; the second
edge was consumed again by the Lost branch. Inline on_tooth timeout likewise
left Lost after recording the baseline. Both paths now enter Unsynced on the
first restart capture. The new restart_uses_exactly_one_baseline_with_or_without_tick
test checks state, next measured interval and tooth index for both paths.

Independently reviewed all five candidate vectors and existing extra/missing
tooth tests. Runtime defaults are missing+3 crank intervals and six cam
revolutions without a setter call. Deadline is capped by timeout_ms; comparison
is strictly greater than the deadline. Cam age counts accepted captures, not
nominal missing slots; expiry runs before gap parity/state selection. Wall time
is also checked by tick/on_tooth, so missing cam cannot remain valid merely
because edges stop (provided the caller services tick). Valid window cam resets
age and parity; subsequent gap resets parity to zero. Crank loss clears phase
and gap history. Counters saturate; serialized MissingToothConfig fields and
derives are unchanged.

Residual limits: six revolutions means accepted-edge count under a valid wheel,
not a wall-time revolution measurement during faults. Callers must service tick
for idle expiry; angle getters alone do not expire state. The 50% filter can
reject legitimate abrupt acceleration/uneven cranking and needs measured input
validation. Existing gradual acceleration does not establish that limit.
These deadlines/filter/recovery policies differ explicitly from Speeduino;
no upstream binary differential comparison was performed. Host simulation only,
no MCU/capture/engine/vehicle evidence; H0–H9 dispositions unchanged.

Independent final verification: check-10 compiler GCC 13.3.0; check-11 fmt exit 0; check-12 strict workspace Clippy exit 0; check-13 workspace tests exit 0 (223 passed, 0 failed); check-14 trigger nocapture exit 0 (26 passed, 0 failed). All Cargo commands ran from ecu with PATH=/home/jlee/tools/bin:$PATH. Historical check-2..4 retained. Corrected implementation accepted for host simulation.


## RUST-TRG-001 independent review remediation (2026-11-13)

Independent Claude review rejected completion for two blocking regressions:
pre-sync short-edge rejection could half-rate lock; fixed 250 ms cam expiry
lost phase during cranking. Review fix 0cff4eaa was adopted by merge
6b2a33f43887a5a25d39f8a190c7ae08e842f96a. Its acceleration regression remains.

Policy: cam age advances on accepted crank edges only; expiry occurs at
age >= (teeth - missing) * cam_loss_revolutions, including the exact bound
and all subsequent edges. No wall-clock cam deadline remains; last_cam_us
was removed. Continuing crank without cam is count-bounded; a stopped
engine is handled by crank timeout, which clears phase. A once-per-720°
cam cannot use an RPM-independent 250 ms deadline.

Minimum cam bound is now three revolutions; two is rejected using the
existing InvalidTimeout variant (retained API limitation). Default six
is exercised over 24 revolutions at 150/250/400 rpm, with cam each 720°,
then continued without cam through and beyond the exact expiry bound.
Review-reported before measurements: 12 Sequential losses at each of
150/250/400 rpm versus two for base; 600 rpm equal. These are supplied
review evidence, not a locally reproduced comparative run. Local after
measurements and assertions are in check-19.log.

Removed the second Lost branch in on_tooth: fail and tick clear
last_tooth_us; the initial no-timestamp branch handles prior loss, and
the inline tick-loss branch returns immediately. Existing restart and
recovery vectors remain.

Remaining limits: pre-sync noise is unfiltered, matching base behavior;
the synchronized 50% short-edge threshold is heuristic. Ratios 3.3,
3.5 and 3.9 are characterized as accepted normal edges (known 3.2x–4x
interval limitation; next task: independently justify rejection policy).
No Speeduino binary parity, MCU compatibility, physical capture, engine
or vehicle validation is claimed. H0–H9 dispositions are unchanged.

Verification commands, tool versions, raw outputs and exit codes:
AUDIT/rust-round-3/check-15.log through check-19.log. Commands run from
ecu with PATH=/home/jlee/tools/bin:$PATH: cc/rustc/cargo --version;
cargo fmt --all -- --check;
cargo clippy --workspace --all-targets --locked -- -D warnings;
cargo test --workspace --locked;
cargo test -p ecu-trigger --locked -- --nocapture.
Implementation fix commit is the commit containing this appended section
(resolve with git log -- ecu/crates/ecu-trigger/src/lib.rs); a follow-up
provenance record supplies its immutable hash.

Final remediation verification: fmt, Clippy, workspace tests and trigger tests all exit 0. Workspace: 226 passed, zero failed; trigger: 29 passed, zero failed. At 150/250/400 rpm: zero post-acquisition non-Sequential edges across 24 revolutions each. Implementation is ready for independent re-review; no independent acceptance is claimed.
