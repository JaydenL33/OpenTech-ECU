# Feature matrix

Status date: 2026-11-12. This is an honest capability map, not a marketing comparison.

## How to read this

**OpenTech status**

- **Implemented** — host-side Rust code exists and type-checks. It is not a claim of completed verification, hardware readiness, or Speeduino parity.
- **Simulated** — the behavior exists only inside the virtual ECU. No physical output path.
- **Planned** — specified or designed, but no implementation in this tree.
- **Blocked** — cannot proceed without hardware, evidence, independent review, or validation.

**Evidence basis**

- **Speeduino reference** — the retained C++ tree is present and was inspected for the named modules. This is not an exhaustive audit, and only two narrow numerical parity tests exist; the full compiled oracle harness has not been completed.
- **Haltech / Link benchmark** — published capabilities summarized in [`ECU_MASTER_BRIEF.md`](../ECU_MASTER_BRIEF.md) §2. These are benchmark claims from manufacturer documentation, not independent measurements. Link's 10 November 2026 PCLink announcement is treated as announced, not shipped, and is excluded from status claims.
- **OpenTech evidence** — `cargo fmt`, `cargo clippy`, and `cargo test --workspace` (130 passed, 0 failed) pass on 2026-11-12; the firmware `thumbv7em-none-eabihf` check passes. The CLI was exercised (`--check-determinism` passed with 2184 byte-identical frames; `analyze` and `apply` completed), the UI passed `npm run typecheck` and `npm run build`, the copilot passed `npx tsc --noEmit` / `npm test` and answered an offline `/api/copilot` query, and a local service answered health/status. See the README verification note.

No row in this table is **parity-verified**. “Implemented” does not mean “the same as Speeduino.”

## Matrix

| Family | Speeduino reference | Haltech/Link benchmark | OpenTech status | Evidence/notes |
|---|---|---|---|---|
| Missing-tooth crank/cam sync | Yes: `speeduino/src/decoders/missingTooth.cpp`, `test/test_decoders/missing_tooth/` | Differential crank/home, dual trigger, broad OEM support | Implemented | `ecu-trigger`: configurable 4–120 teeth, gap validation, dropped-tooth rejection, cam-phase latch, sequential sync, timeout. Default 36-1. Edge-case tests are defined. Only the generic missing-tooth family is implemented. |
| OEM trigger-pattern library | Yes: 29 decoder modules under `speeduino/src/decoders/`; decoder tests under `test/test_decoders/` | Broad OEM trigger support | Planned | The Rust `TriggerPattern::DualWheel` is explicitly rejected as not implemented. No OEM patterns beyond generic missing-tooth. |
| Speed-density fuel | Yes: `speeduino/fuel_calcs.cpp`, `test/test_fuel/` | Traditional and modelled VE, multifuel | Implemented | `ecu-fuel`: ideal-gas air mass, VE table, target lambda, dead time, pressure compensation. Not compared to Speeduino's integer `req_fuel` path; no wall-wetting model. |
| Alpha-N / blended load | Yes: load-strategy configuration in the INI/firmware | Supported | Implemented | `LoadStrategy::{SpeedDensity, AlphaN, Blended}`; blend is TPS-weighted and bounded. |
| Injector characterization | Yes: flow, dead time, battery correction | Injector characterization and import workflows | Simulated | `InjectorConfig` includes flow, rated pressure, dead-time curve, minimum pulse, max duty. No flow-bench data, no characterization import wizard. |
| Sequential injection | Yes: sequential/batch scheduling | Multistage injection | Simulated | 720-degree scheduler supports 1–12 cylinders; the simulator models one injector set and applies scheduled events virtually. No physical outputs. |
| Staged / batch injection | Yes: staging modes and secondary injectors | Staged injection | Planned | `InjectionMode::Staged` exists as a configuration enum, but no staging control or secondary-injector scheduling is implemented. |
| Ignition scheduling, dwell, rev limit | Yes: `scheduler_ignition_controller.cpp`, `test/test_schedules/test_ignition_schedule.cpp` | 12 low-current ignition outputs (Haltech); 8 dedicated + 4 configurable (Link) | Simulated | `ecu-ignition` and `ecu-scheduler`: advance/dwell bounds, 45% dwell-duty cap, rev limiter with hysteresis, 720-degree spark events. Ignition table is synthetic in the default profile. |
| Ignition evidence policy | No equivalent policy; maps are calibration data | Base maps are starting points; timing light required | Implemented | `IgnitionPolicy` and `recommend_from_reference`: no lambda parameter anywhere in `ecu-ignition`; missing/unverified reference returns `InsufficientEvidence`; synthetic tables are `SimulationOnly`; physical export blocked. |
| Closed-loop lambda / STFT | Yes: EGO correction with a configured limit | Dual closed-loop oxygen | Simulated | `ecu-lambda::closed_loop`: bounded PI, ±25% default, conditional integration, gate freeze then decay. Simulator uses one virtual bank; `BankTrims` supports two. |
| Offline Mixture Map | Firmware is not a mixture-map engine; `reference/speeduino.ini` exposes `veAnalyzeMap` for TunerStudio | Link Mixture Map; Haltech target-lambda/trim workflows | Implemented | `ecu-lambda::mixture_map`: log replay, exhaust-delay alignment, shared gates, steady-state checks, hard-fault abstention (`LAMBDA_STUCK_SUSPECTED`), per-cell confidence, bounded proposal, selective apply. The local service requires `approve=true`, pauses the engine, and stores an in-memory undo revision; durable persistence/audit is planned. Never applies to hardware. |
| Cell/bank-aware LTFT | EGO correction in `corrections.cpp`; no native per-cell long-term trim identified in the C++ tree | Short- and long-term fuel trim; Link long-term trim | Simulated | `ecu-lambda::long_term`: per-cell trim, two banks, dwell/sample requirements, bounded learning, selective merge into base VE. Integrated in the simulator code path; unit tests pass. |
| Engine protection / DTCs | Yes: `speeduino/engineProtection.cpp`, `test/test_engineprotect/` | Multi-level engine protection | Implemented | 14 DTCs, latching, freeze frames, `Normal/Warning/Derated/Limp/Cut`, fuel/spark/boost/retard reactions. Thresholds are engineering placeholders, not validated limits. |
| Configurable tables and versioning | TunerStudio INI and firmware pages | 32×32×8 4D maps (Haltech); flexible 3D/4D tables (Link) | Implemented | Rank 1–4, 2–32 points per axis, clamped interpolation, axis resampling, validation, SHA-256 sealed calibration revisions, parent links, table diffs. The service exposes edit, resample, and undo; history is in memory. No 4D UI yet. |
| Setup wizard and preflight | TunerStudio project setup; no firmware wizard | Vendor setup wizards; base maps are not final tunes | Implemented | Programmatic starter calibration and a studio engine-setup form; explicit estimates, preflight checks, timing-light/ignition-offset warnings, physical interlock check. |
| Deterministic virtual ECU | Native test harness (Unity/ArduinoFake), not a physics simulator | No direct equivalent in published specs | Simulated | 1 ms physics-lite plant, seeded noise, 36-1 trigger generation, cam pulse, wideband delay/tau/stuck faults, protection integration, six scenarios. CLI determinism check passed with 2184 byte-identical frames. |
| Host protocol | TunerStudio serial protocol and CAN in the C++ tree | USB, Wi-Fi, CAN, Ethernet/GPS (model dependent) | Implemented | New versioned JSON envelope and binary CRC32 frame format. The local Axum service exposes JSON/WebSocket endpoints; there is no auth/TLS. Not TunerStudio compatible. |
| Logging, replay, CSV | SD logging, TunerStudio logs | 512 MB / 1 GB onboard logging; 300/250 channels | Implemented | In-memory ring buffer, session summary, deterministic replay cursor, fixed CSV header, service `/api/log`, `/api/log/summary`, `/api/log.csv`, and `/api/replay/start` / `/api/replay/stop` endpoints. The Logs page has a replay control. Replay start/stop was exercised (658 frames queued); full WebSocket playback was not independently inspected. No durable file store or high-rate logger. |
| Tuning UI | TunerStudio is external; no bundled application | Vendor tuning applications | Implemented | `apps/tuning-studio`: React/TypeScript/Vite dashboard, fuel/target/ignition maps, 3D surfaces, Mixture Map review/apply, setup, diagnostics, logs, calibration audit, Copilot client. `npm run typecheck` and `npm run build` pass. Vite UI with an Electron shell; map axis resampling and the Logs replay control exist. The shell was not launch-tested. Tauri is not used. |
| Desktop shell | None | Native vendor applications | Implemented | `apps/tuning-studio/electron/main.cjs`: Electron shell launches the Rust service and loads the UI. Not launch-tested here; Tauri is not used. |
| LLM tuning copilot | None | None advertised | Implemented | `apps/tuning-copilot`: Mastra agent with five read-only tools, Zod structured output, `ignitionAdvice: "none"`, prompt-level injection defenses, and a deterministic offline mode. LLM mode is opt-in. It cannot apply or access hardware. |
| Copilot audit and evaluation | None | None advertised | Planned | Per-request `toolCalls` only; no durable audit, actor/model/prompt record, or hash chain. `npm test` runs `tsc --noEmit` and passes; there is no behavioral, refusal, or prompt-injection eval suite. |
| Boost control | Yes: `speeduino/src/controllers/boost/`, `test/test_boostcontrol/` | 4D closed-loop boost; boost by gear/speed | Planned | No Rust boost controller. Simulator has a simple open-loop boost-pressure effect only. |
| VVT | Yes: `speeduino/src/controllers/vvt/`, `test/test_vvtcontrol/` | Four-cam control | Planned | No Rust VVT control. |
| Idle control | Yes: `speeduino/src/controllers/idle/`, `test/test_idle/` | Closed-loop idle | Planned | Simulator has a plant idle-throttle model, not an ECU idle-valve controller. |
| Flex fuel / multi-fuel | Yes: flex-fuel sensor and compensation tables | Flex fuel, modelled multifuel | Planned | Telemetry has an `ethanol_pct` field; no blending or compensation logic. |
| Launch, nitrous, rolling cut | Yes: launch/nitrous controllers and tests; rolling-cut paths in protection | Launch, flat-shift, anti-lag, nitrous | Planned | No Rust implementation. |
| Traction, anti-lag, shift strategies | No dedicated traction/anti-lag controller identified in the retained tree; flat-shift/launch paths exist | Traction, anti-lag, flat/paddle shift, torque management | Planned | No Rust implementation. |
| CAN / OEM integration | Yes: `speeduino/comms_CAN.cpp` | 2–3 CAN networks; OEM modes | Planned | The current protocol is a host link, not a CAN stack. No OEM frames or templates. |
| Firmware / embedded target | AVR, Teensy, STM32 builds | Vendor hardware and firmware | Implemented | `ecu/firmware/stm32` is a `no_std` safety skeleton: capture-to-decoder, protection, fail-closed `EngineOutputs`, `PHYSICAL_OUTPUTS_AUTHORISED = false`. The `thumbv7em-none-eabihf` check passes. No HAL, timer capture, drivers, watchdog, flash, CAN, or target image. |
| Hardware I/O, PDM, DBW | Board definitions in `boards/`; hardware is a separate upstream project | 18 injectors, 12 ignition, PDM outputs, dual DBW (Haltech); 24 drives, dual e-throttle (Link) | Blocked | No hardware design in `ecu/`; physical output interlock is true; no harness, drivers, or bench validation. |
| Harness & I/O documentation | No integrated equivalent | Vendor wiring products | Implemented, unreviewed source | Bench connector/test-point manifest generated from the unreviewed KiCad package; `ecu-harness` validates the manifest. No OEM pinout or vehicle adapter. |
| Vehicle-specific profile (EVO VIII) | Generic Speeduino configuration; no vehicle profile | OEM plug-in base maps and harness data | Simulated | `ecu-calibration/src/vehicle.rs`: research-only EVO VIII JDM GSR profile. Pinout, trigger, injectors, ignition topology and harness are unverified; physical configuration request is blocked. See `AUDIT/OPEN_ISSUES.md`. |
| Functional safety / vehicle validation | Not a certified safety ECU | Motorsport products with installation warnings | Blocked | Audit drafts exist (`AUDIT/`), but no FMEA completion, EMC, environmental, thermal, fault-injection, or independent review. Not road legal or engine-safe. |

## Parity summary

- **Parity-verified families:** 0. One narrow fuel-formula parity pair is tested; this is not family-level parity.
- **Early comparison candidates:** generic missing-tooth decoding, speed-density fuel pulse width, ignition event scheduling/dwell, and protection state transitions. A formula-level fuel test exists (`ecu/crates/ecu-fuel/tests/speeduino_parity.rs`); the full compiled oracle comparison is still planned.
- **Deliberate divergences:** ignition evidence policy, calibration document/checksum model, host protocol, and the Mixture Map/approval workflow. These should be documented as OpenTech behavior, not as Speeduino parity gaps.
- **Largest functional gaps:** OEM decoder library, staged injection, idle/boost/VVT, flex fuel, motorsport features, CAN, target firmware/HAL, hardware, durable audit, copilot evals, and validation.

## Benchmark sources

- Speeduino firmware and tests: <https://github.com/speeduino/speeduino> (retained tree in this repository).
- Haltech fuel trim: <https://support.haltech.com/portal/en/kb/articles/short-and-long-term-fuel-trim>
- Haltech target lambda: <https://support.haltech.com/portal/en/kb/articles/target-lambda>
- Haltech engine protection: <https://support.haltech.com/portal/en/kb/articles/engine-protection>
- Link G5 Voodoo Pro V2: <https://linkecu.com/products/g5-voodoo-pro-2>
- Link PCLink: <https://linkecu.com/pages/pc-link>
- Link motorsport features: <https://support.linkecu.com/learn/latest/product-features/motorsport-features>
- Link fuel equations: <https://support.linkecu.com/learn/latest/get-started/getting-running/fuel-equations>

The full research snapshot and caveats are in [`ECU_MASTER_BRIEF.md`](../ECU_MASTER_BRIEF.md). Manufacturer features and availability can change; verify against a released product before relying on a benchmark claim.
