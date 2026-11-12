# Failure Mode and Effects Analysis (component level, v0.1)

| Field | Value |
|---|---|
| Document ID | AUD-02 |
| Status | Independent audit draft; qualitative FMEA, not a quantitative FMEDA or safety validation |
| Audit date | 2026-11-12 |
| Evidence cut-off | 2026-11-12T05:45Z, working tree; source manifest `9837ba88...` |
| Scope | Host-side Rust trigger, lambda, fuel, ignition, sensor, calibration, service/UI, protocol, scheduler, processor/watchdog, power assumptions, and the no_std firmware skeleton |
| Out of scope | Physical PCB, harness, MCU HAL/timer/driver implementation, EMC, thermal, vehicle installation |
| Related | AUD-01, AUD-03, AUD-04, AUD-05, AUD-08 |

> **Reading note.** "Host effect" is what the audited simulation can do. "Hazard if actuated" is the consequence only if a future firmware/PCB path exists. Many rows are therefore requirements for hardware, not descriptions of current physical behaviour.

## 1. Method and ratings

- Method: source-code review, unit-test inspection, CLI scenario execution, and structured fault enumeration. No hardware fault injection, no field data, no quantitative failure-rate data.
- Severity (S): **S1** catastrophic (death/serious injury, engine destruction, fire), **S2** critical (major engine/vehicle damage, dangerous loss of power), **S3** major (drivability, emissions, component damage), **S4** minor (nuisance/degraded function).
- Likelihood (L) in intended development use: **L1** remote, **L2** unlikely, **L3** possible, **L4** likely, **L5** frequent. These are judgments, not probabilities.
- Detection (D) by current host code: **D1** certain/immediate, **D2** high (automated check in code), **D3** moderate (logged/observable, requires review), **D4** low (only offline inspection), **D5** none.
- "Residual" describes what remains after current controls. It is not an accepted risk.

## 2. Trigger and synchronisation failures

| ID | Failure mode | Cause | Host effect in v0.1 | Hazard if actuated | S | L | D | Detection / current control | Mitigation / required evidence |
|---|---|---|---|---|---|---|---|---|---|
| FM-TRG-01 | Wrong gap configuration | `teeth`/`missing` not matched to wheel | Gap mismatch or wrong sync; simulator may lose sync | Spark/fuel at wrong angle; engine damage | 1 | 2 | 2 | `MissingToothConfig::validate`; `GapMismatch`; test `extra_missing_tooth_causes_sync_loss` | Scope the actual wheel; verify tooth count/gap; make config read-only after seal |
| FM-TRG-02 | Missing normal tooth | Damaged tooth, sensor dropout, noise | `ToothDropped` -> `Lost`; fuel/spark cut | Sudden cut/stall; possible wrong event before detection | 1 | 3 | 1 | `ratio > 1.25 && <1.45` fail; test `dropped_tooth_is_not_guessed` | Immediate cut on first anomaly; no extrapolation; confirm with cam |
| FM-TRG-03 | Extra tooth / noise edge | EMI, VR ringing, damaged wheel | Treated as normal tooth; angle advances one tooth; next gap count mismatch -> `Lost` | One or more events at wrong angle before gap check | 1 | 3 | 3 | Gap tooth-count check only at next gap; no too-short-interval rejection | Reject intervals below a minimum ratio; debounce/blanking; hardware filter |
| FM-TRG-04 | False gap on deceleration | Normal tooth interval stretches past 1.45× expected | Gap accepted if count matches; possible wrong index | Wrong phase/angle; spark at wrong time | 1 | 2 | 3 | Gap ratio and tooth count; test only acceleration | Deceleration tests; speed-dependent gap threshold; cam cross-check |
| FM-TRG-05 | First-gap false sync | Noise creates a gap-like interval before a real gap | First gap has no tooth-count check; when cam is required, `Sequential` now also requires a cam edge inside an explicit window | Wrong tooth index by one or more teeth | 1 | 2 | 3 | `gap_count > 0` check only after first gap; cam-window validation fixed in host code (FM-TRG-07 tests) | Require two consecutive valid gaps before sequential; verify offset |
| FM-TRG-06 | Cam signal lost after phase latch | Cam sensor/connector failure | `cam_phase_known` remains true; state stays `Sequential`; parity toggles on gaps | 360° phase error; sequential fuel/spark on wrong stroke | 1 | 3 | 4 | No cam timeout or cam-loss DTC | Add cam edge timeout/plausibility; degrade to wasted spark/batch or cut |
| FM-TRG-07 | False/spurious cam edge | Noise, wrong cam pattern, double edge | When cam is required, absent/outside window rejects phase, exposes `cam_phase_rejected` and leaves a synced decoder `CrankSynced`; an inside-window edge validates phase | In-window spurious edge or wrong physical window can still give wrong stroke | 1 | 3 | 4 | Cam-window gap fixed in host code: `cam_inside_window_gives_sequential`, `cam_outside_window_is_rejected`, `cam_without_window_is_rejected`, `cam_rejection_revokes_validated_phase`; `cam_not_required_keeps_sequential_without_window` preserves optional-cam behaviour | Physical Evo cam pattern/window and TDC offset remain unverified and open; validate recorded crank/cam traces; cam-loss detection remains open |
| FM-TRG-08 | Crank signal loss | Broken sensor/connector, wheel damage | Decoder remains `Sequential` until `tick()` timeout (250 ms default); angle extrapolated | At 7000 rpm ~29 revolutions of open-loop fuel/spark before cut | 1 | 3 | 2 | `timeout_ms`; `CrankSyncLost`; test `timeout_declares_lost_sync` | Speed-dependent missing-tooth timeout; cut on first missing expected edge; bound extrapolation |
| FM-TRG-09 | Timestamp rollback / zero interval | Timer wrap, bad capture, software bug | `TimestampWentBackwards`/`ZeroInterval` -> `Lost` | Cut/stall; possible missed event | 2 | 2 | 1 | `on_tooth` checks; error count | Use 32/64-bit monotonic capture; wrap tests; fail-closed |
| FM-TRG-10 | RPM stale during acceleration | RPM updated only at gap | Angle extrapolation uses last revolution's RPM | Timing error during hard acceleration/deceleration | 2 | 3 | 3 | No per-tooth RPM; test `acceleration_tracks_rpm_without_false_gap` only checks final RPM | Per-tooth speed estimate; angle error budget; bench scope |
| FM-TRG-11 | Wrong base offset / TDC assumption | `sync_offset_deg` set from guess | Simulator runs with offset; no physical claim | Ignition timing offset by arbitrary degrees | 1 | 3 | 4 | `Docs`/comments say not TDC; preflight warns `IGNITION_OFFSET` | Timing-light verification; record measurement; lock offset |
| FM-TRG-12 | Unsupported dual-wheel pattern | User configures `DualWheel` | `EngineConfig::validate` returns `TRIGGER_NOT_IMPLEMENTED` | Cannot run; no wrong-angle claim | 2 | 2 | 1 | Validation error | Implement or explicitly reject; document supported patterns |
| FM-TRG-13 | Cam required but absent from start | Wrong config or missing cam | Stays `CrankSynced`; fuel cut (`!is_sequential`) | No start; no sequential fuel/spark | 2 | 3 | 1 | `cam_required`; state machine; fuel cut | Clear diagnostic; allow explicit wasted-spark mode only if validated |

**Trigger findings.** The decoder is deterministic and fail-closed for gap mismatch/dropped tooth, which is good. The significant gaps are: (a) no cam-loss detection after phase latch; (b) extra short edges are not rejected until the next gap; (c) first gap has no tooth-count check; (d) 250 ms timeout is long at high RPM and angle is extrapolated during it; (e) deceleration/gap-threshold behavior is untested. These are material for any physical ignition.

## 3. Wideband / lambda failures

| ID | Failure mode | Cause | Host effect in v0.1 | Hazard if actuated | S | L | D | Detection / current control | Mitigation / required evidence |
|---|---|---|---|---|---|---|---|---|---|
| FM-WB-01 | Stuck lean | Sensor poisoned/stuck, wiring, controller fault | Simulator forces 1.18 with status `Ok`; offline heuristic detects constant value and abstains | Fuel table driven lean; over-fuelling, bore wash | 2 | 3 | 3 | `LAMBDA_STUCK_SUSPECTED` in `mixture_map.rs`; no online detector | Online stuck/plausibility monitor; cross-check with fuel trims and operating range |
| FM-WB-02 | Stuck rich | Same | Simulator forces 0.78; offline heuristic may detect | Fuel table driven rich; lean misfire, overheating | 2 | 3 | 3 | Offline heuristic only | Same as above; rich/lean direction check |
| FM-WB-03 | Delayed response | Transport delay, sensor lag, CAN latency | Simulator 60 ms transport + 80 ms lag; Mixture Map aligns by delay | Wrong cell attribution; wrong VE | 2 | 3 | 2 | `aligned_index`, `exhaust_delay_ms`; gate age | Measure delay vs RPM/load; timestamp at sensor; validate alignment |
| FM-WB-04 | Exhaust leak | Cracked manifold/gasket, open sensor boss | Simulator fault biases lambda +0.12; gate flag rejects | False lean -> over-fuelling | 2 | 3 | 2 | `exhaust_leak` gate flag; no detector | Leak detection (pressure/oxygen), physical inspection; do not learn if flag |
| FM-WB-05 | Misfire false lean | Unburned O2, late combustion | Simulator fault biases +0.10; gate flag rejects | False lean -> fuel addition, worse misfire | 2 | 3 | 2 | `misfire` gate flag; no detector | Misfire detection (crank speed/ionization); inhibit learning |
| FM-WB-06 | Not ready / cold | Heater off, condensation, key-on | `lambda_ready=false`; gate rejects; protection warns | No closed loop; base fuel only | 3 | 3 | 1 | `lambda_ready`, `SensorStatus`; test `cold_engine_and_stale_sensor_are_explained` | Heater diagnostics; readiness timer; no learning until stable |
| FM-WB-07 | Sensor aging/drift | Contamination, thermal cycling | None; model has no aging | Slow bias in all fuel learning | 2 | 3 | 5 | None | Free-air calibration, aging model, periodic validation |
| FM-WB-08 | Wrong bank assignment | Config error, swapped sensors | `lambda_bank` filter; wrong bank could be used | Bank-specific fuel error | 2 | 2 | 3 | `options.bank`; `ALIGNED_BANK_MISMATCH` | Verify sensor-to-bank mapping; dual-bank tests |
| FM-WB-09 | Saturation | Very rich/lean beyond sensor range | `SensorStatus::Saturated` exists; gate rejects | Hidden mixture error | 2 | 3 | 2 | Status check | Exercise saturation in tests; sensor datasheet limits |
| FM-WB-10 | CAN time alignment/latency | External controller, bus load | No CAN path; telemetry age hard-coded 20 ms | Stale lambda used as current | 2 | 3 | 3 | `lambda_age_ms` field; gate max 300 ms | Real timestamping; reject stale/out-of-order; bus load tests |
| FM-WB-11 | Heater short/open | Hardware fault | No hardware | Sensor damage, no reading | 2 | 3 | 5 | None | Heater current/voltage diagnostics; fuse; safe-off |
| FM-WB-12 | Condensation thermal shock | Cold start, water | No model | Sensor cracked; false reading | 2 | 2 | 5 | None | Heater ramp; installation angle; datasheet procedure |

**Lambda findings.** The gate implementation is broad and explicitly rejects misfire, exhaust leak, overrun, fuel cut, knock, overboost, critical DTC, stale/implausible lambda, and WOT by default. However, most of those inputs are booleans supplied by the caller; the repository does not implement misfire, exhaust-leak, or online stuck-sensor detection. The simulator deliberately keeps `SensorStatus::Ok` for a stuck signal, so the status bit alone is not sufficient. The offline `LAMBDA_STUCK_SUSPECTED` heuristic requires a long exact-constant run and a wide operating range; noisy stuck sensors can evade it.

## 4. Injector and fuel-delivery failures

| ID | Failure mode | Cause | Host effect in v0.1 | Hazard if actuated | S | L | D | Detection / current control | Mitigation / required evidence |
|---|---|---|---|---|---|---|---|---|---|
| FM-INJ-01 | Clogged/low-flow injector | Deposits, contamination | Simulator `InjectorFlowLow` scales delivered fuel 0.80 | Lean cylinder, detonation, burned valve | 2 | 3 | 3 | Fault code `INJECTOR_FLOW_LOW` in simulator; no per-cylinder measurement | Flow bench; per-cylinder trim; wideband per bank/cylinder if available |
| FM-INJ-02 | Wrong dead time | Wrong datasheet, voltage compensation error | PW offset error, especially at small pulse widths | Lean/rich at idle and light load | 2 | 3 | 3 | `dead_time_at` curve; preflight warns unverified | Bench dead-time characterization across voltage/pressure |
| FM-INJ-03 | Wrong rated flow / pressure | Injector replaced, regulator fault, sensor blind | Flow/pressure factor wrong; simulator `FuelPressureSensorBlind` | Lean at high load; detonation | 2 | 3 | 2 | Pressure factor; protection low fuel pressure; preflight warn | Flow bench; pressure sensor calibration; regulator test |
| FM-INJ-04 | Fuel pressure error | Pump/regulator/filter, returnless vs referenced | Wrong delivered fuel; simulator blind-sensor fault | Lean/rich; unstable mixture | 2 | 3 | 2 | `fuel_pressure_kpa` bounds; `LowFuelPressure` DTC | Pressure sensor + regulator validation; fuel pump control |
| FM-INJ-05 | Minimum pulse clamp | Large injector at idle | `min_pulse_clamped=true`; delivers more than requested | Rich idle, poor emissions, bore wash | 3 | 4 | 1 | `min_pulse_clamped` flag; test `minimum_pulse_is_flagged_and_delivers_more_than_requested` | Size injectors; idle strategy; validate minimum pulse |
| FM-INJ-06 | Duty exceeded | Undersized injector, high load | `InhibitReason::DutyExceeded` in decision; simulator cuts fuel | Lean/detonation if ignored; sudden cut if honored | 2 | 3 | 1 | Duty check; test `excessive_duty_is_reported_not_silently_clamped` | Hardware path must treat inhibited as cut; injector sizing |
| FM-INJ-07 | Wrong density/stoich | E85/gasoline mismatch, config error | Fuel mass error | Lean/rich; cold start, detonation | 2 | 3 | 2 | Bounds; explicit config | Fuel composition sensor or explicit calibration; validate |
| FM-INJ-08 | Driver open/short | Harness, injector coil, driver | No hardware | No fuel or continuous fuel; hydro-lock/fire risk | 1 | 3 | 5 | None | Driver diagnostics; fuse; per-channel current sense; safe-off |
| FM-INJ-09 | Flyback/inductive spike | No clamp/recirculation | No hardware | Driver avalanche, ECU damage, erratic injection | 1 | 3 | 5 | None | Validated low-side driver with flyback; scope test; thermal |
| FM-INJ-10 | Wrong cylinder assignment | Firing order/connector error | Scheduler uses firing order; simulator model | Fuel in wrong cylinder; washing/lean | 2 | 2 | 3 | Firing order validation; no connector check | Pinout verification; per-cylinder test; audit |
| FM-INJ-11 | Staged/batch mode error | `InjectionMode` says staged but `calculate` is one injector | Model mismatch; no staged implementation | Wrong fuel distribution | 2 | 3 | 4 | Enum exists; no staged path | Implement per-injector summation or reject staged config |
| FM-INJ-12 | Fuel pump/relay failure | Wiring, relay, pump | No hardware | No fuel; lean; stall | 2 | 3 | 5 | None | Pump control + pressure feedback; prime logic; fuse |

**Injector findings.** The fuel calculation is dimensionally consistent and reports minimum-pulse/duty conditions rather than silently clamping. The critical gap is that `inhibited` is a return value, not a type-level actuator gate; only the simulator currently honors it. Any future firmware must treat both `inhibited` and protection `cut_fuel` as hard cuts. There is no injector characterization evidence.

## 5. Ignition failures

| ID | Failure mode | Cause | Host effect in v0.1 | Hazard if actuated | S | L | D | Detection / current control | Mitigation / required evidence |
|---|---|---|---|---|---|---|---|---|---|
| FM-IGN-01 | Excessive dwell | Wrong coil data, voltage compensation, high RPM | `limit_dwell` caps at 45% of available charge time | Coil overheating, driver damage, spark failure | 2 | 3 | 2 | Dwell cap; test `dwell_is_reduced_at_high_rpm_but_never_zero` | Coil datasheet; thermal test; per-coil dwell table |
| FM-IGN-02 | Insufficient dwell | High RPM, low voltage, wrong table | Dwell clamped but may be too low for coil | Misfire, lean false reading, power loss | 2 | 3 | 2 | Dwell cap only; no spark-current feedback | Coil characterization; spark detection |
| FM-IGN-03 | Wrong advance | Bad table, wrong offset, interpolation error | Table lookup clamped -10..55; synthetic map | Knock/detonation or overheating | 1 | 3 | 2 | Bounds; evidence policy; preflight | Licensed map; timing light; knock validation; dyno |
| FM-IGN-04 | Lambda used for advance | Future code change | None; no lambda parameter | Knock/detonation | 1 | 2 | 2 | API absence; no runtime test | Review invariant; static check; test that would fail if added |
| FM-IGN-05 | Rev limiter only cuts spark | Limiter design | `RevLimiter` cuts spark; fuel cut handled by protection | Unburned fuel in exhaust; catalyst/backfire | 3 | 3 | 2 | Test `rev_limiter_has_hysteresis`; comment says fuel cut separate | Coordinate fuel cut; validate limiter strategy |
| FM-IGN-06 | Knock retard applied twice | Previous and current protection decisions both subtracted in `compute_ecu_outputs`/`evaluate_protection` | Over-retard in simulation; logged advance lower than intended | Excessive retard, heat, poor torque, emissions | 2 | 3 | 3 | Code reading; no dedicated test | Add test isolating one knock step; fix ordering; log applied retard once |
| FM-IGN-07 | Knock retard not enough / false knock | No signal processing; threshold placeholder | Simulator injects level 3.0; 0-6° retard | Knock damage or over-retard | 1 | 3 | 4 | `KnockHigh`; `knock_retard_deg` | Knock sensor interface, windowing, threshold calibration, audio review |
| FM-IGN-08 | Coil/igniter mismatch | Logic-level vs dumb coil, polarity | No hardware | No spark, wrong dwell, driver damage | 1 | 3 | 5 | None | Verify Evo coil/igniter topology; interface requirement |
| FM-IGN-09 | Wasted-spark/sequential error | Cam phase wrong, outputs per cylinder | Scheduler supports firing order; simulator sequential only | Spark on wrong stroke; backfire | 1 | 3 | 3 | Sequential gate; trigger tests | Phase verification; wasted-spark fallback design |
| FM-IGN-10 | Spark cut not honored | Future actuator path ignores flag | Simulator checks `spark_cut` before counting sparks | Uncontrolled spark during cut | 1 | 2 | 3 | `apply_events` checks; no hardware | Hardware gate + independent inhibit |
| FM-IGN-11 | Timing-light offset not verified | Assumed TDC | Preflight warns; no measurement | Wrong base timing | 1 | 3 | 4 | `IGNITION_OFFSET` warning | Mandatory timing-light procedure; record evidence |

**Ignition findings.** The strongest property is structural: `ecu-ignition` has no lambda input, and synthetic tables are rejected for physical use. The weakest properties are evidence authenticity, knock validation, and the simulator's apparent double retard. The dwell cap is a simplified model, not a coil datasheet or thermal validation.

## 6. Sensor failures

| ID | Failure mode | Cause | Host effect in v0.1 | Hazard if actuated | S | L | D | Detection / current control | Mitigation / required evidence |
|---|---|---|---|---|---|---|---|---|---|
| FM-SEN-01 | MAP stuck | Blocked port, wiring, sensor | Simulator forces 55 kPa; protection `SensorPlausibility` derate | Wrong load -> wrong fuel/ignition | 1 | 3 | 2 | Simulator fault; `sensor_plausible`; no fallback | Plausibility vs TPS/RPM; default load; sensor calibration |
| FM-SEN-02 | MAP disconnected/zero | Hose off, open circuit | `map_kpa=0`; fuel inhibits MAP <5; sensor_stale in Mixture Map | No fuel or wrong default | 2 | 3 | 1 | Fuel bounds; `sensor_stale` heuristic | Open/short diagnostics; default strategy |
| FM-SEN-03 | TPS stuck/noisy | Wiring, pedal, sensor | Fuel `alpha-N`/blend affected; speed-density less so | Wrong load/transient fuel | 2 | 3 | 2 | Range checks in fuel; no TPS DTC | Dual-track plausibility; calibration |
| FM-SEN-04 | CLT stale | Sensor, wiring | Simulator forces 35 °C; protection sensor fault | Wrong warm-up/cooling protection | 2 | 3 | 2 | Simulator fault; `sensor_plausible` | Time-based plausibility; default; fan control independent |
| FM-SEN-05 | IAT stale | Sensor, wiring | Wrong air density | Lean/rich | 2 | 3 | 3 | Bounds only | Plausibility vs ambient/CLT; default |
| FM-SEN-06 | Oil pressure sensor blind | Wiring, sensor | Protection may miss low oil; simulator does not model blind oil | Engine destruction | 1 | 3 | 4 | None; oil pressure input is a number | Sensor diagnostics; independent switch; conservative limit |
| FM-SEN-07 | Fuel pressure sensor blind | Wiring, sensor | Simulator `FuelPressureSensorBlind`; actual pressure differs | Lean at load; detonation | 2 | 3 | 2 | Simulator fault; low pressure DTC may not see it | Pressure sensor plausibility; pump current; fuel pressure relief |
| FM-SEN-08 | Battery voltage sense error | Divider, wiring, ground offset | Wrong dead time/voltage protection | Wrong PW; no low-voltage cut | 2 | 3 | 3 | Range check in fuel; protection low voltage | Calibrated divider; redundant sense; ground offset test |
| FM-SEN-09 | Sensor short to 5 V/ground | Harness damage | Out-of-range value; some checks catch | Wrong control; driver damage | 2 | 3 | 3 | Bounds per sensor; not all | Hardware protection; diagnostics; fail-safe defaults |
| FM-SEN-10 | No per-sensor age/status | Telemetry only has lambda status | Stale MAP/CLT/IAT can look valid | Control on stale data | 2 | 3 | 3 | `SensorSample` type exists but not wired into telemetry | Add status/age to every sensor; control must check |

**Sensor findings.** `SensorStatus` and `SensorSample` are good primitives, but the running telemetry and control paths largely use raw `f32` values. The only real status in telemetry is lambda. Protection receives booleans (`sensor_plausible`, etc.) computed by the caller. This is a structural weakness: the type system does not prevent a stale numeric value from being used.

## 7. Calibration corruption and provenance failures

| ID | Failure mode | Cause | Host effect in v0.1 | Hazard if actuated | S | L | D | Detection / current control | Mitigation / required evidence |
|---|---|---|---|---|---|---|---|---|---|
| FM-CAL-01 | Bit flip/partial write | Storage, transport, crash | `verify()` fails if checksum field unchanged | Wrong fuel/spark | 1 | 2 | 1 | SHA-256 verify; tests | Store redundant copies; atomic commit; flash ECC |
| FM-CAL-02 | Deliberate edit + reseal | Anyone can compute SHA-256 | `verify()` passes; no authenticity | Wrong fuel/spark, forged ignition evidence | 1 | 2 | 4 | None; hash is unkeyed | HMAC/signature with protected key; secure boot; audit log |
| FM-CAL-03 | Forged "verified" ignition policy | Crafted JSON with `VerifiedReference` | Preflight `IGNITION_EVIDENCE` passes; physical export still interlock-blocked | Unverified advance if interlock removed | 1 | 2 | 3 | Policy only; no cross-check with table provenance | Verify policy/table provenance consistency; signed evidence registry |
| FM-CAL-04 | Stale proposal applied | Base calibration changed after analysis | `apply_proposal` rejects checksum/revision mismatch; test `stale_proposal_is_refused` | Wrong VE applied | 2 | 2 | 1 | Checksum/revision check | Keep; add UI diff and approval record |
| FM-CAL-05 | Wrong table shape/axis | Manual JSON, bug | `Table` validation/axis checks reject | Interpolation error | 2 | 2 | 1 | `validate`, `replace_values` | Keep; schema versioning; migration tests |
| FM-CAL-06 | Out-of-bounds cell | Bad merge/import | `replace_values`/`apply_proposal` bounds check | Wrong fuel/ignition | 2 | 2 | 1 | Bounds checks; tests | Keep; independent review of limits |
| FM-CAL-07 | Undo/rollback lost | Service restart; in-memory history | Cannot restore prior revision; history lost | Wrong calibration persists | 2 | 3 | 3 | `history` vector; no persistence | Durable append-only store; export/import; backup |
| FM-CAL-08 | Concurrent edits | Multiple clients | Mutex serializes within process; no version check on generic edit | Lost update | 2 | 2 | 2 | Pause requirement; mutex | Optimistic concurrency/version check; actor identity |
| FM-CAL-09 | Provenance mislabeled | Manual edit, import | `mark_edited` adds note but does not downgrade source | Estimate treated as verified | 2 | 2 | 3 | `estimated()` helper; UI labels | Downgrade provenance on edit; require re-verification |
| FM-CAL-10 | Schema drift | Old/new JSON | `schema_version` check; no migration | Import failure or misinterpretation | 2 | 2 | 1 | `CAL_SCHEMA` error | Versioned migrations; compatibility tests |

**Calibration findings.** The SHA-256 seal is useful for accidental corruption and stale-proposal detection, but it is **not** a signature. Anyone who can edit the JSON can recompute the hash. The "verified reference" ignition policy is metadata, not authenticated evidence. This is the largest trust-model gap after the missing hardware.

## 8. Service and UI failures

| ID | Failure mode | Cause | Host effect in v0.1 | Hazard if actuated | S | L | D | Detection / current control | Mitigation / required evidence |
|---|---|---|---|---|---|---|---|---|---|
| FM-SVC-01 | Unauthorised apply | Any local process/browser posts `approve=true` | Calibration changed in simulation | Wrong fuel/spark if hardware existed | 2 | 2 | 2 | Pause + approve + checksum; localhost bind | Authentication, authorization, CSRF token, audit actor |
| FM-SVC-02 | Permissive CORS | `CorsLayer::permissive()` | Any website can call localhost service | Same as above | 2 | 3 | 3 | Localhost bind only | Restrict origins; no CORS for state-changing routes; token |
| FM-SVC-03 | Edit while running | UI bypass or direct API | Backend rejects `edit_table`/apply while running | Inconsistent state | 2 | 2 | 1 | `if session.running` conflict | Keep; add server-side generation check |
| FM-SVC-04 | Crash during apply | Panic, OOM, kill | Calibration replaced before/after engine rebuild; inconsistent session | Unknown calibration state | 2 | 2 | 2 | No transaction; history in memory | Atomic transaction; persist intent/commit; restart recovery |
| FM-SVC-05 | UI hides warning | CSS/layout, truncation, stale build | Operator misses unvalidated status | Wrong tune accepted | 2 | 2 | 3 | UI labels; preflight page | UX review; accessibility; screenshot tests |
| FM-SVC-06 | DTC label mapping bug | `f.dtc` is snake_case, UI compares uppercase enum to `Pxxxx` | Fault card shows enum name, not code/title | Operator misidentifies fault | 3 | 4 | 2 | Code reading; no test | Map enum to DTC code; UI test |
| FM-SVC-07 | Stale UI data | WebSocket lag, polling | Displayed values older than control state | Wrong operator decision | 3 | 3 | 2 | WS lag warning; polling | Timestamp/age display; sequence gap detection |
| FM-SVC-08 | Service exposed beyond localhost | User config/proxy | Remote calibration control | Unauthorised changes | 1 | 2 | 3 | Binds 127.0.0.1 by default | Enforce bind; TLS/auth if remote; document prohibition |
| FM-SVC-09 | Copilot tool misuse | Prompt injection, model error | Planned copilot; no write tools | Unauthorised proposal/action if tools added | 2 | 2 | 3 | Design says read-only; not implemented | Enforce capability list; schema validation; human approval; evals |
| FM-SVC-10 | Build/deploy mismatch | UI dist stale, wrong service version | UI expects different API | Confusing/incorrect operation | 3 | 3 | 2 | Protocol version; health | Version handshake; build hash; CI |

**Service/UI findings.** State-changing endpoints are reasonably guarded for a local development tool (pause, explicit approve, checksum/revision). They are not secure against a local malicious process or a browser page on the same machine. The lack of durable history and actor identity means the audit trail is incomplete. UI labels are generally candid about unvalidated maps, but the DTC mapping bug and hard-coded `lambda_age_ms` reduce diagnostic trust.

## 9. Protocol failures

| ID | Failure mode | Cause | Host effect in v0.1 | Hazard if actuated | S | L | D | Detection / current control | Mitigation / required evidence |
|---|---|---|---|---|---|---|---|---|---|
| FM-PRO-01 | Payload corruption | Bit flip, noise, bug | CRC32 mismatch rejected; test `corrupted_payload_fails_crc` | Wrong telemetry/command | 2 | 2 | 1 | CRC32; magic/version/length | Keep; CRC header too for commands |
| FM-PRO-02 | Header corruption | Bit flip in length/seq/flags | Length exact check catches most; seq/flags not CRC-covered | Mis-sequenced data | 2 | 2 | 2 | Magic/version/length; no header CRC | Cover header with CRC; sequence gap detection |
| FM-PRO-03 | Version mismatch | Old/new client | `negotiate` rejects wrong protocol; test | No connection | 3 | 2 | 1 | Version check | Keep; capability negotiation |
| FM-PRO-04 | Oversized payload | Malformed length | Rejected if >64 KiB; u16 length | DoS/memory | 2 | 2 | 1 | `MAX_BINARY_PAYLOAD`; length check | Keep; stream limits |
| FM-PRO-05 | Replay/duplicate command | No nonce/sequence enforcement | No command messages; telemetry seq only | Duplicate action if extended | 2 | 2 | 3 | None | Command nonce, idempotency, auth |
| FM-PRO-06 | JSON parse/DoS | Huge/invalid JSON | Axum/serde rejects; no explicit size limit in handler | Service memory/CPU | 2 | 2 | 2 | Framework defaults | Request size limit; timeouts; fuzz tests |
| FM-PRO-07 | WebSocket lag/drop | Slow client | Broadcast lag warning; frames dropped | Operator misses data | 3 | 3 | 2 | Lagged error message | Backpressure policy; snapshot/resync; sequence tracking |

**Protocol findings.** The binary frame is a reasonable telemetry container: magic, version, flags, sequence, length, payload CRC. It is not authenticated and does not protect the header. The JSON/WebSocket path has no CRC or auth; it relies on TCP/WebSocket integrity. Neither path carries calibration writes; calibration changes use HTTP endpoints, which is a separate trust boundary.

## 10. Scheduler and timing failures

| ID | Failure mode | Cause | Host effect in v0.1 | Hazard if actuated | S | L | D | Detection / current control | Mitigation / required evidence |
|---|---|---|---|---|---|---|---|---|---|
| FM-SCH-01 | Event buffer overflow | >12 cylinders or extra event types | `push()` silently drops at `MAX_EVENTS`; `TooManyEvents` never reached | Missing injection/spark | 1 | 2 | 3 | Fixed array; no overflow flag | Return error on push; assert capacity; test boundary |
| FM-SCH-02 | Pulse longer than cycle | Low RPM/high PW | `PulseLongerThanCycle` error; schedule empty | No fuel or continuous injection | 2 | 2 | 1 | Validation; test | Keep; cut fuel; driver timeout |
| FM-SCH-03 | Dwell longer than cycle | Low RPM/high dwell | Validation error | No spark/coil overheating | 2 | 2 | 1 | Validation | Keep; clamp and log |
| FM-SCH-04 | Invalid firing order | Config error | `InvalidFiringOrder`; simulator logs schedule error | Wrong cylinder | 1 | 2 | 1 | Permutation check; test | Keep; setup validation |
| FM-SCH-05 | Event collision/overlap | Dwell/injection windows overlap | No overlap check; 45% dwell cap reduces risk | Driver conflict, wrong timing | 2 | 2 | 3 | `limit_dwell` only | Add explicit overlap check; per-output budget |
| FM-SCH-06 | Angle wrap error | 720° boundary | `events_crossed` wrap logic; test `wrap_crossing_is_detected_once` | Missed/duplicate event | 2 | 2 | 2 | Test | Keep; property tests at boundaries |
| FM-SCH-07 | Quantization/jitter | f32 angles, 1 ms host step | Simulator event time quantized to 1 ms | Timing error on hardware if copied | 2 | 3 | 3 | No hardware timer | Use timer ticks/angle; measure jitter; see AUD-04 |
| FM-SCH-08 | Injection timing not calibrated | Fixed 240° BTDC in simulator | All cells use same injection end | Poor vaporization/driveability | 3 | 3 | 3 | No table | Add injection timing table; validate |
| FM-SCH-09 | Wasted-spark unsupported | `outputs_per_cylinder` u8, sequential assumption | No wasted-spark schedule | Cannot run wasted spark | 3 | 2 | 2 | Enum/logic | Implement or reject explicitly |

**Scheduler findings.** The scheduler computes deterministic angles and validates major impossible inputs. It is not a hardware scheduler and has no jitter evidence. The silent `push()` drop and lack of overlap check are design weaknesses that should be fixed before firmware reuse. The 1 ms host step is not a timing guarantee.

## 11. Processor, watchdog, and firmware failures

| ID | Failure mode | Cause | Host effect in v0.1 | Hazard if actuated | S | L | D | Detection / current control | Mitigation / required evidence |
|---|---|---|---|---|---|---|---|---|---|
| FM-PRC-01 | MCU hang | Firmware bug, clock failure, ESD | Firmware skeleton only; no HAL/driver | Outputs stuck at last state | 1 | 2 | 5 | `watchdog_ok` hard-coded true in simulator; skeleton accepts a fault flag | External window watchdog; safe output hardware |
| FM-PRC-02 | Watchdog not serviced correctly | Task starvation, priority inversion | Skeleton has no watchdog integration | Reset loop or no reset | 1 | 3 | 5 | None | Window watchdog; task check-in; reset-cause log |
| FM-PRC-03 | Watchdog resets during operation | Too-tight timeout, long flash/erase | Not modelled | Repeated resets, no start | 2 | 3 | 5 | None | Measure WCET; separate watchdog task; test |
| FM-PRC-04 | Brownout | Cranking, load dump, bad ground | None | Corruption/reset/actuator glitch | 1 | 3 | 4 | None | UVLO, reset supervisor, safe state; see AUD-03 |
| FM-PRC-05 | Flash corruption | Wear, power loss during write | Calibration checksum catches content change if stored | Wrong calibration/no boot | 1 | 2 | 2 | SHA-256; no dual-bank | Dual-bank/CRC; atomic update; recovery bootloader |
| FM-PRC-06 | Stack/heap overflow | Unbounded recursion/allocation | Host crates mostly bounded; simulator uses Vec; skeleton no heap | Crash/corruption | 2 | 2 | 3 | No stack analysis | Static stack analysis; no heap in ISR; MPU |
| FM-PRC-07 | ISR latency/jitter | Long ISR, priority inversion, cache | Skeleton has no ISR/timer implementation | Wrong spark/injection timing | 1 | 3 | 5 | None | Measure on target; priority design; see AUD-04 |
| FM-PRC-08 | Clock/oscillator failure | Crystal, PLL, EMI | Skeleton has no clock/HAL integration | Timing loss | 1 | 2 | 5 | None | Clock monitor; external watchdog; fail-safe |
| FM-PRC-09 | Debug access in production | Open SWD/JTAG | None | Unauthorised firmware/calibration | 2 | 2 | 4 | None | Disable/lock debug; authenticated recovery |

**Processor findings.** A `no_std` firmware skeleton now exists and compiles for `thumbv7em-none-eabihf`, but it has no HAL, timer capture/compare, injector/ignition driver, watchdog integration, flash/calibration storage, CAN or bootloader. The simulator's watchdog and CAN health inputs are hard-coded true, so the protection state machine's watchdog/CAN DTCs have no evidential value. Any claim of watchdog coverage is unsupported.

## 12. Power and electrical failures

| ID | Failure mode | Cause | Host effect in v0.1 | Hazard if actuated | S | L | D | Detection / current control | Mitigation / required evidence |
|---|---|---|---|---|---|---|---|---|---|
| FM-PWR-01 | Reverse polarity | Jump start, wiring error | None | ECU destruction, fire | 1 | 2 | 5 | None | Reverse-polarity protection; fused input; bench procedure |
| FM-PWR-02 | Load dump | Alternator load rejection | None | Overvoltage damage, actuator glitch | 1 | 2 | 5 | None | TVS/transient protection; validate to defined pulse |
| FM-PWR-03 | Cranking voltage sag | Starter inrush | Simulator battery 11.6 V during crank | Brownout/reset; injector dead time error | 2 | 4 | 3 | Simulator battery model; fuel bounds | UVLO; boost/hold-up; low-voltage calibration |
| FM-PWR-04 | Ground offset | Poor ground, sensor return current | None | Wrong sensor readings; communication errors | 2 | 3 | 4 | None | Star ground; differential inputs; ground test |
| FM-PWR-05 | Injector flyback | Inductive load turn-off | None | Driver avalanche/damage | 1 | 3 | 5 | None | Flyback/clamp; validated driver; scope |
| FM-PWR-06 | Coil back-EMF | Ignition turn-off | None | Driver damage, EMI | 1 | 3 | 5 | None | Igniter/coil interface protection; clamp |
| FM-PWR-07 | Fuse/connector overheating | Undersized wire/terminal | None | Fire/melted harness | 1 | 2 | 5 | None | Current budget; terminal rating; thermal test |
| FM-PWR-08 | Supply sequencing | 5 V/3.3 V/actuator rails | None | Driver glitch, MCU latch-up | 2 | 3 | 5 | None | Sequencing; enable/pulldown; safe default |
| FM-PWR-09 | Wideband heater load | High current at cold start | None | Voltage sag, connector heating | 2 | 3 | 5 | None | Separate rail/fuse; heater diagnostics; current limit |

**Power findings.** These are requirements for the future electrical design, not evidence. No input protection, regulator, ground, driver, flyback, thermal or EMC circuit exists. The simulator's battery model is not a power-supply validation.

## 13. Cross-cutting failure interactions

| Interaction | Why it matters | Current evidence | Required action |
|---|---|---|---|
| False lean + protection cut | A misfire/exhaust leak can read lean; `LeanUnderLoad` may cut fuel and latch, worsening the fault | Gate flags exist; protection uses same lambda; sensor-threshold latch confirmation fixed in host code (tests in §14), default N=3 | Distinguish sensor/combustion faults; validate limits and confirmation timing; cut reactions remain immediate |
| Stuck MAP + fuel + protection | Stuck MAP 55 kPa can produce plausible fuel; protection only derates, no fallback | Simulator `MapSensorStuck`; `sensor_plausible` derate | Plausibility vs TPS/RPM; default load; cut/limp strategy |
| Brownout + watchdog + calibration | Low voltage can corrupt flash and reset MCU; watchdog may not run | No hardware | UVLO, dual-bank flash, reset supervisor, safe outputs |
| Calibration tamper + ignition policy | Unkeyed hash + metadata-only "verified" policy | `seal`/`verify`; `IgnitionPolicy` | Signature/trust anchor; policy/table cross-check |
| Trigger loss + protection timeout | 250 ms extrapolation at high RPM | `tick()` timeout; `CrankSyncLost` | Speed-dependent timeout; immediate cut |
| Double knock retard + fuel learning | Over-retard changes torque/thermal state; logs mislead tuner | Code reading | Fix and test; log applied retard once |
| Service CORS + apply | A local web page can change calibration | `CorsLayer::permissive`; approve=true | Auth/CSRF; restrict origins |
| UI stale data + operator action | Operator applies based on old frame | Polling/WS; no age display | Timestamp/age; disable apply on stale data |

## 14. Test coverage map

| Failure area | Tests present | Coverage assessment |
|---|---|---|
| Trigger gap mismatch/dropped tooth | `extra_missing_tooth_causes_sync_loss`, `dropped_tooth_is_not_guessed`, `extra_tooth_between_gaps_is_rejected` | Good for eventual loss; not for wrong event before detection |
| Trigger acceleration | `acceleration_tracks_rpm_without_false_gap` | Acceleration only; no deceleration, no noise, no cam loss |
| Trigger timeout | `timeout_declares_lost_sync` | Passes; does not test high-RPM extrapolation |
| Trigger cam-window validation | `cam_inside_window_gives_sequential`, `cam_outside_window_is_rejected`, `cam_without_window_is_rejected`, `cam_rejection_revokes_validated_phase` | Fixed in host code; explicit window required when cam is required; physical Evo cam-pattern verification remains open |
| Wideband stuck | `stuck_wideband_is_detected_and_no_proposal_issued` | Simulator exact-constant case; noisy stuck sensor not covered |
| Wideband gate | `every_invalidity_is_explained`, `cold_engine_and_stale_sensor_are_explained` | Good gate-reason coverage; flags are synthetic |
| Fuel pulse/dead time/duty | 9 `ecu-fuel` unit tests + 2 Speeduino parity integration tests | Good unit coverage; no bench data; parity is formula-only |
| Mixture Map | 12 tests, all pass in current snapshot | Broad; trim term is verified with `max_cell_correction=0.20`; default 5% cap still binds |
| Ignition evidence | `missing_reference...`, `synthetic_or_unknown...`, `verified_reference...` | Good policy tests; no forged-policy test |
| Knock retard | `knock_produces_bounded_retard_not_fuel_learning` | Single-step; does not catch double application |
| Protection latching/recovery | 14 tests; new `single_sample_overtemp_does_not_latch`, `n_consecutive_samples_latch`, `clearing_before_n_samples_resets_confirmation`, `crc_mismatch_latches_on_first_evaluation` | Sensor-threshold latch confirmation fixed in host code; overtemperature tests cover one sample, N samples and count reset; existing overtemperature/oil latch tests feed N samples; existing CRC test unchanged; hardware noise/fault-injection validation remains open |
| Scheduler | 6 tests | Good angle/cut tests; no overflow/overlap test |
| Protocol | 7 tests | Good CRC/version tests; no header corruption/auth |
| Calibration | 22 tests including vehicle profile | Good checksum/revision; no signature/durability |
| Simulator scenarios | 10 tests; CLI determinism PASS for fixed and fresh calibration | Deterministic; idle/transient realism remains weak |
| Firmware skeleton | 5 tests; `thumbv7em-none-eabihf` check | No HAL/timer/driver/watchdog; fail-closed interface only |
| Service/UI | No Rust integration tests; UI typecheck/build PASS; Electron launch not tested | No auth/CSRF/concurrency tests; browser/Electron not tested |

## 15. Highest-priority FMEA actions

1. **Trigger integrity:** speed-dependent timeout, immediate cut on missing expected edge, cam-loss detection, extra-edge rejection, deceleration tests.
2. **Protection confirmation fixed in host code (A3):** coolant over-temperature, low oil pressure, lean under load, overboost and low battery voltage are sensor-threshold faults; their latching cut conditions require N consecutive evaluations per DTC (`ProtectionLimits::latch_confirmation_samples`, default 3; zero treated as one). A clear or derate-only sample resets confirmation. Tests are listed in §14; `overtemp_latches_and_requires_explicit_clear` and `low_oil_pressure_cuts_and_latches` now feed N samples. Calibration checksum/CRC and watchdog are deterministic integrity faults and retain immediate first-evaluation latching; the unchanged `calibration_crc_failure_latches_a_cut` and new `crc_mismatch_latches_on_first_evaluation` cover CRC. All cut/derate reactions remain immediate; non-latching faults including low fuel pressure are unchanged. Thresholds are unchanged placeholders; define false-trip vs missed-trip targets and validate confirmation timing and sensors physically.
3. **Calibration trust:** keyed signature/HMAC, policy/table provenance cross-check, durable audit log, actor identity.
4. **Service security:** authentication/authorization/CSRF, restrict CORS, never expose beyond localhost, transactional apply.
5. **Knock path:** fix double retard, add test, implement real knock signal processing and validation.
6. **Mixture Map auditability:** separate STFT/LTFT means, document the default 5% per-cell cap, add noisy-stuck-sensor and delay-sensitivity tests.
7. **Hardware prerequisites:** watchdog, UVLO, flyback, driver diagnostics, safe defaults, thermal/EMC; no hardware exists and the firmware skeleton has no HAL/timer/driver.
8. **Evo evidence:** pinout, connector, trigger, injector, ignition, harness; all unverified and blocking Gate 3.
