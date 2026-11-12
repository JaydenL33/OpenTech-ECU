# Roadmap

Status: planning document. No dates and no delivery promises. Phases are ordered by dependency, not by marketing priority. A phase is complete only when its exit criteria are met and the evidence is recorded.

## Ground rules

- Virtual-first until a physical output path has independent review.
- No phase may weaken the physical output interlock, the ignition evidence policy, or the human approval requirement.
- “Done” means code, tests, evidence, and documentation. A passing build is not validation.
- Hardware-blocked work stays blocked even if the software is ready.
- Every parity claim needs a pinned Speeduino commit and reproducible fixtures.

## Phase 0 — Verification and repository hygiene (current)

**Goal:** make the existing host core reproducible and reviewable.

Status: verification is green on the native toolchain. `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test --workspace` (130 passed, 0 failed) pass; the STM32 `thumbv7em-none-eabihf` skeleton check passes; the CLI, local service, browser build, and offline copilot were exercised on 2026-11-12. The CI workflow exists but no CI run attestation is included.

- Commit the Rust workspace and documentation.
- Run `cargo fmt --check`, `cargo clippy`, and a clean `cargo test --workspace` on a normal Rust host; record exact commands and results.
- Fix the remaining `no_std` gaps: missing `String` import in `ecu-types` and float operations that need `libm` in `ecu-scheduler`; `ecu-trigger` now passes its no-default-features check.
- Run and record the existing CI (`.github/workflows/opentech-ci.yml`) for Rust, UI, and copilot typecheck.
- Add the provenance ledger and pin the Speeduino oracle commit.
- Maintain the formulas and safety case; add the remaining evidence/validation index and unresolved-validation register.

**Exit criteria:** a clean checkout builds and tests on a documented toolchain; CI is green; the current test results and known failures are recorded; no status claim relies on an unrun suite.

## Phase 1 — End-to-end virtual ECU and tuning studio

**Goal:** a real UI connected to the virtual ECU over the versioned protocol.

Status: largely implemented. The Axum service, live telemetry WebSocket, dashboard, maps, 3D surfaces, Mixture Map review, setup, diagnostics, logs, replay endpoints, calibration history, and an Electron desktop shell exist. Remaining: hardening and replay verification.

- [x] Implement the `studio-service` HTTP/WebSocket server: telemetry stream, status, errors, calibration snapshots.
- [x] Add a desktop shell around the Vite UI (Electron; Tauri is not used).
- [x] Live dashboard: RPM, MAP, TPS, lambda, target lambda, ignition, VE cell, STFT/LTFT, injector PW/duty, coolant/IAT, protection state, DTCs.
- [x] Editable maps with axes, cell selection, interpolation, save, and 3D surface view.
- [x] Log replay endpoint and UI control exist; start/stop was exercised (658 frames queued); [ ] full WebSocket playback was not independently inspected. CSV export exists via the service.
- [x] Table diff and calibration revision browser (in-memory).
- [x] No mock controls; the UI uses the Rust service.

**Exit criteria:** a new user can launch the studio, connect to the simulator, run `mixture_sweep`, edit and save a calibration revision, disconnect, and replay the log without a hidden mock path.

## Phase 2 — Auditable fuel calibration workflow

**Goal:** make the Mixture Map a first-class, reviewable UI workflow.

Status: the main manual workflow exists. The UI runs a deterministic sweep or analyzes the current log, shows gate counts, coverage, confidence and proposed cells, then applies selected cells with `approve=true` and supports in-memory undo. Remaining: durable proposals/audit, proposal expiry/provenance, a permanent revision store, and bank/dual-wideband UX.

- [x] Run/select a log; show gate coverage and exclusion reasons per cell.
- [x] Show measured cells, unvisited cells, confidence, sample count, dwell, and trims.
- [x] Preview the VE diff; select/deselect cells; apply per-cell/total bounds.
- [ ] Apply through a service that creates a durable sealed revision and a persistent undo snapshot.
- [x] Implement in-memory undo; [ ] compare arbitrary revisions and persist history.
- [ ] Add bank-aware and dual-wideband workflows only when sensor allocation is real; do not fabricate per-cylinder data.
- [ ] Add full tests for stale checksums, tampered calibration, sparse coverage, and oscillations; the hard-fault abstention exists but the suite has not been executed here.
- [x] Keep STFT, LTFT, and offline Mixture Map separate in the UI and the data model.

**Exit criteria:** a reviewer can reconstruct every proposed cell from the log, gate decisions, trims, and base calibration; apply and undo are reproducible; no proposal can be applied to a mismatched revision.

## Phase 3 — Bounded LLM tuning copilot

**Goal:** an assistant that is useful without having authority.

Status: implemented as a local read-only service with deterministic offline mode. The Mastra agent, five read-only tools, Zod structured output, and the studio Copilot page exist. Remaining: durable audit, proposal provenance/expiry, an automated test/eval suite, and a programmatic cross-check between reported cells and the Rust proposal.

- [x] Add a Copilot client page in the Tuning Studio.
- [x] Implement `apps/tuning-copilot` with Mastra and the read-only tool catalog in [LLM-COPILOT.md](LLM-COPILOT.md).
- [x] Add strict output schemas, a read-only tool allowlist, and the deterministic offline mode.
- [ ] Add the Rust durable audit writer, proposal provenance/expiry, and actor identity.
- [ ] Add refusal evals, prompt-injection evals, stale-proposal tests, and audit reconstruction tests.
- [x] `npm test` runs `tsc --noEmit` and passes; [ ] add behavioral, refusal, and prompt-injection evals.
- [ ] Do not add an apply/write/hardware tool.

**Exit criteria:** mock mode runs offline without an API key; the copilot cannot call a write tool; every tool call and approval is auditable; adversarial log text cannot change tool behavior.

## Phase 4 — Speeduino parity expansion

**Goal:** move from “implemented” to reproducible behavioral comparison.

- Build the differential fixture harness and manifest described in [PORTING-FROM-SPEEDUINO.md](PORTING-FROM-SPEEDUINO.md).
- Parity-verify the generic missing-tooth decoder, fuel pulse width, ignition event scheduling/dwell, and protection transitions.
- Add OEM decoder patterns one family at a time, each with vectors and edge cases.
- Add staged injection, idle control, boost control, VVT, flex fuel, launch/traction/anti-lag/nitrous, and shift strategies as separate bounded projects.
- Keep the ignition evidence policy and new protocol as documented divergences, not as parity failures.

**Exit criteria:** each claimed family has a pinned oracle commit, fixtures, tolerances, edge cases, and a written divergence list. “Parity” is not used as a synonym for “feature exists.”

## Phase 5 — Embedded feasibility

**Goal:** prove the portable core can run on a selected MCU without pretending it is real-time safe.

Status: a `no_std` STM32 safety skeleton exists (`ecu/firmware/stm32`) with a capture-to-decoder path, protection, and fail-closed outputs. It has no HAL, board mapping, timer capture, drivers, watchdog, flash, CAN, or target image.

- Select an MCU family with a first-class Rust HAL and verified timer/capture support.
- Make the portable crates actually pass `no_std` checks; document heap/allocation strategy.
- Add hardware capture/compare for crank teeth and scheduled events.
- Add host-to-target compile checks and a target linker script.
- Define watchdog, brownout, calibration storage, CRC, and rollback behavior.
- Do not claim deterministic ignition timing from async Rust.

**Exit criteria:** a documented development board builds and runs the portable core; timer capture and scheduled output behavior are measured on the bench; limitations are recorded.

## Phase 6 — Bench hardware

**Goal:** a protected 4-cylinder port-injection development board with an external CAN wideband.

- Complete the KiCad schematic/PCB, BOM, pinout, and ERC/DRC reports.
- Include power protection, transient/load-dump considerations, debug access, and documented thermal budgets.
- Validate drivers, sensor inputs, trigger capture, and communication on the bench.
- Keep the physical output interlock until independent review approves a bounded test plan.
- Do not add electronic throttle, direct injection, high-current PDM, or other high-energy actuation without a separate safety review.

**Exit criteria:** bench measurements, fault-injection results, and an independent review are recorded. ERC/DRC alone is not functional-safety validation.

## Phase 7 — Independent validation

**Goal:** determine whether any physical deployment is defensible.

- Functional-safety analysis: hazards, fault trees, independence, diagnostics, safe states.
- EMC, environmental, thermal, vibration, and transient testing.
- On-engine validation with a supervised test plan, timing-light verification, and knock instrumentation.
- Calibration validation by a qualified tuner; base maps remain estimates until proven.
- Security review: firmware update authenticity, rollback, debug access, CAN exposure.
- Legal/regulatory review for the intended market.

**Exit criteria:** independent reviewers accept the safety case and the evidence. Until then, the project remains a development platform, not a vehicle-ready ECU.

## Cross-cutting work

| Workstream | Requirement |
|---|---|
| Safety documentation | Safety case, hazard log, formulas, evidence index, unresolved validation, change control. |
| Licensing | Preserve GPL notices; provenance ledger for derived code; CERN-OHL-S-2.0 for derived hardware. |
| Testing | Unit, property, simulation, differential, fault-injection, and regression tests; record exact commands/results. |
| Data | Versioned log/fixture schemas; checksums; deterministic seeds; no unlicensed calibration data. |
| UX | Explain gates, confidence, uncertainty, and unvisited cells; never hide an abstention. |
| Security | Least privilege, local-only defaults, no secrets in prompts, signed updates before a deployable firmware update path exists. |

## Current next highest-value task

Keep the verified fmt/clippy/test baseline green, close the remaining `no_std` gaps, and record CI results. Then make the calibration/audit path durable (persistent revisions, proposal provenance/expiry, actor identity, append-only audit) and add behavioral copilot evals with a cross-check against the Rust proposal. The end-to-end virtual ECU + studio + copilot slice now exists; hardening it is more valuable than adding feature breadth.
