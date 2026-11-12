# Hardware

There is **no manufacturing-ready OpenTech-ECU board** in this repository yet.

The first hardware target is a **bench prototype**, not a drop-in Mitsubishi Evo
VIII ECU:

- protected 12 V bench input, fuses, reverse-polarity protection, transient
  suppression, current-limited bring-up;
- STM32H7-class MCU with SWD/USB, CAN, watchdog and recovery;
- four injector low-side drivers into **dummy loads**, with flyback/test points;
- four logic-level ignition outputs for an **external, verified igniter** — never
  direct coil drive;
- protected simulated crank/cam inputs, analogue sensor inputs and auxiliary
  low-side outputs;
- labelled screw terminals/breakout, emergency output isolation, scope points.

The actual Evo VIII connector/pinout is UNVERIFIED. Do not build a vehicle
adapter harness until the measurements in
[`AUDIT/ECU_PINOUT_VERIFICATION.md`](../AUDIT/ECU_PINOUT_VERIFICATION.md) are
complete. See [`AUDIT/HARDWARE_STATUS.md`](../AUDIT/HARDWARE_STATUS.md),
[`AUDIT/EVO8_BENCH_TEST_PLAN.md`](../AUDIT/EVO8_BENCH_TEST_PLAN.md) and
[`AUDIT/ELECTRICAL_DESIGN_REVIEW.md`](../AUDIT/ELECTRICAL_DESIGN_REVIEW.md).

A separate, time-boxed KiCad attempt was delegated to an agent in an isolated
workspace. If any KiCad files from that attempt are added here, they must be
marked **UNREVIEWED** until ERC/DRC, netlist review, component sourcing and
independent electrical review are recorded. A KiCad file is not a safe circuit.
