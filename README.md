# OpenTech-ECU

**A Rust-first, open-source engine management platform: a deterministic virtual ECU, auditable fuel calibration, evidence-gated ignition, and a human-approved tuning copilot.**

> **Status: v0.1, virtual-first.** The host Rust core, headless CLI, local service, browser/Electron tuning studio, local read-only copilot, and a `no_std` STM32 safety skeleton are present. There is no target firmware image, no physical output path, and no vehicle actuation. Nothing in this repository is an engine-safe calibration.

OpenTech-ECU is a new, original aftermarket ECU platform. It is being built as a Rust port/reimplementation of Speeduino behavior, with a feature target informed by Haltech Nexus R5 and Link G5 Voodoo Pro-class capabilities, and a tuning workflow designed around explainable and reviewable changes rather than hidden auto-tuning.

**Not a Speeduino fork.** OpenTech-ECU is a separate, Rust-first project based on Speeduino. The original Speeduino C++ tree is retained in `speeduino/`, `test/`, `reference/`, `lib/`, and `boards/` as a behavioral oracle for parity testing. It is not the product firmware, and it has not been replaced or deleted. Upstream Speeduino is at <https://github.com/speeduino/speeduino>; its documentation describes the reference tree, not this Rust platform.

## Why this is different

1. **Virtual ECU first, deterministic by construction.** A 1 ms host simulation runs a physics-lite four-stroke engine, a configurable missing-tooth crank decoder with cam phase, a 720-degree injection/ignition scheduler, a wideband model with transport delay, sensor lag, noise and fault injection, and an independent protection state machine. The loop uses fixed 1 ms steps and seeded noise so runs are reproducible; determinism is a design requirement, not a hard real-time guarantee.

2. **Fuel calibration you can audit.** Closed-loop short-term trim, cell/bank-aware long-term trim, and an offline Mixture Map share one learning-gate implementation. Proposals are bounded, show sample counts and confidence, distinguish measured from unvisited cells, account for active trims, and expose a diff for preview. Apply requires explicit human approval in the service, creates a new sealed calibration revision, and stores an in-memory undo history; durable audit/persistence is planned.

3. **Ignition refuses to guess.** Lambda is never an ignition input. A synthetic ignition table exists only to make the simulator run; physical ignition export is blocked. A recommendation requires a licensed, engine-specific verified reference, and still requires timing-light verification and knock review. The ignition API has no lambda parameter by design.

4. **An LLM copilot with a hard trust boundary.** The Mastra/TypeScript copilot can read telemetry and logs, run Mixture Map analysis, and explain gates and DTCs. It cannot write a calibration, change ignition, disable a protection, or touch hardware. Its output is schema-validated untrusted input; a human approves every apply. It also runs in a deterministic offline tool mode with no API key.

5. **A real tuning UI, not a mock.** The repository includes a React/TypeScript/Vite tuning studio with live telemetry, editable maps and 3D surfaces, Mixture Map review, diagnostics, logs, and a Copilot client page. An Electron desktop shell launches the local Rust service and presents the UI; it is not Tauri, and it was not launch-tested in this headless environment.

6. **Evidence over claims.** The project distinguishes implemented, simulated, planned, and hardware-blocked work. See the [feature matrix](docs/FEATURE-MATRIX.md), [porting strategy](docs/PORTING-FROM-SPEEDUINO.md), [safety case](docs/SAFETY-CASE.md), and the source specification in [ECU_MASTER_BRIEF.md](ECU_MASTER_BRIEF.md).

## Status at a glance

Status terms: **Implemented** means host-side code exists and type-checks; it is not a claim of completed verification, parity, or hardware readiness. **Simulated** means the behavior exists only inside the virtual ECU. **Planned** means design only. **Blocked** means prohibited or impossible until hardware and independent validation exist.

| Area | Status | Notes |
|---|---|---|
| Rust workspace | Implemented, host-only | `cargo fmt`, `cargo clippy`, and `cargo test --workspace` pass (128 tests); see the verification note. |
| Deterministic virtual ECU | Simulated | 1 ms physics-lite engine, trigger generation, scheduler, wideband lag/delay/faults, protection, telemetry; six built-in scenarios. |
| Calibration and tables | Implemented, host-only | 1D–4D tables, clamped interpolation, axis resampling, validation, SHA-256 sealed revisions, table diffs, starter wizard, preflight, in-memory revision history. |
| Lambda fuel calibration | Implemented, host-only | Shared gates, bounded STFT, cell/bank-aware LTFT, offline Mixture Map with hard-fault abstention, confidence, proposal preview, selective apply and undo. |
| Ignition policy | Implemented, host-only | Evidence-gated recommendations; synthetic simulation-only tables; physical export blocked. |
| Engine protection | Implemented, host-only | 14 DTCs, latching, freeze frames, warning/derate/limp/cut reactions. |
| Protocol | Implemented, served locally | JSON/WebSocket envelopes and a length-delimited binary frame with CRC32; the local service exposes the JSON link. |
| `ecu-sim-cli` | Implemented, verified | Scenario runner, deterministic replay check, Mixture Map analysis and approval-gated apply. See the quick start. |
| `studio-service` | Implemented, local-only | Axum HTTP/WebSocket service for live telemetry, calibration edits/undo, axis resampling, log replay, preflight, Mixture Map, faults and logs. In-memory state; no auth/TLS. |
| Tuning Studio (browser + Electron) | Implemented, verified UI | React/TypeScript/Vite app; `npm run typecheck` and `npm run build` pass. Map axis resampling and the Logs replay control exist. `electron/main.cjs` is a desktop shell; it was not launch-tested (no display). Tauri is not used. |
| Mastra tuning copilot | Implemented, local-only | `apps/tuning-copilot`: read-only tools, structured output, `ignitionAdvice: none`, and a deterministic offline mode. LLM mode is opt-in with an API key. Per-request tool audit only; no durable audit store. |
| Firmware / embedded target | Implemented, skeleton | `ecu/firmware/stm32`: `no_std` safety skeleton with capture-to-decoder path, protection, and fail-closed `EngineOutputs`. The `thumbv7em-none-eabihf` check passes. No HAL, timer capture, drivers, watchdog, flash, or target image. |
| Durable audit / persistence | Planned | Calibration history is in memory; there is no durable audit log or signed provenance store. |
| Physical injector/coil actuation | Blocked | `PHYSICAL_OUTPUT_INTERLOCK_ENGAGED = true`; no code path energizes hardware. |
| Vehicle use | Blocked | No bench, engine, EMC, or functional-safety validation. |

### Verification note

Verified on 2026-11-12 with the native Rust toolchain:

- `cargo fmt --all -- --check` — pass.
- `cargo clippy --workspace --all-targets -- -D warnings` — pass.
- `cargo test --workspace` — 128 passed, 0 failed.
- `cargo check -p opentech-firmware-stm32 --target thumbv7em-none-eabihf --no-default-features` — pass. This is a compile check for the `no_std` skeleton, not hardware validation.
- CLI `mixture_sweep` — determinism PASS, 2184 frames byte-identical; `analyze` produced 3–4 bounded cells; `apply --approve` produced sealed revision 2.
- Tuning Studio — `npm run build` passes.
- Copilot — `npx tsc --noEmit` passes; `npm test` now runs `tsc --noEmit` and passes. The offline `/api/copilot` path was exercised against the Rust service and returned tool-audit entries. LLM mode was not called.

The Electron shell was not launch-tested because this environment has no display. Axis resampling and replay endpoints/UI exist; the replay start/stop endpoints were exercised (658 frames queued), but full WebSocket playback was not independently inspected. Axis resampling was not exercised in this verification pass. Re-run the commands on a clean host before relying on these results.

## Reference vehicle and release gates

The target reference is a JDM Mitsubishi Lancer Evolution VIII GSR (CT9A, 4G63T, six-speed manual), but the specific ECU variant, connector count, pinout, trigger pattern, injector data, ignition topology, and harness are **UNVERIFIED**. `ecu-calibration/src/vehicle.rs` contains a simulator-only template for this profile; the physical configuration request is hard-blocked. It is not a plug-and-play definition and must not be used to build a harness or connect hardware.

**Current status: Gate 0 only. No vehicle connection, no harness construction, no ECU flashing, and no engine start are permitted.** Gate 0 is a software-readiness stage, not a safety certification.

Release gates follow [`ECU_MASTER_BRIEF.md`](ECU_MASTER_BRIEF.md) exactly:

| Gate | Scope | Current status |
|---|---|---|
| **0** | Software simulation | Current stage; host simulation, CLI, UI, copilot, and `no_std` skeleton checks |
| **1** | Unpowered hardware | Not started; no ECU, harness, or bench hardware |
| **2** | Powered bench | Not started; no drivers, loads, or fault injection |
| **3** | Vehicle electrical compatibility | Blocked; ECU variant and pinout unverified |
| **4** | Controlled engine testing | Blocked; requires Gates 1–3 and independent review |
| **5** | Vehicle operation | Blocked; requires Gates 1–4, insurance/legal review, and validation |

Gate definitions are maintained in [release gates](AUDIT/RELEASE_GATES.md). Where an audit table uses older labels, the master-brief numbering above is authoritative. See also [vehicle compatibility](AUDIT/VEHICLE_COMPATIBILITY.md), [pinout verification](AUDIT/ECU_PINOUT_VERIFICATION.md), and [open issues](AUDIT/OPEN_ISSUES.md).

## Architecture

```
                        HOST / VIRTUAL-FIRST (today)

  +----------------------------+      +-------------------------------+
  | Tuning Studio (browser/Electron) |  | Tuning Copilot (offline/LLM)  |
  | React + TypeScript + Vite  |      | Mastra / TypeScript           |
  +-------------+--------------+      +---------------+---------------+
                |                                     |
                |  review, approve, undo              |  read-only tools,
                |                                     |  proposals, explanations
                v                                     v
  +-------------------------------------------------------------------+
  | studio-service (local Axum HTTP + WebSocket)                      |
  | telemetry, revisions, preflight, Mixture Map, faults, logs, UI    |
  +-------------------------------+-----------------------------------+
                                  |  ecu-protocol
                                  |  JSON envelopes + binary CRC32
                                  v
  +-------------------------------------------------------------------+
  | ecu-simulator: 1 ms deterministic loop                            |
  | trigger -> scheduler -> fuel/ignition -> protection -> telemetry  |
  +------+-----------+-----------+-----------+-----------+------------+
         |           |           |           |           |
   ecu-trigger  ecu-fuel  ecu-ignition  ecu-lambda  ecu-protection
         +-----------+-----------+-----------+-----------+
                         ecu-types / ecu-calibration
                         ecu-protocol / ecu-diagnostics

  ---------------------------------------------------------------------
  firmware/stm32 (no_std skeleton; no HAL/drivers) -> physical I/O blocked
```

The virtual ECU is the integration point: it owns the engine plant, sensors, trigger wheel, wideband, and simulated actuators, while the control crates compute fuel, spark, learning decisions, and protection reactions. The host protocol is a new, versioned protocol; it is not Speeduino's TunerStudio serial protocol and does not write raw ECU memory.

Detailed layering, data flow, host/firmware boundaries, and the copilot trust boundary are in [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md).

## Safety non-negotiables

1. **No physical actuation in v0.1.** A `no_std` STM32 safety skeleton exists, but it has no HAL, timer capture/compare, injector or ignition drivers, flashing path, or vehicle interface. `PHYSICAL_OUTPUTS_AUTHORISED` is false and outputs are forced off; preflight reports the physical export blocked.
2. **Lambda is never ignition evidence.** A wideband sensor measures combustion oxygen/fueling consequences. It cannot establish MBT, knock margin, or safe spark advance. No automatic spark advance is permitted, and physical ignition export is blocked.
3. **Fuel corrections are bounded and reviewable.** Learning gates cover engine state, sensor validity, fuel/oil pressure, injector duty, misfire, exhaust leak, overrun, knock, overboost, and DTC state; the offline Mixture Map additionally enforces steady-state rate checks by default. Proposals are previewed, selectively applied, and sealed into a new revision; the service stores an in-memory undo revision, with durable persistence still planned.
4. **The LLM is not in the control loop.** Copilot output is untrusted, schema-validated input. The copilot cannot apply changes, change ignition, disable gates or protections, clear latched faults, or access hardware.
5. **Calibrations are estimates until proven otherwise.** Starter VE, target-lambda, and dwell values are explicit engineering estimates. The synthetic ignition surface is simulation-only. A licensed reference and timing-light verification are required before any physical ignition use.
6. **No road, track, or engine use.** This is a development platform, not a certified or validated ECU. Independent engineering review, bench testing, and functional-safety work are mandatory before any physical deployment.

## Quick start

Requirements for a normal build host: Rust 1.80 or newer and a C linker (for example `build-essential` on Debian/Ubuntu). The repository workspace is under `ecu/`.

```bash
# From the repository root: headless virtual ECU.
cd ecu
cargo run -p ecu-sim-cli -- --help
cargo run -p ecu-sim-cli -- scenarios
cargo run -p ecu-sim-cli -- run --scenario mixture_sweep --out runs/sweep --check-determinism

# Analyze the log with the offline Mixture Map; emits a proposal, never applies.
cargo run -p ecu-sim-cli -- analyze \
  --log runs/sweep/log.json --cal runs/sweep/calibration.json \
  --out runs/sweep/report.json --min-samples 12

# Apply a reviewed proposal to a new sealed revision (virtual calibration only).
cargo run -p ecu-sim-cli -- apply \
  --cal runs/sweep/calibration.json --report runs/sweep/report.json \
  --out runs/sweep/calibration-applied.json --approve

# Local tuning service. Defaults to 127.0.0.1:8765 and serves the built UI.
cargo run -p studio-service -- --port 8765
```

The browser tuning studio lives in `ecu/apps/tuning-studio`:

```bash
cd ecu/apps/tuning-studio
npm install
npm run dev        # Vite on 127.0.0.1:5173, /api proxied to 127.0.0.1:8765
npm run build      # production build in dist/, served by studio-service
```

Tauri is not used because the WebKitGTK development libraries required for a Tauri build are unavailable in this environment. An Electron desktop shell (`ecu/apps/tuning-studio/electron/main.cjs`) starts the local service and loads the UI; it needs a desktop session and a built service, so it was not launch-tested here. The Copilot page talks to the Mastra service on port 8787. Start it in a third terminal:

```bash
cd ecu/apps/tuning-copilot
npm install

# Deterministic offline mode: no LLM, no API key. Requires the Rust service on 8765.
COPILOT_OFFLINE=1 npm run start

# Optional LLM mode: opt in explicitly and provide a provider key.
COPILOT_ENABLE_LLM=1 DEEPSEEK_API_KEY=... npm run start
```

The copilot has only read-only tools. It cannot apply a calibration; the Mixture Map page is the approval path.

To reproduce the verified host baseline: `cd ecu && cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace`.

## What is implemented in the host core

| Crate | Responsibility |
|---|---|
| `ecu-types` | Unit-tagged quantities, engine/injector configuration, sensor quality and wideband state, telemetry frames, provenance, CSV log rows. |
| `ecu-calibration` | Rank 1–4 tables, clamped interpolation, axis validation, immutable calibration revisions, SHA-256 sealing, table diffs, starter wizard, preflight, ignition policy. |
| `ecu-fuel` | Speed-density air mass, target-lambda fuel mass, injector pulse width, dead time, pressure compensation, duty and minimum-pulse diagnostics, alpha-N/blended load. |
| `ecu-ignition` | Advance/dwell bounds, dwell-duty limiting, rev limiter with hysteresis, evidence-gated recommendations, simulation-only surfaces. |
| `ecu-trigger` | Configurable missing-tooth crank decoder, gap and dropped-tooth checks, cam-phase latching, sequential sync state machine, timeout to lost sync. |
| `ecu-scheduler` | Deterministic 720-degree injection, dwell, and spark events for 1–12 cylinders and arbitrary firing orders. |
| `ecu-lambda` | Shared learning gates, bounded STFT, cell/bank-aware LTFT, offline Mixture Map with exhaust-delay alignment, confidence, proposals, selective apply, and undo metadata. |
| `ecu-protection` | Independent 14-DTC protection state machine with latching, freeze frames, and bounded fuel/spark/boost/retard reactions. |
| `ecu-protocol` | Protocol version negotiation, JSON server events, binary framing with magic/version/flags/sequence/length/CRC32. |
| `ecu-simulator` | Physics-lite engine plant, trigger generation, sensor and wideband models, fault windows, closed-loop/LTFT integration, protection, deterministic telemetry logs. |
| `ecu-diagnostics` | Session ring buffer, event log, session summaries, deterministic replay cursor. |

### Applications

| Path | Responsibility |
|---|---|
| `apps/ecu-sim-cli` | Headless scenario runner, deterministic replay check, offline Mixture Map analysis, approval-gated apply, calibration verification. |
| `apps/studio-service` | Local Axum HTTP/WebSocket service: live telemetry, calibration revisions and undo, axis resampling (`/api/calibration/resample`), log replay (`/api/replay/start` and `/api/replay/stop`), preflight, Mixture Map analysis/apply, faults, CSV/log export, static UI hosting. |
| `apps/tuning-studio` | React/TypeScript/Vite UI: dashboard, fuel/target/ignition maps with axis resampling, 3D surface view, Mixture Map review/apply, setup, diagnostics, logs with replay control, calibration audit, Copilot client. `electron/main.cjs` is the desktop shell; it has no ECU logic or hardware access. |
| `apps/tuning-copilot` | Mastra/TypeScript copilot: read-only telemetry/fault/calibration/log/Mixture Map tools, structured answer schema, offline tool mode, optional LLM mode, per-request tool audit. |

## Repository layout

```text
OpenTech-ECU/
  README.md                 this document
  ECU_MASTER_BRIEF.md       source specification (not modified by the docs pass)
  docs/                     architecture, porting, feature matrix, copilot, safety, formulas, roadmap
  ecu/                      Rust Cargo workspace
    crates/                 eleven host library crates
    apps/ecu-sim-cli/       headless simulator and calibration CLI
    apps/studio-service/    local HTTP/WebSocket tuning service
    apps/tuning-studio/     React/TypeScript/Vite UI with Electron shell
    apps/tuning-copilot/    Mastra/TypeScript read-only copilot service
    firmware/stm32/         no_std safety skeleton (no HAL/drivers)
  AUDIT/                    independent audit drafts and open issues
  research/                 EVO VIII source research
  NOTICE.md                 provenance and licensing notice
  speeduino/                retained Speeduino C++ reference tree (GPL-2.0)
  test/                     retained Speeduino native/unit tests
  reference/                retained Speeduino INI, bootloaders, and reference data
  lib/                      retained Speeduino test dependencies (ArduinoFake)
  boards/                   retained Speeduino board definitions
```

## Documentation

- [Architecture](docs/ARCHITECTURE.md) — layers, data flow, protocol, simulation, host/firmware split, trust boundary.
- [Porting from Speeduino](docs/PORTING-FROM-SPEEDUINO.md) — what “based on Speeduino” means, oracle strategy, GPL obligations, comparison plan.
- [Feature matrix](docs/FEATURE-MATRIX.md) — Speeduino parity and Haltech/Link feature families, with honest status.
- [LLM tuning copilot](docs/LLM-COPILOT.md) — Mastra design, tools, schemas, approval flow, prompt-injection defenses, audit.
- [Safety case](docs/SAFETY-CASE.md) — v0.1 virtual-only argument, hazards, evidence status, unresolved validation, interlock release criteria.
- [Calibration workflow](docs/CALIBRATION.md) — profile creation, sealed revisions, table editing/resampling, Mixture Map apply.
- [Formulas](docs/FORMULAS.md) — implemented fuel, trim, mixture-map, ignition, dwell, and scheduler equations, with assumptions.
- [Speeduino parity](docs/PARITY.md) — retained C++ oracle, current formula-level evidence, and the planned compiled comparison harness.
- [Engineering log](docs/ENGINEERING-LOG.md) — factual milestone record, decisions, known gaps and next work.
- [Roadmap](docs/ROADMAP.md) — phased plan from the current host core to parity, embedded work, and validation.
- [ECU_MASTER_BRIEF.md](ECU_MASTER_BRIEF.md) — the source research and engineering mandate; preserved as the specification.

### Audit and research (outside `docs/`)

Audit documents are snapshot drafts. Where an audit file reports an older test result or gate status, the verification note above and a fresh command run take precedence.

- [Engine control safety argument](AUDIT/ENGINE_CONTROL_SAFETY.md) — independent audit draft; not a certification.
- [Failure mode analysis](AUDIT/FAILURE_MODE_ANALYSIS.md) — FMEA-style hazard review.
- [Electrical design review](AUDIT/ELECTRICAL_DESIGN_REVIEW.md) — power, grounding, and I/O review.
- [ECU pinout verification](AUDIT/ECU_PINOUT_VERIFICATION.md) — connector and pin evidence status.
- [Release gates](AUDIT/RELEASE_GATES.md) — Gate 0–5 definitions and current status.
- [Vehicle compatibility](AUDIT/VEHICLE_COMPATIBILITY.md) — vehicle-specific unknowns and compatibility matrix.
- [Open issues](AUDIT/OPEN_ISSUES.md) — blocking EVO VIII research items.
- [Firmware timing analysis](AUDIT/FIRMWARE_TIMING_ANALYSIS.md) — timing and scheduling limits.
- [EVO VIII sources](research/evo8/sources.md) — research provenance.

## Licensing and provenance

- The root [`LICENSE`](LICENSE) is the GNU General Public License, version 2, and is preserved.
- Original Rust code in `ecu/` is declared **GPL-2.0-or-later** in the workspace `Cargo.toml`, package metadata and `NOTICE.md`, matching the Speeduino reference licence.
- The retained Speeduino C++ tree is **GPL-2.0-or-later** per its file headers and remains under its original terms. It is not relicensed by this project.
- A Rust rewrite does not remove GPL obligations. Any future module derived from Speeduino must retain the applicable notices and carry a recorded provenance entry. No code is copied from Speeduino or another project without provenance.
- Speeduino-derived hardware, if used, is subject to CERN-OHL-S-2.0. No new hardware design is included in this snapshot.
- “Speeduino” and associated marks belong to their respective owners. OpenTech-ECU does not claim endorsement by the Speeduino project.

## Contributing

The most useful contributions right now are verification, not features: run the test suite on a normal Rust host, add golden-vector comparisons against the retained C++ oracle, and review the safety and ignition evidence rules. Do not add a physical output path, an ignition auto-advance feature, or a copilot write path.
