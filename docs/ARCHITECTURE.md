# Architecture

Status: host-first v0.1. The Rust library crates, virtual ECU, headless CLI, local service, browser/Electron tuning UI, local read-only copilot, and a `no_std` STM32 safety skeleton exist. A target firmware image, HAL, timer drivers, and hardware path do not. This document describes what is present and marks planned parts explicitly.

## 1. Design principles

- **Virtual first.** The first integration target is a deterministic host executable, not a PCB or a vehicle. Physical actuation is blocked until independent validation exists.
- **One source of truth for safety decisions.** Fuel learning, trim, and mixture-map analysis share the same `ecu-lambda` gate implementation. Protection is a separate, independent state machine.
- **Trust is separate from value.** A number is not usable because it is a float. `SensorStatus`, sample age, and readiness travel with sensor data, and control code must check them.
- **Evidence is a first-class field.** Calibrations and ignition recommendations carry provenance (`VerifiedReference`, `EngineeringEstimate`, `SyntheticSimulationOnly`, `Unknown`) rather than a prose disclaimer alone.
- **Deterministic and bounded.** No wall-clock scheduling in the control path, no unbounded event lists, no unsafe code, and fixed-capacity core structures where practical.
- **Errors are explicit.** Fuel and ignition calculations return inhibited/error states instead of silently clamping or inventing values.
- **Host and embedded are separate concerns.** Async Rust can serve UI, logging, and communications. It cannot provide an ignition timing guarantee; that requires hardware capture/compare.

## 2. System layers

| Layer | Present? | Responsibility |
|---|---|---|
| Tuning Studio (React/TypeScript + Vite) | Yes, browser + Electron | Live dashboard, table editors, 3D surfaces, Mixture Map review, diagnostics, logs, Copilot client. Electron shell present; Tauri not used (WebKitGTK development libraries unavailable). |
| Tuning Copilot (Mastra/TypeScript) | Yes, local | Read-only tools, structured answers, offline tool mode or optional LLM. |
| `studio-service` (Rust) | Yes, local | HTTP/WebSocket service, live telemetry, calibration revisions, in-memory undo, preflight, analysis. No auth/TLS. |
| `ecu-protocol` | Yes, served locally | Versioned host protocol; JSON envelopes and binary CRC32 frames. |
| `ecu-simulator` | Yes, host | Virtual engine, sensors, trigger, wideband, faults, protection integration, telemetry. |
| Control crates | Yes, host | Types, calibration, fuel, ignition, trigger, scheduling, lambda learning, protection, diagnostics. |
| Firmware (`no_std`, MCU) | Yes, skeleton | Capture-to-decoder path, protection, fail-closed output trait. No HAL, timer capture, drivers, watchdog, flash, CAN, or target image. |
| Hardware | No, blocked | Protected 4-cylinder port-injection board would be the first target; no design exists in `ecu/`. |

## 3. Crate map

```text
ecu-types
  +-- ecu-calibration ---- ecu-lambda ---- ecu-simulator
  +-- ecu-fuel ---------------^               ^
  +-- ecu-ignition -----------^               |
  +-- ecu-trigger ----------------------------+
  +-- ecu-scheduler --------------------------+
  +-- ecu-protection -------------------------+
  +-- ecu-protocol
  +-- ecu-diagnostics
```

| Crate | Depends on | `std` requirement | Role |
|---|---|---|---|
| `ecu-types` | serde | `no_std` intent, unverified | Units, engine/injector config, sensor status, telemetry, provenance. |
| `ecu-calibration` | types, serde_json, sha2 | std | Tables, interpolation, revisions, wizard, preflight, ignition policy. |
| `ecu-fuel` | types, libm | `no_std` intent, unverified | Speed-density/alpha-N fuel mass and injector pulse width. |
| `ecu-ignition` | types | `no_std` intent, unverified | Advance/dwell limits, rev limiting, evidence-gated recommendations. |
| `ecu-trigger` | serde | `no_std` check passes | Missing-tooth decoder and sync state machine. |
| `ecu-scheduler` | serde | `no_std` intent, unverified | 720-degree injection/dwell/spark event computation. |
| `ecu-lambda` | types, calibration | std | Gates, STFT, LTFT, offline Mixture Map, apply/undo. |
| `ecu-protection` | serde | `no_std` check passes | DTCs, latching, freeze frames, bounded reactions. |
| `ecu-protocol` | types, serde_json | std | Version negotiation, JSON events, binary framing, CRC32. |
| `ecu-diagnostics` | types | std | Session log, summary, replay cursor. |
| `ecu-simulator` | all control crates | std | Physics-lite plant and integration harness. |

All crates use `#![forbid(unsafe_code)]`.

## 4. Runtime data flow

### 4.1 Virtual ECU step

The simulator advances in 1 ms ticks. The order in `SimEngine::step_once` is:

1. Update driver/throttle command and the plant model (MAP, coolant, IAT, oil, fuel pressure, battery, torque, RPM).
2. Update sensor readings (noise, stuck/stale faults, wideband readiness).
3. Generate crank teeth and the cam pulse from the true crank angle; feed the decoder.
4. Run the decoder timeout check.
5. Compute ECU outputs (VE cell, target lambda, STFT gate, fuel pulse, ignition advance/dwell, limiter).
6. Evaluate independent protection and apply cut/derate/retard reactions.
7. Apply scheduled injection and spark events for the angle crossed in this tick.
8. Update the wideband transport delay and first-order sensor response.
9. Record fault-window transitions and advance time.

This is a simulation harness, not a hard real-time scheduler. Event angles and durations are computed deterministically; their physical execution would require MCU timer hardware.

The host service wraps this loop: a 20 ms Tokio tick advances the simulator by 20 ms and broadcasts telemetry frames over `/api/ws`; HTTP endpoints control the scenario, inspect/calibrate/resample tables, run the Mixture Map, start/stop log replay, read faults/logs, and serve the built Vite application. Replay re-broadcasts recorded frames over the WebSocket; start/stop was exercised (658 frames queued), but full WebSocket playback was not independently inspected. The service binds `127.0.0.1:8765`, uses permissive CORS, and has no authentication or TLS. Calibration history is an in-memory `Vec<Calibration>`; there is no durable store. The CLI wraps the same simulator without a server.

### 4.2 Fuel path

```text
MAP, IAT, RPM, displacement, VE table, target lambda
        |
        v
ideal-gas air mass per 720-degree cycle
        |
        v
fuel mass = air mass / (stoich AFR * target lambda)
        |
        v
injector flow (rated flow, fuel density, sqrt pressure correction)
        |
        v
pulse width = fuel mass / flow + dead time(volts)
        |
        v
min-pulse clamp, duty check, explicit inhibit reasons
        |
        v
ecu-scheduler injection event -> simulated fuel delivery -> exhaust lambda
```

STFT and LTFT are explicit inputs to the fuel calculation, not hidden state. The caller can log exactly what correction was applied.

### 4.3 Ignition path

```text
ignition table (synthetic simulation-only or licensed reference)
        |
        v
base advance, dwell(volts), rev limiter, knock retard from protection
        |
        v
bounded advance and dwell -> ecu-scheduler -> simulated spark
```

There is intentionally no lambda parameter in `ecu-ignition`. `recommend_from_reference` returns `InsufficientEvidence` rather than a guess when no verified engine-specific reference exists. `simulation_only_recommendation` exists only to keep the virtual engine running.

### 4.4 Fuel-learning path

Three workflows share `ecu-lambda::gate`:

1. **STFT** (`closed_loop`): bounded proportional-integral trim at the wideband rate. Invalid gates freeze the last trim briefly, then decay it toward zero. Conditional integration prevents windup.
2. **LTFT** (`long_term`): per-cell learned trim, bank-aware, kept separate from base VE. Learning requires dwell, samples, a valid gate, and an active STFT. Merging selected cells into base values is a review action and preserves the source table.
3. **Offline Mixture Map** (`mixture_map`): replays a log, aligns lambda to the fuel command using exhaust transport delay, applies the shared gate plus steady-state rate checks, aggregates per-cell statistics, computes confidence, and proposes bounded VE changes. Apply requires a matching base checksum and creates a new sealed revision; the input calibration is never mutated. The result carries the undo revision; persisting the prior snapshot is the service's job.

The Mixture Map correction is a steady-state starting estimate:

```text
raw          = (mean_measured_lambda / mean_target_lambda) * (1 + mean_stft + mean_ltft)
bounded_total = clamp(raw, 1 - max_total, 1 + max_total)
bounded_cell  = clamp(bounded_total, 1 - max_cell, 1 + max_cell)
proposed_ve  = clamp(current_ve * bounded_cell, table minimum, table maximum)
```

The trim term avoids double-correcting base VE when trims are already active. This is documented as an estimate, not a complete real-world control law. Unvisited cells remain untouched.

## 5. Calibration model

- A `Calibration` contains engine, fuel, sensor, I/O, tables, ignition policy, notes, revision, parent, and checksum.
- Tables support rank 1–4, 2–32 breakpoints per axis, explicit units, value bounds, provenance, and a revision.
- Interpolation clamps at the axis ends and never extrapolates. Invalid coordinates are rejected or clamped according to the API; control code must not treat extrapolation as measured data.
- `seal` computes SHA-256 over canonical JSON with the checksum field blank. `verify` recomputes it. Edits create a new revision with a parent link.
- Preflight covers checksum, engine profile, fuel config, required maps, VE/target axis compatibility, injector characterization, sensor calibration flags, wideband assignment, target-lambda bounds, trigger support, ignition evidence, timing-light requirements, ignition offset, fuel pressure, I/O conflicts, and the physical output interlock.
- `IgnitionPolicy` is one of `SyntheticSimulationOnly`, `VerifiedReference`, or `InsufficientEvidence`. Only `VerifiedReference` is hardware-eligible, and it still requires timing-light verification.

## 6. Protocol

The protocol is new to OpenTech-ECU. It is not Speeduino's TunerStudio serial protocol.

### JSON envelope (host/WebSocket)

```json
{ "protocol": 1, "seq": 42, "event": { "type": "telemetry", "seq": 42, "calibration_checksum": "...", "frame": { } } }
```

`ServerEvent` variants are `hello`, `telemetry`, `status`, and `error`. `negotiate` rejects a version mismatch explicitly and advertises capabilities such as `telemetry_json`, `binary_frames_crc32`, `calibration_snapshots`, `mixture_map_offline`, `stft_ltft`, and `replay`.

### Binary frame (embedded/serial)

```text
magic(2) = "OE" | version(2 LE) | flags(2 LE) | sequence(4 LE)
| length(2 LE) | crc32(4 LE) | payload
```

- Header length: 16 bytes.
- Maximum payload: 64 KiB.
- Empty payloads are rejected.
- CRC-32/ISO-HDLC over the payload only.

The protocol never writes raw ECU memory. Calibration changes are explicit, checksum-verified documents with revision history. The local Axum service exposes this protocol over HTTP/WebSocket, binds loopback, and serves the built UI; it has no authentication or TLS.

## 7. Simulation model

The simulator is deliberately physics-lite, not a high-fidelity engine model:

- Four-stroke torque from fuel power, combustion efficiency, friction, pumping, accessories, starter torque, and dyno/load torque.
- MAP from throttle, speed, and optional boost; thermal models for coolant and IAT; oil and fuel pressure models; battery voltage.
- True VE differs from the ECU's VE table by a scenario bias and optional load/RPM gradients, so the Mixture Map has a known error to recover.
- A configurable missing-tooth crank wheel (default 36-1) generates tooth timestamps. A cam pulse once per 720 degrees provides phase. The decoder rejects extra/missing teeth and times out to lost sync.
- Wideband: heater readiness delay, exhaust transport delay, first-order response, noise, stuck-lean/stuck-rich, not-ready, and exhaust-leak/misfire false-lean behavior.
- Fault windows: low fuel pressure, fuel-pressure sensor blind, coolant stale, MAP stuck, trigger dropout, knock, overheat, low injector flow, and others.
- Determinism: SplitMix64 seeded noise and fixed 1 ms steps. The PRNG is tested as repeatable; full-run determinism is a design goal and should be asserted by a regression test rather than assumed.
- Built-in scenarios: `idle_warmup`, `mixture_sweep`, `transient`, `fault_recovery`, `sensor_stuck_lean`, `trigger_loss`.

Logs carry calibration ID/revision/checksum, scenario/seed, schema and software version, frames, and structured events. CSV export uses a fixed header.

## 8. Host vs firmware

Most current code runs on the host. Six crates are written with `no_std` attributes and `alloc`/`core` types, but portability is not yet fully verified:

- `ecu-protection` and `ecu-trigger` pass `cargo check --no-default-features`.
- `ecu-types` and `ecu-scheduler` still fail a no-default-features check (missing `String` import and float methods such as `rem_euclid`/`round` that require `libm` or a std-only path). `ecu-fuel` and `ecu-ignition` fail through their dependency on `ecu-types`.
- `ecu/firmware/stm32` is a `no_std` safety skeleton: capture-to-decoder, independent protection, and a fail-closed `EngineOutputs` trait. `PHYSICAL_OUTPUTS_AUTHORISED` is false; `apply_reaction` calls `all_off` even after `arm`.
- There is no STM32 HAL, board pin mapping, hardware timer capture/compare, injector/ignition driver, watchdog integration, flash/calibration store, CAN stack, bootloader, linker script, or target firmware image. The CI target check is a compile check, not a timing or electrical validation.

A physical implementation would need hardware capture/compare for tooth timing and spark/injection scheduling. Async tasks may handle communications, UI state, and slow peripherals, but they must not be presented as a deterministic ignition guarantee.

## 9. LLM copilot trust boundary

The copilot is implemented as `apps/tuning-copilot` (Mastra/TypeScript). The studio's Copilot page calls it on `127.0.0.1:8787`; the copilot calls the Rust service on `127.0.0.1:8765`.

```text
telemetry/logs (untrusted data)  ->  Mastra agent / offline tool mode
                                          |
                                          v
                         Zod-validated structured answer + read-only tools
                                          |
                                          v
                     Rust service (checksum/revision/bounds; no write tool)
                                          |
                                          v
                              human approval in the Mixture Map UI
```

- Implemented tools are read-only: `read_telemetry`, `read_faults`, `read_calibration_summary`, `run_mixture_map_analysis`, and `read_log_summary`. There is no apply, write, ignition, DTC-clear, or hardware tool.
- The agent's structured answer schema requires `ignitionAdvice: "none"`, limits findings/cells/actions, and labels `requiresHumanApproval` when a proposal exists. The system prompt forbids ignition advice, hardware instructions, safety claims, and bypassing factory safety systems.
- Tool output and log text are explicitly treated as untrusted data in the system prompt; user messages are length-limited and history is truncated. There is no separate prompt-injection classifier, so this remains a prompt-level defense.
- Offline mode executes the same tools deterministically without an LLM call and records each tool use. LLM mode is opt-in via `COPILOT_ENABLE_LLM=1` plus a provider key.
- Per-request tool audit is returned to the UI as `toolCalls`; it is **not persisted** and has no hash chain, actor identity, or approval record. Durable audit is planned.
- The copilot cannot apply a calibration. The human uses the Mixture Map page, which calls the Rust service with `approve=true`; the service pauses the engine and rechecks checksum/revision and cell bounds. Proposal provenance/expiry/actor validation is not implemented.
- Ignition changes, gate/protection disabling, latched-DTC clearing, firmware flashing, and direct actuation are forbidden.

Full design and current gaps: [LLM-COPILOT.md](LLM-COPILOT.md).

## 10. Directory map

```text
/home/jlee/OpenTech-ECU/
  README.md
  ECU_MASTER_BRIEF.md          source specification (not modified)
  LICENSE                      GPL-2.0 text
  docs/
    ARCHITECTURE.md
    PORTING-FROM-SPEEDUINO.md
    FEATURE-MATRIX.md
    LLM-COPILOT.md
    SAFETY-CASE.md
    FORMULAS.md
    CALIBRATION.md
    ENGINEERING-LOG.md
    ROADMAP.md
  ecu/
    Cargo.toml                 workspace, GPL-2.0-or-later
    crates/
      ecu-types/
      ecu-calibration/
      ecu-fuel/
      ecu-ignition/
      ecu-trigger/
      ecu-scheduler/
      ecu-lambda/
      ecu-protection/
      ecu-protocol/
      ecu-diagnostics/
      ecu-simulator/
    apps/
      ecu-sim-cli/             headless simulator/calibration CLI
      studio-service/          local HTTP/WebSocket service
      tuning-studio/           React/TypeScript/Vite UI + Electron shell
        dist/                  Vite build output (generated)
      tuning-copilot/          Mastra/TypeScript read-only copilot
    firmware/stm32/            no_std safety skeleton (no HAL/drivers)
    target/                    build artifacts, not source
  speeduino/                   retained C++ oracle
  test/                        retained Speeduino tests
  reference/                   retained Speeduino INI/bootloaders
  lib/                         retained ArduinoFake test support
  boards/                      retained Speeduino board definitions
  AUDIT/                       independent audit drafts and open issues
  research/                    EVO VIII source research
  NOTICE.md                    provenance and licensing notice
```

Planned but absent: a target firmware image/HAL, `ecu/hardware`, `ecu/tests/fixtures`, durable audit/persistence, and a standalone evidence/validation index. The Electron shell is present but was not launch-tested here.

## 11. Known gaps

- No target firmware image, HAL, drivers, or hardware; the STM32 crate is a skeleton only.
- `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test --workspace` (128 passed, 0 failed) pass. The `thumbv7em-none-eabihf` check for the firmware skeleton passes. See the README verification note.
- `no_std` portability checks pass for `ecu-types`, `ecu-fuel`, `ecu-ignition`, `ecu-scheduler`, `ecu-trigger`, `ecu-protection`, and the firmware skeleton. This is host-verified portability, not target-hardware behaviour.
- No authentication, TLS, multi-user isolation, durable calibration store, or durable audit log; both services are local-only. The copilot's tool audit is per-request and not persisted.
- No proposal provenance/expiry/actor validation beyond checksum/revision/bounds; the API can be called directly with a crafted proposal JSON.
- No CAN stack, OEM CAN templates, flashing, or bootloader.
- No staged injection, idle control, boost control, VVT, flex fuel, launch/traction/anti-lag/nitrous, or transmission control in Rust.
- The EVO VIII vehicle profile is research-only and unverified; see `AUDIT/OPEN_ISSUES.md`.
- No independent functional-safety certification, EMC testing, bench validation, or vehicle validation.
