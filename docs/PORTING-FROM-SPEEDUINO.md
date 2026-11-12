# Porting from Speeduino

Status: port in progress. No completed Speeduino parity is claimed. This document defines what “based on Speeduino” means here, how the retained C++ tree is used as an oracle, and what must be true before a feature can be called parity-verified.

## 1. Relationship to Speeduino

OpenTech-ECU is a new Rust-first platform **based on Speeduino**. It is a Rust port/reimplementation of Speeduino behavior, not a rebrand and not a continuation of the C++ firmware.

The original Speeduino C++ source tree is retained in this repository:

- `speeduino/` — firmware source, decoders, controllers, scheduling, fuel/ignition calculations, protection.
- `test/` — Speeduino native/unit tests and the ArduinoFake-based harness.
- `reference/` — TunerStudio INI, bootloaders, and reference data.
- `lib/` — test dependencies such as ArduinoFake.
- `boards/` — board definitions.

The C++ tree is not the product, is not modified by the Rust port, and is not deleted or replaced. It is the behavioral oracle: a concrete, executable definition of what Speeduino does, including its integer math, decoder state machines, configuration pages, and edge-case behavior.

## 2. What “port” means in this repository

The Rust code is an independent implementation of the same behavioral domain. It is not a mechanical line-by-line translation of the C++.

- At this snapshot, no Rust module carries a recorded line-by-line derivation from a specific Speeduino function. This is the current provenance state, not a legal conclusion. Future work that adapts Speeduino code must record the source file, function, commit, license, and nature of the adaptation before merge.
- The Rust workspace `Cargo.toml`, package metadata and `NOTICE.md` declare GPL-2.0-or-later, matching the Speeduino reference licence. Derived material must retain the applicable notices.
- A language change does not remove GPL obligations. See section 7.

## 3. What exists in Rust today

This is implementation coverage, not parity coverage.

| Area | Rust implementation | Parity status |
|---|---|---|
| Missing-tooth crank decoding | `ecu-trigger`: configurable 4–120 teeth, gap validation, dropped-tooth rejection, cam-phase latch, sequential sync, timeout | Not parity-tested; only the generic missing-tooth family, not Speeduino's OEM decoder set |
| Fuel calculation | `ecu-fuel`: ideal-gas speed density, VE, target lambda, injector dead time, pressure compensation, alpha-N/blended load | Not parity-tested; floating-point model, not Speeduino's integer `req_fuel` path |
| Ignition calculation | `ecu-ignition`: advance/dwell bounds, dwell-duty cap, rev limiter, evidence-gated recommendations | Not parity-tested; ignition policy is a deliberate OpenTech divergence |
| Scheduling | `ecu-scheduler`: 720-degree injection/dwell/spark events, 1–12 cylinders, firing order, wrap crossing | Not parity-tested; no hardware timer execution |
| Protection | `ecu-protection`: 14 DTCs, latching, freeze frames, warning/derate/limp/cut | Not parity-tested; thresholds are placeholders |
| Calibration/tables | `ecu-calibration`: rank 1–4 tables, clamped interpolation, sealed revisions, wizard, preflight | New model; not Speeduino config-page compatible |
| Lambda learning | `ecu-lambda`: shared gates, STFT, LTFT, offline Mixture Map | New functionality; Speeduino's INI exposes TunerStudio VE Analyze, not a firmware mixture-map engine |
| Simulation | `ecu-simulator`: 1 ms plant, trigger generation, wideband model, faults | New test harness; Speeduino's native tests are not a physics simulator |
| Protocol | `ecu-protocol`: JSON envelopes + binary CRC32 frames | New protocol; not TunerStudio serial compatible |

The Rust implementation also includes features Speeduino does not have in its firmware, such as a first-class ignition evidence policy and an offline Mixture Map designed for explicit approval/undo. Those are not parity items and must not be presented as “Speeduino features.”

## 4. Oracle strategy

### 4.1 Pin the oracle

Record the exact upstream Speeduino commit used for a comparison in the fixture manifest. Do not compare against a floating branch. The manifest must include:

- upstream repository URL and commit SHA;
- Speeduino license and notice;
- build environment and command;
- board/configuration used;
- input fixture and seed;
- output units, scaling, and rounding;
- tolerance and rationale;
- known divergences.

### 4.2 Build the native reference tests

Speeduino's own test suite is the first oracle. The repository's CI uses PlatformIO and a native simulator environment:

```bash
cd /home/jlee/OpenTech-ECU
platformio test -v -e megaatmega2560_sim_unittest
```

This requires PlatformIO and the project's Python/test dependencies. The exact environment may change upstream; record what was actually run. The native tests use Unity and ArduinoFake and cover decoders, fuel, ignition, scheduling, protection, sensors, tables, and more.

### 4.3 Unit vectors

Start with pure, deterministic functions and record input/output vectors:

- injector dead time vs voltage;
- required fuel / VE / target-lambda scaling;
- pulse-width limits, minimum pulse, and duty calculations;
- advance corrections and dwell limits;
- table interpolation and axis lookup;
- CRC and protocol byte layouts where applicable.

Rules:

- Integer and boolean outputs should match exactly unless a documented divergence exists.
- Floating-point outputs need a per-fixture tolerance, not a global “close enough” claim.
- Record units and scaling explicitly. A number without a unit is not a vector.
- Preserve the reference output verbatim in the fixture; do not regenerate expected values from the Rust implementation.

### 4.4 Decoder edge cases

Trigger decoding is where “works on a normal log” is weakest. The comparison set must include:

- constant-speed 36-1 and other missing-tooth patterns;
- acceleration and deceleration within one revolution;
- startup from zero, first gap, and first cam edge;
- missing/extra teeth, dropped teeth, and noise;
- cam phase present, absent, late, or noisy;
- 360/720 wrap and sequential phase;
- sync loss, timeout, and re-synchronisation;
- dual-wheel and OEM patterns only when the Rust decoder actually supports them.

Compare sync state transitions, reported RPM, crank angle, cycle angle, tooth index, and error counters. A decoder that produces the right RPM but the wrong phase is not parity.

### 4.5 Fuel and ignition scheduling I/O

Use fixed engine/calibration configurations and replay identical sensor snapshots:

- Compare injector pulse width, dead time, duty, staging split, and cut behavior.
- Compare spark advance, dwell, limiter state, and event angle.
- Compare injection and spark event order across the 720-degree cycle.
- Compare behavior at minimum pulse, maximum duty, low voltage, and rev-limit boundaries.
- Include cranking, idle, transient, overrun, and protection-cut cases.

If the reference output depends on AVR/Teensy integer widths or ISR timing, record that context. Do not compare a host floating-point result to a fixed-point firmware result without a documented conversion.

### 4.6 CSV/log fixtures

The most useful end-to-end evidence is a replayable log:

1. Run the Speeduino native tests or an instrumented reference build under a fixed configuration and scenario.
2. Export a time-ordered CSV/log with all inputs needed to reproduce the decision: RPM, MAP, TPS, IAT, CLT, battery, lambda/target, injector PW/duty, advance/dwell, sync state, trims, protection state, and active faults.
3. Store the fixture and its manifest under a versioned path such as `ecu/tests/fixtures/speeduino/<commit>/`.
4. Replay the same inputs through the Rust control crates.
5. Diff the outputs with per-channel tolerances and fail the test on unexplained divergence.

Do not commit proprietary or unlicensed calibration data. Fixtures must be original, licensed, or synthetic with provenance.

Self-consistency is not parity. The CLI's `run --check-determinism` proves the Rust simulator repeats byte-identically (it passed with 2184 frames for `mixture_sweep`); it says nothing about matching Speeduino.

## 5. Suggested porting order

This is an engineering order, not a promise or schedule.

1. **Infrastructure first.** Add a fixture manifest, a differential test harness, and a CI job that builds the reference tests. No parity claim without reproducible fixtures.
2. **Missing-tooth decoder.** Compare the generic decoder against Speeduino's missing-tooth tests and add edge-case vectors. This is the highest-value early parity target because the Rust simulator depends on it.
3. **Fuel pulse width.** Compare dead time, required fuel, VE lookup, target lambda, pressure correction, and limits under fixed inputs.
4. **Ignition scheduling and dwell.** Compare event angles, dwell limits, rev limiting, and cut behavior. Keep the OpenTech evidence policy as a documented divergence.
5. **Protection.** Compare DTC thresholds, latching, and reaction priority. Treat Speeduino thresholds as reference behavior, not validated engine limits.
6. **Decoder family expansion.** Add OEM patterns only with their own vectors and edge cases. Do not claim “all Speeduino decoders” from a generic decoder.
7. **Corrections and advanced control.** Idle, boost, VVT, launch, nitrous, flex fuel, and similar families need separate behavioral contracts and fixtures.

## 6. Parity definitions

Use precise language:

- **Parity-verified**: the Rust implementation produces the same observable outputs as the pinned Speeduino reference for the documented fixture set, within recorded tolerances, including edge cases, with known divergences listed.
- **Behaviorally aligned**: the same control intent and state machine, but outputs are not yet compared across the full fixture set.
- **Implemented**: Rust code exists and type-checks. This says nothing about Speeduino equivalence.
- **Divergent by design**: OpenTech intentionally does something different, such as the ignition evidence policy or the new calibration/protocol model.

No feature in this repository is currently **parity-verified**. The table in section 3 is implementation coverage only.

## 7. GPL and provenance obligations

- The retained Speeduino tree is GPL-2.0 (GPLv2-or-later per its file headers). Its notices and license must remain intact.
- The Rust workspace declares GPL-2.0-or-later. Distribution of binaries or derived works must satisfy the GPL: source availability, license text, notices, and no additional restrictions.
- Do not copy code from Speeduino, RusEFI, or another project without recording the source and license. A copied function must retain its copyright notice and be compatible with the project license.
- Do not assume that rewriting code in Rust makes it independent. Conversely, do not claim GPL violation based only on behavioral similarity; record the actual provenance and let the license analysis follow the code.
- Speeduino hardware designs are CERN-OHL-S-2.0. Any derived hardware must comply with that license. No new hardware is included in this snapshot.
- “Speeduino” marks and branding remain with their owners. This project does not claim endorsement.

### Provenance ledger

Every derived module should eventually have an entry in a provenance ledger (for example `docs/PROVENANCE.md`) with:

| Field | Meaning |
|---|---|
| Rust path | File/module in this repository. |
| Source project | Speeduino, RusEFI, datasheet, paper, or original. |
| Source version | Commit SHA or document revision. |
| Source location | File/function/table. |
| License | SPDX identifier and notice requirements. |
| Method | Translated, adapted, reference-only, or original. |
| Author/date | Who made the change and when. |
| Tests | Fixtures that exercise the derived behavior. |
| Notes | Known divergences and review status. |

## 8. Known risks

- **Integer vs float math.** Speeduino's fixed-point/integer behavior may round differently from Rust `f32`. A tolerance is not a substitute for understanding the conversion.
- **Units and scaling.** RPM, kPa, milliseconds, degrees, and injector flow may be scaled differently in configuration pages and telemetry.
- **Table axes.** Speeduino's table sizes and axis ordering may not match the Rust `Table` model. Do not assume axis compatibility.
- **Scheduler context.** Speeduino schedules on MCU timers with ISR latency; the Rust scheduler computes angles on the host. Event angles can match while real-time behavior does not.
- **Configuration pages.** TunerStudio INI pages are a wire/configuration format. The OpenTech calibration document is a different model; configuration parity is a separate project.
- **Feature interactions.** Corrections, trims, cut behavior, and protection often interact. A unit vector can pass while the integrated behavior diverges.
- **Reference test scope.** Speeduino's native tests use mocks and may not represent every board or firmware path. Use both unit tests and recorded firmware logs where possible.
- **License drift.** Upstream notices and file headers can change. Re-check the pinned commit and license before distributing derived work.

## 9. What is explicitly not claimed

- No completed Speeduino parity.
- No claim that the Rust code is a translation of the C++ tree.
- No claim that all Speeduino decoders, corrections, or controllers are implemented.
- No claim that the OpenTech calibration or protocol is compatible with TunerStudio or Speeduino configuration pages.
- No claim that the C++ tree has been validated for this project's use beyond its role as a reference.
- No claim that passing a fixture set makes an ECU safe for an engine.
