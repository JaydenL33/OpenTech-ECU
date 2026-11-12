# Release Gates (Gate 0-5)

| Field | Value |
|---|---|
| Document ID | AUD-08 |
| Status | Gate definitions and current status; not a release approval |
| Date | 2026-11-12 |
| Evidence cut-off | 2026-11-12T05:45Z, working tree |
| Source manifest | `733355ccfefcc18b31806f1493b9d7b8de8f4504b841e4bb0a83335690c82397` |
| Current status | **Gate 0 host software evidence: PASS, scoped. Gate 1-5: BLOCKED.** |
| Related | AUD-01 through AUD-10 |

> **No automatic promotion.** A gate changes status only when a named reviewer records a decision against an exact source hash, calibration checksum, hardware revision and evidence bundle. Code existence, a green build, a passing test suite, a demo, or an LLM summary cannot promote a gate. A regression resets the affected gate until re-reviewed.
>
> **Gate 0 pass is not a certification.** It means the implemented host behaviour has reproducible evidence. It does not authorise hardware, engine, or vehicle use, and it does not establish timing, EMC, electrical safety, or Evo VIII compatibility.

## 1. Canonical gate model (must match `ECU_MASTER_BRIEF.md`)

| Gate | Name | Purpose | Physical actuation allowed? | Current status |
|---:|---|---|---|---|
| 0 | Software simulation | Prove host software behaviour, determinism, invariants and evidence | No | **PASS, scoped** |
| 1 | Unpowered hardware verification | Prove the board exists, is assembled correctly, and is electrically safe to power | No | **BLOCKED** |
| 2 | Powered bench verification | Prove firmware, timing, safe state, protection and fault response with dummy loads | Dummy loads only, no fuel/engine | **BLOCKED** |
| 3 | Vehicle electrical compatibility | Prove safe interface with the vehicle harness/systems, engine not running | No engine; staged checks | **BLOCKED / UNVERIFIED** |
| 4 | Controlled engine testing | Prove start/idle/fuel/ignition/protection on a bench engine or dyno under supervision | Yes, supervised bench/dyno | **BLOCKED** |
| 5 | Vehicle operation | Prove reliable, legal, insured vehicle use | Yes, road/track | **BLOCKED** |

**Numbering rule.** Use exactly this 0-5 sequence in every AUDIT document and cross-reference. Do not substitute "bench electronics" for Gate 1, "HIL" for Gate 2, or "vehicle integration" for Gate 3. Gate 1 is **unpowered**; Gate 2 is **powered bench**; Gate 3 is **vehicle electrical compatibility without engine operation**.

## 2. Gate 0 — Software simulation

### 2.1 Objective

Demonstrate that the host-side virtual ECU, calibration workflow, protocol, service, UI, copilot typecheck and firmware skeleton build behave as specified; that safety invariants hold; and that evidence is reproducible. No physical actuator.

### 2.2 Entry criteria

- Source snapshot identified by commit or manifest hash; Cargo lock recorded.
- Build toolchain and environment recorded.
- Test/acceptance criteria defined.
- No physical output path present or all physical outputs are forced off.

### 2.3 Exit criteria and current evidence

| # | Criterion | Required evidence | Current status |
|---:|---|---|---|
| 0.1 | Formatting clean | `cargo fmt --all -- --check` exit 0 | **PASS** |
| 0.2 | Clippy clean with `-D warnings` | Raw log | **PASS** |
| 0.3 | Full host test suite green | `cargo test --workspace` | **PASS**: 130/130 |
| 0.4 | Firmware skeleton target check | `thumbv7em-none-eabihf`, no-default-features | **PASS** |
| 0.5 | Full core `no_std` portability | Per-crate checks | **PASS**: ecu-types, ecu-fuel, ecu-ignition, ecu-scheduler, ecu-trigger, ecu-protection pass |
| 0.6 | CLI determinism for a recorded calibration | Two runs, same log hash | **PASS** for fixed calibration |
| 0.7 | Fresh-generation determinism or documented seed/time policy | Repeat-run hashes | **PASS**: starter calibration timestamp is deterministic (0); repeat generation yields identical calibration checksum |
| 0.8 | Mixture Map apply requires approval/checksum/revision | CLI/API refusal; tests | **PASS** |
| 0.9 | Protection latching/recovery tests | Test log | **PASS** |
| 0.10 | Trigger sync/loss tests | Test log | **PASS** (known timing gaps noted in AUD-02/AUD-04) |
| 0.11 | UI typecheck/build | Raw log; `dist/` | **PASS** |
| 0.12 | Copilot typecheck | `tsc --noEmit` | **PASS** |
| 0.13 | UI browser/Electron interaction | Recording/test log | **PARTIAL**: headless Chromium smoke test passed for dashboard, maps, Harness, Mixture Map, Setup, Diagnostics, Logs and Copilot; Electron not launched |
| 0.14 | Live copilot/LLM/tool test | Runtime log/evals | **MISSING** |
| 0.15 | Service smoke test, no physical outputs | Health/preflight logs | **PASS** (local) |
| 0.16 | Service security posture documented | Review record | **PARTIAL**: no auth, permissive CORS, localhost bind |
| 0.17 | Invariants INV-01..INV-20 reviewed | Review record | Partial; see AUD-07 |
| 0.18 | Known gaps/limitations documented | AUD set | Present |
| 0.19 | Independent software review | Signed record | **MISSING** |
| 0.20 | Source committed/reproducible | Commit or manifest hash | **PASS**: local branch source commit `27b91df90695eb7d1a8870b552befe3fcb400577`; Rust tree clean; tested manifest hash recorded in AUD-05 |

**Gate 0 decision: PASS for host software evidence, scoped.** Formatting, clippy, 130 tests, UI build, copilot typecheck, firmware skeleton target check, core no_std checks, CLI determinism and apply refusal all pass. The pass is conditional on the documented limitations: headless Chromium only (Electron not launched), no live LLM test, and no independent review. It does **not** authorise Gate 1+.

### 2.4 Conditions for maintaining Gate 0

1. Re-run the full command set on every source change; a regression resets Gate 0.
2. Keep the firmware skeleton interlock false; no physical output path.
3. Do not claim target-hardware behaviour from a host no_std check.
4. Add an Electron launch test, accessibility checks and copilot behavioral evals before claiming UI/copilot readiness.
5. Commit the source or keep the manifest hash current.
6. Obtain independent software review; record findings.

## 3. Gate 1 — Unpowered hardware verification

### 3.1 Objective

Prove a physical ECU board exists, is assembled correctly, has no obvious electrical faults, and is safe to power for the first time. No battery, no actuators, no engine.

### 3.2 Entry criteria

- Gate 0 host evidence current for the firmware/software revision intended for the board.
- Schematic, PCB, BOM, gerbers, drill, assembly drawing, build notes.
- ERC/DRC report with rule set, exceptions and reviewer rationale.
- Board revision/serial number; assembly inspection record.
- An unreviewed bench-prototype schematic package exists at `hardware/kicad/evo8-bench-ecu/`; it has an unrouted PCB draft with 14 DRC violations (13 dangling vias + 1 clearance) and 499 unconnected items; it cannot be used as Gate 1 evidence until reviewed and completed.

### 3.3 Exit criteria

| # | Criterion | Required evidence | Current status |
|---:|---|---|---|
| 1.1 | Schematic/PCB/BOM/gerbers exist and match reviewed revision | Design package with hashes | **BLOCKED**: unreviewed schematic/BOM; PCB draft has 14 DRC violations (13 dangling vias + 1 clearance) and 499 unconnected items; no gerbers |
| 1.2 | ERC/DRC completed; violations resolved/accepted | Tool report; reviewer sign-off | **BLOCKED**: ERC JSON exists (0 violations from generator) but not independently reviewed; no DRC |
| 1.3 | BOM derating/lifecycle review | BOM review | **BLOCKED** |
| 1.4 | Visual inspection | Photos/checklist | **BLOCKED** |
| 1.5 | Continuity/rails/no shorts | Measurement record | **BLOCKED** |
| 1.6 | Power-input resistance/reverse assembly | Measurement record | **BLOCKED** |
| 1.7 | Connector/keying/pinout verified | Continuity record; second person | **BLOCKED / UNVERIFIED** |
| 1.8 | Outputs default off with no power | Schematic review; measurement | **BLOCKED** |
| 1.9 | Independent electrical review | Signed record | **MISSING** |

**Gate 1 decision: BLOCKED.** An unreviewed bench schematic/BOM and an unrouted PCB draft exist, but DRC fails (14 violations (13 dangling vias + 1 clearance), 499 unconnected), there are no gerbers/assembly, and there is no independent review. It does not satisfy Gate 1.

## 4. Gate 2 — Powered bench verification

### 4.1 Objective

Power the board from a current-limited supply with fuses and dummy loads, prove safe state, timing, protection and fault response, and produce measured evidence. No fuel, no engine, no vehicle.

### 4.2 Entry criteria

- Gate 1 passed.
- Bench fixture, current-limited supply, fuses, dummy injector/ignition loads, E-stop, scope/logic analyser.
- Firmware image/build hash, calibration checksum, debug probe, recovery method.
- AUD-09 test plan reviewed; stop conditions/emergency procedure briefed.
- Two-person rule for first power per local policy.

### 4.3 Exit criteria

| # | Criterion | Required evidence | Current status |
|---:|---|---|---|
| 2.1 | Power-up current/rails within limits; no overheating | Scope/DMM/thermal | **BLOCKED** |
| 2.2 | Reset/brownout/watchdog outputs off | Scope; reset log | **BLOCKED** |
| 2.3 | Trigger sync across RPM with simulated crank/cam | Scope/logic analyser | **BLOCKED** |
| 2.4 | Injector pulse/timing/flyback within SOA | Current probe/scope | **BLOCKED** |
| 2.5 | Ignition dwell/spark timing within budget | Scope/logic analyser | **BLOCKED** |
| 2.6 | Rev limiter/protection cuts behave | Scope; fault injection | **BLOCKED** |
| 2.7 | Sensor open/short/out-of-range safe | Fault injection | **BLOCKED** |
| 2.8 | CAN/UI load does not disturb control timing | Bus flood/UI stall + scope | **BLOCKED** |
| 2.9 | Thermal soak at defined load/ambient | Chamber/thermocouples | **BLOCKED** |
| 2.10 | EMC pre-compliance (if claimed) | Test report | **BLOCKED** |
| 2.11 | Anomalies closed/accepted | Review record | **BLOCKED** |
| 2.12 | Independent bench review | Signed record | **MISSING** |

**Gate 2 decision: BLOCKED.** A no_std skeleton exists, but no HAL, timer capture/compare, drivers, watchdog integration, flash image, bench fixture or measured timing exists.

## 5. Gate 3 — Vehicle electrical compatibility

### 5.1 Objective

Prove the ECU can interface with the vehicle's electrical system **without engine operation**, with no damage and no unsafe actuation. This gate is about wiring, sensors, power, grounds, CAN/factory systems, not tuning.

### 5.2 Entry criteria

- Gates 0-2 passed for the exact hardware/firmware.
- Exact Evo VIII ECU part number, connector, pinout, harness condition documented.
- Trigger pattern, sensor types, injector/coil topology measured.
- Vehicle immobilized, battery disconnected for continuity checks; approved procedure.
- Owner authorization and insurance/legal review for vehicle work.
- Back-probe/adapter method that does not damage the harness.

### 5.3 Exit criteria

| # | Criterion | Required evidence | Current status |
|---:|---|---|---|
| 3.1 | ECU part number/connector identified | Photos/label/service data | **UNVERIFIED** |
| 3.2 | Pinout verified by continuity, second person | Pin-to-pin record | **UNVERIFIED** |
| 3.3 | Crank/cam trigger verified by scope | Raw scope at cranking | **UNVERIFIED** |
| 3.4 | Sensor transfer functions/polarity verified | Measurement/calibration | **UNVERIFIED** |
| 3.5 | Injector resistance/inductance/flow verified | Bench data | **UNVERIFIED** |
| 3.6 | Coil/igniter topology verified | Measurement/service data | **UNVERIFIED** |
| 3.7 | Power/ground/transient compatibility verified | Measurement/design review | **BLOCKED** |
| 3.8 | CAN/ACD/AYC/immobiliser/dashboard documented; no spoofing | CAN capture/review | **UNVERIFIED** |
| 3.9 | No factory security bypass | Review record | **PROHIBITED** |
| 3.10 | Legal/emissions/insurance review | Jurisdiction review | **UNVERIFIED** |
| 3.11 | No unintended actuation with ECU connected, engine off | Test record | **BLOCKED** |
| 3.12 | Independent vehicle electrical review | Signed record | **MISSING** |

**Gate 3 decision: BLOCKED / UNVERIFIED.** All Evo-specific evidence is missing. Do not connect the vehicle.

## 6. Gate 4 — Controlled engine testing

### 6.1 Objective

Run a real engine on a bench/dyno under supervision, verify start, idle, fuelling, ignition, protection and shutdown, and produce engine-specific calibration evidence. No road use.

### 6.2 Entry criteria

- Gates 0-3 passed.
- Bench engine/dyno with fuel, fire safety, ventilation, E-stop, spill kit, first aid.
- Timing light, calibrated knock instrumentation/audio, wideband, pressure/temperature sensors.
- Verified calibration for the exact engine; all maps estimates until proven.
- Supervised test plan with abort/stop conditions, roles, communications.
- Insurance/legal approval for bench running.

### 6.3 Exit criteria

| # | Criterion | Required evidence | Current status |
|---:|---|---|---|
| 4.1 | Engine starts/idles without unsafe condition | Log/scope/video | **BLOCKED** |
| 4.2 | Base ignition offset verified with timing light | Measurement record | **BLOCKED** |
| 4.3 | Knock detection validated; no unsafe advance | Knock data/audio review | **BLOCKED** |
| 4.4 | Fuel pressure/injector data validated | Bench/engine data | **BLOCKED** |
| 4.5 | Protection cuts/derates under fault injection | Test record | **BLOCKED** |
| 4.6 | Shutdown/E-stop verified | Test record | **BLOCKED** |
| 4.7 | Thermal/mechanical limits respected | Logs/thermocouples | **BLOCKED** |
| 4.8 | Calibration review by qualified tuner | Signed record | **MISSING** |
| 4.9 | No unresolved safety findings | Review record | **MISSING** |

**Gate 4 decision: BLOCKED.** No engine, dyno, timing light, knock instrumentation or calibration evidence exists.

## 7. Gate 5 — Vehicle operation

### 7.1 Objective

Operate the vehicle on road/track with a validated, legal, insured configuration and demonstrated reliability.

### 7.2 Entry criteria

- Gates 0-4 passed.
- Vehicle mechanically/electrically inspected; harness/fuel/ignition compatible.
- Emissions/roadworthiness/insurance/legal review for jurisdiction and use.
- Validated calibration/protection; data logging; emergency plan.
- Driver/operator briefing; no passenger unless approved.

### 7.3 Exit criteria

| # | Criterion | Required evidence | Current status |
|---:|---|---|---|
| 5.1 | Road/track legal and insured | Documents | **BLOCKED** |
| 5.2 | Reliability demonstration | Test logs | **BLOCKED** |
| 5.3 | Emissions/roadworthiness | Test report | **BLOCKED** |
| 5.4 | Protection/limp behaviour validated | Test record | **BLOCKED** |
| 5.5 | Incident/emergency procedure | Plan/rehearsal | **BLOCKED** |
| 5.6 | Independent final review | Signed record | **MISSING** |

**Gate 5 decision: BLOCKED.** No legal, insurance, reliability or vehicle evidence exists.

## 8. Current evidence status

| Evidence | Available? | Location / note |
|---|---|---|
| Rust tests | Yes, green | 130 tests; AUD-05 |
| Formatting | Yes | `cargo fmt --check` pass |
| Clippy | Yes | `-D warnings` pass |
| Firmware skeleton target check | Yes | `thumbv7em-none-eabihf` pass; skeleton only |
| Full core `no_std` | Pass for core control crates | All six pass; `ecu-protocol`/`ecu-lambda` remain std-only; firmware skeleton has no HAL/drivers |
| CLI determinism | Yes, fixed and fresh | `mixture_sweep` PASS, 2184 frames; fresh `idle_warmup` checksum stable |
| Mixture Map analysis/apply | Yes, host | 3 cells; approval/CLI refusal |
| UI typecheck/build | Yes | PASS; headless Chromium smoke test passed; Electron not launched |
| Copilot typecheck | Yes | PASS; live LLM/tools not tested |
| Service smoke | Yes, local | Health/preflight/status; no auth |
| CI workflow | Defined | `.github/workflows/opentech-ci.yml`; not observed running |
| KiCad/schematic/PCB/BOM/gerbers | Partial, unreviewed | Unreviewed schematic/BOM/ERC; PCB draft fails DRC; no gerbers |
| Firmware HAL/timer/driver | No | Skeleton only |
| Bench fixture/E-stop | No | Plan only (AUD-09) |
| Evo pinout/connector/trigger/harness | No | All unverified |
| Independent review | No | None |

## 9. Approval record template

| Field | Entry |
|---|---|
| Gate | 0 / 1 / 2 / 3 / 4 / 5 |
| Candidate | Software version / firmware hash / hardware serial |
| Source hash | Git commit or manifest SHA-256 |
| Calibration | ID, revision, checksum |
| Hardware revision | Schematic/PCB/BOM revision and serial |
| Evidence bundle | Raw logs, scope files, photos, reports, hashes |
| Reviewer(s) | Name, organization, role |
| Independence | Yes/No; conflicts declared |
| Findings | Open findings with severity/owner |
| Decision | PASS / CONDITIONAL / FAIL |
| Conditions (if conditional) | Exact evidence required, owner, due date |
| Expiry | Date or "until source/hardware changes" |
| Signature/date | Signed record |

**Approval rules**

1. Reviewer must see raw evidence, not only a summary.
2. A conditional pass must not permit physical actuation unless explicitly limited to non-actuating work.
3. Any source, calibration, hardware or harness change invalidates the gate until re-reviewed.
4. Gate decisions are tied to exact hashes/checksums; "latest" is not a revision.
5. A reviewer cannot approve their own work or a gate with a conflict.
6. The project owner cannot self-promote a gate.
7. A failed gate can be re-reviewed; do not overwrite the failure record.

## 10. Regression and change control

| Change | Required action |
|---|---|
| Any Rust source change | Re-run Gate 0 command set; re-hash; review safety invariants |
| Calibration schema/algorithm change | Re-run Mixture Map/trim tests; re-check apply/undo; new checksum |
| Interlock/ignition/gate/protection change | New safety review; not a normal PR |
| Firmware change | Re-run Gate 2 timing/safe-state tests; new firmware hash |
| Schematic/PCB/BOM change | Re-run Gate 1 review; new hardware revision |
| Harness/connector change | Re-run Gate 3 continuity/pinout; new record |
| New sensor/actuator | Re-validate transfer function/driver; update FMEA |
| Documentation change | Re-check claims vs evidence; do not "fix" a gate by editing prose |

## 11. Current release statement

> **OpenTech-ECU v0.1 Gate 0 host software evidence passes for the implemented behaviour: formatting, clippy, 130 tests, firmware skeleton target check, CLI determinism, Mixture Map approval/apply refusal, UI build and copilot typecheck. This is not a certification and does not authorise Gate 1+. Gate 1 is blocked because only an unreviewed bench schematic/BOM and failing PCB draft exist, with no gerbers; Gate 2 is blocked because the firmware is a no_std skeleton with no HAL/timing/drivers/watchdog; Gate 3 is blocked because all Evo pinout/connector/trigger/harness evidence is unverified; Gates 4-5 are blocked because no engine or vehicle validation exists. Do not connect this software or any prototype to a vehicle. Do not treat any map as engine-safe.**

The next step is not a vehicle test. It is: keep the Gate 0 evidence current, complete a software review, produce a reviewed hardware design and bench fixture, and only then seek Gate 3 evidence.
