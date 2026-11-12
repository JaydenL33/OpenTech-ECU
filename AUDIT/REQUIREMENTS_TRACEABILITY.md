# Requirements Traceability (v0.1 snapshot)

| Field | Value |
|---|---|
| Document ID | AUD-06 |
| Status | Audit traceability matrix; not a certification or release approval |
| Date | 2026-11-12 |
| Evidence cut-off | 2026-11-13, working tree; source manifest `a262e995...` |
| Sources | `ECU_MASTER_BRIEF.md`; project context; `ecu-calibration/src/vehicle.rs`; code/tests in `ecu/` |
| Related | AUD-01, AUD-02, AUD-05, AUD-08, AUD-10 |

## 1. How to read this matrix

**Status labels**

| Label | Meaning in this audit |
|---|---|
| Implemented | Host-side code exists and at least one passing unit test or direct command demonstrates the behaviour. Host-only; not hardware-validated. |
| Simulated | Behaviour exists only inside the virtual ECU/simulator. |
| Partial | Some code/evidence exists, but coverage is incomplete or a material limitation remains. |
| Planned | Design/roadmap only; no working code or evidence. |
| Blocked | Prohibited or impossible until hardware, vehicle evidence, or independent review exists. |
| Unverified | No evidence to assess; treat all values as unknown. Never fill from assumption. |

**Rule:** a requirement is not "done" because a module or UI page exists. The evidence column must identify a test, command, source invariant, or explicit absence. A passing host unit test does not establish engine or vehicle safety.

## 2. System and platform requirements

| ID | Requirement | Source | Implementation / evidence | Status | Notes / blocker |
|---|---|---|---|---|---|
| SYS-001 | Rust-first Cargo workspace with separate core crates | Brief §4 | `ecu/Cargo.toml`; 15 workspace members (12 libs, 2 apps, firmware skeleton) | Implemented | Host-only; Tauri shell absent; current UI uses Electron |
| SYS-002 | Deterministic host virtual ECU with fixed-step engine model | Brief §1, §5 | `ecu-simulator`; `step_once` 1 ms; CLI determinism PASS | Simulated | Deterministic; starter calibration uses a deterministic zero timestamp |
| SYS-003 | No physical output path in v0.1 | Project context | No hardware driver/PCB; firmware skeleton forces outputs off; `PHYSICAL_OUTPUT_INTERLOCK_ENGAGED=true` | Implemented (snapshot) | Source committed; interlock enforcement still needs independent review |
| SYS-004 | Versioned protocol with CRC and explicit negotiation | Brief §4 | `ecu-protocol`; 7 tests | Implemented | Binary CRC covers payload, not header; no auth |
| SYS-005 | Real tuning UI, not mock controls | Brief §1, §7 | `apps/tuning-studio` React/Vite + Electron main; build PASS; headless Chromium smoke test PASS | Partial | Electron launch not tested; no formal visual/accessibility suite; Tauri not used because WebKitGTK libs are unavailable |
| SYS-006 | Bounded memory, no heap allocation in hard real-time paths | Brief §4 | Core control crates use fixed arrays; skeleton has no heap | Partial | Simulator uses `Vec`; no ISR/timer exists to assess |
| SYS-007 | Embedded-relevant crates compile under `no_std` | Brief §4 | All six core control crates pass no-default-features; firmware skeleton target check passes | Implemented | `ecu-protocol`/`ecu-lambda` remain std-only; no HAL/drivers yet |
| SYS-008 | `cargo fmt`, clippy, tests, CI | Brief §4 | fmt/clippy/test pass locally; CI workflow defined | Partial | CI not observed running |
| SYS-009 | Preserve GPL/CERN-OHL-S licences and provenance | Brief §2 | GPL-2.0-or-later workspace; Speeduino tree retained | Partial | No per-module provenance registry; no hardware design yet |
| SYS-010 | Independent engineering review before physical use | Brief §7 | None | Planned | Gate 0-5 process; no reviewer sign-off exists |
| SYS-011 | Numeric parity pin against Speeduino fuel arithmetic | Brief §2, §4 | `ecu/crates/ecu-fuel/tests/speeduino_parity.rs`; 2 tests | Implemented (narrow) | Formula comparison only; not a full C++ binary/oracle build |

## 3. Simulation requirements

| ID | Requirement | Source | Implementation / evidence | Status | Notes / blocker |
|---|---|---|---|---|---|
| SIM-001 | 4-stroke engine model, 1-12 cylinders, firing order, displacement, compression, aspiration | Brief §3A | `ecu-types::EngineConfig`; `ecu-simulator` | Simulated | Physics-lite; not a validated engine model |
| SIM-002 | Configurable crank/cam trigger pattern | Brief §3A | Missing-tooth decoder; `TriggerConfig` | Simulated | Dual-wheel not implemented; Evo pattern unverified |
| SIM-003 | Crank-tooth timestamp model and missing-tooth decoder | Brief §3A | `ecu-trigger`; 10 tests | Implemented | Host model only; no capture hardware |
| SIM-004 | Phase/sync state machine with deterministic loss response | Brief §3A | `SyncState`; gap/cam/timeout tests | Implemented | Cam-loss timeout absent; 250 ms extrapolation gap |
| SIM-005 | Strongly typed sensor pipeline with invalid/stale status | Brief §3A | `SensorStatus`, `SensorSample`, `TelemetryFrame` | Partial | Control paths mostly use raw `f32`; only lambda status is wired through |
| SIM-006 | Deterministic seeded scenarios and logs | Brief §5 | 6 scenarios; CLI `--check-determinism` PASS; fresh calibration checksum deterministic | Simulated | `idle_warmup`/`transient` are not realistic engine validation |
| SIM-007 | Telemetry log, CSV, replay | Brief §3G | `LogFile`, `to_csv`, `ReplayCursor`; 4 diagnostics tests | Implemented (host) | Replay start/stop and Logs UI exercised; logs ephemeral |
| SIM-008 | Fault injection: sensor, sync, overtemp, overboost, lean, CAN, watchdog | Brief §3E | `SimFault` windows; 6 scenarios | Partial | Watchdog/CAN inputs hard-coded healthy; no hardware fault injection |
| SIM-009 | Cranking -> sync -> idle -> transient -> fault -> recovery acceptance | Brief §5 | `idle_warmup`, `fault_recovery`, `trigger_loss` | Partial | Observed idle/transient mean RPM very low; tests assert only some `Running` frame |
| SIM-010 | Crank acceleration/deceleration, rollover, missing teeth | Brief §5 | `acceleration_tracks_rpm_without_false_gap`; missing-tooth tests | Partial | Deceleration and high-RPM extrapolation not tested |
| SIM-011 | Exhaust delay, sensor lag, noise model | Brief §3B | `update_wideband`; scenario parameters | Simulated | Fixed delay/lag; no real sensor validation |
| SIM-012 | Deterministic replay for same seeded log | Brief §5 | `deterministic_for_same_log`; CLI PASS | Implemented | Fresh starter calibration is deterministic; logs are reproducible |

## 4. Fuel calculation requirements

| ID | Requirement | Source | Implementation / evidence | Status | Notes / blocker |
|---|---|---|---|---|---|
| FUEL-001 | Speed-density air mass with documented assumptions | Brief §3A | `air_mass_mg`; `air_mass_matches_hand_calculation` | Implemented | Ideal gas; no wall wetting; VE is table input |
| FUEL-002 | Injector pulse width, dead time, pressure compensation | Brief §3A | `calculate`; 9 tests | Implemented | No flow bench; pressure model is returnless sqrt |
| FUEL-003 | Duty limit and explicit inhibition, no silent clamp | Brief §3A | `InhibitReason::DutyExceeded`; test `excessive_duty_is_reported_not_silently_clamped` | Implemented | Caller must honor inhibition; no hardware gate |
| FUEL-004 | Alpha-N / blended load | Brief §3A | `blended_load`; test | Implemented | Not integrated into simulator beyond default speed-density |
| FUEL-005 | Transient enrichment, wall wetting, cranking, after-start, warm-up | Brief §3A, P1 | None | Planned | No fuel-film model |
| FUEL-006 | Sequential, batch, staged injection | Brief §3A | Simulator sequential schedule; one injector config | Partial | Staged/batch/per-injector summation not implemented |
| FUEL-007 | Per-cylinder trims | Brief §3A, P1 | None | Planned | One PW for all cylinders |
| FUEL-008 | Fuel pressure compensation and regulator strategy | Brief §3C | `pressure_compensation`; `fuel_pressure_kpa`; preflight warning | Partial | Returnless sqrt only; no regulator dynamics |
| FUEL-009 | Injector characterization import/library | Brief §3B, P2 | None | Planned | Preflight only warns that data is unverified |
| FUEL-010 | Minimum-pulse diagnostics | Brief §3A | `min_pulse_clamped`; test | Implemented | Delivers more than requested; reported |

## 5. Ignition requirements

| ID | Requirement | Source | Implementation / evidence | Status | Notes / blocker |
|---|---|---|---|---|---|
| IGN-001 | Editable ignition table with bounded advance | Brief §3A, §3D | `ignition` table; `corrected_advance`; 7 tests | Implemented | Synthetic map is simulation-only |
| IGN-002 | Dwell table and dwell-duty limit | Brief §3A | `limit_dwell`; test `dwell_is_reduced_at_high_rpm_but_never_zero` | Implemented | Generic 45% cap; no coil datasheet |
| IGN-003 | Rev limiter with hysteresis | Brief §3A | `RevLimiter`; test | Implemented (host) | Cuts spark only; fuel cut separate |
| IGN-004 | Evidence-gated ignition recommendation | Brief §3D, §5 | `recommend_from_reference`; 7 tests | Partial | "Verified reference" is caller metadata, not authenticated |
| IGN-005 | Lambda never an ignition input | Brief §1, §3D | No lambda parameter in `ecu-ignition`; tests | Implemented | Review invariant; future edit could violate |
| IGN-006 | Timing-light calibration workflow | Brief §3D, §7 | Docs/checklist only | Planned / Blocked | No measurement record |
| IGN-007 | Knock monitoring/retard | Brief §3D, §3E | `KnockHigh`; simulator fault; bounded retard | Partial | No signal processing; apparent double-retard bug |
| IGN-008 | Per-cylinder ignition trims/individual coils | Brief §3A, P1 | None | Planned | Scheduler supports firing order only |
| IGN-009 | Coil/igniter compatibility | Brief §3H | None | Unverified | Evo topology unknown |
| IGN-010 | No automatic spark advance based on lambda | Brief §1, §3D | No code path | Implemented | Preserve as invariant |
| IGN-011 | Import licensed verified map with provenance | Brief §3D, P1 | `IgnitionPolicy::VerifiedReference`; wizard path | Partial | No licence registry; no signature |

## 6. Trigger requirements

| ID | Requirement | Source | Implementation / evidence | Status | Notes / blocker |
|---|---|---|---|---|---|
| TRG-001 | Missing-tooth crank decoder | Brief §3A | `MissingToothDecoder`; 10 tests | Implemented (host) | 36-1 is simulation default, not Evo-verified |
| TRG-002 | Cam phase latching and sequential state | Brief §3A | `on_cam`; `SyncState::Sequential` | Implemented | No cam-loss timeout |
| TRG-003 | Gap mismatch/dropped tooth -> sync loss | Brief §3A | `GapMismatch`, `ToothDropped`; tests | Implemented | Extra short edge not rejected until next gap |
| TRG-004 | Timeout -> lost sync/fuel cut | Brief §3A | `tick`; `CrankSyncLost`; test | Implemented | 250 ms default too long at high RPM; angle extrapolated |
| TRG-005 | Config validation | Brief §3A | `MissingToothConfig::validate`; tests | Implemented | Dual-wheel rejected |
| TRG-006 | Actual Evo crank/cam pattern | Project context | `vehicle.rs` flags; no scope data | Unverified | Do not assume 36-1 |
| TRG-007 | Absolute TDC/offset calibration | Brief §3D | `sync_offset_deg`; preflight warning | Planned / Blocked | Timing light required |
| TRG-008 | Dual-wheel/other patterns | Brief §3A P1 | `TriggerPattern::DualWheel` rejected | Planned | Not implemented |
| TRG-009 | Trigger noise/EMI robustness | Brief §3H | None | Planned / Blocked | Hardware/harness test needed |

## 7. Scheduler requirements

| ID | Requirement | Source | Implementation / evidence | Status | Notes / blocker |
|---|---|---|---|---|---|
| SCH-001 | 720-degree injection/dwell/spark schedule | Brief §3A | `build_schedule`; 6 tests | Implemented (host) | Not executed by hardware timers |
| SCH-002 | 1-12 cylinders and arbitrary firing order | Brief §3A | `MAX_CYLINDERS`; test 12-cyl | Implemented | 12-cyl buffer exactly full |
| SCH-003 | Independent fuel/spark cut | Brief §3A | `fuel_cut`/`spark_cut`; test | Implemented | Simulator only |
| SCH-004 | Overflow/collision handling | Brief §4 | Fixed buffer; no overlap check | Partial | `push()` silently drops; `TooManyEvents` unreachable |
| SCH-005 | Hardware timer execution with bounded jitter | Brief §4 | None | Planned / Blocked | See AUD-04 |
| SCH-006 | Configurable injection timing | Brief §3A P1 | Fixed 240° BTDC in simulator | Planned | No table |
| SCH-007 | Wasted-spark/batch schedule | Brief §3A P1 | None | Planned | Sequential assumption |
| SCH-008 | Event ordering/wrap tests | Brief §5 | `wrap_crossing_is_detected_once`; sorted test | Implemented (host) | No property/fuzz tests |

## 8. Lambda and fuel-learning requirements

| ID | Requirement | Source | Implementation / evidence | Status | Notes / blocker |
|---|---|---|---|---|---|
| LAM-001 | Target-lambda table distinct from AFR | Brief §3B | `target_lambda`; `stoich_afr`; wizard | Implemented | Table values are estimates |
| LAM-002 | Shared learning gate with reasons | Brief §3B | `gate.rs`; `every_invalidity_is_explained` | Implemented | Several inputs are caller-supplied booleans |
| LAM-003 | Bounded STFT with anti-windup/freeze/decay | Brief §3B | `ClosedLoop`; 6 tests | Implemented | Gains not hardware-validated |
| LAM-004 | Cell/bank-aware LTFT with limits/persistence | Brief §3B P1 | `LongTermTrimTable`; 5 tests | Implemented (host) | Persistence is JSON round-trip, not service durability |
| LAM-005 | Offline Mixture Map with coverage/confidence/abstention | Brief §3B, §5 | `mixture_map.rs`; 12 tests pass | Partial | `mean_ltft=0`; default 5% per-cell cap binds; provenance labels need improvement |
| LAM-006 | Time alignment to exhaust delay | Brief §3B | `aligned_index` | Implemented | Fixed/estimated delay; no real sensor timing |
| LAM-007 | WOT/high-load excluded by default | Brief §3B | `allow_wot=false`; tests | Implemented | Explicit opt-in exists |
| LAM-008 | Reject misfire/exhaust leak/overrun/knock/DTC | Brief §3B | Gate flags; simulator faults; tests | Partial | Real detectors absent |
| LAM-009 | Bank separation / dual wideband | Brief §3B P1 | `BankTrims`; one sensor default | Partial | Bank 2 optional; no dual-CAN path |
| LAM-010 | Avoid double-correcting with active trims | Brief §3B P1 | `raw = lambda ratio * (1+trim)`; trim test passes with `max_cell_correction=0.20` | Partial | Default cap still 5%; test/config distinction must be documented |
| LAM-011 | Bounded proposal, preview, selective apply/undo | Brief §3B, §5 | `apply_proposal`; CLI; service | Implemented (host) | History in memory; no actor identity |
| LAM-012 | Never apply directly to physical injectors | Brief §3B | No physical path; apply returns calibration only | Implemented (snapshot) | Future hardware must preserve |
| LAM-013 | Per-cylinder lambda/trim | Brief §3B P1 | None | Planned | One sensor per bank cannot resolve cylinders |
| LAM-014 | Sensor aging/free-air calibration | Brief §3B P1 | None | Planned | No sensor health model |

## 9. Protection requirements

| ID | Requirement | Source | Implementation / evidence | Status | Notes / blocker |
|---|---|---|---|---|---|
| PROT-001 | Independent protection state machine | Brief §3E | `ecu-protection`; separate crate | Implemented (software) | Same process; no hardware monitor |
| PROT-002 | DTC set with codes/titles | Brief §3E | 14 DTCs; `Dtc::ALL` | Implemented | Codes are generic OBD-style, not Evo-specific |
| PROT-003 | Latching and explicit clear | Brief §3E | `clear_latched`; tests | Implemented | No operator identity; endpoint unauthenticated |
| PROT-004 | Freeze frames | Brief §3E | `FreezeFrame`; stored per fault | Implemented | Not durably persisted |
| PROT-005 | Warn/derate/limp/cut reactions | Brief §3E | `ProtectionState`; `Reaction` | Partial | `Limp` never selected; thresholds placeholder |
| PROT-006 | Oil/overtemp/overboost/lean/low-voltage | Brief §3E | `evaluate`; tests | Implemented (host) | No debounce; false/missed trip risks |
| PROT-007 | Watchdog DTC and safe state | Brief §3E | `Dtc::Watchdog`; input hard-coded true | Partial / Blocked | No hardware watchdog |
| PROT-008 | CAN-loss DTC | Brief §3E | `Dtc::CanBusLost`; input hard-coded true | Partial / Blocked | No CAN driver |
| PROT-009 | Debounce/confirmation | Brief §3E P1 | None | Planned | Single-sample latch is a material gap |
| PROT-010 | Engine-specific thresholds | Brief §3E P1 | Defaults only | Blocked | Evo limits unknown |
| PROT-011 | Hardware fault injection | Brief §3E P2 | None | Blocked | Bench plan only |
| PROT-012 | FTA/diagnostic coverage | Brief §3E P1 | Qualitative FMEA only | Planned | No quantitative FMEDA |

## 10. Calibration and wizard requirements

| ID | Requirement | Source | Implementation / evidence | Status | Notes / blocker |
|---|---|---|---|---|---|
| CAL-001 | 1D-4D tables, bounded axes | Brief §3C | `table.rs`; 8 tests | Implemented | Max 32 points/axis |
| CAL-002 | Clamped interpolation, no extrapolation | Brief §3C | `interpolate`; tests | Implemented | Nearest-cell learning separate |
| CAL-003 | Schema/axis/value validation | Brief §3C | `validate`; tests | Implemented | Some semantic checks only in preflight |
| CAL-004 | SHA-256 sealed revisions with parent | Brief §3C, §4 | `seal`/`verify`/`fork`; tests | Implemented | Unkeyed hash; not authenticity |
| CAL-005 | Signature/trusted provenance | Brief §3C, §3D | None | Planned | Critical for future physical ignition |
| CAL-006 | Starter wizard with estimated maps | Brief §3C | `generate_starter`; 4 tests | Implemented | Estimates only; no engine validation |
| CAL-007 | Preflight blocking hazardous combinations | Brief §3C P1 | `preflight`; 3 tests | Partial | Warnings (sensor/injector/timing light) do not block `hardware_candidate` |
| CAL-008 | Ignition policy/provenance | Brief §3D | `IgnitionPolicy`; wizard path | Partial | Policy is metadata; no table/policy cross-check |
| CAL-009 | Versioned import/export | Brief §3C P1 | JSON load/export; CLI `--cal`; UI export | Partial | No import UI; no migration tests |
| CAL-010 | Diff/compare/rollback | Brief §3C P1 | `diff_tables`; revision history; `undo` | Partial | History in memory; no two-way merge |
| CAL-011 | Durable apply/undo audit | Brief §3C P1 | Service history vector | Partial | Lost on restart; no actor identity |
| CAL-012 | Map templates with provenance | Brief §3C P1 | Provenance fields; synthetic labels | Implemented (host) | No curated licensed template library |
| CAL-013 | I/O pin conflict detector | Brief §3C P1 | Preflight duplicate-pin check | Implemented | Only declared assignments; no hardware pinmux |
| CAL-014 | Sensor calibration workflow | Brief §3C P1 | Flags only | Partial | No transfer-function editor/validation |
| CAL-015 | Injector data validation | Brief §3C P1 | Bounds; preflight warning | Partial | No flow bench/import wizard |
| CAL-016 | Evo simulation profile | Project context | `vehicle.rs`; `evo_profile_is_fail_closed` | Implemented (sim) | Explicitly unverified; 12 blocking items |
| CAL-017 | Physical Evo profile | Project context | `physical_request()` refuses | Blocked | Requires Gate 3 evidence |

## 11. Protocol, service and UI requirements

| ID | Requirement | Source | Implementation / evidence | Status | Notes / blocker |
|---|---|---|---|---|---|
| COM-001 | JSON envelope with protocol version | Brief §4 | `JsonEnvelope`; test | Implemented | No auth/CRC |
| COM-002 | Binary frame with magic/version/length/CRC | Brief §4 | `encode/decode`; 5 tests | Implemented | Header not CRC-covered |
| COM-003 | No blind writes to ECU memory | Brief §4 | Protocol has no memory-write message; calibration via service | Implemented | Future firmware must preserve |
| COM-004 | Calibration apply via authoritative service | Brief §4 | `mixture_apply`; checks pause/approve/checksum | Implemented (host) | Local unauth HTTP |
| COM-005 | Authentication/authorization | Brief §4 | None | Planned | Must precede hardware |
| COM-006 | CAN interface / external wideband | Brief §3H | No CAN driver; protocol only | Planned / Blocked | No hardware |
| COM-007 | Live dashboard/maps/mixture/diagnostics UI | Brief §3G | React pages + Electron main; build PASS; headless Chromium smoke test PASS | Partial | Electron not launched; no accessibility/visual-regression suite |
| COM-008 | Tauri v2 desktop package | Brief §3G | `apps/tuning-studio/electron` exists; no Tauri `src-tauri` | Planned / Partial | Brief specified Tauri; current shell is Electron; runtime not validated |
| COM-009 | Read-only Mastra copilot | Brief §3G, §7 | `apps/tuning-copilot`; read-only tools, structured output, offline mode exercised; `tsc` PASS | Implemented (host) | LLM mode not called; no prompt-injection eval suite or durable audit |
| COM-013 | Bench harness/I/O visibility | Project extension | `GET /api/harness`; Harness page; 2 `ecu-harness` manifest tests | Implemented (host) | Bench manifest is UNREVIEWED; no measurement or physical approval |
| COM-010 | Log record/replay/CSV | Brief §3G | `LogFile`, CSV, `ReplayCursor` | Implemented (host) | No browser replay validation |
| COM-011 | Protocol capability negotiation | Brief §4 | `negotiate` lists capabilities | Implemented | Capabilities are informational only |
| COM-012 | Message size/backpressure limits | Brief §4 | Binary 64 KiB; WS lag warning | Partial | JSON body limit not explicit |

## 12. Firmware skeleton requirements

| ID | Requirement | Source | Implementation / evidence | Status | Notes / blocker |
|---|---|---|---|---|---|
| FW-001 | Core control logic shall have a `no_std` compilation path for the selected MCU | Brief §4 | `opentech-firmware-stm32` target check passes for `thumbv7em-none-eabihf` | Implemented (skeleton) | Skeleton only; no HAL/timer/driver |
| FW-002 | Trigger capture to decoder path shall be wired to a hardware timer capture | Brief §4 | No HAL/capture implementation | Planned / Blocked | Gate 2 |
| FW-003 | Injector/ignition outputs shall be driven by hardware compare/one-shot timers | Brief §4 | No output compare/driver | Planned / Blocked | Gate 2 |
| FW-004 | Watchdog, reset-cause and safe-state integration | Brief §3H | Skeleton accepts a watchdog flag; no hardware watchdog | Planned / Blocked | Gate 2 |
| FW-005 | Flash/calibration storage, bootloader and recovery | Brief §4 | None | Planned / Blocked | Gate 2 |
| FW-006 | Physical output interlock remains false until Gate 1-4 evidence | Brief §4, §7 | `PHYSICAL_OUTPUTS_AUTHORISED=false`; 5 tests | Implemented (skeleton) | No hardware path |

## 13. Hardware requirements

| ID | Requirement | Source | Implementation / evidence | Status | Notes / blocker |
|---|---|---|---|---|---|
| HW-001 | MCU trade study (STM32H7 vs S32K) | Brief §3H | **v1 baseline decision:** Arduino Mega 2560 shield (Speeduino v0.4.4d) for firmware compatibility. Long-term STM32H7/S32K trade study remains open | Partial | v1 uses the proven AVR platform; future MCU selection still needs a dev-board proof (`AUDIT/HARDWARE_BASELINE.md` §3) |
| HW-002 | KiCad schematic/PCB/BOM/gerbers | Brief §3H | Native KiCad 9 review copy of Speeduino v0.4.4d: 7 sheets, routed PCB (0 unconnected), gerbers/drill/BOM/position/IPC-D-356 generated | Partial | `hardware/prototype-v1/`; H2 exceptions, BOM MPNs and assembly drawing remain open |
| HW-003 | ERC/DRC with issue log | Brief §3H | ERC 101 (44 errors); DRC 600 (230 errors) after zone refill and R-01…R-03; 0 unconnected; connectivity PASS 514/514 | Partial / Blocked | `AUDIT/PCB_CHECK_RESULTS.md`; exceptions must be resolved/accepted before H2 |
| HW-004 | Power input protection/transients | Brief §3H | Upstream v0.4.4d input protection circuit present; not independently reviewed or tested | Partial | No load-dump/transient test; vehicle use blocked |
| HW-005 | Injector low-side drivers/flyback | Brief §3H | Upstream 4-channel injector drivers present; Evo injector impedance/current unverified | Partial | Bench dummy-load test required; see `AUDIT/PCB_CHECK_RESULTS.md` |
| HW-006 | Ignition outputs/igniter compatibility | Brief §3H | Upstream 4 logic-level ignition outputs present; Evo coil/igniter topology unverified | Partial | Do not connect coils; external verified igniter only |
| HW-007 | Knock interface | Brief §3H | **Not present on v0.4.4d** | Blocked | Requires future hardware expansion (`HW-EXP-01`) |
| HW-008 | CAN physical layer | Brief §3H | **Not present on v0.4.4d** | Blocked | Requires future hardware expansion; serial/USB only on v1 |
| HW-009 | Connector/adapter harness | Brief §3H | IDC40 + screw terminals present; Evo connector unknown | Blocked | Bench breakout `HW-BENCH-01`; vehicle harness blocked |
| HW-010 | Watchdog/safe state | Brief §3H | AVR watchdog available in firmware; no external hardware watchdog or validated safe-state circuit | Partial | Bench fault injection required |
| HW-011 | Thermal/EMC/mechanical | Brief §3H | No thermal/EMC test; no assembly/enclosure drawing | Blocked | `HW-MECH-01`; H3 package incomplete |
| HW-012 | Bench fixture/E-stop | Brief §3H | Plan only (AUD-09); no IDC40 breakout | Planned | `HW-BENCH-01` |
| HW-013 | Automotive qualification | Brief §3H | None | Blocked | No AEC-Q/ISO tests |
| HW-014 | Functional safety case | Brief §3H | Qualitative draft (AUD-01) | Partial | Not ISO 26262/IEC 61508 |

## 14. Evo VIII target requirements and unverified items

The following are **not** requirements satisfied by code. They are evidence items that must be resolved against the physical vehicle. The repository's `vehicle.rs` currently sets `verification: Unverified`, `ecu_part_number: None`, and `harness_documented/connector_documented/pinout_documented/trigger_verified/injector_data_verified/ignition_data_verified = false`. `physical_request()` refuses with `EVO_PHYSICAL_CONFIG_BLOCKED` and lists 12 blocking items.

| ID | Item | Why it blocks | Verification method | Status |
|---|---|---|---|---|
| EVO-001 | Exact ECU part number | Connector/adapter selection impossible | Photograph label/housings; record all markings | **Unverified** |
| EVO-002 | ECU connector part numbers/keying/terminals | Plug-and-play cannot be claimed | Housing markings; service data; terminal series | **Unverified** |
| EVO-003 | ECU pinout | Wrong pin can destroy ECU/vehicle circuits | Key-off continuity, pin-to-pin checks, second-person review | **Unverified** |
| EVO-004 | Crank/cam trigger pattern | Wrong decoder/angle; engine damage | Scope crank/cam at cranking; count teeth/gap/phase | **Unverified** |
| EVO-005 | Injector type/flow/dead time/resistance | Fuel model and driver rating | Markings; bench flow/dead-time; resistance/inductance | **Unverified** |
| EVO-006 | Coil/igniter topology | Output stage may be incompatible | Trace outputs; measure polarity/primary resistance | **Unverified** |
| EVO-007 | Fuel pump/relay control | Failsafe/polarity unknown | Verify relay coil/control; key-on/key-off | **Unverified** |
| EVO-008 | Knock sensor interface | Gain/filter/window unknown | Sensor type/shielding; controlled excitation | **Unverified** |
| EVO-009 | MAP/TPS/CLT/IAT sensor transfer functions | Wrong load/fuel | Datasheet/measurement; calibration | **Unverified** |
| EVO-010 | Wideband provision/controller | No lambda without it | Decide external controller/CAN; document | **Unverified** |
| EVO-011 | ACD/AYC CAN interaction | Factory chassis control may break | Capture CAN with factory ECU; authoritative data | **Unverified** |
| EVO-012 | Immobiliser/security | Bypass prohibited; vehicle may not start | Document factory behaviour; legitimate integration | **Unverified / prohibited to bypass** |
| EVO-013 | VSS/gear/clutch signals | Six-speed functions need them | Scope/measure; gear ratios | **Unverified** |
| EVO-014 | Harness condition/repairs | Previous damage/repairs unknown | Physical inspection; continuity | **Unverified** |
| EVO-015 | Legal/emissions/insurance | Road use may be unlawful | Jurisdiction review | **Unverified** |

**Do not invent any of these values.** A forum post, a similar model, a pinout image without provenance, or an LLM answer is not verification. The current simulation template uses 36-1, 1997 cc, 8.8:1, 560 cc/min and firing order 1-3-4-2 as **explicit synthetic placeholders**; none is a verified Evo value.

## 15. Release gates (summary; details in AUD-08)

| Gate | Definition | Current status | Blocking evidence |
|---|---|---|---|
| Gate 0 | Software simulation | **PASS, scoped** | 130/130 tests; fmt/clippy pass; firmware skeleton target check; UI build; copilot typecheck. Not a certification |
| Gate 1 | Unpowered hardware verification | **Blocked (partial evidence)** | Routed v0.4.4d baseline + native KiCad + exports exist, 0 unconnected; ERC/DRC exceptions, BOM gaps, assembly drawing and independent review still open |
| Gate 2 | Powered bench verification | **Blocked** | No HAL/timer/driver firmware image, bench fixture, dummy loads, E-stop |
| Gate 3 | Vehicle electrical compatibility | **Blocked / Unverified** | No Evo pinout/connector/trigger/harness evidence |
| Gate 4 | Controlled engine testing | **Blocked** | No bench engine, timing light, knock instrumentation, supervision |
| Gate 5 | Vehicle operation | **Blocked** | No legal/insurance/validation; no road use |

## 16. Coverage summary and gaps

A numeric percentage would be misleading because requirements differ in scope, evidence quality and criticality. The qualitative summary is:

| Area | Strongest current evidence | Dominant gap |
|---|---|---|
| System/platform | Cargo workspace, protocol, UI build, CI workflow, all core no_std checks pass | CI not observed running; protocol/lambda std-only |
| Simulation | Deterministic fixed and fresh runs; six scenarios | Idle/transient realism; no hardware |
| Fuel | Unit-tested air mass, pulse width, dead time, duty inhibition; 2 Speeduino parity tests | No flow bench; no transient/wall-wetting; no per-cylinder |
| Ignition | API excludes lambda; bounded advance/dwell; evidence policy | No authenticated evidence; no knock validation; apparent double retard |
| Trigger | Gap/dropped-tooth tests; sync state machine | Cam-loss detection; extra-edge latency; 250 ms timeout; Evo pattern unverified |
| Scheduler | Angle-domain tests; firing-order validation | Silent overflow path; no overlap check; no hardware timer |
| Lambda | Shared gate, bounded STFT/LTFT, Mixture Map tests pass | Real misfire/leak/stuck detection absent; default cap; trim labels misleading |
| Protection | Separate state machine; latching; 10 tests | No debounce; placeholder thresholds; watchdog/CAN hard-coded healthy |
| Calibration | Tables, checksum revisions, wizard, preflight | Unkeyed hash; warnings not blocking; no durable audit trail |
| Protocol/UI | CRC frame tests; UI build and headless Chromium smoke test; copilot typecheck | No auth/CSRF; header not CRC-covered; no Electron launch or live LLM test; harness source unreviewed |
| Firmware | no_std skeleton target check; 5 tests | No HAL/timer/driver/watchdog; not a controller |
| Hardware | Unreviewed schematic/BOM/ERC; unrouted PCB with failing DRC; no gerbers | No manufacturing package, thermal/EMC, bench |
| Evo evidence | Fail-closed `vehicle.rs` profile | All 15 items unverified; physical configuration refused |

The dominant pattern is: **host software exists for the core algorithms; hardware, vehicle evidence, security, timing and independent review are absent.**

## 17. Traceability maintenance rules

1. Every code change that touches safety invariants (interlock, ignition evidence, gates, protection, apply/approval, trigger sync) must update the relevant requirement row and AUD-05 evidence.
2. A requirement may move from Partial to Implemented only with a named test or command result, not because "the code looks right".
3. A requirement may not move from Unverified to Verified without a recorded measurement, source document, and reviewer.
4. Gate status may not be promoted by a code change alone; see AUD-08.
5. If a requirement is deleted or descoped, record why and who approved it. Do not silently remove a safety requirement.
