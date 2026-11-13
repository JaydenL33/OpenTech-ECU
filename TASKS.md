# OpenTech-ECU — Engineering Task Ledger

| Field | Value |
|---|---|
| Document ID | PROG-TASKS |
| Status | Living ledger; states are evidence-based |
| Date | 2026-11-13 Australia/Sydney |
| Baseline | `master` at `71763bbd` (cycle close; see git log for later ledger commits) |
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
| H1 — complete PCB design package | PARTIAL | Routed v0.4.4d review copy; mounting holes, assembly/fab drawings and library fixes added; BOM MPNs still missing |
| H2 — engineering checks | COMPLETE WITH DOCUMENTED EXCEPTIONS; focused independent review ACCEPT WITH FINDINGS | ERC 49, DRC 397 + 199 per-item exclusions, 0 unconnected; `AUDIT/PCB_CHECK_RESULTS.md` |
| H3 — independent fabrication review candidate | NOT READY | BOM MPNs, footprint-library audit and human acceptance of H2/mechanical exceptions outstanding |
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
| State | **DONE (design package) — integrated `3d191045`.** Focused independent review: ACCEPT WITH FINDINGS. Lead rerun: ERC 49, DRC 397 + 199 per-item exclusions, 0 unconnected, connectivity PASS, upstream 32/32 OK. Findings: crowded assembly annotations need human readability/orientation review; standoff detail is in notes; header MPN, three-hole support strategy, fab assumptions and H4 approval remain human decisions. H2 complete with exceptions, not a pass; H3 not ready. |

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
| State | **DONE (host) — integrated `b710d2b9`.** Independent re-review: ACCEPT WITH FINDINGS. Both blocking defects fixed (pre-sync half-rate lock, low-RPM wall-clock cam expiry); mutation tests fail on the old behavior. 226 workspace / 29 trigger tests; 0 post-acquisition cam drops at 150/250/400 rpm. Residual medium pre-sync noise false-gap behavior is pre-existing and queued as RUST-TRG-002. |

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
| HW-DRAW-01 | Hardware | Improve assembly-drawing annotation readability and add explicit standoff/J3 strain-relief detail | Review found crowded/pale annotations and standoff detail only in notes; human readability decision remains |
| HW-FOOT-01 | Hardware | Audit the 143 embedded KiCad 5 footprint/library mismatches | H3 blocker identified by the hardware review |
| RUST-TRG-002 | Rust | Pre-sync noise false first-gap lock | Re-review measured 100/136 probe cases; require a minimum baseline-tooth count or confirmed second gap before first sync |
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
| HW-H2-01 mechanical/ERC/DRC-exception package | `3d191045`; focused review ACCEPT WITH FINDINGS; H2 complete with exceptions, H3 not ready |
| RUST-TRG-001 bounded crank/cam loss handling | `b710d2b9`; re-review ACCEPT WITH FINDINGS; 226 workspace / 29 trigger tests; RUST-TRG-002 queued |
| TUNE-AUD-01 durable calibration audit store | `c35323f6`; review ACCEPT WITH FINDINGS; 235 workspace / 36 studio-service tests; TUNE-UI-002 queued |
| Status reconciliation | `71763bbd`; README, RELEASE_GATES, HARDWARE_STATUS, traceability and blockers updated |

## 6. Ledger rules

1. Only the lead changes task state at integration; delegates propose evidence.
2. A task may be split, but file ownership must remain disjoint between
   concurrent delegates.
3. Every accepted task must add raw command output, exit codes and commit
   hashes to `AUDIT/`; summaries alone are insufficient.
4. Never delete a historical failure. Supersede it with a later evidence file.
5. No task may self-promote H4 or any physical/vehicle gate.

## 7. Cycle 2026-11-13 close

| Check | Result |
|---|---|
| `cargo fmt --all -- --check` | exit 0 |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | exit 0 |
| `cargo test --workspace --locked` | **244 passed, 0 failed** |
| `cargo test -p ecu-trigger --locked -- --nocapture` | 29 passed; 0 post-acquisition cam drops at 150/250/400 rpm |
| `cargo test -p studio-service --locked -- --nocapture` | 36 passed; restart, tamper, SIGKILL crash proxy and failed-write rollback |
| Tuning Studio `npx tsc --noEmit` / `npx vite build` | exit 0 / built |
| `hardware/tools/check_prototype_v1.sh` | exit 0; ERC 49, DRC 397 + 199 per-item exclusions, 0 unconnected |
| `hardware/tools/verify_upstream_baseline.sh` | exit 0; 32/32 upstream hashes |
| Independent hardware review | ACCEPT WITH FINDINGS; 199/199 exclusions matched, jumpers verified, drawings rendered |
| Independent Rust re-review | ACCEPT WITH FINDINGS; mutation tests prove the two blocking fixes |
| Independent tuning review | ACCEPT WITH FINDINGS; prior blocking durability finding resolved |

Gate disposition: Gate 0 host evidence remains PASS, scoped. H0 PASS, H1 PARTIAL,
H2 COMPLETE WITH DOCUMENTED EXCEPTIONS, H3 NOT READY, H4 human-only. No physical,
engine, vehicle or fabrication gate advanced. No purchase or fabrication occurred.

## 8. Next cycle tasks — 2026-11-13

### HW-BOM-01 — Real MPNs, supplier data and AUD cost comparison

| Field | Value |
|---|---|
| Workstream | Hardware |
| Owner | `@hardware-engineer` |
| Priority | P0 (H3 blocker) |
| Depends on | HW-H2-01 design package; header housing decision from the focused review |
| Owned files | `hardware/prototype-v1/**` (MPN fields and regenerated BOM only), `AUDIT/BOM_REVIEW.md`, `docs/HARDWARE_COST.md`, hardware BOM evidence |
| Expected output | Real, purchasable MPNs for the 11 grouped rows / 17 placements lacking them; supplier, alternates, availability, dated AUD price; assembled-vs-build comparison; header MPN with housing ≤ 2.54 mm per position; explicit substitution restrictions |
| Acceptance | Release BOM has no blank MPN or placeholder; every price has a dated source URL; arithmetic checked; no safety-critical substitution justified by pin compatibility alone; no purchase made; BOM still matches the schematic/netlist |
| Verification | Regenerate BOM via the pinned KiCad image; audit rows against `AUDIT/BOM_REVIEW.md`; cross-check quantities and references |
| Safety | Read-only web research; no purchases, fabrication or energising |
| State | CLAIMED — delegated in this cycle |

### RUST-TRG-002 — Pre-sync noise false first-gap lock

| Field | Value |
|---|---|
| Workstream | Rust / simulation |
| Owner | `@rust-engineer` |
| Priority | P1 |
| Depends on | RUST-TRG-001 integration (`b710d2b9`) |
| Owned files | `ecu/crates/ecu-trigger/**`, `AUDIT/RUST_SIMULATION_TASKS.md`, `AUDIT/RUST_TRIGGER_WAVEFORM_EVIDENCE.md`, `AUDIT/rust-round-4/**` |
| Expected output | Require a minimum baseline-tooth count or a confirmed second gap before first sync; synthetic pre-sync noise vectors at several injection ratios; documented sync-latency trade-off |
| Acceptance | False first-gap lock no longer occurs in the reviewer's probe class; existing 29 trigger tests still pass; new tests fail on the old behavior; no hardware claim |
| Verification | fmt, strict clippy, full workspace tests, trigger tests with `--nocapture`; raw logs committed |
| Safety | Host simulation only |
| State | CLAIMED — delegated in this cycle |

### TUNE-STORE-02 — Single-writer store lock

| Field | Value |
|---|---|
| Workstream | Tuning |
| Owner | `@tuning-engineer` |
| Priority | P1 |
| Depends on | TUNE-AUD-01 integration (`c35323f6`) |
| Owned files | `ecu/apps/studio-service/**`, `AUDIT/TUNING_WORKSTREAM_EVIDENCE.md` |
| Expected output | Refuse a second service instance on the same data directory; safe stale-lock behaviour after a crash; test with two processes/instances |
| Acceptance | Two concurrent writers cannot corrupt the store; stale lock is recovered deliberately, not silently; no physical output path or approval change |
| Verification | fmt, strict clippy, full tests, targeted store tests with `--nocapture` |
| Safety | Local evidence store only; no hardware path |
| State | CLAIMED — delegated in this cycle |

### TUNE-UI-002 — Incremental audit paging and cancellation

| Field | Value |
|---|---|
| Workstream | Tuning |
| Owner | `@tuning-engineer` |
| Priority | P2 (LOW review finding) |
| Depends on | TUNE-AUD-01 UI |
| Owned files | `ecu/apps/tuning-studio/**`, `AUDIT/TUNING_WORKSTREAM_EVIDENCE.md` |
| Expected output | Incremental "load more" paging or bounded fetch with cancellation; no silent 1,000-entry truncation; stale responses cannot overwrite newer state |
| Acceptance | Typecheck/build pass; read-only panel; browser rendering is explicitly not claimed unless actually tested |
| Verification | `npx tsc --noEmit`, `npx vite build` |
| Safety | Read-only UI; no writes or hardware |
| State | CLAIMED — delegated in this cycle |
