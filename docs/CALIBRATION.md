# Calibration workflow

This document describes how a calibration moves through OpenTech-ECU v0.1. It
does not declare any calibration safe.

## 1. Create a profile

`Engine setup` (or `WizardRequest` in Rust) collects:

- engine name, displacement, cylinders, firing order, compression ratio;
- aspiration, rev limit, idle target;
- injector flow at a rated pressure, dead time, pressure;
- fuel stoichiometry and density;
- trigger configuration and sensor assignment (future/verified values only);
- optional licensed ignition reference.

The wizard generates VE, target-lambda, ignition and dwell tables. VE and target
lambda are explicitly marked `engineering_estimate`. The ignition table is marked
`synthetic_simulation_only` unless a verified engine-specific reference is
supplied. Every document is sealed with SHA-256 over canonical JSON.

**All generated fuel maps are unvalidated starting estimates.** The Evo VIII
profile is a simulation-only template; its trigger, injector, ignition and
pinout data are UNVERIFIED.

## 2. Validate before running

Preflight blocks virtual operation on invalid checksums, invalid axes, missing
required tables, implausible targets, unsupported trigger configuration or
failed engine/fuel validation. It reports physical blocking separately:

- physical export is always blocked in v0.1 by the interlock;
- ignition without a verified reference is always physically blocked;
- unverified sensors/injectors, timing-light checks and I/O conflicts are
  reported as explicit checks.

Passing preflight means the virtual configuration is coherent, not engine-safe.

## 3. Run and record

The deterministic simulator records telemetry at 50 Hz, including calibration
checksum, seed, scenario, sensor status, trims, fault flags, lambda age and
exhaust-delay metadata. Use a guided warm sweep for steady-state fuel coverage.
Logs can be exported to CSV; the full JSON includes events and freeze frames.

## 4. Analyze — not auto-apply

The offline Mixture Map:

1. time-aligns lambda to the fuel command that produced it;
2. applies shared learning gates (warm coolant, sequential sync, valid wideband,
   no fuel cut/overrun/misfire/exhaust leak/knock/overboost/critical DTC,
   plausible fuel/oil pressure, injector duty, steady RPM/MAP/TPS);
3. attributes accepted samples to the nearest VE cell;
4. computes sample count, dwell, lambda mean/spread and confidence;
5. accounts for STFT/LTFT to produce a pre-trim equivalent;
6. bounds the cell change (default 5% per apply, 20% lifetime);
7. abstains on insufficient coverage, high variance, hard faults or a suspected
   stuck wideband;
8. emits a proposal with a preview/diff. **Nothing is written yet.**

WOT/high-load cells are excluded unless the operator explicitly opts in.

## 5. Review and apply

The operator selects cells and approves in the Tuning Studio. The service:

- re-verifies the base calibration checksum and revision (stale proposals fail);
- rejects unknown/duplicate cell selections;
- applies only selected cells to a child revision;
- updates provenance notes and seals a new checksum;
- retains the prior revision in memory for rollback.

There is no path from the LLM copilot or Mixture Map to physical injector
outputs. Applying a revision only changes calibration data for the virtual ECU.

## 6. Map editing and axis resampling

The 2D grid supports keyboard navigation, copy/paste, local undo/redo, a live
operating-cell outline and a 3D surface. Axis resampling is an editing aid:
linear interpolation changes the grid, not the evidence. VE and target-lambda
axes are resampled together to preserve cell alignment.

## 7. Long-term learning

LTFT is cell/bank-aware and learns toward an active, gated STFT after sustained
dwell. It is separate from base VE and from STFT. Merging learned cells into a
base map is an explicit review action; it is not performed automatically during
live simulation. Bank 2 requires its own sensor and controller.

## 8. Rollback and audit

Each revision records parent, revision number, checksum and creation time.
In-memory history is currently not durable across service restart; durable
signed storage is planned. Export the calibration JSON before a service restart
if the revision must be retained.

## 9. What is missing

- vehicle-specific verified trigger/ignition/injector/sensor data;
- hardware validation and durable calibration storage;
- staged injection and full per-cylinder bank assignments;
- Evo OEM connector/pinout verification;
- independent review and bench evidence.

Do not treat a successful virtual sweep or a green preflight as an engine-safe
calibration.
