# OpenTech-ECU — Engineering Task Ledger

| Field | Value |
|---|---|
| Document ID | PROG-TASKS |
| Status | Living ledger; states are evidence-based |
| Date | 2026-11-13 Australia/Sydney |
| Baseline | `master` at `f88bb772c8b7b84f0772e199ee38c7b9dd0dee9c` |
| Orchestration | T3 Code native delegation; no shell-launched agents |
| Recovery record | `AUDIT/PREVIOUS_SWARM_RECOVERY.md` |

Task states: `TODO → CLAIMED → IMPLEMENTING → TESTING → REVIEW → DONE`, plus
`BLOCKED`. A task is `DONE` only when its acceptance criteria and verification
evidence are committed. Passing host checks do not promote a hardware gate.

## 1. Current gate / readiness status

| Area | Status | Evidence |
|---|---|---|
| Gate 0 — host software evidence | PASS, scoped | `cargo fmt`/`clippy`/`test` exit 0; UI build; see `AUDIT/RELEASE_GATES.md` |
| H0 — verified upstream hardware reference | PASS | `AUDIT/UPSTREAM_PROVENANCE.md`, `AUDIT/HARDWARE_BASELINE.md` |
| H1 — complete PCB design package | PARTIAL | Routed v0.4.4d review copy; no mounting holes/assembly drawing; BOM gaps |
| H2 — engineering checks | NOT PASSED | ERC 101, DRC 600, 0 unconnected; `AUDIT/PCB_CHECK_RESULTS.md` |
| H3 — independent fabrication review candidate | NOT READY | Review package incomplete; no independent review |
| H4 — fabrication approved | BLOCKED — human only | Independent review + explicit human approval required |
| H5–H9 | NOT STARTED / BLOCKED | No fabricated board, no power, no engine/vehicle evidence |

## 2. Workstream ownership

| Workstream | Owner | Owned paths |
|---|---|---|
| Hardware | Claude Code `@hardware-engineer` | `hardware/**`, `HARDWARE_BLOCKERS.md`, hardware `AUDIT/*` |
| Rust / simulation | Codex `@rust-engineer` | `ecu/crates/**`, `ecu/apps/ecu-sim-cli/**`, Rust `AUDIT/*` |
| Tuning / calibration | Claude Code `@tuning-engineer` | `ecu/apps/studio-service/**`, `ecu/apps/tuning-studio/**`, `apps/tuning-copilot/**`, tuning `AUDIT/*` |
| Orchestration / integration | OpenCode lead | `TASKS.md`, `AUDIT/PREVIOUS_SWARM_RECOVERY.md`, integration commits |

Concurrent delegates work in isolated Git worktrees. The lead integrates only
after verifying their committed changes and evidence.

## 3. Active tasks — cycle 2026-11-13

### HW-H2-01 — Close the mechanical and ERC package for H2/H3

| Field | Value |
|---|---|
| Workstream | Hardware |
| Owner | `@hardware-engineer` (Claude Code) |
| Priority | P0 |
| Depends on | H0 baseline (`hardware/prototype-v1/`) |
| Owned files | `hardware/prototype-v1/**`, `hardware/tools/**`, `AUDIT/PCB_CHECK_RESULTS.md`, `AUDIT/HARDWARE_DIFF.md`, `HARDWARE_BLOCKERS.md` |
| Expected output | Mounting-hole decision/placement; assembly and fabrication drawings; stack-up/mounting notes; ERC library-link cleanup and justified no-connect/PWR_FLAG fixes; reduced unexplained ERC/DRC; regenerated manufacturing package |
| Acceptance | Native KiCad files remain valid; 0 unconnected items; connectivity still PASS; every remaining ERC/DRC violation individually classified; manufacturing exports regenerated; Docker KiCad 9.0.9 commands and raw outputs committed |
| Verification | `hardware/tools/check_prototype_v1.sh`; inspect `verification/erc.rpt`, `drc.rpt`, `summary.txt`; `hardware/tools/verify_upstream_baseline.sh` |
| Safety | No fabrication, no purchase, no energising; no global rule disabling; no invented mechanical approval |
| State | REVIEW — independent review in progress. Commit `eca1bec5`; lead rerun: ERC 49, DRC 397 + 199 per-item exclusions, 0 unconnected, connectivity PASS, upstream 32/32 OK. H2 is complete with exceptions, not a pass; H3 not ready; H4 human-only. |

### RUST-TRG-001 — Crank/cam loss handling and recovery vectors

| Field | Value |
|---|---|
| Workstream | Rust / simulation |
| Owner | `@rust-engineer` (Codex) |
| Priority | P0 |
| Depends on | Host Rust baseline (`cargo test` 217 pass) |
| Owned files | `ecu/crates/ecu-trigger/**`, `ecu/crates/ecu-simulator/**` only as needed, `AUDIT/RUST_SIMULATION_TASKS.md`, `AUDIT/RUST_TRIGGER_WAVEFORM_EVIDENCE.md`, new `AUDIT/rust-round-3/**` |
| Expected output | Bounded tooth/cycle-based signal-loss handling; cam validity invalidation on loss/recovery; synthetic dropout, short-edge and re-sync vectors; explicit differences from Speeduino documented |
| Acceptance | No fixed-wall-clock-only latching; deterministic recovery tests for cam loss and crank loss; tests fail before and pass after; no regressions in workspace suite |
| Verification | `cargo fmt --all -- --check`; `cargo clippy --workspace --all-targets --locked -- -D warnings`; `cargo test --workspace --locked`; targeted `ecu-trigger` tests with `--nocapture`; raw logs committed |
| Safety | Host simulation only; no claim of hardware timing or Speeduino binary equivalence |
| State | REVIEW — independent re-review in progress. Commit `bd6f1a17`; 226 workspace / 29 trigger tests; 0 post-acquisition cam drops at 150/250/400 rpm; clippy/fmt clean. Prior reviews found and this branch fixes the pre-sync half-rate lock and low-RPM cam expiry. |

### TUNE-AUD-01 — Durable calibration audit and revision persistence

| Field | Value |
|---|---|
| Workstream | Tuning / calibration |
| Owner | `@tuning-engineer` (Claude Code) |
| Priority | P0 |
| Depends on | Gate 0 tuning service; sealed calibration revisions already exist in memory |
| Owned files | `ecu/apps/studio-service/**`, `ecu/crates/ecu-calibration/**` if required, `ecu/apps/tuning-studio/**`, `AUDIT/TUNING_WORKSTREAM_EVIDENCE.md` |
| Expected output | Append-only on-disk calibration revision/audit store; checksum-verified load; approval record with actor/time/reason; restart recovery; service endpoints and minimal UI history view; tests |
| Acceptance | Revision survives service restart; tampered/corrupt store is refused or flagged; audit entries are append-only and include before/after checksums; no physical output path; existing HTTP behaviours unchanged |
| Verification | `cargo fmt --all -- --check`; `cargo clippy --workspace --all-targets --locked -- -D warnings`; `cargo test --workspace --locked`; new service persistence test with `--nocapture`; `npx tsc --noEmit` and `npx vite build` in `ecu/apps/tuning-studio` |
| Safety | No autonomous calibration write to hardware; no vehicle path; store is local evidence, not signed certification |
| State | **DONE (host) — integrated `c35323f6`.** Independent review: ACCEPT WITH FINDINGS. Prior blocking durability finding resolved (fsynced temp+rename metadata, parent-directory fsync, absolute env paths, full audit paging, failed-write rollback). 235 workspace / 36 studio-service tests. LOW residual TUNE-UI-002. Host evidence only; no gate advanced. |

## 4. Queued tasks (not started)

| ID | Workstream | Task | Why queued |
|---|---|---|---|
| HW-BOM-01 | Hardware | Assign real MPNs to 11 BOM rows and remove placeholders | Needs sourcing research; next hardware cycle |
| HW-BOM-02 | Hardware | Dated AUD supplier quotes and purchase-vs-build comparison | Depends on BOM; no purchases |
| HW-BENCH-01 | Hardware | IDC40 breakout schematic/assembly + safe power-up procedure | Design-only until H4 |
| HW-MECH-01/02 | Hardware | Assembly drawing, stack-up, mounting holes | Folded into HW-H2-01 |
| RUST-TIM-001 | Rust | Discrete-event timing/causality vectors | After TRG-001; no hardware needed |
| RUST-REF-001 | Rust | Upstream differential testing | Needs reproducible upstream reference captures |
| TUNE-LAM-017 | Tuning | Unit-aware importer for percent trims, psi/bar MAP, `.msl` headers | Needs real log sample for acceptance |
| TUNE-UI-001 | Tuning | Browser test of file-upload/import flow | Needs a browser session; do not fake |
| TUNE-UI-002 | Tuning | Incremental audit paging and refresh cancellation | LOW review finding: UI currently fetches/retains the entire history; unbounded logs can amplify requests/memory |
| HW-FW-02 | Firmware / hardware | Pin an official Speeduino firmware tag for the release package | Low priority; build path already works |
| HW-REVIEW-01 | Human | Independent hardware review, then H4 decision | AI cannot grant H4 |
| HW-VEH-01 | Human / research | Evo VIII connector, pinout and trigger verification | Vehicle evidence required |

## 5. Completed in prior cycles (reference)

| Area | Evidence |
|---|---|
| Recovery of three swarm worktrees and integration into `master` | `AUDIT/PREVIOUS_SWARM_RECOVERY.md`; commits `ed9375a4`, `8fd7d2ee`, `f88bb772` |
| Speeduino v0.4.4d baseline selection, provenance, routed review copy | `AUDIT/HARDWARE_BASELINE.md`, `UPSTREAM_PROVENANCE.md`, `PCB_CHECK_RESULTS.md` |
| Firmware Mega 2560 build evidence | `AUDIT/FIRMWARE_TEST_EVIDENCE.md` |
| Rust trigger waveform/timing evidence rounds 1–2 | `AUDIT/rust-round-1/`, `AUDIT/rust-round-2/` |
| Tuning rounds 1–7, including HTTP e2e and CSV import | `AUDIT/TUNING_WORKSTREAM_EVIDENCE.md` |

## 6. Ledger rules

1. Only the lead changes task state at integration; delegates propose evidence.
2. A task may be split, but file ownership must remain disjoint between
   concurrent delegates.
3. Every accepted task must add raw command output, exit codes and commit
   hashes to `AUDIT/`; summaries alone are insufficient.
4. Never delete a historical failure. Supersede it with a later evidence file.
5. No task may self-promote H4 or any physical/vehicle gate.
