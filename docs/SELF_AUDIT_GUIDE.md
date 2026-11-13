# Self-audit guide — OpenTech-ECU prototype-v1

You do not need to be a PCB designer to audit this board. You need to be
systematic, keep the schematic and PCB cross-checked, and write down what you
actually verified. This guide is for the selected board in
`hardware/prototype-v1/`, not the legacy `hardware/kicad/evo8-bench-ecu/` draft.

## 0. What “audit” means here

An audit is not “the DRC passed.” It is a recorded check that:

1. the right files and revision are being reviewed;
2. every functional block is present and connected as intended;
3. every exception is understood and accepted or rejected;
4. the BOM/assembly matches the schematic;
5. the board can be powered and tested without an obvious hazard;
6. a human reviewer has seen the evidence and signed a decision.

Record the date, the Git commit, the KiCad/Docker version, the file hashes, and
what you checked. “Looks fine” is not evidence.

## 1. Get the tools and the exact revision

From the repository root:

```bash
git rev-parse HEAD
hardware/tools/check_prototype_v1.sh
hardware/tools/verify_upstream_baseline.sh
```

The check script uses the pinned KiCad 9.0.9 Docker image and writes reports to
`hardware/prototype-v1/verification/`. Read:

- `verification/summary.txt` — counts, unconnected items, manufacturing files
- `verification/erc.rpt` — schematic electrical rules
- `verification/drc.rpt` — PCB design rules (reported violations)
- `verification/drc-with-exclusions.rpt` — the 199 excluded courtyard markers
- `verification/drc_exclusions.csv` — one justification per exclusion
- `verification/connectivity.txt` — schematic vs PCB node comparison
- `verification/mounting-holes.txt` — hole clearance check

Then read `AUDIT/PCB_CHECK_RESULTS.md` and `AUDIT/HARDWARE_DIFF.md`. The reports
are only useful with the written dispositions.

## 2. Open the right KiCad project

Open:

```text
hardware/prototype-v1/v0.4.4d.kicad_pro
```

The root schematic is `v0.4.4d.kicad_sch`; it has six hierarchical sheets. The
PCB is `v0.4.4d.kicad_pcb`.

If you see “DO NOT FABRICATE: UNROUTED”, you have opened
`hardware/kicad/evo8-bench-ecu/`. Close it.

## 3. Schematic audit, sheet by sheet

Use `Ctrl+F` to find a reference or net. Click a wire and use **Highlight Net**
(right-click) to follow it. Use the cross-probe tool to select a component in
the schematic and see it highlighted on the PCB. Press `E` on a selected symbol
or footprint to inspect its properties and footprint assignment.

Check each block:

### Power and protection (power sheet)
- 12 V input path, fuse F1/F2, reverse-polarity diode D14, TVS D15.
- 5 V regulator U8 (LM2940), decoupling, regulator input/output nets.
- `Vdrive` selection JP1: pin 1 = 5 V, pin 3 = 12 V-SW, pin 2 = Vdrive.
  Confirm which position your build uses and why.
- GND/PGND partition and the single connection point; do not assume all ground
  symbols are one net without checking.

### Inputs
- Crank/cam conditioning, VR/Hall jumpers JP2/JP3, pull-up jumpers JP4/JP5.
- MAP sensor U5, TPS, CLT, IAT, wideband input.
- ADC clamp/protection U6; check the four spare channels are unused.
- Every input has a ground reference and a defined range.

### Fuel and ignition
- Four injector channels: U1/U3/U7/U9 VNLD5090 low-side drivers, D5/D6/D11/D12
  flyback diodes, outputs INJ-1..4 to J4.
- Four ignition outputs: U2/U4 IXDN602 gate drivers, 10 Ω series resistors
  R13/R14/R27/R28, outputs IGN-1..4 to J4.
- **There are no IGBTs or coil primary drivers on this board.** The ignition
  outputs are gate-driver / logic-level outputs for an external igniter or a
  smart coil. Read `COIL_INJECTOR_OPTIONS.md`.
- Dwell is set by the Mega/Speeduino firmware, not by this PCB.

### Outputs and connector
- J4 IDC40 pinout: check every pin against
  `hardware/bench-interface/idc40_pinout.csv` and the netlist.
- J3 12 V input, J1/J5/J6/J8 auxiliary headers.
- JP6–JP11 solder jumpers: default bridges; JP11 must be cut before external
  MAP is connected to IDC40 pin 21.
- Check connector pin 1/orientation against the assembly drawing.

### Mechanical
- H1–H3 3.2 mm NPTH mounting holes; no top-right hole.
- Header stack/standoff height; J3 strain relief.
- 100.05 × 100.05 mm outline; confirm the board-house quote uses that size.

## 4. PCB audit

- Inspect board outline, four copper layers? No: exactly **2 layers** F.Cu/B.Cu.
- Check the GND/PGND pours and the return paths for the injector/ignition
  drivers; high-current returns should not share a thin signal return.
- Inspect the 0.25/0.3/0.4/0.5/0.75/1.0 mm tracks; the 1.0 mm tracks carry
  higher current. Confirm the expected current and temperature rise with the
  board house or an EE; the project has not done a thermal calculation.
- Check the 0.05 mm minimum via annular ring against the chosen fab. It is tight
  for low-cost processes.
- Inspect the 199 DRC exclusions in `drc_exclusions.csv`; they are all
  courtyard overlaps. The three header-to-header pairs depend on the chosen
  header MPN. Do not accept them without that MPN.
- Check the 31 remaining DRC errors: they are the JP6–JP11 solder jumpers. Verify
  each is intentional and matches the netlist.
- Check the four ERC errors: U6 spare SP720 channels. Confirm they are unused,
  not engine I/O.
- Check test points, connector orientation, and mounting-hole clearance.

## 5. BOM and assembly audit

- `hardware/prototype-v1/manufacturing/assembly/v0.4.4d-bom.csv` — 47 grouped
  rows; all populated rows need a real MPN.
- `hardware/prototype-v1/manufacturing/jlcpcb/bom-jlcpcb.csv` — 139 machine
  placements; 9 connector/socket rows are unresolved.
- `cpl-jlcpcb.csv` — 139 placements; every origin inside the outline.
- `bom-hand-solder.csv` — XA1 shield headers and A1 carrier sockets.
- `bare-copper-jumpers.csv` — JP6–JP11, no parts.
- `docs/HARDWARE_COST.md` — cost estimate and EOL/stock blockers.

Do not upload the assembly BOM until the nine connector/socket MPNs are chosen
and the EOL/zero-stock parts (especially MPX4250AP) are resolved or consigned.
The J3 CSV quoting defect is fixed; re-run `generate.py` after any BOM change.

## 6. Bench audit

Before power:

1. Inspect the bare board under magnification for bridges, missing pads, and
   damaged silkscreen.
2. Continuity: 12 V to GND, 5 V to GND, each injector output to GND (no short),
   each ignition output to GND (no short).
3. Confirm no jumper is fitted in the wrong position; JP1 must match the
   intended 5 V/12 V drive.
4. Power from a current-limited supply, initially at low current, with fuses.
5. Measure 5 V, 3.3 V (if present), and the regulator temperatures.
6. Use dummy injector/ignition loads, not real coils/injectors, for the first
   output tests. Scope the outputs; verify dwell/pulse width and polarity.
7. Only then consider a known smart coil/test injector, one channel at a time.

Stop conditions: any smell, smoke, hot component, supply current limit, or
unexpected output. Disconnect power and record the fault.

## 7. Independent review

AI review is not a substitute for a human EE. Your brother declining means the
project’s H4 gate is not satisfied. Options:

- a freelance automotive/electronics engineer with PCB review experience;
- a local makerspace or university electronics lab;
- an online PCB review service (ask for a written findings list, not a chat);
- a motorsport wiring/ECU shop with an electronics engineer.

Give them `docs/START_HERE_HARDWARE.md`, this guide, the KiCad project, the
verification reports, `HARDWARE_BLOCKERS.md`, `FABRICATION_NOTES.md` and the
assembly pack. Ask for a written pass/fail with findings, not a verbal “looks
fine.”

## 8. Minimum self-audit record

Write down:

- commit hash and file hashes;
- check-script exit codes and counts;
- each ERC/DRC exception you accept and why;
- BOM/connector choices and EOL alternatives;
- mechanical/standoff decisions;
- bench measurements (supply current, rail voltages, output waveforms);
- anything you could not verify, and who must verify it.
