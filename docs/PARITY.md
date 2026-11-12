# Speeduino parity and the behavioural oracle

OpenTech-ECU is a Rust-first project **based on Speeduino**. The Speeduino C++
source remains in this repository (`speeduino/`, `test/`, `reference/`, `lib/`,
`boards/`) under GPL-2.0. It is used as a reference implementation and eventual
behavioural oracle, not as the product firmware.

Keeping the C++ tree is deliberate:

- decoder edge cases and tooth-count behaviour can be compared against a known
  implementation;
- fuel/ignition scheduling and correction ordering can be checked for unintended
  semantic changes;
- log fixtures and unit-test vectors can be reused with provenance;
- reviewers can see which Rust behaviour is a port, a deliberate divergence, or
  a new feature.

## Current parity evidence

This is **not** a full port and not a claim of parity.

| Area | Rust implementation | Speeduino reference | Parity evidence |
|---|---|---|---|
| Required fuel / paired 4-stroke rule | `ecu-fuel` physical model; compatibility rule in test | `fuel_calcs.cpp`, `test/test_fuel/test_pw_calculateRequiredFuel.cpp` | Upstream test vectors reproduced in `ecu/crates/ecu-fuel/tests/speeduino_parity.rs` |
| Speed-density pulse-width form | `ecu-fuel::calculate` | `calcPrimaryPulseWidth`, `test_pw_calcPrimaryPulseWidth.cpp` | At MAP=100 kPa, VE/lambda matched, Rust physical fuel math matches `reqFuel*VE + deadtime` within 0.002 ms |
| Missing-tooth sync | `ecu-trigger` | `src/decoders/missingTooth.cpp`, decoder tests | Behavioural cases implemented independently (gap, dropped tooth, timeout, cam phase); no compiled side-by-side test yet |
| Injection/spark angles and dwell | `ecu-scheduler`, `ecu-ignition` | ignition/fuel schedule tests | Event ordering/dwell tests in Rust; no side-by-side schedule comparison yet |
| Protection reactions | `ecu-protection` | `engineProtection.cpp` | Rust state-machine tests; thresholds intentionally differ and are simulator placeholders |
| Closed-loop/offline trim | `ecu-lambda` | Speeduino EGO correction; TunerStudio `veAnalyzeMap` | New workflow, deliberately not a line-by-line port |
| Host protocol | `ecu-protocol` | TunerStudio serial protocol | Deliberately different, versioned and checksummed |

The parity tests are evidence for a narrow formula relationship, not proof that
the Rust ECU behaves like Speeduino across all inputs.

## Planned oracle harness

A future compiled oracle should:

1. Build Speeduino's native test/fixture target in an isolated, pinned toolchain
   (PlatformIO/ArduinoFake/Unity as required), with the upstream revision and
   licence notice recorded.
2. Feed identical deterministic inputs (rpm, MAP, IAT, VE, corrections,
   injector data, trigger edges) to the C++ functions and the Rust crates.
3. Serialise outputs as a stable JSON/CSV schema with units and rounding rules.
4. Compare:
   - required fuel and pulse width at boundary values;
   - missing-tooth/gap/dropped-tooth decoder state transitions;
   - ignition/injection event ordering and dwell;
   - protection state transitions (while documenting deliberate threshold
     differences);
   - calibration import/export semantics where equivalent.
5. Store golden fixtures and a hash in the test evidence. A diff must be
   reviewed as either a Rust bug or a documented divergence.

Do not silently translate Speeduino code. If an implementation is adapted,
preserve GPL-2.0 provenance and identify the source file/revision in the Rust
module and commit.

## Deliberate differences

- OpenTech-ECU uses typed quantities and explicit sensor quality/age.
- Fuel and ignition evidence are separated; lambda never feeds ignition.
- Calibrations are sealed documents with SHA-256 and revision history.
- Learning gates, confidence, abstention, selective apply and rollback are
  first-class.
- The protocol is a new versioned host protocol, not TunerStudio's.
- Physical outputs are interlocked off in v0.1.

These differences should be listed in the feature matrix and must not be
described as Speeduino bugs or as verified parity.

## Licence

Speeduino firmware is GPL-2.0. The OpenTech-ECU Rust workspace is GPL-2.0-or-later
to preserve compatibility. See `NOTICE.md` and the root `LICENSE`. Hardware
derivatives from Speeduino Hardware would require the CERN-OHL-S-2.0 obligations;
no such hardware is distributed here yet.
