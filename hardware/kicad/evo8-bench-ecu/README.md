# Evo VIII bench-prototype ECU — unreviewed KiCad schematic + PCB draft

> **STATUS: UNREVIEWED BENCH SCHEMATIC + UNROUTED PCB DRAFT. NOT MANUFACTURABLE.
> NOT AUTOMOTIVE-QUALIFIED. NOT AN EVO VIII PLUG-IN. DO NOT FABRICATE OR CONNECT TO A VEHICLE.**

This package was produced in a time-boxed independent Codex hardware pass and
then imported into the repository for review. It has **not** been designed or
signed off by a qualified automotive electronics engineer. Treat every
component value, pin, net and protection choice as a hypothesis to be reviewed.

## What exists

| Artefact | Present? | Notes |
|---|---|---|
| KiCad 9 project + hierarchical schematic | Yes | Seven sheets, 305 BOM lines/components |
| Netlist (KiCad XML) | Yes | 198 nets |
| BOM | Yes | Real manufacturer part numbers; sourcing/derating not reviewed |
| Pin map | Yes | Bench connector names, **not** OEM Evo cavities |
| Test point list | Yes | Scope points for rails, MCU, injectors, triggers, CAN |
| ERC report | Yes | 0 violations reported by the generator's KiCad CLI |
| Schematic SVG | Yes | `reports/schematic-svg/` |
| PCB layout draft | Yes, **NOT USABLE** | `.kicad_pcb` with 305 footprints; 37 tracks/vias, 499 unconnected items, 14 DRC violations. Unrouted placement draft, not a manufacturable board |
| Manufacturing outputs | **No** | No gerbers, drill, assembly or fabrication drawing |
| Thermal/EMC/transient validation | **No** | Not started |
| Evo connector/pinout | **No** | OEM harness remains UNVERIFIED |
| Independent review | **No** | Required before any build |

ERC passing means the captured schematic has no KiCad electrical-rule
violations under the chosen settings. It does **not** prove the circuit is safe,
correct, manufacturable, automotive-qualified or compatible with an Evo VIII.

## Intended architecture (as captured)

- **Power:** fused 12 V bench input, reverse-polarity MOSFET, TVS, e-fuse
  (TPS259260), 5 V/3.3 V rails, reset supervisor, bench ARM pushbutton and
  hardware-inhibit path. No load-dump/automotive qualification.
- **MCU:** STM32H743VIT6, external HSE, SWD, USB, CAN transceiver, watchdog.
- **Injectors:** four low-side bench sinks (IRLZ44N TO-220) with flyback,
  clamp, 0.22 Ω shunts and test points. **No peak/hold current control, no
  injector characterisation and no vehicle injector approval.**
- **Ignition:** four logic-level outputs intended for **external, verified
  igniters**; no direct coil drive and no high-voltage section.
- **Triggers:** two simulated crank/cam inputs with comparator conditioning and
  scope points. These are not proof of the 4G63 trigger pattern.
- **Sensors:** four protected analogue channels (not a complete Evo sensor set).
- **Auxiliary:** four low-side outputs and one relay driver, bench loads only.
- **Connectors:** Phoenix-style bench terminal blocks and headers. Pin numbers
  are **not** Mitsubishi OEM cavity numbers.

## Review blockers before fabrication

1. Complete the PCB routing and fix all DRC violations; review every clearance,
   creepage, current path and thermal pad. The current draft has 499 unconnected
   items and 14 DRC violations.
2. Review the power input for reverse polarity, load dump, transient energy,
   fuse coordination, inrush and brownout.
3. Verify the TPS259260 configuration, e-fuse current limit, SOA and thermal
   design for the intended dummy loads.
4. Review injector sink SOA, flyback energy, shunt power, gate drive and
   default-off behaviour. No real injector until the driver is proven.
5. Review the ignition logic-level interface against a **verified** igniter
   input; do not assume TTL/CMOS thresholds or polarity.
6. Review CAN transceiver protection, termination, ground offsets and EMC.
7. Verify STM32H743 pin assignments against the exact package datasheet and
   alternate-function table. The pin map is not independently checked.
8. Verify all BOM parts are active, sourceable, correctly rated and derated.
9. Add a hardware watchdog and output-interlock circuit review; the firmware
   skeleton's interlock is software, not an independent safety barrier.
10. Run the unpowered and powered bench plan in
    [`AUDIT/EVO8_BENCH_TEST_PLAN.md`](../../../../AUDIT/EVO8_BENCH_TEST_PLAN.md)
    with current-limited supplies and dummy loads.

## Licence

The original hardware design files in this directory are intended to be
distributed under **CERN-OHL-S-2.0** (see `LICENSE-CERN-OHL-S-2.0.txt`). They are
not derived from the Speeduino Hardware repository. Review the licence and
provenance before manufacture or redistribution.

## Reproduction and evidence

The files were generated with a local KiCad 9 installation fetched into an
isolated workspace. The generator script is retained under `tools/` for review,
but it depends on local KiCad libraries and is not a manufacturing build system.
The ERC JSON is in `reports/erc.json`. A DRC run on the unrouted placement draft
is in `reports/drc.json`: **14 violations and 499 unconnected items**. It is not
a routable/manufacturable board and must not be fabricated.

Do not claim Gate 1 or Gate 2 from this package. See
`AUDIT/HARDWARE_STATUS.md`, `AUDIT/ELECTRICAL_DESIGN_REVIEW.md` and
`AUDIT/RELEASE_GATES.md`.
