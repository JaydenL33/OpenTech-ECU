# Previous Swarm Recovery — OpenTech-ECU

| Field | Value |
|---|---|
| Document ID | AUD-REC-01 |
| Status | Recovery record; historical evidence, not a validation claim |
| Date | 2026-11-13 |
| Baseline inspected | `master` at `f88bb772c8b7b84f0772e199ee38c7b9dd0dee9c` |
| Historical orchestration | `tools/ecu-swarm/` (Bash/tmux), three Git worktrees, JSONL logs |
| Replacement orchestration | T3 Code native delegation (this program) |
| Related | `AUDIT/UPSTREAM_PROVENANCE.md`, `AUDIT/HARDWARE_BASELINE.md`, `AUDIT/RUST_SIMULATION_TASKS.md`, `AUDIT/TUNING_WORKSTREAM_EVIDENCE.md` |

> This document distinguishes **verified repository evidence** from agent
> claims. It does not reactivate the historical swarm and does not treat any
> recovered artifact as fabrication, bench or vehicle approval.

## 1. Historical architecture

The previous programme used a custom Bash/tmux coordinator (`tools/ecu-swarm/`)
with three long-lived specialist workers and one Git worktree per worker:

| Role | Provider / command | Worktree | Branch |
|---|---|---|---|
| Hardware | OpenCode, `deepseek/deepseek-flash` | `/home/jlee/OpenTech-ECU-swarm-worktrees/hardware` | `swarm/hardware-20261112T222821Z` |
| Rust / simulation | Codex CLI (`codex exec`) | `/home/jlee/OpenTech-ECU-swarm-worktrees/rust` | `swarm/rust-20261112T222821Z` |
| Tuning | Claude Code (`claude -p`) | `/home/jlee/OpenTech-ECU-swarm-worktrees/tuning` | `swarm/tuning-20261112T222821Z` |

State and logs were written under `/home/jlee/OpenTech-ECU-swarm-state/`:
`baseline.sha`, `index/*.latest`, and JSONL/stderr/meta files per run. The
coordinator wrote to a terminal-attached tmux session, which is why several
runs ended with exit code 130 (SIGINT) rather than a worker failure.

The T3 Code orchestrator replaces this system. `tools/ecu-swarm/` is retained
as historical evidence only; it must not be run.

## 2. Branches and commit evidence

All three worker branches are **fully contained in `master`**; their tips are
ancestors of the integration commits. No recoverable commit remains unmerged.

| Branch | Tip | Relationship to `master` |
|---|---|---|
| `swarm/hardware-20261112T222821Z` | `3e666981f6e0deb590c5137c2600c835a74fd38c` | Ancestor (merged by `ed9375a4`) |
| `swarm/rust-20261112T222821Z` | `2a2491ea8e9404db937cc790130316a0a1a8ae8f` | Ancestor (merged by `8fd7d2ee`) |
| `swarm/tuning-20261112T222821Z` | `e47ff46e49d660474558d0cce6702c19afa5b870` | Ancestor (merged by `f88bb772`) |
| `feature/support-ui-harness` | `1e53f883e7d64b9da46793acb014f24b64013ed7` | Ancestor; earlier UI/safety/PCB work |
| `public/snapshot` | `fb4e38afaf4afcaccc6d04f606f17d43b4b634f2` | Older public snapshot; superseded by `master` |
| `wisp/opentech-ecu-autonomous-multi-ha-536128` | `cadebd8a` | Separate recovery-merge branch, **not** an ancestor; its content is duplicated by the three integration commits. Its registered worktree `/home/jlee/OpenTech-ECU-wisp-recovery` no longer exists (prunable). |

`master` is 20 commits ahead of `origin/master` (`d73d5481`) at the time of
this record. The local commits contain the recovered work and have not been
pushed by this workstream.

Integration commits:

| Commit | Content |
|---|---|
| `ed9375a4` | Hardware: routed Speeduino v0.4.4d SMD review copy, KiCad 9 files, manufacturing exports, H0–H2 audits |
| `8fd7d2ee` | Rust: trigger waveform export, output timing verification, scheduler/simulator fixes, round-1/2 evidence |
| `f88bb772` | Tuning: external CSV datalog import, lambda gating, ignition advisory, trim-compensated VE calibration |

## 3. Recovered assets by workstream

### 3.1 Hardware

- `hardware/upstream-speeduino/v0.4.4d/` — byte-preserved upstream source,
  commit `ee6a9242834891877744ff2137663b6ca8d79ed6`, with `SHA256SUMS`.
- `hardware/prototype-v1/` — native KiCad 9 schematic/PCB review copy of the
  v0.4.4d SMD board, plus `verification/` and `manufacturing/`.
- `hardware/bench-interface/` — netlist-derived IDC40 pin map (40 pads).
- `hardware/tools/` — baseline verifier, KiCad check/export runner, minimal
  revision script, connectivity comparison.
- `AUDIT/HARDWARE_BASELINE.md`, `HARDWARE_DIFF.md`, `PCB_CHECK_RESULTS.md`,
  `UPSTREAM_PROVENANCE.md`, `BOM_REVIEW.md`, root `HARDWARE_BLOCKERS.md`.
- Firmware build evidence: `evidence/hardware/` and
  `AUDIT/FIRMWARE_TEST_EVIDENCE.md` (Arduino Mega 2560 image, not flashed).

Verified facts (from the committed reports, not re-derived here): the PCB has
0 unconnected items, 514/514 common schematic-to-PCB nodes, a closed
100.05 × 100.05 mm outline, 9 Gerber layers, drill file, BOM, position file,
and 141 real placements after the proto-pad exclusion. ERC 101 and DRC 600
violations remain; H2 is not passed.

### 3.2 Rust / simulation

- `ecu/` Cargo workspace: `ecu-types`, `ecu-fuel`, `ecu-ignition`,
  `ecu-scheduler`, `ecu-trigger`, `ecu-protection`, `ecu-lambda`,
  `ecu-simulator`, `ecu-protocol`, `ecu-calibration`, `ecu-harness`,
  `opentech-firmware-stm32`, plus `ecu-sim-cli` and `studio-service`.
- Round-1/2 evidence: `AUDIT/rust-round-1/`, `AUDIT/rust-round-2/`,
  `AUDIT/RUST_OUTPUT_TIMING_EVIDENCE.md`,
  `AUDIT/RUST_TRIGGER_WAVEFORM_EVIDENCE.md`, `AUDIT/RUST_SIMULATION_TASKS.md`.
- Open work: `RUST-TRG-001` (crank/cam loss handling and recovery vectors),
  `RUST-TIM-001` (discrete-event timing/causality), `RUST-REF-001` (upstream
  differential testing), `RUST-HIL-001` (blocked on physical hardware).

Verified in this recovery round: `cargo fmt --all -- --check` exit 0,
`cargo clippy --workspace --all-targets --locked -- -D warnings` exit 0, and
`cargo test --workspace --locked` exit 0 with 217 passing test cases across
suites. This is host evidence only; it is not target-hardware validation.

### 3.3 Tuning

- `ecu/apps/tuning-studio/` — React/TypeScript/Vite tuning studio (maps, 3D
  surfaces, Mixture Map, diagnostics, logs, harness page, copilot page).
- `ecu/apps/studio-service/` — local Axum HTTP/WebSocket service.
- `ecu/crates/ecu-lambda/` — shared learning gates, STFT/LTFT, Mixture Map,
  external CSV import.
- `apps/tuning-copilot/` — read-only Mastra copilot with offline mode.
- `AUDIT/TUNING_WORKSTREAM_EVIDENCE.md` — seven rounds of evidence, including
  the first compiled/executed HTTP end-to-end tests in round 5 and CSV import
  round-trip in round 7.

## 4. Historical failures observed in the logs

| Observation | Evidence | Current disposition |
|---|---|---|
| OpenCode rejected an unsupported model with `provider.invalid-request` | `logs/hardware-20261112T222831Z-4040144-1.jsonl`; exit 1 | Replaced by T3 provider discovery; model validated before delegation |
| Two hardware runs ended with exit 130 | `.meta` files for 22:42:57 and 22:45:15 | Caused by the terminal-attached coordinator, not a code defect |
| Rust worker hit sandbox/file-write errors | `rust-…-1.stderr.log` (`Failed to write file …AUDIT/…`) and early sandbox failure | Later Codex run with a usable sandbox completed and was integrated |
| Tuning session ended quickly in an early round | `tuning-20261112T222831Z…` meta; later rounds succeeded | Superseded by rounds 5–7 with executed verification |
| Wisp recovery worktree is missing | `git worktree list` marks it prunable | Branch preserved; content duplicated in `master`; no action needed |
| Log dashboard visibility was incomplete | Historical `tools/ecu-swarm` runner/dashboard | Coordinator retired; T3 owns session history and monitoring |

## 5. Verified versus unverified claims

**Verified in this repository (reproducible commands or committed raw output):**

- The upstream Speeduino hardware revision is pinned and hash-verified.
- The prototype-v1 PCB is routed with 0 unconnected items and connectivity
  PASS under the recorded KiCad 9.0.9 image.
- The Arduino Mega 2560 firmware build succeeded; the image hash is recorded.
- The Rust workspace formats, lints and tests clean on the host.
- The tuning UI type-checks/builds and the service has executed HTTP tests.

**Not verified / must not be implied:**

- Any automotive qualification, EMC, thermal or vibration suitability.
- Manufacturing readiness: ERC 101, DRC 600, no assembly drawing, 11 BOM rows
  without MPNs, no independent review.
- Target-hardware firmware behaviour: no flash, no power, no scope capture.
- Real datalog import: the CSV importer is self-tested only; no real
  Speeduino/TunerStudio log or calibration has been supplied.
- Evo VIII connector, pinout, trigger or harness compatibility.
- Any H4+ gate: human approval is required and has not been granted.

## 6. Integration recommendations

1. Treat `master` (`f88bb772` plus this record) as the only development base.
2. Keep the three historical worktrees read-only; do not merge from them again.
3. Use `hardware/prototype-v1/` for hardware work, not the older unrouted
   `hardware/kicad/evo8-bench-ecu/` draft.
4. Continue the open tasks in `TASKS.md`: hardware H2/mechanical package,
   `RUST-TRG-001`, and durable tuning audit/persistence.
5. Re-run the exact verification commands in each workstream before accepting
   a change; append evidence, never overwrite historical failures.
6. Do not claim that recovered artifacts are validated hardware. The recovery
   establishes provenance and a baseline, not safety.

## 7. Superseded work

- `hardware/kicad/evo8-bench-ecu/` — custom unreviewed schematic; unrouted PCB
  (499 unconnected items). Superseded by `hardware/prototype-v1/`.
- `public/snapshot` — earlier Gate 0 snapshot; superseded by `master`.
- `wisp/opentech-ecu-autonomous-multi-ha-536128` — recovery-merge branch;
  superseded by the three integration commits.
- `tools/ecu-swarm/` — retired coordinator; evidence only.

## 8. Remaining engineering blockers

- Hardware H2: ERC 101, DRC 600 (199 courtyard overlaps, 19 mask bridges,
  12 shorting items, 143 library mismatches), no assembly drawing, no mounting
  holes, 11 BOM rows without MPNs, no dated AUD sourcing comparison.
- Hardware H3/H4: review package incomplete; independent review and explicit
  human approval outstanding.
- Rust: `RUST-TRG-001`, `RUST-TIM-001`, `RUST-REF-001`; `RUST-HIL-001` blocked
  on physical hardware.
- Tuning: no real datalog/calibration; unit handling and thresholds unvalidated;
  durable calibration audit/persistence not implemented; browser upload flow
  untested.
- Vehicle: Evo VIII pinout, connector, trigger and harness remain unverified.
