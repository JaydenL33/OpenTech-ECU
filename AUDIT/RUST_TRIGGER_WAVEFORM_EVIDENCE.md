# Rust trigger waveform evidence — round 2

Date: 2026-11-13 Australia/Sydney. Base commit: `82f466a021e4b98af8931b5b3da9a3ba237a069b`. This is host simulation evidence for RUST-SIM-002. The source hashes in `rust-round-2/manifest.json` identify the tested Rust snapshot. No upstream Speeduino, KiCad, BOM, UI or ignition calibration source changes.

## Implemented behavior

`ecu-simulator::trigger_waveform::TriggerWaveform` generates conditioned crank and cam input edges independently. Each revolution contains exactly `teeth - missing` crank edges. All consecutive missing positions are omitted. The first tooth after each gap is placed at the configured `sync_offset_deg` in the plant; the default cam is at 690° relative to tooth zero over 720°. The standalone generator accepts a separate cam phase, including a cam edge between teeth or within the crank gap. The simulator retains its fixed synthetic cam/window model; this is not an Evo trigger definition.

Intervals are (start, end], bounded to at most 720° per call. Zero movement emits nothing. Invalid angles, movement, cam phases, zero duration and overflowing timestamp horizons return explicit errors. Crossings are chronological; coincident crank is delivered before cam. Input capture timestamps truncate fractional microseconds, preserving the existing default input convention. The prior round's output command timestamps still round to nearest microsecond.

The decoder now checks its documented cam window in degrees **from the last gap**, accounting for sync offset. Cycle phase uses revolution parity relative to tooth zero, including when crank angle wraps before the next gap. Extrapolation preserves 720° phase when crossing a gap before the next capture arrives. These changes do not add cam-loss detection, short-edge rejection or a new timeout policy.

## Client artifacts

The CLI writes version-1 `triggers.json` alongside existing `outputs.json`, `log.json` and CSV. `SimEngine::simulated_trigger_trace()` exposes the same record to host clients. Each delivered input has sequence, timestamp, plant angle, crank/cam identity and decoder status immediately after that edge. Status angle is the latest crank capture angle; it is not an interpolated angle at an off-tooth cam edge. Control-loop timeout transitions without an edge remain visible through telemetry, rather than a new input record.

Only the latest **8192** inputs are retained. `events_emitted`, `events_dropped`, first retained sequence and capture horizon disclose truncation. Metadata includes scenario seed and calibration checksum/revision; configuration and synthetic cam phase are explicit. `--check-determinism` compares serialized input traces as well as telemetry/output traces. No UI/service route is added. `TriggerDropout` continues to suppress both inputs at plant-step resolution.

```sh
cd ecu
cargo run --locked -p ecu-sim-cli -- run --scenario trigger_loss --seed 43 \
  --out runs/trigger-waveform --check-determinism
# Inspect triggers.json alongside outputs.json and log.json.
```

## Independent references and results

| Verification | Observed result / scope |
|---|---|
| `full_cycles_match_independent_f64_wheel_vectors` | 6,432 edge comparisons across 36-1, 36-2, 60-2, 7-2; offsets 0/25/95/355°; speeds 600/1000/3000/6000/7500/9000 rpm. Independent f64 nominal-slot/time equations; every input timing error ≤1.01 µs. 7-2 checks generator geometry only. |
| Split interval hand vector | 60-2, 700°→725°: crank 702° at 1200 µs, independent cam 707.5° at 1750 µs, crank 0° at 3000 µs. Splitting at cam edge preserves exactly the same records; coincident default crank/cam delivered in order. |
| `cam_in_missing_region_decodes_without_a_synthetic_crank_tooth` | Four cycles of 60-2 at 1000 rpm; 116 crank edges/cycle plus independent cam at 707.5°. Sequential phase accepted within a configured gap-relative 340–355° window; zero decode errors; extrapolated cam phase error <0.01°. |
| Independent decoder vectors | Integer-degree timestamps, eight revolutions at 1000 rpm; 36-1/36-2/60-2 at four offsets. After phase acquisition: Sequential, capture cycle-angle error <0.001°, extrapolated angle error <0.01°, final RPM error <0.01 rpm, zero decode errors. Source bypasses production waveform generator. |
| Gap extrapolation regression | Last crank capture at 700°, then 4000 µs at 1000 rpm: expected cycle angle 4°, not 364°. Separate integer-timestamp fixture. |
| Seeded real simulator integration | 36-2/60-2 at offsets 0/95/355° run for 5 s; sequential sync, zero decode errors, output records present, non-operational ignition policy retained. Separate 15 s 60-2/95° runs test bounded tail, sequence/loss accounting, monotonic timestamps and JSON round-trip; seed 42 repeats byte-for-byte, seed 43 differs. |
| Pinned prior Rust trigger-loss differential | Exact round-1 log hash reproduced from archived commit in an isolated build directory. Eight telemetry samples (6.04–6.24 s during dropout before timeout) change only by 360° in cycle phase as expected from the wrap correction; every other telemetry field, metadata, log event and output command matches. See `rust-round-2/reference-and-diff.txt`. |
| Existing inline-four baseline | Calibration SHA-256 `d9ddcdf2ef600a610ae319698b97eedacbad94291e67c8bf7099edb9ae5e5a89` and idle-warmup log SHA-256 `ff6d5a675d321f073cea847aa9c16683e7a48364c65198c4db6c9851d98a2c9a` unchanged. |
| `cargo fmt --all -- --check` | Exit 0. |
| `cargo clippy --locked --workspace --all-targets -- -D warnings` | Exit 0. |
| `cargo test --locked --workspace` | **177 passed, 0 failed**, including seven new tests; zero doc-tests. Existing two narrow source-derived Speeduino fuel comparisons retained. |
| Six control crates, `cargo check --locked ... --no-default-features` | Exit 0 on host; no MCU build/compatibility claim. |
| CLI mixture_sweep, seed 42, repeated | 2184 frames; 11,628 output commands (4096 retained); 68,900 delivered inputs (8192 retained, 60,708 evicted). All three serialized records identical on repeat. |
| CLI trigger_loss, seed 43, repeated | 1144 frames; 541 output commands; 7321 delivered inputs, no evictions. All three serialized records identical on repeat. |

The timing bound is an ideal constant-speed input representation check, not physical capture/ISR/output accuracy or a variable-RPM accuracy budget. Full-cycle test durations use integer microseconds (the six requested RPM labels can therefore have a tiny period quantization error). No electrical noise, conditioner hysteresis, hardware jitter, pulse levels, pulse widths or physical waveform playback is modeled by this generator.

## Reproduction and provenance

With Rust and a working C linker on PATH, from the repository root:

```sh
python3 ecu/tools/verify-output-timing.py AUDIT/rust-round-2
python3 ecu/tools/compare-trigger-reference.py AUDIT/rust-round-2
```

This host reuses the isolated GCC/LLD setup from round 1 without a new download or system installation. Prepend `/tmp/ecu-rust-round1-toolchain/bin` to PATH on this host. Rust/Cargo 1.99.1 and GCC 13.3.0 versions are recorded in `rust-round-2/verification.txt`; exact wrapper/package hashes remain in `rust-round-1/toolchain.txt`. The reference comparator uses Python's `tarfile` data filter and requires Python 3.12+ (or a version with that filter backported).

The main verifier retains commands, combined stdout/stderr, exit codes, Rust source/lockfile/verifier hashes, generated artifact hashes and compact input/output excerpts under `rust-round-2/`. Full generated client artifacts are ignored under `ecu/runs/output-timing/`. The comparator archives the pinned prior Rust commit, builds it offline in a temporary source directory with a separate `ecu/target/trigger-reference` build directory, verifies the resulting log against the recorded round-1 hash and compares fields/output commands with this round's generated artifacts. Its raw evidence is `rust-round-2/reference-and-diff.txt`; `reference-verifier-sha256.txt` identifies that comparison script. This is a Rust-to-Rust differential reference, not upstream Speeduino binary parity.

Initial development findings: the buffer test's original 5 s idle fixture emitted only 3553 inputs and correctly failed its 8192-capacity expectation; the bounded-tail fixture now runs 15 s. An intermediate extrapolation expression changed floating-point operation order, changing 108 idle-warmup cycle-angle values while all other fields and output commands matched. Advancing before adding parity restored the unchanged baseline fingerprint; no golden hash was replaced. An initial verification format check failed because formatting was invoked from the repository root without a manifest path; the retained `verification-initial-format-failure.txt` captures the resulting unformatted test diff. Formatting was then applied with the correct manifest path; final verification supersedes that attempt. The first reference rebuild shared the candidate Cargo build directory and did not reproduce the pinned log hash; its raw output is preserved in `reference-initial-shared-cache.txt`. A separate reference build directory reproduces the exact recorded prior hash and isolates the eight expected phase differences. The three affected candidate Cargo package caches were then cleaned and the CLI rebuilt; `cache-restoration.txt` records the repeated trigger-loss determinism check and confirms all 10 candidate artifact hashes still match the final manifest. The repository source was unchanged by cache cleanup.

## Traceability, remaining tasks and gates

Project directive §3/§8: independent crank/cam generation, host trigger decoding, timing vectors, deterministic regression and client telemetry. Requirement links: TRG-001/002/005 (synthetic missing-tooth/cam/config evidence), SIM-002 (generalized generator), SIM-006/007/012 (seeded logs, client records, repeatability). Existing ignition remains `SyntheticSimulationOnly`; no on-engine timing advance or calibration writes are added.

RUST-TRG-001 is complete for the host decoder (cam-loss expiration, speed-aware crank loss, short-edge rejection and recovery vectors); see the completion section below. RUST-TIM-001 remains autonomous: discrete-event schedule replacement/fault causality, pulse-end execution and dwell cancellation. RUST-REF-001 still requires a selected, reproducibly built upstream Speeduino firmware and captured reference outputs. RUST-HIL-001 requires approved physical fixtures and measured captures. Arbitrary wheel/cam electrical compatibility and the actual Evo pattern remain unverified.

H0–H9 dispositions are retained in `RUST_SIMULATION_TASKS.md`: software evidence only, no hardware gate advanced. Speeduino source remains the known upstream reference; this round does not port Rust to Arduino Mega or replace that firmware path. H4 needs independent engineering review and explicit human approval; H5+ need physical evidence.


### RUST-TRG-001 initial checkpoint (superseded)

> Retained as historical evidence: the worktree initially had no C linker, so
> compilation exited 101 before any test ran. The lead supplied the local GCC
> wrapper, fixed two real recovery defects, and added the missing vectors.

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

### RUST-TRG-001 completion (2026-11-13)

Host-only completion; independent review pending. Full details, commands and
residual limits are in `RUST_SIMULATION_TASKS.md` §RUST-TRG-001 completion
verification and `AUDIT/rust-round-3/check-5` … `check-9`.

- `cargo fmt --all -- --check` exit 0; `cargo clippy --workspace --all-targets
  --locked -- -D warnings` exit 0; `cargo test --workspace --locked` exit 0
  with **222 passed, 0 failed**; `cargo test -p ecu-trigger --locked -- --nocapture`
  exit 0 with **25 passed, 0 failed**.
- New vectors: cam loss expiry and re-acquisition, crank stall timeout and
  recovery, short-edge rejection without poisoning sync, cam-outside-window
  rejection and recovery, and loss-bound validation.
- Two real defects were found during verification: an eight-revolution
  single-cam test fixture (now one cam per 720° cycle), and post-stall recovery
  using the stall gap as a tooth interval (now a fresh baseline edge).
- Bounds (50 % short-edge threshold, six-revolution/250 ms cam expiry) are
  engineering choices, not values validated on an engine or Speeduino capture.
  No MCU, hardware capture or physical timing claim; no gate advanced.
