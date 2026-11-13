Current round: [Rust trigger waveform evidence — round 2](RUST_TRIGGER_WAVEFORM_EVIDENCE.md), based on `82f466a021e4b98af8931b5b3da9a3ba237a069b`. The round 1 evidence below is historical and retained intact.

# Rust host output timing evidence — round 1

Date: 2026-11-13 Australia/Sydney (2026-11-12 UTC). Base commit: `d73d5481b7b01ffc39ae176cf80e7c2fcfa1751e`, branch `swarm/rust-20261112T222821Z`. Evidence is for the changes accompanying this document; the source SHA-256 manifest identifies the tested Rust snapshot. Older `TEST_EVIDENCE.md` and `FIRMWARE_TIMING_ANALYSIS.md` describe earlier snapshots and do not include this change.

## Change and scope

RUST-SIM-001 corrects three observed source defects: output markers were stamped at step start, wrapped schedule traversal returned low angles before high angles, and pre-wrap markers inherited the next cycle number. The engine now uses the saved starting phase, interpolates each crossing in the 1 ms plant step, rounds to the nearest microsecond, returns markers in traversal order and attributes pre-wrap markers to the preceding cycle. Dwell command markers are now retained alongside injection and spark markers.

`ecu-sim-cli run --seed <decimal-u64>` overrides the existing deterministic scenario seed. `--check-determinism` now compares serialized complete telemetry logs and output traces. Seeded sensor noise affects the plant/RPM and its deterministic synthetic crank/cam inputs; no stochastic electrical trigger-jitter model is introduced.

`outputs.json` is a separate version-1 client artifact. It contains scenario seed, calibration checksum/revision, unchanged ignition provenance, simulation-only label, capture horizon and explicit total/evicted event counts. Each marker carries sequence number, timestamp, plant cycle, cylinder, output channel, kind, angle and **requested** duration. Only the newest 4096 markers are retained; `events_dropped` and the first retained sequence expose missing history. Consumers must not interpret this bounded tail as a complete session. The public `SimEngine::simulated_output_trace()` method provides the same record for a future service/UI adapter. This round does not modify the UI or add a service route.

```sh
cd ecu
cargo run --locked -p ecu-sim-cli -- run --scenario mixture_sweep --seed 42 \
  --out runs/output-timing/mixture_sweep --check-determinism
# Read runs/output-timing/mixture_sweep/outputs.json alongside log.json.
```

No calibration generation, ignition estimate, physical-output interlock or upstream source is changed. Synthetic ignition remains explicitly non-operational through the existing `SyntheticSimulationOnly` policy, now copied into the trace.

## Reference comparisons

| Test | Independent expectation / limit |
|---|---|
| `wrapped_traversal_is_chronological_and_half_open` | Hand angles: 718°→2° must yield 719°, 0°, 2°. Split intervals neither duplicate nor omit endpoints; equal endpoints yield nothing. |
| `wrap_events_have_crossing_timestamps_and_correct_cycles` | 7500 rpm, 690°→15° in 1 ms: injection at 692.5°, dwell at 697.5°, spark at 720°. At start 1,000,000 µs expect 1,000,056 / 1,000,167 / 1,000,667 µs, cycles 7 / 7 / 8. |
| `constant_rpm_outputs_match_independent_f64_time_reference` | Six speeds: 600, 1000, 3000, 6000, 7500, 9000 rpm; one-cylinder fixed request over three cycles. Separate f64 TDC-based time equations yield 54 expected command timestamps. Every error must be ≤1 µs; dwell-to-spark separation must be exactly 3000 µs. No expected angles/times are read from production schedules. |
| `cuts_and_lost_sync_suppress_command_markers` | Fuel cut suppresses injection; spark cut suppresses dwell/spark; combined cuts or lost sync suppress all commands. This is step-state gating, not pulse cancellation or sub-step fault-latency proof. |
| `seeded_runs_export_repeatable_bounded_traces_with_loss_accounting` | Seed 42 repeated twice yields identical serialized telemetry and traces; seed 43 changes plant/telemetry and output records. Tail sequence, monotonic timestamps/cycles, explicit eviction count, JSON round-trip and non-operational ignition policy are checked. |
| Existing `inline4_outputs_match_pre_change_baseline` | Fixed calibration/log SHA-256 comparison against the repository's existing four-cylinder snapshot; independent of the new trace schema. |
| Existing `ecu-fuel/tests/speeduino_parity.rs` | Two narrow source/formula-derived comparisons retained. These do not execute a Speeduino firmware binary and do not establish whole-ECU parity. |

## Reproduction and provenance

With a normal Rust + C linker toolchain:

```sh
python3 ecu/tools/verify-output-timing.py AUDIT/rust-round-1
```

The verifier saves exact commands, raw combined stdout/stderr and exit codes in [`rust-round-1/verification.txt`](rust-round-1/verification.txt). [`rust-round-1/manifest.json`](rust-round-1/manifest.json) records Rust source, lockfile, verifier and generated artifact hashes. Generated logs/traces are under ignored `ecu/runs/output-timing/`; compact trace metadata/examples are retained in the evidence directory.

This host initially failed `cargo test -p ecu-scheduler -p ecu-simulator` with exit 101: `error: linker cc not found`. No code tests ran in that attempt. Local setup used these commands:

```sh
mkdir -p /tmp/ecu-rust-round1-toolchain
cd /tmp/ecu-rust-round1-toolchain
apt-get download gcc-13-x86-64-linux-gnu libgcc-13-dev
for package in /tmp/ecu-rust-round1-toolchain/*.deb; do
  dpkg-deb -x "$package" /tmp/ecu-rust-round1-toolchain/root
done
mkdir -p /tmp/ecu-rust-round1-toolchain/bin
ln -sf /home/jlee/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/x86_64-unknown-linux-gnu/bin/rust-lld \
  /tmp/ecu-rust-round1-toolchain/bin/ld.lld
```

The executable `bin/cc` wrapper is reproduced verbatim in [`rust-round-1/toolchain.txt`](rust-round-1/toolchain.txt), together with actual compiler/linker versions and downloaded package hashes. The final verification command prepended that directory to `PATH`. GCC driver/library package version: `13.3.0-6ubuntu2~24.04.1`; Rust/Cargo: `1.99.1`; LLD: `23.1.1`. Package files were downloaded from the configured Ubuntu archive and extracted locally; no system compiler installation or hardware purchase occurred.

Initial new-test corrections: the hand vector first requested injection end at 0°, which the existing scheduler correctly rejects; the fixture now uses allowed 5° timing. Strict Clippy initially rejected a constant `chunks_exact(3)` loop; the test now checks dwell intervals within its existing indexed loop. The original lint failure is preserved in `verification-initial-clippy-failure.txt`. A subsequent format-only check failed until `cargo fmt --all` was run; final raw results below supersede those attempts.

## Results

| Command / check | Exit | Observed result |
|---|---|---|
| `cargo fmt --all -- --check` | 0 | Pass |
| `cargo clippy --workspace --all-targets -- -D warnings` | 0 | Pass |
| `cargo test --workspace` | 0 | **170 passed, 0 failed**, including five new regression tests; zero doc-tests |
| Six control crates, `cargo check ... --no-default-features` | 0 | Pass on host: `ecu-types`, `ecu-fuel`, `ecu-ignition`, `ecu-trigger`, `ecu-scheduler`, `ecu-protection`; no new MCU target claim |
| CLI `mixture_sweep`, seed 42, `--check-determinism` | 0 | 2184 frames; serialized telemetry + trace identical on repeat; 11,628 commands emitted, 4096 retained, 7532 evicted |
| CLI `trigger_loss`, seed 43, `--check-determinism` | 0 | 1144 frames; serialized telemetry + trace identical on repeat; 541 commands emitted/retained, zero evicted |

The full suite includes both existing Speeduino fuel comparisons and the unchanged inline-four calibration/log fingerprint test. Raw output records 12 scheduler unit tests, 14 simulator unit tests and four simulator integration tests passing. These are current checkout counts; the older 130-test audit snapshot is historical.

Requirement links: SIM-006/SIM-012 (seeded reproducibility), SIM-007 (offline client log artifacts), SCH-001 (injection/dwell/spark command markers), SCH-003 (cut gating) and SCH-008 (chronological wrap/endpoints). SCH-005 (hardware timer execution) remains unmet. No H4+ gate is advanced.

## Limits and next tasks

The ≤1 µs criterion concerns **representation of a constant-speed synthetic plant crossing**, not ignition accuracy, ISR jitter or a physical timing budget. The production plant still integrates in 1 ms steps, schedules against plant phase, evaluates decoder/protection for the whole step before applying commands and rebuilds schedules at cycle wrap. Injection pulse-end execution, dwell pulse state/cancellation, hardware capture delay and driver latency are absent. Requested dwell need not equal elapsed dwell under variable RPM or schedule changes.

The simulator's existing general-wheel generator has multiple-missing-tooth and cam-position restrictions; this round exercises its default 36-1 zero-offset scenario path. Cam-loss timeout and short-edge rejection remain open. Exact blockers, next verification actions and H0–H9 disposition are in [`RUST_SIMULATION_TASKS.md`](RUST_SIMULATION_TASKS.md). This host evidence grants no physical release gate.

Speeduino remains the upstream behavioural reference in `speeduino/` and `test/`; no such files are modified. The last commit touching `speeduino/decoders.cpp` in this checkout is `5d217bf4471fa1391a5acd0aa341848ec43f5431` (`Unit test the Ford TFI decoder (#1732)`). That is source provenance, not a newly selected firmware release or a build result. No Arduino Mega Rust target/toolchain compatibility is asserted, and no upstream firmware build was attempted this round.
