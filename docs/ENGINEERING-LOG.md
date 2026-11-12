# Engineering log — 2026-11-12

This is a factual build log, not a release note or safety approval. Times are
local working notes; the final verification commands are recorded in
`AUDIT/TEST_EVIDENCE.md`.

## Milestone 1 — virtual ECU, UI, lambda tuning

Completed:

- Cargo workspace with typed units, engine/calibration models and 1D–4D tables.
- SHA-256 sealed calibrations, starter wizard, preflight checks and revision diffs.
- 36-1 crank/cam decoder with sync-loss, dropped-tooth and timeout handling.
- Physical speed-density fuel model with injector dead time, pressure correction,
  duty inhibition and unit/hand-calculation tests.
- 720° injection/ignition scheduler and evidence-gated ignition policies.
- Bounded STFT with anti-windup and sensor-loss decay.
- Cell/bank LTFT with dwell gates and selective merge.
- Offline Mixture Map with time alignment, shared gates, confidence, bounded
  proposals, trim-aware correction, stuck-sensor abstention, apply/undo and
  deterministic output.
- Independent protection state machine with DTCs, freeze frames, latching and
  recovery.
- Deterministic 1 ms simulator with trigger generation, wideband delay/noise/
  faults, warm-up, transient, fault-recovery and trigger-loss scenarios.
- Rust Axum HTTP/WebSocket service, versioned JSON/binary protocol, logs/CSV,
  axis resampling and replay.
- React/Vite/Electron tuning studio with live dashboard, editable maps, 3D
  surfaces, Mixture Map review, diagnostics, calibration audit and logs.
- Mastra TypeScript copilot with read-only structured tools and an offline tool
  mode; no write path, no ignition advice, human approval required.
- no_std STM32 safety skeleton that compiles for `thumbv7em-none-eabihf`, with
  a hard physical-output interlock.
- Evo VIII GSR simulator-only vehicle profile with a fail-closed physical request
  and an UNVERIFIED pinout/trigger/injector/ignition matrix.

Verification at the end of the milestone:

- `cargo fmt --all -- --check` pass.
- `cargo clippy --workspace --all-targets -- -D warnings` pass.
- `cargo test --workspace` 128 passed, 0 failed.
- `cargo check -p opentech-firmware-stm32 --target thumbv7em-none-eabihf --no-default-features` pass.
- `npm run build` in the tuning studio pass.
- `npx tsc --noEmit` in the copilot pass.
- Headless mixture sweep deterministic (2184 frames byte-identical); analysis
  produced bounded VE cells; apply produced a sealed revision 2.
- Local service health reported `physical_outputs: false`; a browser smoke test
  showed live telemetry, map UI, setup/profile and Mixture Map apply.
- Copilot offline mode returned tool audit entries and structured output.

## Key engineering decisions

1. **Fuel and ignition are separate evidence domains.** Lambda is never an
   ignition input; the ignition recommendation API has no lambda parameter.
2. **Learning is gated and reviewable.** Online STFT, LTFT and offline Mixture
   Map are separate; proposals never auto-apply; stale checksums fail.
3. **The virtual ECU is deterministic and fail-closed.** Fixed 1 ms steps,
   seeded noise, output interlock, protection independent of learning.
4. **Do not guess Evo details.** The reference profile is simulation-only and
   lists blocking unknowns. The C++ Speeduino tree remains as an oracle.
5. **Electron rather than Tauri in this environment.** Electron is a desktop
   shell around the same local Rust service/UI because WebKitGTK development
   libraries are unavailable here. Tauri remains an option on a normal desktop
   toolchain.

## Known issues / unresolved

- Vehicle pinout, connector family, trigger pattern/phase, injector, ignition
  and factory-system behaviour are UNVERIFIED.
- No reviewed manufacturing PCB/BOM/gerbers; no bench hardware; KiCad toolchain
  absent. A separate unreviewed Codex attempt does not change Gate 1.
- No MCU HAL, timer capture/compare, watchdog, flash or target image.
- Calibration history is in memory, not durable or signed for provenance.
- Replay UI start was exercised, but full visual playback was not independently
  audited.
- No Speeduino C++ oracle test suite is yet compiled; parity remains planned and
  must not be claimed as verified.
- LLM mode was not called; offline tool mode was tested.
- UI/service have no authentication or TLS and must remain local-only.

## Next highest-value work

1. Independent Rust review of the trigger/protection/fuel scheduling paths.
2. Build a compiled Speeduino oracle harness and comparison fixtures.
3. Review/produce bench hardware; run Gate 1/2 tests with dummy loads.
4. Complete Evo pinout/trigger/injector/ignition verification on the actual car
   with the engine off.
5. Durable, signed calibration/audit storage and firmware flash safety.
