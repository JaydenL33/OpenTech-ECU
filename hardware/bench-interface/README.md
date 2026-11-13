# Bench interface — IDC40 (J4) pin map

| Field | Value |
|---|---|
| Status | **Netlist-derived map only. No breakout has been built or continuity-checked.** |
| Source | `hardware/prototype-v1/v0.4.4d.kicad_pcb`, connector J4 (`IDC-Header_2x20_P2.54mm_Vertical`) |
| Extraction date | 2026-11-13 |
| Tool | KiCad 9.0.9 `pcbnew` Python API |
| Raw mapping | `idc40_pinout.csv` (40 pads, net names from the PCB) |
| Related | `HARDWARE_BLOCKERS.md` `HW-BENCH-01`, `AUDIT/PCB_CHECK_RESULTS.md`, `AUDIT/EVO8_BENCH_TEST_PLAN.md` |

> **Do not wire this to a vehicle.** The pin numbers below are the **PCB pad
> numbers** of J4 as defined in the footprint. They are not a verified physical
> pin-1 orientation, mating-connector view, or Evo VIII harness pinout. Before
> energising anything, buzz out every pin to the schematic/board with a
> multimeter and confirm the J4 pin-1 silkscreen marking.

## 1. What J4 provides

- 4 injector low-side outputs, doubled pins for current (1/2, 3/4, 5/7, 9/11).
- 4 logic-level ignition outputs (13–16). These are **not** coil drivers.
- 2 high-current auxiliary outputs (6, 10).
- Idle, boost, fuel-pump-related and stepper outputs on adjacent pins.
- Crank/cam conditioned signals (32, 34), VR negative inputs (28, 30).
- MAP external input (21), CLT/TPS/IAT/O2 sensor inputs (37–40).
- Analog rail VDDA (25, 26) and four ground pins (17, 19, 23, 36).
- Five spare/prototype nets (27, 29, 31, 33, 35).

**Not present on J4:** 12 V battery power, a CAN bus, a knock input, VVT, or
drive-by-wire. 12 V enters through J3 (`GND`, `12V-SW`). Do not assume a spare
pin is a 12 V or 5 V supply.

## 2. Safe first-use constraints

- Power the board from a **current-limited supply** through J3, with a fuse and
  an emergency disconnect. Do not use a vehicle battery for first power-up.
- Injector and high-current outputs must drive **dummy loads** (resistors or
  sealed test loads), never injectors or solenoids, until the bench plan is
  approved and executed.
- Ignition outputs must go to an **external, verified logic-level igniter** or a
  scope/logic analyser, never directly to a coil.
- Measure VDDA and the 5 V rail before connecting any sensor.
- Verify the crank/cam signal source and polarity before connecting a simulator.
- Keep all outputs off at reset; confirm with a scope before enabling anything.

## 3. Proposed breakout (not built)

A bench breakout should provide:

1. A keyed IDC40 mating connector with strain relief, labelled pin 1.
2. Screw terminals or 4 mm jacks for each injector/ignition/aux output, grouped
   by function and colour-coded.
3. A fused, switched 12 V input with a current-limited bench supply and an
   emergency stop that removes actuator power.
4. Dummy-load positions for four injectors and four ignition indicators.
5. Test points on GND, 12 V-SW, 5 V, VDDA, CRANK-IN, CAM-IN and each output.
6. Series resistors/current-limited probes on signal lines; no direct scope
   ground connection to a vehicle chassis.
7. A clear silkscreen/legend that mirrors `idc40_pinout.csv`; re-verify every
   pin against the board before use.

This is a specification, not a finished schematic or assembly. `HW-BENCH-01`
remains open until the breakout schematic, assembly drawing and continuity
procedure exist.

## 4. Verification status

| Check | Status |
|---|---|
| J4 pad-to-net extraction | Done from the native PCB (40 pads) |
| Schematic↔PCB net partition | PASS (514/514 common nodes) |
| Physical J4 pin-1 marking | **Not verified on hardware** (no board exists) |
| Continuity/buzz-out | **Not performed** (no board exists) |
| Mating connector part number | **Not selected** (`HW-BOM-01`) |
| Breakout schematic/assembly | **Not built** (`HW-BENCH-01`) |

A pin map is not a harness. Do not use it as an Evo VIII pinout.
