# Engine Control Safety Argument (v0.1, host simulation)

| Field | Value |
|---|---|
| Document ID | AUD-01 |
| Status | Independent audit draft. Not a certification, not a safety case for an engine, not an approval to connect hardware. |
| Audit date | 2026-11-12 |
| Evidence cut-off | 2026-11-12T05:45Z, working tree |
| Repository | `/home/jlee/OpenTech-ECU` |
| Git HEAD | `fdf25420bb6925389f17a0e2d1fa365159acba3d` (source commit `041be84f`; docs/audit commits after; Rust tree clean at cut-off) |
| Rust source manifest | SHA-256 `9837ba880320ae30dc4913a7e077b6b1b5f2b19b528e0630ab1d069ea9dec724` over all `.rs` files under `ecu/crates`, `ecu/apps/ecu-sim-cli`, `ecu/apps/studio-service`, `ecu/firmware/stm32` |
| Cargo lock | SHA-256 `71ecae98186c1bc59d46293bae54f3dc2db3cca86bba79c0e77f9b837a2f329e` |
| Test status at cut-off | 128 tests (126 unit + 2 integration): 128 passed, 0 failed; `cargo fmt` and clippy pass; firmware no_std skeleton target check passes. See AUD-05. Gate 0 host software evidence is **PASS, scoped**; not a certification |
| Related documents | AUD-02 through AUD-10; `docs/SAFETY-CASE.md`; `docs/FORMULAS.md`; `ECU_MASTER_BRIEF.md` |

> **Scope statement.** This document audits the host-side Rust code in the snapshot above. It does not audit firmware, a PCB, a harness, a sensor installation, an engine, or a vehicle. The only defensible v0.1 claim is narrow: the audited host code has no path that energises a physical injector or ignition coil. That claim is a property of this source snapshot, not a permanent property of the project.

> **Unverified target.** The target vehicle is described as a JDM Mitsubishi Lancer Evolution VIII GSR, CT9A, 4G63T, six-speed manual. The exact ECU part number, connector, pinout, crank/cam trigger pattern, injector data, ignition topology and harness are **UNVERIFIED**. Nothing in this document supplies them. The repository's own `ecu-calibration/src/vehicle.rs` marks them `None`/`false` and blocks physical configuration.

## 1. Safety concept

The v0.1 safety concept is **containment by absence of an actuator path**, supported by software invariants and review gates. It is not a functional-safety concept in the ISO 26262 or IEC 61508 sense, and it has no independent monitor, lockstep core, external watchdog, or certified toolchain.

| Layer | Mechanism in this snapshot | Evidence | Fundamental limitation |
|---|---|---|---|
| L0 Physical | No MCU HAL, timer capture/compare, injector/coil driver, or harness; a no_std firmware skeleton exists but forces outputs off; an unreviewed bench schematic/BOM and unrouted PCB draft exist, with failing DRC and no gerbers | `ecu/firmware/stm32`; `PHYSICAL_OUTPUTS_AUTHORISED=false`; `hardware/kicad/evo8-bench-ecu/` is UNREVIEWED | A future commit can add a path; this audit expires |
| L1 Interlock | `PHYSICAL_OUTPUT_INTERLOCK_ENGAGED = true` in `ecu-calibration/src/preflight.rs` | Constant; preflight reports physical export blocked | Compile-time constant, not a hardware inhibit; one edit removes it |
| L2 Preflight | Calibration shape, checksum, ignition evidence, target-lambda bounds | `preflight()` and tests | Warnings do not block `hardware_candidate`; metadata can be forged |
| L3 Ignition evidence | No lambda input in `ecu-ignition`; synthetic map rejected for physical use | API shape; `recommend_from_reference`; tests | "Verified reference" is a caller-supplied label, not an authenticated source |
| L4 Fuel-learning gates | Shared gate for STFT, LTFT and offline Mixture Map | `ecu-lambda/src/gate.rs`; tests | Detection thresholds are unvalidated; several inputs are booleans owned by callers |
| L5 Protection | Separate `ecu-protection` state machine, DTCs, latching, freeze frames; N-sample sensor-threshold latch confirmation | 14 tests; simulator integration; confirmation tests in §7 | Same process, no independent hardware; thresholds are placeholders; cut reactions remain immediate; integrity faults latch immediately |
| L6 Human approval | Mixture Map apply requires `approve=true`, matching checksum/revision, paused engine | `mixture_apply`; CLI `--approve`; tests | Local unauthenticated HTTP; history is in memory |
| L7 Release gates | Gate 0-5 process, no automatic promotion | AUD-08 | Process only; not enforced by code or CI |

**Defence-in-depth caveat.** Layers L1-L6 are not independent. They run in one process, share the same calibration object, and are not protected by a separate safety monitor. A software defect, a bad merge, or a deliberate edit can bypass several layers at once. The argument therefore rests primarily on L0 (no actuator path), not on the runtime controls.

## 2. System boundary and actuator path

| Element | Present? | Path to a physical actuator? | Notes |
|---|---|---|---|
| Rust control crates | Yes | No | Compute fuel, ignition, trigger state, trims, protection decisions |
| Virtual ECU (`ecu-simulator`) | Yes | No | Plant model, simulated sensors, simulated schedule; 1 ms host loop |
| Studio service | Yes | No | Local HTTP/WebSocket, calibration editing, simulation control |
| Tuning UI | Yes, host build | No | React/Vite + Electron main; Electron launch not tested; no hardware transport |
| Firmware | Yes, no_std skeleton | No | `firmware/stm32`; no HAL, timer capture/compare, injector/ignition driver, watchdog integration, or flash image |
| PCB / harness | Unreviewed schematic/BOM and failing PCB draft | No | `hardware/kicad/evo8-bench-ecu/`; DRC 14 violations, 499 unconnected, no gerbers, harness or connector definition |
| Physical injector/coil | No | No | No code path; `PHYSICAL_OUTPUT_INTERLOCK_ENGAGED` is true |

The simulator's "injection" and "spark" events are calls in `apply_events` that update model state (`fuel_cycle_kg`, `sparks_this_cycle`). They do not call a GPIO, timer, CAN transmit API, or driver. This is confirmed by source inspection, not by a runtime sandbox.

## 3. Hazard analysis (qualitative FMEA-style)

Ratings are engineering judgments for prioritisation, not quantitative risk acceptance. **S** = severity if the hazardous event reaches a physical engine (S1 catastrophic, S2 critical, S3 major, S4 minor). **L** = likelihood in the intended development use (L1 remote, L2 unlikely, L3 possible, L4 likely, L5 frequent). **D** = detectability by the current host code (D1 certain, D2 high, D3 moderate, D4 low, D5 none). "Host effect" is what v0.1 can actually do; "physical consequence" applies only if a future actuator path exists.

| ID | Hazard | Cause | Host effect in v0.1 | Physical consequence if actuated | S | L | D | Current control | Residual risk / gap |
|---|---|---|---|---|---|---|---|---|---|
| H-01 | Physical actuation despite virtual-only status | Future code adds a driver, or interlock is edited | None; no actuator API; firmware skeleton forces outputs off | Uncontrolled fuel/spark/throttle | 1 | 1 | 2 | No hardware driver; `PHYSICAL_OUTPUTS_AUTHORISED=false`; interlock; preflight | Depends on change control and review; not technically enforced against deliberate edits |
| H-02 | Ignition advance inferred from lambda | Developer adds lambda/AFR as an ignition input | None; `ecu-ignition` has no lambda parameter | Knock, detonation, engine destruction | 1 | 2 | 2 | API absence; evidence policy; synthetic table blocked | A future edit can violate the invariant; no runtime test enforces absence |
| H-03 | False-lean fuel addition | Exhaust leak, misfire, stuck-lean sensor, air injection, sensor not ready | Wrong VE proposal or trim in simulation | Over-fuelling, rich misfire, bore wash, catalyst damage | 2 | 3 | 2 | Shared gate rejects flags; stuck heuristic; confidence | Leak/misfire detection is not implemented; flags are caller-supplied |
| H-04 | Wrong fuel mass from injector data | Wrong flow, dead time, rated pressure, density, stoich | Plausible but wrong PW in simulation | Lean/rich mixture, detonation or misfire | 2 | 3 | 3 | Bounds, pressure compensation, dead-time curve | No flow bench or on-engine verification; preflight only warns |
| H-05 | Trim windup or oscillation | Sensor lag, aggressive gains, persistent error | Bounded STFT/LTFT in simulation | Unstable mixture, hidden correction | 2 | 3 | 2 | Bounds, conditional integration, deadband, freeze/decay | Gains and sensor dynamics not validated on hardware |
| H-06 | Stale or tampered calibration | Bit flip, partial write, wrong revision, deliberate edit | Wrong fuel/ignition data in simulation | Wrong fuel/spark | 1 | 2 | 2 | SHA-256 checksum; revision/parent; stale-proposal rejection | Hash is unkeyed; no signature; history is in memory; no flash journal |
| H-07 | Trigger mis-sync or wrong phase | Wrong gap config, missing/extra tooth, cam loss, noise | Wrong angle or sync loss in simulation | Spark at wrong crank angle, valve/piston damage | 1 | 2 | 2 | Gap check, tooth-count check, cam latch, timeout | Cam loss after phase latch is not detected; extra short edge can shift angle for one revolution |
| H-08 | Crank signal loss with angle extrapolation | Broken sensor, connector, wheel damage | Model continues on extrapolated angle until timeout | Spark/fuel continue at predicted angle; late cut | 1 | 3 | 2 | `tick()` timeout (default 250 ms); sync DTC | 250 ms is many revolutions at high RPM; timeout is not speed-dependent |
| H-09 | Protection false trip | Single-sample spike, noise, threshold too tight | Sudden cut/derate in simulation | Loss of power, traffic hazard, engine stall | 2 | 3 | 2 | Latching, freeze frame, configurable limits; sensor-threshold latch confirmation fixed in host code (§7 tests) | N consecutive samples confirm sensor-threshold latches; instantaneous cut/derate reactions and unvalidated thresholds remain |
| H-10 | Protection missed fault | Placeholder thresholds, blind sensor, stuck signal | Fault not detected in simulation | Engine damage, overboost, lean burn | 1 | 3 | 4 | DTCs and limits | Thresholds are not Evo limits; sensor plausibility is limited |
| H-11 | Watchdog failure | MCU hang, clock failure, firmware bug | `watchdog_ok` is hard-coded `true` in simulator; skeleton accepts a watchdog fault input but has no hardware integration | Outputs remain at last state; no reset | 1 | 2 | 5 | DTC exists; skeleton fail-closed path | No hardware watchdog; no HAL/timer integration; not testable on target |
| H-12 | Power transient / brownout | Cranking sag, load dump, reverse polarity, ground offset | None; simulation only | MCU reset, actuator glitch, driver damage | 1 | 3 | 3 | None | Electrical requirements only; see AUD-03 |
| H-13 | Unauthorised calibration apply | Any local process or browser page posts to service | Calibration changed in simulation | Wrong fuel/spark if hardware existed | 2 | 2 | 2 | Pause + `approve=true` + checksum/revision | No auth; CORS permissive; localhost bind only; no actor identity |
| H-14 | Protocol corruption or spoofing | Bit flip, wrong version, forged telemetry | Wrong display/decision in simulation | Wrong control action if used for actuation | 2 | 2 | 2 | CRC32 payload, magic/version, length | Header not CRC-covered; no authentication; JSON has no CRC |
| H-15 | Scheduler overflow or event collision | More events than buffer, overlapping dwell/injection, bad firing order | Missing/duplicate simulated events | Wrong spark/injection timing | 2 | 2 | 2 | Fixed array, validation, 45% dwell cap | `push()` silently drops at capacity; no overlap check; `TooManyEvents` unreachable |
| H-16 | Knock retard applied twice | Previous and current protection decisions both subtracted | Over-retard in simulation | Excessive retard, heat, poor torque | 2 | 3 | 3 | Knock retard bounded 0-6° | Code-reading finding; no dedicated test; see AUD-02 FM-IGN-06 |
| H-17 | Mixture Map misattribution | Wrong exhaust delay, bank mismatch, combined trim label | Wrong VE proposal | Wrong fuel table | 2 | 3 | 2 | Time alignment, bank filter, confidence | `mean_ltft` is always 0 and `mean_stft` is combined; limited auditability |
| H-18 | Evo pinout/trigger/harness error | Unverified vehicle data | None; physical configuration blocked | Wrong wiring, damaged ECU, wrong timing | 1 | 3 | 5 | `vehicle.rs` flags; Gate 3 | No pinout/connector/trigger evidence exists |
| H-19 | Immobiliser/security bypass | Attempt to make a replacement ECU run | None | Theft/stranded vehicle, legal breach | 2 | 1 | 3 | Explicitly prohibited in `vehicle.rs` and docs | No integration strategy; must remain prohibited |
| H-20 | Operator treats estimate as validated | Synthetic VE/ignition/target tables | Wrong tune accepted in simulation | Engine damage | 2 | 3 | 2 | Provenance, UI warnings, preflight | Warnings can be ignored; no independent sign-off |

## 4. Safety requirements (ECS-SR)

These are requirements for the audited software and for any future hardware programme. "Status" uses the project labels: Implemented (host code plus at least one test), Partial, Simulated, Planned, Blocked, Unverified. No requirement is certified.

| ID | Requirement | Rationale | Verification method | Status |
|---|---|---|---|---|
| ECS-SR-001 | v0.1 shall have no code path that can energise a physical injector, coil, fuel pump, throttle, or other actuator. | Primary containment | Source inspection; no hardware driver/PCB; firmware skeleton forces outputs off; preflight | Implemented (host + skeleton), bounded by snapshot |
| ECS-SR-002 | The physical output interlock shall be a single explicit constant and shall be reported by preflight; changing it shall require a new safety case and independent review. | Prevent accidental enablement | `PHYSICAL_OUTPUT_INTERLOCK_ENGAGED`; test `synthetic_calibration_is_virtual_only` | Implemented; process control not enforced |
| ECS-SR-003 | Lambda, AFR, STFT, LTFT and wideband state shall never be inputs to ignition advance, dwell, or knock-limit decisions. | Lambda is consequence evidence, not timing evidence | API absence in `ecu-ignition`; source grep; tests `missing_reference_returns_insufficient_evidence_not_a_guess` | Implemented; future edits need review |
| ECS-SR-004 | Physical ignition use shall require a licensed, engine-specific reference plus timing-light verification and knock review. | Synthetic/estimated timing is not engine-safe | `IgnitionPolicy::VerifiedReference`; preflight; docs | Partial: metadata is caller-supplied and not authenticated |
| ECS-SR-005 | Fuel calculation shall return an explicit inhibition for invalid inputs or excessive duty; every actuator path shall treat inhibition as fuel cut. | Prevent silent clamping/guessing | `FuelDecision.inhibited`; simulator checks; tests | Partial: no hardware path; no type-level enforcement |
| ECS-SR-006 | Any lambda-derived fuel correction shall be blocked unless all learning gates pass; gates shall fail closed. | Prevent learning from invalid conditions | `ecu-lambda/src/gate.rs`; tests `every_invalidity_is_explained` | Implemented (host); detection inputs not validated |
| ECS-SR-007 | STFT and LTFT shall be hard-bounded; integrator windup shall be prevented; invalid sensor periods shall freeze then decay trim. | Prevent hidden/oscillatory correction | `ClosedLoop`; tests `integrator_never_winds_beyond_limit`, `missing_lambda_gate_freezes_then_decays_to_zero` | Implemented (host); gains unvalidated |
| ECS-SR-008 | Mixture Map proposals shall be bounded, show coverage/confidence, distinguish measured cells, and never apply without explicit human approval, matching checksum, and matching revision. | Reviewable fuel calibration | `mixture_map.rs`; `apply_proposal`; CLI `--approve`; tests | Implemented (host); history not durable |
| ECS-SR-009 | Calibration integrity shall be verified before use; changes shall create a new sealed revision with parent and undo metadata. | Detect corruption/staleness | SHA-256 `seal`/`verify`; revision tests | Partial: unkeyed hash, not authenticity; no flash journal |
| ECS-SR-010 | The trigger decoder shall report sequential sync only after a valid gap and cam phase; a gap mismatch or dropped tooth shall cause sync loss, not a guessed angle. | Wrong angle can destroy an engine | `ecu-trigger`; tests `syncs_and_measures_constant_speed`, `extra_missing_tooth_causes_sync_loss`, `dropped_tooth_is_not_guessed`, `cam_inside_window_gives_sequential`, `cam_outside_window_is_rejected`, `cam_without_window_is_rejected`; `timeout_declares_lost_sync` still passes | Cam-window gap fixed in host code: when cam is required, only an edge inside an explicit window validates phase; absent/outside window rejects phase. Physical Evo cam-pattern verification remains open; other trigger gaps in AUD-02 |
| ECS-SR-011 | Loss of crank sync shall cut fuel and spark promptly; recovery shall require re-synchronisation. | Limit consequences of lost position | Protection `CrankSyncLost`; simulator test `trigger_loss_causes_cut_then_resync` | Simulated; timeout is 250 ms and not speed-dependent |
| ECS-SR-012 | Protection shall be logically independent of fuel/ignition calculation and shall latch faults where silent recovery is hazardous. | Independent last line of defence | Separate crate; latching tests | Implemented in software; not an independent hardware monitor |
| ECS-SR-013 | Latched faults shall clear only when the physical condition is inactive and an explicit operator action occurs after a hold period. | Prevent silent recovery from oil/overtemp/overboost | `clear_latched`; tests | Implemented; no operator identity; clear endpoint unauthenticated |
| ECS-SR-014 | Protection thresholds and reactions shall be engine-specific, debounced, and validated by fault injection before physical use. | Avoid false and missed trips | Sensor-threshold latch confirmation fixed in host code (§7 tests); current limits are placeholders; no hardware | Partial (host confirmation); engine-specific validation remains blocked |
| ECS-SR-015 | A hardware watchdog with a defined reset safe state shall monitor the control loop; watchdog health shall not be a hard-coded `true`. | Detect hang/clock failure | Simulator hard-codes `watchdog_ok=true` | Planned / Blocked |
| ECS-SR-016 | Power loss, brownout, reverse polarity and load-dump shall drive outputs to a defined safe state and shall not cause unintended actuation. | Automotive electrical environment | No hardware | Planned / Blocked; see AUD-03 |
| ECS-SR-017 | Injector and spark events shall be generated by hardware capture/compare timers with bounded jitter, not by async tasks or the UI loop. | Deterministic timing | Firmware skeleton has no HAL/timer capture or output compare | Planned / Blocked; see AUD-04 |
| ECS-SR-018 | Every actuator path shall independently honour `cut_fuel`, `cut_spark`, `inhibited`, and the physical interlock. | Defence in depth | No actuator path exists | Planned; must be designed before hardware |
| ECS-SR-019 | Calibration edits and applies shall require a paused engine, explicit approval, and a recorded actor/revision/checksum. | Auditability and safe sequencing | Service checks pause/approve; history in memory | Partial: no actor identity, no durable log |
| ECS-SR-020 | The UI shall display unvalidated/estimated status, physical-export status, active faults, and gate reasons without hiding them. | Operator awareness | UI labels; preflight page | Implemented (host); Electron/browser runtime not validated; DTC label mapping bug |
| ECS-SR-021 | Before any physical ignition, base offset shall be verified with a timing light and knock detection shall be validated on the engine. | Timing-light and knock are mandatory | Checklist only | Planned / Blocked |
| ECS-SR-022 | Evo VIII pinout, connector, trigger, injector, ignition and harness data shall be verified against the physical vehicle and recorded before Gate 3. | Prevent wiring/trigger damage | `vehicle.rs` blocking items; no evidence yet | Unverified / Blocked |
| ECS-SR-023 | Release gates shall not self-promote; each gate shall require attached evidence and independent sign-off. | Prevent claims without evidence | AUD-08 template | Process only; not enforced by CI |
| ECS-SR-024 | The service shall not be reachable by untrusted network clients and shall authenticate/authorise calibration changes before any physical actuator path exists. | Prevent remote/CSRF apply | Binds 127.0.0.1, but CORS permissive and no auth | Not implemented; currently virtual-only |
| ECS-SR-025 | The operator shall have an independent emergency stop that removes actuator power, not merely a software cut. | Human last resort | Bench plan only | Planned / Blocked |
| ECS-SR-026 | Sensor validity and staleness shall be tracked per sensor and checked before control; a numeric value alone shall not imply trust. | Prevent control on stale data | `SensorStatus` type; lambda gate; no per-sensor age in telemetry | Partial |
| ECS-SR-027 | The safety argument shall be re-issued for the exact source revision, calibration checksum, and hardware revision. | Evidence traceability | This document; audit hash | Partial: source committed; no CI attestation |

## 5. Separation of fuel and ignition evidence

This separation is the most important design rule in the project. It is enforced structurally in the Rust API, not by a runtime check that could be forgotten.

| Evidence | What it measures | May inform fuel? | May inform ignition? | Code evidence | Test evidence |
|---|---|---|---|---|---|
| Wideband lambda | Oxygen concentration in exhaust after combustion | Yes, under gates and bounds | **No** | `ecu-ignition` has no lambda parameter; `recommend_from_reference` signature excludes lambda | `missing_reference_returns_insufficient_evidence_not_a_guess` |
| STFT / LTFT | Fuel correction history | Yes, as pre-trim context and learning input | **No** | `ecu-lambda` modules; no ignition dependency | Closed-loop and LTFT tests |
| VE table | Estimated engine breathing | Yes, fuel mass | No direct use | `ecu-fuel` | `air_mass_matches_hand_calculation` |
| Target lambda | Desired mixture | Yes | **No** | `ecu-fuel` | Fuel tests |
| Knock level | Cylinder pressure/acceleration signal (model only) | No | Yes, bounded retard; not advance | `ecu-protection` reaction; no signal processing | `knock_produces_bounded_retard_not_fuel_learning` |
| Verified timing reference | Engine-specific spark advance source | No | Yes, offline recommendation with bounds | `ecu-ignition::recommend_from_reference` | `verified_reference_produces_bounded_offline_recommendation` |
| Coolant/IAT | Thermal state | Yes, corrections | Yes, bounded corrections | `corrected_advance` | `advance_is_clamped_and_corrections_bounded` |
| Fuel pressure | Fuel delivery | Yes, flow/pulse | No | `ecu-fuel` pressure factor | `pressure_compensation_increases_flow_and_shortens_pulse` |
| Oil pressure | Lubrication/protection | No | No | `ecu-protection` | `low_oil_pressure_cuts_and_latches` |

**Key statement.** A wideband sensor can tell you that combustion produced more oxygen than expected. It cannot tell you whether the spark occurred at MBT, whether the knock margin is adequate, or whether the advance is safe for the fuel, boost, compression, temperature and combustion chamber. The code's refusal to accept lambda in `ecu-ignition` is correct and should remain a review invariant.

**What would be required for ignition evidence.** A licensed, engine-specific timing map or dyno-derived data with provenance; verification of the base offset with a timing light; calibrated knock detection and audible review; confirmation of fuel octane, compression, boost, IAT, coolant and thermal state; supervised dyno changes; and explicit uncertainty bounds. None of these exist for the Evo VIII in this repository.

## 6. Lambda limitations

| Limitation | Effect | Current control | Residual risk |
|---|---|---|---|
| Measures oxygen, not fuel mass | AFR/lambda conversion depends on fuel stoichiometry and sensor calibration | `stoich_afr` explicit; lambda separate from AFR | Wrong fuel stoich or sensor calibration biases everything |
| Exhaust transport delay | A measured lambda corresponds to an earlier combustion event | Simulator models 60 ms; Mixture Map aligns by `exhaust_delay_ms` | Real delay varies with RPM/load; fixed/estimated delay misattributes cells |
| Sensor response lag | Fast transients are smoothed | Simulator first-order lag (80 ms model); gates reject transients for learning | Real sensor dynamics and sampling are not validated |
| Heater/readiness | Cold sensor reads invalid or plausible-but-wrong | `lambda_ready`, `SensorStatus`; gate blocks | Readiness logic is modelled, not a real controller |
| Exhaust leak | External air reads false lean | `exhaust_leak` gate flag; no detector | Flag is caller-supplied; a real leak may be missed |
| Misfire | Unburned oxygen reads false lean | `misfire` gate flag; simulator fault | No misfire detection; a real misfire can cause fuel addition if flag absent |
| Stuck sensor | Constant plausible value | Offline `LAMBDA_STUCK_SUSPECTED` heuristic; gate status | Simulator deliberately keeps status `Ok` when stuck; heuristic requires a wide operating range and exact constant |
| Sensor poisoning/aging | Slow drift, lead/silicone/oil contamination | None beyond status | No aging model or free-air check |
| One sensor per bank | Cannot resolve individual cylinders | `lambda_bank`; bank-aware LTFT | Per-cylinder mixture imbalance is invisible |
| Rich/lean saturation | Sensor stops responding at limits | `SensorStatus::Saturated` exists; gates reject | Not exercised by simulator faults; real saturation behavior unknown |
| Pressure/temperature sensitivity | Reading depends on exhaust pressure/temperature | None | No compensation model |
| Sampling/CAN latency | Stale value used as current | `lambda_age_ms`; gate max 300 ms | Simulator hard-codes age 20 ms; no CAN timestamp path |

**No lambda-based ignition.** There is no code path that converts lambda error into spark advance. The Mixture Map report carries `ignition_untouched: true` and an `ignition_effect` string; that is an assertion in data, not a cryptographic proof. A reviewer should verify the ignition table checksum is unchanged after any fuel workflow.

## 7. Protection reactions and thresholds

Thresholds below are `ProtectionLimits::default()` values. They are **engineering placeholders for the simulator**, not Evo VIII limits and not validated engine limits.

| DTC | Code | Trigger (default) | Action | Latched? | Clear | Test | Notes |
|---|---|---|---|---|---|---|---|
| Crank sync lost | P0335 | `sync_ok=false` | Cut fuel + spark | No | 100 ms hold after sync returns | `sync_loss_cuts_fuel_and_spark_then_recovers` | Decoder must actually report non-sequential |
| Coolant over-temperature | P0217 | ≥110 °C derate; ≥118 °C cut | Derate / Cut all | After N consecutive cut samples | Condition inactive + 1 s + explicit clear | `overtemp_latches_and_requires_explicit_clear` | Sensor-threshold latch confirmation fixed; single-sample cut remains immediate |
| Low oil pressure | P0524 | RPM >1200 and oil <60 kPa | Cut all | After N consecutive cut samples | Condition inactive + 1 s + explicit clear | `low_oil_pressure_cuts_and_latches` | Sensor-threshold fault; below 1200 rpm not evaluated; no oil-pressure sensor validation |
| Low fuel pressure | P0087 | Fuel pressure <180 kPa | Cut fuel | No | 1 s after recovery | None dedicated | Triggers at key-on if sensor reads 0; no prime logic |
| Lean under load | P0171 | MAP ≥90 kPa and lambda >1.12 derate; >1.22 cut | Derate / Cut fuel | After N consecutive cut samples | Condition inactive + 1 s + explicit clear | `lean_under_load_is_a_protection_fault_but_not_an_ignition_signal` | Sensor-threshold fault; false lean (misfire/leak) can still cause immediate cut |
| Overboost | P0234 | MAP gauge >95 kPa derate; >120 kPa cut | Derate / Cut fuel | After N consecutive cut samples | Condition inactive + 1 s + explicit clear | `overboost_derates_then_cuts_with_hysteresis` | Sensor-threshold fault; 120 kPa gauge = 220 kPa absolute; not an Evo target |
| Low battery voltage | P0562 | <10 V derate; <8 V cut | Derate / Cut fuel | After N consecutive cut samples | Condition inactive + 1 s + explicit clear | `fault_free_recovery_after_hold` | Sensor-threshold fault; ECU may brown out before 8 V; no hardware UVLO |
| Wideband not ready | P0133 | `wideband_ready=false` | Warn | No | Auto after condition clears | `wideband_not_ready_warns_without_cutting_engine` | Learning gate blocks separately; note test asserts `learning_inhibited == false` |
| Sensor plausibility | P0113 | `sensor_plausible=false` | Derate | No | 1 s after recovery | None dedicated | No fallback value; fuel may use stuck sensor |
| Calibration checksum | P0601 | `calibration_crc_ok=false` | Cut all | First evaluation | Explicit clear | `calibration_crc_failure_latches_a_cut` (unchanged), `crc_mismatch_latches_on_first_evaluation` | Integrity fault; no confirmation delay; no signature; attacker can reseal |
| Watchdog | P0606 | `watchdog_ok=false` | Cut all | First evaluation | Explicit clear | None dedicated | Integrity fault; no confirmation delay; simulator hard-codes `true`; no hardware |
| CAN bus lost | U0100 | `can_ok=false` | Warn | No | Auto | None dedicated | Simulator hard-codes `true`; no CAN |
| Injector duty high | P0213 | >95% | Derate | No | 1 s after recovery | None dedicated | Fuel layer also inhibits duty > max; two limits can disagree |
| Knock high | P0325 | ≥1.5 warn/retard; ≥4.0 derate | Warn/derate + retard 0-6° | No | Auto | `knock_produces_bounded_retard_not_fuel_learning` | Simulator injects level 3.0; no knock signal processing |

**Reaction semantics.** `Warn` changes state to Warning but no actuator reaction. `Derate` sets `max_rpm=4500`, `boost_limit_gauge_kpa=95`, `learning_inhibited=true`. `CutFuel` sets `cut_fuel=true`, `max_rpm=0`, boost limit 0. `CutAll` also sets `cut_spark=true`. `Limp` exists in the enum but `evaluate()` never selects it. Faults with higher action rank overwrite lower rank for the same DTC; latched faults remain until explicit clear.

**Sensor-threshold latch confirmation fixed in host code (A3).** Coolant over-temperature, low oil pressure, lean under load, overboost and low battery voltage latch only after N consecutive cut-condition evaluations per DTC. `ProtectionLimits::latch_confirmation_samples` defaults to 3 (zero is treated as one); a clear or derate-only sample resets that DTC's count. Tests: `single_sample_overtemp_does_not_latch`, `n_consecutive_samples_latch`, `clearing_before_n_samples_resets_confirmation`; the existing `overtemp_latches_and_requires_explicit_clear` and `low_oil_pressure_cuts_and_latches` now feed N samples. Calibration checksum/CRC and watchdog failures are deterministic integrity faults and retain their first-evaluation latch. The unchanged `calibration_crc_failure_latches_a_cut` and new `crc_mismatch_latches_on_first_evaluation` cover CRC latching. Cut/derate reactions remain immediate, and non-latching faults (including low fuel pressure) retain their behaviour. Thresholds remain unchanged placeholders; stuck plausible sensors and physical fault-injection validation remain gaps.

## 8. GSN-style safety argument (text)

This is a structured argument sketch, not a formal GSN model and not a compliance claim.

```text
G1: In the audited v0.1 snapshot, no physical injector, ignition coil, fuel pump,
    throttle, or other engine actuator can be energised by OpenTech-ECU.
  C1: Scope is the host Rust workspace plus the no_std firmware skeleton in
      the stated snapshot; no HAL, PCB, harness, or vehicle is in scope.
  A1: The operator does not deliberately add or bypass an actuator path.
  A2: The source snapshot hash and dependency lock are the ones recorded here.
  S1: Argue over every possible output path.
    G2: No hardware driver or MCU HAL exists; the firmware skeleton forces outputs off.
      E1: `firmware/stm32` is a no_std skeleton with no HAL, timer capture/compare,
          injector/ignition driver, watchdog integration, CAN, or bootloader.
      E2: `PHYSICAL_OUTPUTS_AUTHORISED = false`; `apply_reaction` calls `all_off()`.
    G3: The physical-output interlock is engaged and preflight reports it.
      E3: PHYSICAL_OUTPUT_INTERLOCK_ENGAGED = true.
      E4: Preflight test synthetic_calibration_is_virtual_only passes.
    G4: The only host transport is a local service/UI with no actuator command.
      E5: studio-service routes are calibration/simulation/telemetry only.
      E6: UI has no hardware transport; Electron/Tauri runtime is not validated.
  S2: Argue that fuel calibration cannot silently become unsafe.
    G5: Lambda-derived fuel changes are gated, bounded, and human-approved.
      E7: Shared gate rejects invalid/stale/transient/high-load samples.
      E8: STFT/LTFT bounded; Mixture Map bounded and approval-gated.
      E9: Apply checks checksum/revision and requires approve=true.
      L1: Gate 0 host software evidence passes (128/128); UI/copilot runtime,
          independent review, and hardware/timing evidence remain gaps.
    G6: Calibration changes are sealed and revisioned.
      E10: SHA-256 seal/verify; parent/revision; stale proposal refused.
      L2: Hash is unkeyed and history is in memory; not an authenticity control.
  S3: Argue that ignition cannot be inferred from lambda.
    G7: The ignition API has no lambda input.
      E11: ecu-ignition signature/source inspection.
      E12: Tests reject missing/synthetic references.
      L3: "Verified reference" is caller-supplied metadata; no trust anchor.
  S4: Argue that faults are contained.
    G8: An independent protection state machine latches critical faults.
      E13: Separate crate; DTCs; freeze frames; 14 tests; firmware skeleton integration.
      L4: Same process, no hardware monitor; thresholds placeholders; cuts immediate.
          Sensor-threshold latches require N samples; integrity latches are immediate.
  R1: Residual goal: hardware watchdog, power safe state, timing evidence,
      sensor/injector validation, Evo pinout/harness verification, independent review.
      These are not satisfied and are tracked in AUD-08.
```

**Assurance deficit.** The argument is a source-code and process argument. There is no formal verification, no independent review, no hardware fault injection, no EMC/thermal test, and no quantitative diagnostic coverage. The green host suite improves the software evidence but does not establish physical safety or full runtime coverage.

## 9. Operator responsibilities

| Role | Responsibilities | Prohibited actions |
|---|---|---|
| Developer | Keep the interlock; do not add lambda to ignition; add tests for every safety invariant; preserve provenance/licences; update evidence after code changes | Adding a physical output path, weakening gates, or marking synthetic data verified |
| Tuner | Treat all maps as unvalidated estimates; review gate reasons and cell coverage; apply only selected cells; record checksum/revision | Accepting a proposal because it "looks smooth"; using WOT cells without explicit review |
| Bench operator | Use a current-limited supply, fuses, dummy loads, isolation, E-stop; verify safe state before power; record scope/log evidence | Connecting a vehicle harness, energising real injectors with fuel, or working alone on high-energy tests |
| Independent reviewer | Re-run commands, inspect source, challenge claims, sign gate records; declare conflicts of interest | Self-approving a gate or accepting a screenshot as proof of timing |
| Project owner | Maintain release gates, no self-promotion, fund independent review, keep docs and code consistent | Claiming automotive qualification, plug-and-play, EMC, or engine readiness |

**No vehicle operation.** The repository has no validated hardware, harness, calibration, timing evidence, or insurance/legal basis for vehicle use. The Evo VIII must not be connected to this software or to a prototype board.

## 10. Explicit non-claims

| Non-claim | Why it is not claimed |
|---|---|
| ISO 26262 / IEC 61508 functional safety | No safety lifecycle, ASIL/SIL allocation, independent assessment, or certified tools |
| Automotive qualification | No AEC-Q100/Q200, PPAP, environmental, vibration, thermal or EMC evidence |
| EMC compliance | No pre-compliance scan, filter design, transient test, or enclosure |
| Electrical safety | No validated schematic, PCB, isolation, fusing, thermal or creepage/clearance review |
| Hardware watchdog / safe state | No hardware watchdog/HAL integration; simulator input is hard-coded true; skeleton only accepts a flag |
| Deterministic engine timing on hardware | Host 1 ms simulation is not a timer capture/compare guarantee |
| Calibration authenticity | SHA-256 detects accidental corruption; it is not a signature or trust anchor |
| Plug-and-play Evo VIII compatibility | Exact ECU part number, connector, pinout, trigger and harness are unverified |
| Engine-safe calibration | All fuel/ignition tables are estimates or synthetic; no dyno/engine validation |
| Knock-safe ignition | No knock signal processing or engine-specific threshold validation |
| Wideband accuracy | Only a modelled sensor; no controller, heater, free-air or aging validation |
| Injector characterization | No flow bench or dead-time measurement; preflight warns |
| Watchdog/CAN/power protection coverage | Paths are simulated or absent |
| Cybersecurity | No authentication, authorization, TLS, secure boot, or update signing |
| Legal/regulatory compliance | No emissions, roadworthiness, insurance, or motorsport rule review |

## 11. Highest-priority open safety issues from this audit

| Rank | Issue | Why it matters | Required action |
|---|---|---|---|
| 1 | No reviewed hardware/firmware path exists | Every physical safety claim is blocked | Complete AUD-03/AUD-04/AUD-08; do not connect a vehicle |
| 2 | Gate 0 pass is scoped, not a certification | Runtime UI/Electron, live copilot and independent review remain gaps | Add runtime tests; obtain independent review |
| 3 | Trigger timeout is 250 ms and angle is extrapolated | At 7000 rpm this is ~29 revolutions before sync loss; wrong-angle events possible | Speed-dependent timeout, bounded prediction, immediate cut on missing teeth |
| 4 | Cam-window validation fixed in host code (ECS-SR-010 tests); cam loss is not detected | Phase can remain "sequential" after cam failure; an in-window spurious edge is not a validated physical cam pattern | Add cam timeout/plausibility; physical Evo cam-pattern and TDC verification remain open |
| 5 | Sensor-threshold latch confirmation fixed in host code (§7 tests); thresholds remain placeholders | Immediate single-sample cut/derate reactions or missed faults remain possible | Validate engine-specific limits and confirmation timing by fault injection; retain immediate integrity-fault latches |
| 6 | Preflight warnings do not block hardware_candidate | If interlock were removed, unverified sensors/injectors/timing-light could still export | Make physical prerequisites hard failures, not warnings |
| 7 | Ignition "verified reference" is unauthenticated metadata | A crafted calibration can claim verified provenance | Signed provenance or external evidence registry; cross-check policy vs table provenance |
| 8 | SHA-256 checksum is unkeyed; history is in memory | Tamper detection is not authenticity; audit trail lost on restart | Keyed signature/HMAC, durable append-only audit log, actor identity |
| 9 | Service has no auth and permissive CORS | Any local page/process can edit/apply while service runs | Bind local only for now; add auth/CSRF protection before hardware |
| 10 | Apparent double knock retard in simulator | Logged advance may be over-retarded; distorts tuning evidence | Add a unit test and fix the ordering; re-run knock scenarios |
| 11 | Mixture Map trim labels are misleading (`mean_ltft=0`) | Reviewers cannot separate STFT/LTFT contributions | Preserve separate means and report them; add test |
| 12 | `no_std` checks across core crates | Embedded portability claim was previously not met | Resolved 2026-11-12: `alloc` imports and `rem_euclid` replaced/localised; all core crates now pass no-default-features checks |
| 13 | Evo pinout/trigger/harness unverified | Wiring or trigger error can destroy hardware | Complete AUD-06/AUD-10 evidence and Gate 3 checks |

## 12. Conclusion

In the audited snapshot, OpenTech-ECU is a host-side simulation and calibration-analysis platform. It has no physical output path, its ignition API refuses lambda, its fuel learning is gated and bounded, and its protection state machine is separate. Those are meaningful software properties.

They do **not** make it an engine-safe ECU. The host test suite is green and Gate 0 host software evidence passes for the implemented behaviour, but the trigger and protection layers still have material gaps, the calibration trust model is not authenticated, runtime UI/copilot behaviour is not fully tested, `ecu-protocol`/`ecu-lambda` remain std-only, and no reviewed hardware, timing, EMC, thermal, sensor, injector, harness or vehicle evidence exists. The correct status is **Gate 0 host software evidence: PASS, scoped; Gate 1 and above blocked**. Any stronger statement requires the evidence and independent sign-off defined in AUD-08.
