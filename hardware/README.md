# Hardware

> **Start here:** [`docs/START_HERE_HARDWARE.md`](../docs/START_HERE_HARDWARE.md).
> The selected project is `prototype-v1/v0.4.4d.kicad_pro`. The older
> `kicad/evo8-bench-ecu/` board is an **unrouted legacy draft** whose title block
> says “DO NOT FABRICATE: UNROUTED”; it is not the prototype.

There is **no manufacturing-approved OpenTech-ECU board** in this repository.
The current candidate is a **bench prototype**, not a drop-in Mitsubishi Evo VIII
ECU.

## Selected prototype baseline (H0 hardware reference)

The primary hardware path is now a **Speeduino v0.4.4d SMD** design with a
documented minimal PCB revision (R-01…R-03; no net or topology change):

| Path | Contents |
|---|---|
| `upstream-speeduino/v0.4.4d/` | Byte-exact upstream source, commit `ee6a9242834891877744ff2137663b6ca8d79ed6`, CERN-OHL-S-2.0 |
| `prototype-v1/` | Native KiCad 9 review copy, manufacturing exports and verification evidence |
| `bench-interface/` | Netlist-derived IDC40 pin map and safe-breakout specification (not a built fixture) |
| `tools/check_prototype_v1.sh` | Reproducible ERC/DRC/export/connectivity run |
| `tools/apply_v1_minimal_revision.py` | Idempotent R-01…R-03 PCB revision script (KiCad container) |
| `tools/verify_upstream_baseline.sh` | SHA-256 verification of the preserved upstream tree |

The v0.4.4d board is a 100 × 100 mm, 2-layer Arduino Mega 2560 shield with four
injector and four ignition channels, an IDC40 interface and screw terminals. It
is fully routed (KiCad DRC: 0 unconnected items). It is **not** fabrication
approved: ERC/DRC exceptions, missing BOM MPNs and the bench breakout package
remain open. See:

- [`AUDIT/UPSTREAM_PROVENANCE.md`](../AUDIT/UPSTREAM_PROVENANCE.md)
- [`AUDIT/HARDWARE_BASELINE.md`](../AUDIT/HARDWARE_BASELINE.md)
- [`AUDIT/PCB_CHECK_RESULTS.md`](../AUDIT/PCB_CHECK_RESULTS.md)
- [`HARDWARE_BLOCKERS.md`](../HARDWARE_BLOCKERS.md)

Reproduce the checks:

```bash
hardware/tools/check_prototype_v1.sh
hardware/tools/verify_upstream_baseline.sh
```

The selected board is the simplest firmware-compatible baseline: it matches the
upstream Speeduino `megaatmega2560` firmware target. It is not an Evo VIII
plug-in, has no verified vehicle connector/pinout, and must not be connected to a
vehicle. The vehicle interface remains blocked on the measurements in
[`AUDIT/ECU_PINOUT_VERIFICATION.md`](../AUDIT/ECU_PINOUT_VERIFICATION.md).

## Design intent for the bench prototype

- protected 12 V bench input, fuses, reverse-polarity protection, transient
  suppression, current-limited bring-up;
- four injector channels into **dummy loads**, with flyback/test points;
- four logic-level ignition outputs for an **external, verified igniter** — never
  direct coil drive;
- protected simulated crank/cam inputs, analogue sensor inputs and auxiliary
  low-side outputs;
- labelled IDC40 breakout, emergency output isolation, scope points;
- USB/serial communications; optional CAN interface on the expansion header.

No power has been applied and no output has been actuated.

## Legacy custom board (not the baseline)

`hardware/kicad/evo8-bench-ecu/` contains an **UNREVIEWED** STM32H743
hierarchical KiCad bench schematic, netlist, BOM, pin map, test-point list and
ERC report from a time-boxed independent Codex pass. It has an **unrouted PCB
placement draft** (KiCad 9 DRC re-check on 2026-11-13: 499 unconnected items,
14 violations), **no manufacturing outputs and no independent review**. It is
retained as a separate exploratory design, not as the prototype-v1 baseline.
ERC passing is not electrical or functional safety. See its README before
touching it.

The Evo VIII connector/pinout is UNVERIFIED. Do not build a vehicle adapter
harness until the measurements in
[`AUDIT/ECU_PINOUT_VERIFICATION.md`](../AUDIT/ECU_PINOUT_VERIFICATION.md) are
complete. See [`AUDIT/HARDWARE_STATUS.md`](../AUDIT/HARDWARE_STATUS.md),
[`AUDIT/EVO8_BENCH_TEST_PLAN.md`](../AUDIT/EVO8_BENCH_TEST_PLAN.md) and
[`AUDIT/ELECTRICAL_DESIGN_REVIEW.md`](../AUDIT/ELECTRICAL_DESIGN_REVIEW.md).
