# Independent Review Guide

| Field | Value |
|---|---|
| Document ID | AUD-07 |
| Status | Reviewer instructions; not itself an independent review |
| Date | 2026-11-12 |
| Evidence cut-off | 2026-11-12T05:45Z, working tree; source manifest `733355cc...` |
| Audience | Automotive controls engineer, embedded Rust reviewer, PCB reviewer, functional-safety reviewer, project owner |
| Related | AUD-01 through AUD-10 |

> **Independence requirement.** A reviewer who designed or wrote the code, owns the vehicle, or has a commercial interest in approval is not independent for a release gate. Record conflicts of interest. No self-approval. A review that only reads this document is not a review; re-run the commands and inspect the source.

## 1. Reading order

| Order | Document / path | Why read it | Approx. time |
|---:|---|---|---|
| 1 | `README.md` | Project claims/status. Treat as **stale** where it conflicts with code; verify each claim | 15 min |
| 2 | `ECU_MASTER_BRIEF.md` | Source requirements and non-negotiable distinctions | 30 min |
| 3 | `docs/SAFETY-CASE.md` | Existing safety argument; note its test-status table is stale relative to AUD-05 | 15 min |
| 4 | `docs/FORMULAS.md` | Implemented equations and assumptions; check against code | 20 min |
| 5 | `docs/ARCHITECTURE.md` | Layers/data flow/host-firmware boundary | 15 min |
| 6 | `AUDIT/ENGINE_CONTROL_SAFETY.md` | Hazard analysis, safety requirements, GSN, non-claims | 30 min |
| 7 | `AUDIT/FAILURE_MODE_ANALYSIS.md` | Component FMEA and coverage gaps | 30 min |
| 8 | `AUDIT/ELECTRICAL_DESIGN_REVIEW.md` | Hardware requirements; confirm the schematic/PCB draft is unreviewed and DRC fails | 20 min |
| 9 | `AUDIT/FIRMWARE_TIMING_ANALYSIS.md` | Timing budget concept; confirm no timing evidence | 20 min |
| 10 | `AUDIT/TEST_EVIDENCE.md` | Commands/results; reproduce the current evidence | 20 min |
| 11 | `AUDIT/REQUIREMENTS_TRACEABILITY.md` | Requirement-to-code/test mapping; Evo unverified items | 20 min |
| 12 | `AUDIT/RELEASE_GATES.md` | Gate criteria and current status | 15 min |
| 13 | `AUDIT/EVO8_BENCH_TEST_PLAN.md` | Future bench procedure; marked not executed | 20 min |
| 14 | `AUDIT/HARDWARE_STATUS.md` | What hardware exists vs does not | 15 min |
| 15 | Source: `ecu-types` -> `ecu-calibration` -> `ecu-fuel` -> `ecu-ignition` -> `ecu-trigger` -> `ecu-scheduler` -> `ecu-lambda` -> `ecu-protection` -> `ecu-protocol` -> `ecu-simulator` -> apps | Dependency order; verify invariants | 2-4 h |
| 16 | Tests: run the suite and inspect the firmware skeleton | Evidence, not prose | 30 min |
| 17 | UI: `apps/tuning-studio/src` and build | Check warnings/labels; not a hardware path | 30 min |

**Stale-document warning.** At this snapshot, `README.md` says the CLI/service are stubs and the UI does not exist. The code now contains a working CLI, service and UI build. Do not use README as evidence of absence or presence; use the source and commands.

## 2. Commands to reproduce

Run from `OpenTech-ECU/ecu` with a Rust toolchain on `PATH`.

```bash
# 1. Full suite. Expect 130/130 passing.
cargo test --workspace

# 2. Formatting and clippy. Expect both to pass.
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings

# 3. Type-check all targets.
cargo check --workspace --all-targets

# 4. Firmware no_std skeleton target check.
cargo check -p opentech-firmware-stm32 --target thumbv7em-none-eabihf --no-default-features

# 5. Core no_std checks. All listed crates are expected to pass.
for c in ecu-types ecu-fuel ecu-ignition ecu-trigger ecu-scheduler ecu-protection; do
  cargo check -p "$c" --no-default-features
done

# 6. CLI scenarios and deterministic run.
cargo run -q -p ecu-sim-cli -- scenarios
cargo run -q -p ecu-sim-cli -- run --scenario mixture_sweep --out /tmp/review/mixture --check-determinism
cargo run -q -p ecu-sim-cli -- analyze --log /tmp/review/mixture/log.json --cal /tmp/review/mixture/calibration.json --out /tmp/review/mixture/report.json
cargo run -q -p ecu-sim-cli -- apply --cal /tmp/review/mixture/calibration.json --report /tmp/review/mixture/report.json --out /tmp/review/mixture/applied.json --approve

# 7. Apply without approval must be refused.
cargo run -q -p ecu-sim-cli -- apply --cal /tmp/review/mixture/calibration.json --report /tmp/review/mixture/report.json --out /tmp/review/should-not-exist.json

# 8. UI and copilot type/build checks.
cd apps/tuning-studio && npm run typecheck && npm run build
cd ../tuning-copilot && npm run build

# 9. Local service smoke test (different port if 8765 is in use).
cd ../..
cargo run -q -p studio-service -- --port 18766 --no-loop
# In another terminal:
curl -sS http://127.0.0.1:18766/api/health
curl -sS http://127.0.0.1:18766/api/preflight
curl -sS http://127.0.0.1:18766/api/vehicles
```

**Expected at this snapshot**

| Command | Expected result |
|---|---|
| `cargo test --workspace` | 130 tests; 130 pass; 0 fail |
| `cargo fmt` / `clippy` | Pass |
| `cargo check --workspace --all-targets` | Pass |
| Firmware target check | Pass; skeleton only, no HAL/timer/driver |
| Core `no_std` loop | `ecu-types`, `ecu-fuel`, `ecu-ignition`, `ecu-scheduler`, `ecu-trigger`, `ecu-protection` pass |
| CLI determinism | PASS for fixed calibration; 2184 frames |
| CLI apply without `--approve` | Exit 1, refusal message |
| UI typecheck/build | PASS; headless Chromium smoke test PASS; Electron not launched |
| Copilot typecheck | PASS; live LLM/tools not tested |
| Service health | `physical_outputs:false`, interlock engaged |

If your results differ, record the exact command, toolchain, hash and output. Do not assume the audit's result is current.

## 3. Code invariants to check

Each invariant is a claim that should be falsifiable by source inspection or a test. "Violation example" describes what would disprove it.

| ID | Invariant | Where to check | Violation example | Current status |
|---|---|---|---|---|
| INV-01 | No physical actuator path exists | Search for GPIO/timer/driver/CAN transmit in `ecu/`; inspect `firmware/stm32`; inspect `apply_events` | A function that writes a hardware register or calls a driver | Holds for snapshot; firmware skeleton forces outputs off |
| INV-02 | Physical interlock is engaged and reported | `preflight.rs` constant; `preflight()` check; `/api/health` | Constant false; preflight `physical_export_allowed=true` | Holds for snapshot |
| INV-03 | Lambda is absent from ignition decisions | `grep -R "lambda\\|afr\\|wideband" ecu/crates/ecu-ignition/src`; inspect signatures | Lambda parameter in `corrected_advance`, `recommend_from_reference`, dwell | Holds |
| INV-04 | Synthetic/unknown ignition reference is rejected | `recommend_from_reference`; tests `synthetic_or_unknown...`, `missing_reference...` | A synthetic reference returns `VerifiedReference` | Holds for policy, but metadata can be forged |
| INV-05 | Fuel calculation reports inhibition, not silent clamp | `calculate`; `FuelDecision.inhibited`; duty test | Excessive duty returns `inhibited=None` or silently clamps | Holds |
| INV-06 | Actuator path honors `inhibited`/`fuel_cut`/`spark_cut` | Simulator `compute_ecu_outputs`, `apply_events`; no hardware | A future driver ignores inhibition | Holds in sim; no hardware |
| INV-07 | Learning gate fails closed | `gate::evaluate`; `every_invalidity_is_explained`; `nan_lambda_rejected` | Invalid sample returns `allowed=true` | Holds for checked conditions |
| INV-08 | STFT/LTFT bounded; no windup | `ClosedLoop::update`; `LongTermTrimTable::update`; tests | Trim exceeds `max_stft`/`max_trim`; integral grows while saturated | Holds |
| INV-09 | Mixture Map never auto-applies | `apply_proposal`; service `approve`; CLI `--approve`; test `stale_proposal_is_refused` | Analyze endpoint mutates calibration; apply without approve succeeds | Holds |
| INV-10 | Apply requires matching base checksum/revision | `apply_proposal`; `StaleProposal`; test | Changed calibration accepted | Holds |
| INV-11 | Calibration seal detects accidental change | `seal`/`verify`; `checksum_detects_tampering` | Edited JSON verifies without reseal | Holds; **does not detect reseal** |
| INV-12 | Trigger only sequential after gap + cam | `on_tooth`, `on_cam`; `syncs_and_measures_constant_speed` | `Sequential` with no cam when `cam_required=true` | Holds |
| INV-13 | Gap mismatch/dropped tooth -> `Lost` | `on_tooth`; tests | Gap mismatch returns `None`/wrong angle | Holds |
| INV-14 | Sync loss cuts fuel/spark | `ProtectionController`; `trigger_loss...` test | Sync loss leaves `cut_fuel=false` | Holds in sim |
| INV-15 | Protection is separate and latches | Separate crate; `set`/`clear_latched`; tests | Protection logic embedded in fuel/ignition; latch auto-clears | Holds in software |
| INV-16 | Scheduler validates firing order and bounds | `build_schedule`; tests | Invalid order returns Ok; negative dwell accepted | Holds |
| INV-17 | Protocol rejects bad magic/version/CRC/length | `decode_binary_frame`; 5 tests | Corrupt frame accepted | Holds |
| INV-18 | Calibration edits require paused engine | Service `edit_table`/`mixture_apply` | Running engine accepts edit | Holds in service |
| INV-19 | UI displays unvalidated/blocked status | UI source; preflight page | UI says "safe"/"validated" or hides fault | Mostly holds; DTC mapping bug |
| INV-20 | Safety docs do not claim certification | AUD docs; README | "ISO 26262 compliant"/"plug-and-play" | Holds in AUD docs; check other docs |

## 4. How to falsify specific claims

| Claim to test | Falsification experiment | What would disprove it | Safety note |
|---|---|---|---|
| "No physical output path" | Search all Rust for hardware register/driver calls; inspect `apply_events`; inspect `firmware/stm32`; confirm no reviewed PCB | Any hardware write or driver call | Do not add one to test |
| "Interlock cannot be bypassed by calibration" | Inspect `PHYSICAL_OUTPUT_INTERLOCK_ENGAGED`; run preflight on a synthetic cal | Preflight returns `physical_export_allowed=true` | Do not edit the constant in the repo |
| "Lambda never informs ignition" | Inspect `ecu-ignition` API and grep; run ignition tests | Any lambda/AFR parameter or conversion | Static review is sufficient for the snapshot |
| "Mixture Map cannot auto-apply" | Call `/api/mixture/apply` without `approve`; CLI without `--approve` | Success without approval | Use a copy/throwaway calibration |
| "Stale proposal refused" | Analyze, change calibration, apply old proposal | Apply succeeds | Use `/tmp` copy |
| "Checksum detects tampering" | Copy calibration JSON to `/tmp`, change a VE value, verify | Verify still returns true | Do not edit repo files |
| "Checksum is authenticity" | Edit JSON, recompute SHA-256, verify | Verify returns true **and** system treats it as trusted | This is expected; proves it is not a signature |
| "Gate fails closed" | Inspect `evaluate`; run `every_invalidity_is_explained`; try NaN lambda | Invalid sample allowed | Host test only |
| "STFT bounded" | Run `integrator_never_winds_beyond_limit`; inspect clamp | Trim > 0.25 | Host test only |
| "Protection latches" | Run overtemp/low-oil tests; inspect `clear_latched` | Fault auto-clears without explicit action | Host test only |
| "Trigger detects gap mismatch" | Run `extra_missing_tooth_causes_sync_loss`; inspect `on_tooth` | Mismatch accepted | Host test only |
| "Deterministic simulation" | Run twice with same `--cal`; compare log hashes; run twice without `--cal` and compare calibration checksums | Different frames/checksums | Fresh generation is deterministic at this cut-off; record checksums |
| "UI is buildable" | `npm run typecheck && npm run build` | TypeScript/build errors | Headless Chromium passed; Electron launch not tested |
| "Service blocks physical export" | `/api/preflight`; `/api/health` | `physical_export_allowed=true` or `physical_outputs=true` | Local only |
| "Evo profile is unverified" | `/api/vehicles`; inspect `vehicle.rs` flags | Any flag true without evidence | Do not fill from assumption |
| "Hardware exists" | Inspect `hardware/kicad`; read ERC/DRC; `find` for gerbers | A reviewed package with clean DRC/gerbers exists | Unreviewed schematic/BOM; PCB draft has 14 DRC violations, 499 unconnected; no gerbers |

## 5. Missing evidence checklist

A reviewer should mark each item as **provided**, **missing**, or **not applicable**. "Missing" is not a pass.

### Gate 0 — software

- [x] Full test suite green: 130/130.
- [x] `cargo fmt --check` and clippy clean.
- [x] Firmware no_std skeleton target check passes.
- [x] Full core `no_std` build for `ecu-types`, `ecu-fuel`, `ecu-ignition`, `ecu-scheduler`, `ecu-trigger`, `ecu-protection`.
- [x] UI typecheck/build passes.
- [x] Headless Chromium interaction test (dashboard, maps, Harness upload, Mixture Map, Setup, Diagnostics, Logs, Copilot).
- [ ] Electron launch test (missing).
- [x] Copilot typecheck passes.
- [ ] Live LLM/tool/approval test (missing).
- [x] Deterministic run with a recorded calibration checksum (fixed cal).
- [x] Fresh-generation determinism (starter calibration timestamp is zero; repeat checksums match).
- [ ] Service auth/CSRF or explicit "local-only development" restriction (missing).
- [ ] CI run attestation (workflow defined; not observed running).
- [x] Source committed with a reproducible hash (Git HEAD `27b91df9`; Rust tree clean; local branch not pushed).
- [ ] Independent software review (missing).

### Gate 1 — unpowered hardware

- [ ] Reviewed schematic, PCB, BOM, gerbers, assembly drawing. An unreviewed schematic/BOM and failing PCB draft exist; no gerbers/assembly.
- [ ] Reviewed ERC and DRC report with rule set and exceptions. ERC JSON exists (0 violations from generator) but is unreviewed; no DRC.
- [ ] BOM derating/lifecycle review (missing).
- [ ] Visual/continuity inspection procedure and results (missing).
- [ ] Connector/pinout verification (missing).
- [ ] Independent electrical review (missing).

### Gate 2 — powered bench

- [ ] Firmware build/hash and MCU selection proof (missing).
- [ ] Current-limited supply, fuses, dummy loads, E-stop (missing).
- [ ] Scope/logic-analyser timing evidence (missing).
- [ ] Safe-state/reset/brownout/watchdog tests (missing).
- [ ] Fault injection: open/short/reverse/load dump/sensor loss (missing).
- [ ] Thermal/EMC pre-compliance (missing).

### Gate 3 — vehicle electrical compatibility

- [ ] Exact ECU part number/connector/pinout (unverified).
- [ ] Trigger pattern and sensor types (unverified).
- [ ] Injector/coil/igniter data (unverified).
- [ ] Harness condition and continuity (unverified).
- [ ] ACD/AYC/immobiliser/dashboard/CAN interaction (unverified).
- [ ] Legal/insurance/emissions review (unverified).

### Gate 4-5 — engine/vehicle

- [ ] Bench engine/dyno, timing light, knock instrumentation (missing).
- [ ] Supervised test plan and E-stop (plan only).
- [ ] Independent sign-off (missing).
- [ ] Road/track legal/insurance (missing).

## 6. Reviewer sign-off checklist

| Item | Requirement | Reviewer initial/date |
|---|---|---|
| Identity | Name, organization, role, contact | |
| Independence | No design/ownership/commercial conflict; conflict declared | |
| Scope | Exact documents, source hash, calibration checksum, hardware revision | |
| Commands | Commands run, toolchain, environment, raw outputs attached | |
| Findings | Open findings listed with severity and owner | |
| Safety claims | Every claim checked against evidence; no unsupported claim accepted | |
| Test status | Green/red stated exactly; failing tests not waived silently | |
| Hardware status | No schematic/PCB/firmware claim without artefacts | |
| Evo status | Pinout/trigger/injector/harness marked unverified unless evidence attached | |
| Gate decision | Pass / conditional / fail; conditions and expiry | |
| Signature | Signed record, not an email "looks good" | |

**No self-promotion.** A reviewer cannot approve a gate for work they authored. A project owner cannot approve their own gate. A "conditional pass" must list the conditions, evidence required, and an expiry; it is not a pass for physical use.

## 7. Questions to ask the team

1. What is the exact Evo VIII ECU part number, and where is the photograph/label record?
2. What is the verified crank/cam trigger pattern (tooth count, gap, cam phase, sensor type)? Show the scope capture.
3. What are the verified injector flow, dead time, resistance, inductance and rated pressure? Show the bench data.
4. What is the verified coil/igniter topology and dwell data? Show the measurement.
5. What is the fuel pressure/regulator strategy and verified sensor transfer function?
6. What wideband controller is used, how is it timestamped, and what is its latency/aging behavior?
7. Where is the hardware watchdog, and what is the reset safe state? Show the fault injection.
8. What is the measured spark/injection jitter at 7000 rpm and at temperature/voltage corners?
9. What happens on crank loss, cam loss, extra tooth, brownout, reverse polarity, and CAN loss? Show the test.
10. What is the emergency stop, and does it remove actuator power independently of software?
11. What is the calibration trust model? Is the "verified" ignition reference signed or just a JSON label?
12. What is the durable audit trail for apply/undo, and who is the actor?
13. What is the legal/insurance/emissions basis for any vehicle use?
14. Which claims in README/docs are stale, and who owns correcting them?
15. What independent review has been performed, by whom, and against which source hash?

## 8. Prohibited actions during review

- **Do not connect the software or a prototype board to the Evo VIII or any vehicle.**
- Do not energise real injectors with fuel, real coils, or a fuel pump.
- Do not bypass, edit, or "temporarily disable" the physical output interlock.
- Do not add a lambda input to ignition, auto-advance, or a write path to test a theory.
- Do not expose the studio service beyond localhost or disable CORS/auth "for convenience".
- Do not spoof ACD/AYC/immobiliser/dashboard CAN frames or bypass security.
- Do not flash an OEM ECU, defeat an immobiliser, or alter emissions equipment.
- Do not run engine/dyno tests without a supervised plan, E-stop, fire safety and insurance.
- Do not edit safety documents, test evidence, or calibration checksums to make a gate pass.
- Do not use an LLM/copilot output as safety evidence or as an approval.
- Do not treat a green KiCad DRC, a passing unit test, or a screenshot as functional-safety proof.

## 9. Stop conditions / red flags

Stop the review and escalate if any of the following is observed:

| Red flag | Why |
|---|---|
| Any physical actuator energises during a "virtual" test | Interlock/path violation |
| Test suite is reported green while a known test fails | Evidence integrity |
| Calibration checksum is recomputed to hide an edit | Tamper/authenticity failure |
| "Verified" ignition reference has no citation/licence/measurement | Unsupported ignition evidence |
| Pinout/trigger values appear without a source or measurement | Fabrication risk |
| Watchdog/CAN health is hard-coded true in a safety claim | False assurance |
| Protection thresholds are described as Evo limits without data | Unvalidated safety limit |
| Service is reachable remotely or CORS is permissive in a hardware context | Unauthorised control |
| A reviewer is asked to sign without raw evidence | Process failure |
| Vehicle test is proposed before Gate 3 | Safety/legal violation |

## 10. Evidence package a reviewer should receive

1. This AUD set and the exact source manifest hash.
2. Raw test logs (not summaries), including any failed or skipped test.
3. CLI logs/CSV/report/calibration checksums for each scenario.
4. UI build log and a headless Chromium interaction recording; Electron launch is still missing (if claiming UI).
5. Calibration JSON and checksum for every claim.
6. Hardware schematic/PCB/BOM/gerbers/ERC/DRC (if claiming hardware).
7. Firmware source/hash, map file, build log, debug probe output (if claiming firmware).
8. Scope/logic-analyser raw files with probe/timebase/setup metadata (if claiming timing).
9. Fault-injection records and photos (if claiming protection).
10. Evo vehicle evidence: ECU label, connector, pinout continuity, trigger scope, injector/coil measurements, harness inspection (if claiming compatibility).
11. Independent reviewer sign-off with scope, findings and conditions.
12. A list of known limitations and stale documents.

## 11. Reviewer conclusion template

```text
Reviewer:
Organization:
Date:
Source hash / calibration checksum:
Scope reviewed:
Commands run / raw evidence:
Findings (severity, owner):
Gate decision: PASS / CONDITIONAL / FAIL
Conditions (if conditional):
Expiry:
Independent? Yes/No; conflicts:
Signature:
```

A review is not a certification. It is evidence that a specific reviewer examined a specific snapshot and found specific facts. Keep it tied to the hash and checksum.
