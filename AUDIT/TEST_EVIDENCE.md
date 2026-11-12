# Test Evidence (2026-11-12, current snapshot)

| Field | Value |
|---|---|
| Document ID | AUD-05 |
| Status | Evidence record for the audited host snapshot; not a certification, not a physical-safety approval |
| Date | 2026-11-12 |
| Evidence cut-off | 2026-11-12T05:45Z, working tree |
| Repository | `/home/jlee/OpenTech-ECU` |
| Git HEAD | `fdf25420bb6925389f17a0e2d1fa365159acba3d` |
| Rust source manifest | SHA-256 `9837ba880320ae30dc4913a7e077b6b1b5f2b19b528e0630ab1d069ea9dec724` over all `.rs` files under `ecu/crates`, `ecu/apps/ecu-sim-cli`, `ecu/apps/studio-service`, `ecu/firmware/stm32` |
| Cargo lock | SHA-256 `71ecae98186c1bc59d46293bae54f3dc2db3cca86bba79c0e77f9b837a2f329e` |
| Test result | **128 tests (126 unit + 2 integration): 128 passed, 0 failed**; 0 doc-tests |
| Formatting | `cargo fmt --all -- --check` passed |
| Linting | `cargo clippy --workspace --all-targets -- -D warnings` passed |
| Firmware target check | `cargo check -p opentech-firmware-stm32 --target thumbv7em-none-eabihf --no-default-features` passed |
| Gate status | **Gate 0 host software evidence: PASS for implemented behaviour; not a certification, not Gate 1+** |
| Overall limitation | No physical hardware, no reviewed PCB, no MCU timing capture, no vehicle evidence; several UI/copilot/runtime behaviours remain untested |

> **Scope.** This evidence supports host-side software behaviour only. It does not establish engine safety, timing accuracy, EMC, electrical safety, or Evo VIII compatibility. Gate 0 pass means "the implemented host behaviour has reproducible evidence", not "the ECU is safe or ready".

> **Source-control note.** Source commit `041be84f` is the code revision used for this evidence; current HEAD `fdf25420` contains documentation/audit commits only. The Rust source tree was clean at cut-off (`git status --short ecu` empty). The manifest hash above matches that tree. Re-hash the source after any source change.

## 1. Environment

| Item | Value / status |
|---|---|
| Host | Linux (x86_64), audit container |
| Rust | `rustc 1.99.1 (58cf8f9c3 2026-10-13)` |
| Cargo | `cargo 1.99.1 (5f94df478 2026-08-27)` |
| Node | `v24.14.1` |
| npm | `11.11.0` |
| `rustfmt` | Available and passing |
| `clippy` | Available and passing with `-D warnings` |
| KiCad / `kicad-cli` | Not installed on the host PATH; an isolated Codex toolchain produced ERC (0) and DRC (14 violations (13 dangling vias + 1 clearance), 499 unconnected) on the unreviewed schematic/PCB draft. Not independently reviewed. |
| Electron | Dependency present; **application launch not exercised** in this headless audit |
| WebKit/Tauri | Not used by the current Electron shell; no Tauri validation; Electron launch untested |
| Physical hardware | **None**; no assembled board/PCB, harness, sensors, injectors, coils, or vehicle; unreviewed schematic only |
| Network/service | Local smoke test on `127.0.0.1:18769`; no external exposure |

## 2. Command summary

| # | Command | Exit | Result |
|---:|---|---:|---|
| 1 | `cargo fmt --all -- --check` | 0 | Clean |
| 2 | `cargo clippy --workspace --all-targets -- -D warnings` | 0 | Clean, including the firmware skeleton |
| 3 | `cargo test --workspace` | 0 | 128 passed, 0 failed, 0 ignored; 0 doc-tests, including two Speeduino formula-parity tests |
| 4 | `cargo check -p opentech-firmware-stm32 --target thumbv7em-none-eabihf --no-default-features` | 0 | Passes; skeleton only, no HAL/timer/driver |
| 5 | `cargo check -p ecu-types -p ecu-fuel -p ecu-ignition -p ecu-trigger -p ecu-scheduler -p ecu-protection --no-default-features` | 0 | Core no_std checks pass |
| 6 | `cargo run -q -p ecu-sim-cli -- run --scenario mixture_sweep --out ... --check-determinism` | 0 | Determinism PASS, 2184 frames byte-identical |
| 7 | `cargo run -q -p ecu-sim-cli -- analyze --log ... --cal ... --out ...` | 0 | 1378 used / 806 rejected; 3 proposed cells; ignition untouched |
| 8 | `cargo run -q -p ecu-sim-cli -- apply ... --approve` | 0 | Applied 3 cells; revision 1 -> 2 |
| 9 | `cargo run -q -p ecu-sim-cli -- apply ...` without `--approve` | 1 | Refused, as designed |
| 10 | `npm run typecheck` in `apps/tuning-studio` | 0 | Pass |
| 11 | `npm run build` in `apps/tuning-studio` | 0 | Vite build, 44 modules, `dist/` produced |
| 12 | `npm run build` in `apps/tuning-copilot` (`tsc --noEmit`) | 0 | Pass |
| 13 | Local `studio-service` smoke test | 0 | Health/preflight/status; physical outputs false |
| 14 | KiCad schematic ERC + PCB DRC on `hardware/kicad/evo8-bench-ecu` | 0 / 5 | Unreviewed 7-sheet schematic, 0 ERC; PCB draft 14 DRC violations + 499 unconnected; no gerbers |

## 3. Rust test results

### 3.1 Per-crate counts

| Crate | Tests | Passed | Failed | Notes |
|---|---:|---:|---:|---|
| `ecu-calibration` | 22 | 22 | 0 | Includes vehicle profile tests |
| `ecu-diagnostics` | 4 | 4 | 0 | |
| `ecu-fuel` | 11 | 11 | 0 | 9 unit + 2 Speeduino parity integration tests |
| `ecu-ignition` | 7 | 7 | 0 | |
| `ecu-lambda` | 28 | 28 | 0 | Includes trim/Mixture Map tests |
| `ecu-protection` | 10 | 10 | 0 | |
| `ecu-protocol` | 7 | 7 | 0 | |
| `ecu-scheduler` | 6 | 6 | 0 | |
| `ecu-simulator` | 10 | 10 | 0 | Full-run scenario tests |
| `ecu-trigger` | 10 | 10 | 0 | |
| `ecu-types` | 8 | 8 | 0 | |
| `opentech-firmware-stm32` | 5 | 5 | 0 | No_std safety skeleton tests |
| `ecu-sim-cli` | 0 | 0 | 0 | Binary; no unit tests |
| `studio-service` | 0 | 0 | 0 | Binary; no unit tests |
| Doc-tests | 0 | 0 | 0 | No doc-tests |
| **Total** | **128** | **128** | **0** | |

### 3.2 Mixture Map trim behaviour

The current `active_trims_are_included_in_pre_trim_equivalent` test passes. It now sets:

```rust
max_cell_correction: 0.20
```

The production default remains `0.05` (5%). Therefore:

- The test proves the trim term is included when the per-cell bound is widened.
- The default Mixture Map still caps a single proposal at 5% per cell; the `mixture_sweep` proposals all hit +5.0%.
- This is a conservative bound, not a claim that 5% is the correct calibration limit.

No failing test is present in this snapshot.

## 4. Formatting, linting and firmware target

### 4.1 Format and clippy

```text
cargo fmt --all -- --check
FMT_EXIT=0

cargo clippy --workspace --all-targets -- -D warnings
Finished `dev` profile ... 
CLIPPY_EXIT=0
```

Both commands include the firmware skeleton and all workspace targets. Clippy is configured to deny warnings, so this is stronger than a warning-only check.

### 4.2 Firmware skeleton target check

```text
cargo check -p opentech-firmware-stm32 --target thumbv7em-none-eabihf --no-default-features
Finished `dev` profile ...
FW_EXIT=0
```

What this proves:

- The `opentech-firmware-stm32` crate compiles for `thumbv7em-none-eabihf` without default features.
- It has `#![cfg_attr(not(test), no_std)]`, `#![forbid(unsafe_code)]`, and 5 unit tests.
- It links `ecu-trigger` and `ecu-protection` in no_std mode.
- It forces outputs off; `PHYSICAL_OUTPUTS_AUTHORISED = false`; arming cannot override the interlock.

What this does **not** prove:

- No STM32 HAL, pin mapping, timer capture/compare, injector/ignition driver, watchdog integration, flash/calibration storage, CAN, sensors, or bootloader exists.
- No timing, jitter, interrupt latency, stack usage, power-up state, or hardware safe state has been measured.
- The skeleton is not a flashable ECU firmware image and has no physical output authorisation.

### 4.3 Core `no_std` portability

`cargo check -p <crate> --no-default-features` results:

| Crate | Result | Evidence | Interpretation |
|---|---|---|---|
| `ecu-types` | PASS | Core no_std check | no_std-capable |
| `ecu-fuel` | PASS | Core no_std check | no_std-capable |
| `ecu-ignition` | PASS | Core no_std check | no_std-capable |
| `ecu-trigger` | PASS | Core no_std check | no_std-capable |
| `ecu-scheduler` | PASS | Core no_std check | no_std-capable |
| `ecu-protection` | PASS | Core no_std check | no_std-capable |
| `ecu-protocol` | PASS (earlier) | none | **Not a no_std crate**; no `#![no_std]`, uses `std` |
| `ecu-lambda` | PASS (earlier) | none | **Not a no_std crate**; uses `std` collections/strings |

**Conclusion:** all six core control crates and the firmware skeleton pass their no_std checks. `ecu-protocol` and `ecu-lambda` are still std-only by design; do not claim the entire workspace is no_std. The firmware skeleton still has no HAL/timer/driver/timing evidence.

## 5. Headless CLI evidence

### 5.1 Scenario list

`cargo run -q -p ecu-sim-cli -- scenarios` lists:

| ID | Duration | Name |
|---|---:|---|
| `idle_warmup` | 25 s | Cold crank to idle |
| `mixture_sweep` | 42 s | Guided lambda mixture sweep |
| `transient` | 28 s | Throttle transients |
| `fault_recovery` | 34 s | Fault injection and recovery |
| `sensor_stuck_lean` | 30 s | Stuck-lean wideband trap |
| `trigger_loss` | 22 s | Crank signal loss |

### 5.2 Determinism

`mixture_sweep` with `--check-determinism`:

```text
determinism: PASS (2184 frames byte-identical)
scenario: mixture_sweep
frames:   2184
duration: 42.0s
rpm mean/max: 2769 / 4566
coolant max: 89.6 C
lambda mean: 0.977 (2184 samples)
faults:   ["P0335", "P0133"]
```

Previously observed runs of `trigger_loss`, `fault_recovery`, `sensor_stuck_lean`, `idle_warmup` and `transient` also reported PASS. Determinism is per calibration; the current starter calibration is also deterministic (below).

### 5.3 Fresh-calibration determinism

The current working tree sets `created_unix_ms: 0` in `generate_starter` (committed in the OpenTech source revision). Two fresh `idle_warmup` runs without `--cal` produced identical calibration checksums and byte-identical logs:

| Run | `created_unix_ms` | Calibration checksum prefix | Log SHA-256 prefix |
|---|---:|---|---|
| Fresh 1 | 0 | `fee18418` | `ade065d3` |
| Fresh 2 | 0 | `fee18418` | `ade065d3` |

Both reported `determinism: PASS (1300 frames byte-identical)`. This resolves the earlier timestamp caveat; the change is committed and the evidence was reproduced on the committed source. For audit evidence, still record the calibration checksum with the log.

### 5.4 Mixture Map analysis

```text
samples used/rejected: 1378/806  cells proposed: 3  hard faults: []
  gates: {"CRITICAL_DTC": 12, "ENGINE_NOT_RUNNING": 13, "FUEL_CUT": 13,
          "LAMBDA_IMPLAUSIBLE": 32, "LAMBDA_NOT_READY": 32, "LAMBDA_STATUS": 32,
          "MAP_TRANSIENT": 196, "RPM_LOW": 4, "RPM_TRANSIENT": 447,
          "SYNC_NOT_SEQUENTIAL": 8, "TPS_TRANSIENT": 204, "WOT_BOUNDARY": 411}
proposal: mix-e56c7b554ec2b0bd (3 cells, requires approval)
   1000 rpm   50 kPa  VE 46.71 -> 49.05  (+5.0%, n=170, conf=0.78)
   2100 rpm   50 kPa  VE 59.16 -> 62.12  (+5.0%, n=431, conf=0.85)
   2900 rpm   60 kPa  VE 66.13 -> 69.44  (+5.0%, n=259, conf=0.92)
ignition untouched: true
```

Observations:

- All three proposals hit the default +5% per-cell cap.
- The gate counts show substantial rejection for transients and WOT exclusion.
- `ignition_untouched: true` is a data assertion; the applied calibration's ignition table must be independently compared if that matters.

### 5.5 Apply evidence

With `--approve`:

```text
applied 3 cells; revision 1 -> 2; checksum b91cda78fe065037
diff: 3 cells, max |delta| 3.31 VE points
undo snapshot is the input calibration: .../calibration.json
```

Without `--approve`:

```text
Error: refusing to apply without --approve. This applies a proposal to a virtual calibration only;
it is not engine-safe validation.
```

Exit code 1. This is the intended fail-closed behaviour.

### 5.6 Stuck-wideband abstention

The `sensor_stuck_lean` analysis reports `LAMBDA_STUCK_SUSPECTED` and no proposal. This is an abstention in the exact-constant simulator case, not proof of real sensor diagnostics.

## 6. UI and copilot evidence

### 6.1 Tuning studio

```text
npm run typecheck
TYPECHECK_EXIT=0

npm run build
vite v6.4.6 building for production...
✓ 44 modules transformed.
dist/index.html                   0.78 kB
dist/assets/index-CL8twMhr.css   17.30 kB
dist/assets/index-BbCp81ft.js   285.32 kB
✓ built in 1.15s
BUILD_EXIT=0
```

The build produces `apps/tuning-studio/dist/`. The UI source includes a dashboard, VE/target/ignition maps, Mixture Map, setup wizard, diagnostics, calibration audit, logs and a copilot page. Source inspection confirms visible "virtual only", "unverified" and "physical export blocked" messaging.

**Untested:** Electron application launch (headless audit), browser interaction, visual layout, accessibility, replay UI, WebSocket reconnect/lag handling, and the DTC label mapping (source inspection suggests a snake_case/enum-to-P030x mismatch). A TypeScript/Vite build is not a UI acceptance test.

### 6.2 Copilot

```text
cd apps/tuning-copilot
npm run build     # tsc --noEmit
COPILOT_BUILD_EXIT=0
```

**Untested:** live Mastra/LLM call, tool execution against the service, prompt-injection resistance, approval boundary, offline mode. Typecheck only proves the TypeScript is syntactically/type valid.

## 7. Local service smoke test

Command: `cargo run -q -p studio-service -- --port 18769 --no-loop`

| Endpoint | Observed |
|---|---|
| `/api/health` | `status:ok`, protocol 1, `physical_outputs:false`, `interlock:PHYSICAL_OUTPUT_INTERLOCK_ENGAGED` |
| `/api/preflight` | `error_count:2`, `warning_count:4`, `hardware_candidate:false`, `physical_export_allowed:false`, `virtual_run_allowed:true`; summary says physical use blocked |
| `/api/sim/status` after ~6 s | `running:true`, 544 frames, RPM ~909, `sync:"Sequential"`, `protection:"Warning"`, revision 1 |
| `/api/vehicles` (earlier) | Evo profile `verification:"unverified"`, `ecu_part_number:null`, `pinout_documented:false`, `trigger_verified:false`, 12 blocking items |

The service binds localhost by default but has no authentication and uses permissive CORS. It is a development service, not a secure production interface.

## 8. CI workflow

`.github/workflows/opentech-ci.yml` defines:

- Rust: `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test --workspace`, firmware no_std target check, deterministic simulator run/analyze, artifact upload.
- UI: `npm ci` + `npm run build` in `apps/tuning-studio`.
- Copilot: `npm ci` + `npx tsc --noEmit` in `apps/tuning-copilot`.

The workflow was **not observed running** in this audit. The local commands above are equivalent evidence for this host; CI status is not claimed. The workflow is a good control, but it is not proof that the current branch has run it.

## 9. Test-to-requirement mapping

| Requirement / claim | Evidence | Result | Caveat |
|---|---|---|---|
| ECS-SR-001: no physical output path | No hardware driver; firmware skeleton `PHYSICAL_OUTPUTS_AUTHORISED=false`; tests | Pass (host/skeleton) | Host/skeleton evidence; working tree changed after source commit |
| ECS-SR-002: interlock reported | `physical_interlock_constant_is_false`; `/api/health`; preflight | Pass | Compile-time constant |
| ECS-SR-003: lambda never ignition | `ecu-ignition` API; 7 tests | Pass | Review invariant; future edits |
| ECS-SR-006: shared learning gate | `every_invalidity_is_explained`, `nan_lambda_rejected` | Pass | Inputs partially caller-supplied |
| ECS-SR-007: trim bounds | `integrator_never_winds_beyond_limit`, `deadband_prevents_chatter` | Pass | Gains unvalidated |
| ECS-SR-008: Mixture Map approval | CLI analyze/apply; `apply_proposal` tests | Pass | History in memory |
| ECS-SR-009: calibration seal | `checksum_detects_tampering`, revision tests | Pass | Hash unkeyed; not authenticity |
| ECS-SR-010: trigger sync | 10 trigger tests | Pass | Cam-loss/extra-edge timing gaps |
| ECS-SR-011: sync-loss cut | `trigger_loss_causes_cut_then_resync` | Pass (sim) | 250 ms timeout risk |
| ECS-SR-012: independent protection | 10 protection tests; firmware skeleton integration | Pass (software) | No hardware monitor |
| ECS-SR-013: latching/clear | overtemp/low-oil tests | Pass | No debounce/actor identity |
| ECS-SR-017: hardware timer execution | Firmware skeleton has no timer implementation | **Not met** | Gate 2+ |
| ECS-SR-019: apply pause/approval | CLI no-approve refusal; service pause checks | Pass | No auth/CSRF |
| ECS-SR-020: UI warnings | UI build; source inspection | Partial | No browser/Electron test; DTC bug |
| ECS-SR-021: timing light/knock | None | **Blocked** | Gate 4 |
| ECS-SR-022: Evo data | `vehicle.rs`; `/api/vehicles` | **Unverified** | Gate 3 |
| Gate 0 host software | 128 tests, fmt, clippy, UI build, copilot typecheck, firmware target check, two narrow Speeduino formula-parity tests | **Pass, scoped** | Not a certification; not physical; no full Speeduino oracle build |
| Gate 1 | Hardware package | **Blocked** | Unreviewed schematic/netlist/BOM and PCB draft; DRC fails; no gerbers/review |
| Gate 2 | Powered bench | **Blocked** | No firmware image/timing/fixture |
| Gate 3-5 | Vehicle/engine/road | **Blocked** | No Evo evidence/engine/legal |

## 10. What is still untested

| Item | Why it matters | Status |
|---|---|---|
| Electron launch | The desktop shell may fail even if Vite builds | Not run |
| Browser interaction / replay UI | UI may be unusable or show stale data | Headless Chromium smoke test: dashboard, map, setup, Mixture Map apply, replay start; visual review limited |
| Live Mastra/LLM call | Copilot may fail, leak, or exceed authority | Not run |
| Copilot tool approval boundary | Untrusted output must not write | Not tested |
| WebSocket lag/reconnect | Operator may see stale data | Not tested |
| Full core `no_std` | All core crates and firmware skeleton pass | Pass |
| KiCad ERC/DRC | Unreviewed: ERC 0 violations; DRC 14 violations + 499 unconnected items | Not independently reviewed; no gerbers |
| Hardware/bench | Unrouted PCB draft only; no fixture or built ECU | Not run |
| Scope timing | No firmware/timer | Not run |
| Evo pinout/trigger/harness | Unverified | Not run |
| EMC/thermal/power | No hardware | Not run |

## 11. What cannot be concluded

- Gate 0 pass is **not** a certification and **not** permission for Gate 1+.
- No physical output, timing, EMC, thermal, watchdog, power, sensor, injector, coil, harness or vehicle test exists.
- No evidence that the Evo VIII is compatible, plug-and-play, or safe to connect.
- No evidence that the firmware skeleton is flashable, timed, or safe on an MCU.
- No evidence that the UI/copilot behave correctly at runtime; only type/build checks.
- The manifest hash matches the clean committed source tree at cut-off; re-hash after any source change. HEAD contains documentation-only commits after the source commit.
- Only the six control crates listed in §4.3 are proven no_std-capable; `ecu-protocol` and `ecu-lambda` remain std-only by design.

## 12. Reproduction commands

From `/home/jlee/OpenTech-ECU/ecu`:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo check -p opentech-firmware-stm32 --target thumbv7em-none-eabihf --no-default-features

for c in ecu-types ecu-fuel ecu-ignition ecu-trigger ecu-scheduler ecu-protection; do
  cargo check -p "$c" --no-default-features
done

cargo run -q -p ecu-sim-cli -- scenarios
cargo run -q -p ecu-sim-cli -- run --scenario mixture_sweep --out /tmp/audit/mixture_sweep --check-determinism
cargo run -q -p ecu-sim-cli -- analyze --log /tmp/audit/mixture_sweep/log.json --cal /tmp/audit/mixture_sweep/calibration.json --out /tmp/audit/mixture_sweep/report.json
cargo run -q -p ecu-sim-cli -- apply --cal /tmp/audit/mixture_sweep/calibration.json --report /tmp/audit/mixture_sweep/report.json --out /tmp/audit/mixture_sweep/applied.json --approve

cd apps/tuning-studio && npm run typecheck && npm run build
cd ../tuning-copilot && npm run build
```

Expected: all listed commands pass at this snapshot, including the core `no_std` checks and the firmware target check.

## 13. Raw evidence locations

- `/tmp/opencode/final-test.log` — 128-test run.
- `/home/jlee/OpenTech-ECU/evidence/VERIFICATION.txt` — companion verification record with the same final counts; re-check hashes before relying on it.
- `/tmp/opencode/audit-current/` — log, CSV, summary, calibration, report, applied calibration.
- `/tmp/opencode/studio-final.log` — local service smoke log.
- `/tmp/opencode/cli3-run.log`, `/tmp/opencode/cli3-analyze.log` — CLI output.

These are ephemeral copies outside the repository. Re-run the commands; do not rely on a screenshot or a single log line.
