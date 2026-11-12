# Safety case (v0.1, virtual-only)

Status: **development safety argument, not a certified safety case.** This document explains why the current v0.1 system is safe only as a host simulation, and what would have to be proven before any physical output. It is not an ISO 26262, IEC 61508, or automotive functional-safety case. No independent review has been performed.

## 1. Scope and system boundary

In scope:

- the Rust host library crates under `ecu/crates/`;
- the virtual ECU in `ecu/crates/ecu-simulator`;
- the calibration/preflight model in `ecu-calibration`;
- the evidence-gated ignition policy in `ecu-ignition` and `ecu-calibration`;
- the lambda learning and Mixture Map modules in `ecu-lambda`;
- the independent protection state machine in `ecu-protection`;
- the headless CLI, local studio service, browser/Electron tuning studio, and local Mastra copilot service;
- the `no_std` STM32 safety skeleton in `ecu/firmware/stm32`.

Out of scope / absent:

- a target firmware image, STM32 HAL, board pin mapping, timer capture/compare, drivers, watchdog integration, flash/calibration storage, CAN, and bootloader;
- injector, coil, throttle, boost, VVT, nitrous, or PDM actuation;
- hardware design, harness, sensors, power supply, and EMC;
- durable audit/persistence, authentication, and TLS;
- on-engine, dyno, bench, or vehicle validation.

The v0.1 safety claim is therefore narrow: **the code can compute and simulate, but it has no path to energize a physical actuator.**

## 2. Top-level claims

| ID | Claim | Basis |
|---|---|---|
| C1 | v0.1 has no physical output path. | No target firmware image or driver code; `PHYSICAL_OUTPUT_INTERLOCK_ENGAGED = true`; firmware `PHYSICAL_OUTPUTS_AUTHORISED = false` and `apply_reaction` forces `all_off`; preflight reports physical export blocked; service `/api/health` returns `physical_outputs: false`. |
| C2 | Lambda is never used as ignition evidence. | `ecu-ignition` has no lambda input; `recommend_from_reference` requires a verified engine-specific reference; synthetic surfaces are simulation-only; physical export is blocked. |
| C3 | Fuel learning is gated, bounded, and reviewable. | Shared `ecu-lambda::gate`; STFT/LTFT bounds; Mixture Map caps and hard-fault abstention; apply requires a matching checksum/revision and `approve=true`; the service supports selective cell application. |
| C4 | Calibration changes are sealed. | SHA-256 canonical checksum; revision/parent; preflight; apply returns a new revision and the service stores an in-memory undo revision. Durable traceability/audit is not implemented. |
| C5 | Protection is independent and latched where required. | Separate `ecu-protection` state machine; DTCs, freeze frames, latching, explicit clear. |
| C6 | The LLM cannot actuate or write. | The Mastra copilot exposes five read-only tools; no apply/write/hardware tool; Zod structured output with `ignitionAdvice: "none"`; human approval through the Rust service. |
| C7 | Users are told the maps are estimates. | Provenance fields, preflight warnings, synthetic ignition marking, README and feature-matrix status labels. |

These claims are about the current code and design. They are not claims of engine safety, reliability, or regulatory compliance.

## 3. Hazard analysis

| Hazard | Cause | Consequence | Mitigation in v0.1 | Residual risk |
|---|---|---|---|---|
| H1: Physical actuation despite virtual-only status | A future output path is added or the interlock is changed | Injector/coil/throttle actuation without validation | No HAL/driver/target image; interlock constants in host and firmware; fail-closed `apply_reaction`; preflight fails physical export; docs prohibit it | Residual only if someone deliberately changes code and bypasses review. The two interlocks are in the same source tree and not independent hardware inhibits. |
| H2: Ignition timing inferred from lambda | A developer adds lambda as an ignition input or auto-advance feature | Knock, detonation, engine damage | `ecu-ignition` has no lambda parameter; evidence policy; synthetic table marked simulation-only; physical export blocked | Future code could violate the design. Review and tests must enforce the invariant. |
| H3: False-lean fuel addition | Exhaust leak, misfire, stuck-lean sensor, sensor not ready, exhaust delay | Over-fueling, rich misfire, bore wash, catalyst damage | Shared gate rejects exhaust leak, misfire, not-ready, stale/implausible lambda; Mixture Map excludes cells and flags `LAMBDA_STUCK_SUSPECTED` hard faults; confidence/abstention | Detection thresholds and sensor placement are not validated. Real faults can be missed. |
| H4: Wrong fuel mass from injector data | Incorrect flow, dead time, fuel pressure, density, or stoich | Lean/rich mixture, detonation or misfire | Input bounds; pressure compensation; dead-time curve; preflight warns that injector data is unverified | No flow-bench or on-engine validation. Wrong data still produces plausible numbers. |
| H5: Trim windup or oscillation | Sensor lag, aggressive gains, persistent error | Unstable mixture, hidden correction | Bounded STFT/LTFT; conditional integration; deadband; freeze/decay; rate and dwell gates | Tuning and sensor dynamics are not validated on hardware. |
| H6: Stale or tampered calibration applied | Checksum mismatch, concurrent edit, wrong revision, crafted proposal | Wrong fuel/ignition data | Checksum verify; revision/parent; `apply_proposal` rejects mismatched base and out-of-bounds cells; service pauses before apply and requires `approve=true` | The service does not verify proposal provenance/expiry or actor, and history is in memory; multi-user concurrency/persistence are not implemented. |
| H7: Prompt injection via logs or notes | Adversarial text in telemetry/log/calibration text | Unauthorized proposal or action | Copilot is read-only; system prompt treats tool/log text as untrusted data; Zod output; no write tool. | No prompt-injection eval suite; model output is not programmatically cross-checked against the Rust proposal; no durable audit. Must be tested before exposure. |
| H8: Protection bypass or silent recovery | Latched fault cleared automatically, threshold disabled | Operation after low oil pressure, overtemp, overboost, or sync loss | Independent state machine; latching; explicit clear; freeze frames; cut/derate reactions | Thresholds are placeholders; no hardware fault injection. |
| H9: User treats an estimate as validated | Synthetic ignition, generic VE, target lambda presented without context | Engine damage or poor driveability | Provenance fields; preflight; visible UI warnings on synthetic ignition and estimated tables; status labels; timing-light requirement | The UI exists but warnings can be ignored; there is no authenticated release gate or external safety review. |
| H10: Unverified build/test state | No independent CI run attestation; copilot behavioral evals missing; core no_std checks pass but no target hardware runs them | Undetected regression | `cargo fmt`, `cargo clippy`, `cargo test --workspace` (128/0), firmware target check, CLI determinism/analyze/apply, UI build, copilot tsc/offline query, and service smoke tests pass; CI workflow exists | CI run evidence, behavioral copilot evals, and hardware validation are still missing. This is a release blocker for any stronger claim. |

## 4. Evidence status

| Evidence | Status |
|---|---|
| Source inspection of all Rust crates | Done for this documentation pass. |
| `cargo check` on library crates, test targets, and the CLI/service binaries | Passed on 2026-11-12 in the documentation environment. |
| `cargo fmt --all -- --check` / `cargo clippy --workspace --all-targets -- -D warnings` | Both pass on 2026-11-12. |
| `cargo test --workspace` | 128 passed, 0 failed on 2026-11-12. |
| Firmware target check | `cargo check -p opentech-firmware-stm32 --target thumbv7em-none-eabihf --no-default-features` passes; compile-only, not timing validation. |
| Unit/property tests | Included in the 128-test pass. |
| CLI scenario + determinism | `mixture_sweep` completed; `--check-determinism` passed with 2184 byte-identical frames. |
| CLI Mixture Map | `analyze` produced a three-cell proposal (1348 accepted / 836 rejected); `apply --approve` created revision 2. |
| Tuning studio build | `npm run typecheck` and `npm run build` passed. |
| Service smoke test | `/api/health` returned `physical_outputs: false`; `/api/sim/status` returned live telemetry. |
| Copilot build | `npm run build` passed. |
| Copilot offline mode | `/health` reported `offline-tools`; an offline `/api/copilot` query returned a three-cell proposal summary and per-request tool audit entries. |
| Copilot tests | `npm test` runs `tsc --noEmit` and passes; no behavioral, refusal, or prompt-injection eval suite. |
| STM32 skeleton | Target check passes; `PHYSICAL_OUTPUTS_AUTHORISED = false`; fail-closed tests exist. No HAL, driver, or timing validation. |
| Hardware/bench tests | None. |
| Independent review | None. |
| EMC/environmental/thermal testing | None. |
| Functional-safety analysis | This document is an initial hazard list, not a quantitative analysis. |

## 5. Assumptions

- The operator understands that this is a development platform, not a tuned ECU.
- No physical output path is added without a new safety review and change control.
- Calibration data used in simulation is synthetic or appropriately licensed.
- The virtual ECU is not used to certify a real engine calibration.
- The copilot remains outside the real-time control path and has no write credentials.
- Any future hardware uses independent protection, validated sensors, and a reviewed safe state.

## 6. Unresolved validation

The following are explicitly unresolved and must not be implied as complete:

- No actual engine or vehicle validation.
- No timing-light verification of base offset on a real engine.
- No knock-detection validation or knock-limit calibration.
- No injector flow/dead-time characterization on a bench.
- No sensor calibration validation (MAP, CLT, IAT, wideband).
- No fuel-pressure/regulator dynamics validation.
- No EMC, load-dump, brownout, thermal, vibration, or environmental testing.
- No target firmware build, HAL, timer capture/compare, watchdog integration, or flash/boot integrity testing.
- No hardware fault injection.
- No quantitative FMEA/FTA or diagnostic coverage analysis.
- No cybersecurity review of the service, copilot, firmware update, or CAN interfaces.
- No authentication or TLS on the local HTTP/WebSocket service; CORS is permissive.
- No durable audit log, proposal provenance/expiry/actor validation, or persistent undo.
- No behavioral copilot tests, prompt-injection evals, or programmatic cross-check that reported cells match the Rust proposal. `npm test` is only a TypeScript typecheck.
- No security review of the copilot service, model provider, or local API exposure; no authentication/TLS.
- No multi-user/concurrency validation of calibration apply/undo.
- No clean-build CI run evidence in this snapshot; the workflow exists but no run record is included.

## 7. Interlock release criteria

Release gate numbering follows [`ECU_MASTER_BRIEF.md`](../ECU_MASTER_BRIEF.md) exactly; definitions and evidence are also tracked in [`AUDIT/RELEASE_GATES.md`](../AUDIT/RELEASE_GATES.md). Current status: **Gate 0 only**.

| Gate | Scope | Status |
|---|---|---|
| 0 | Software simulation | Current; host simulation, CLI, UI, copilot, and skeleton checks |
| 1 | Unpowered hardware | Not started |
| 2 | Powered bench | Not started |
| 3 | Vehicle electrical compatibility | Blocked; ECU variant/pinout unverified |
| 4 | Controlled engine testing | Blocked |
| 5 | Vehicle operation | Blocked |

The physical output interlock must remain engaged until all of the following are true:

1. A reviewed hardware design exists with protected outputs, watchdog, brownout behavior, and a defined safe state.
2. Timer capture/compare and scheduled output behavior are measured on the bench.
3. Injector/coil drivers, sensors, and power stages are validated against their datasheets and thermal limits.
4. A hazard analysis with fault trees and diagnostic coverage is reviewed by an independent engineer.
5. Bench fault injection demonstrates safe response to sync loss, sensor failure, stuck outputs, and communication loss.
6. A supervised engine test plan exists, including timing-light verification, knock instrumentation, and an emergency stop.
7. Calibration data is verified for the specific engine; base maps are not treated as validated.
8. Legal/regulatory and insurance requirements for the intended use are understood.

Passing a compile, a unit test, or a KiCad DRC does not satisfy these criteria.

## 8. Change control

- Any change to the interlock, ignition policy, learning gates, protection thresholds, or apply/approval path requires a documented safety review.
- Any new physical output path requires a new safety case, not an edit to this one.
- Any derived code must keep its license and provenance; safety evidence must identify the exact source revision.
- Audit records must capture calibration revision, checksum, actor, proposal, approval, and apply/undo actions.

## 9. Conclusion

v0.1 is a host-side simulation, analysis, and tuning platform with a CLI, local service, browser/Electron UI, a read-only Mastra copilot, and a fail-closed STM32 skeleton. It has no physical actuation path, its ignition policy refuses lambda-based timing, and its fuel-learning path is gated and bounded. Those properties make it suitable for development and review, not for an engine.

The system is **not** a validated ECU, **not** a certified safety component, and **not** ready for a vehicle. The unresolved items in section 6 are mandatory before any stronger claim.
