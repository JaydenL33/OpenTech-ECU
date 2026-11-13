# Self-audit guide — OpenTech-ECU prototype-v1

You do not need to be a PCB designer to audit this board. You need to be
systematic, cross-check the schematic and PCB, and record what you actually
verified. This guide is for `hardware/prototype-v1/`, not the legacy
`hardware/kicad/evo8-bench-ecu/` draft. It has been corrected against
`AUDIT/HARDWARE_NEXT_STEPS_REVIEW.md`.

## 0. What “audit” means here

An audit is not “the DRC passed.” It is a recorded check that:

1. the right files and revision are being reviewed;
2. every functional block is present and connected as intended;
3. every exception is understood and accepted or rejected;
4. the BOM/assembly matches the schematic;
5. the board can be powered and tested without an obvious hazard;
6. a human reviewer has seen the evidence and signed a decision.

Record the date, Git commit, KiCad/Docker version, file hashes, and what you
checked. “Looks fine” is not evidence.

## 1. Tools and exact revision

From the repository root:

```bash
git rev-parse HEAD
hardware/tools/check_prototype_v1.sh
hardware/tools/verify_upstream_baseline.sh
git status --short
```

The check script uses the pinned KiCad 9.0.9 Docker image and **rewrites tracked
files** in `verification/` and `manufacturing/`. After it runs, inspect
`git status` / `git diff --stat`; any non-timestamp change in Gerbers, drill,
BOM or position files is a finding. Docker and the ~2 GB image are required.

Read:

- `verification/summary.txt` — counts, unconnected items, manufacturing files
- `verification/erc.rpt` — schematic electrical rules
- `verification/drc.rpt` — PCB design rules (reported violations)
- `verification/drc-with-exclusions.rpt` — the 199 excluded courtyard markers
- `verification/drc_exclusions.csv` — one justification per exclusion
- `verification/connectivity.txt` — schematic vs PCB node comparison
- `verification/mounting-holes.txt` — hole clearance check

Then read `AUDIT/PCB_CHECK_RESULTS.md` and `AUDIT/HARDWARE_DIFF.md`.

## 2. Open the right KiCad project

Open `hardware/prototype-v1/v0.4.4d.kicad_pro`. The root schematic is
`v0.4.4d.kicad_sch`; the PCB is `v0.4.4d.kicad_pcb`.

If you see “DO NOT FABRICATE: UNROUTED”, you opened the legacy
`hardware/kicad/evo8-bench-ecu/` project. Close it.

## 3. Schematic audit, sheet by sheet

Use `Ctrl+F` to find a reference or net. Click a wire and press the backtick
(`` ` ``) hotkey to highlight the net (menu location varies by KiCad version).
Use the cross-probe tool to select a component in the schematic and see it on
the PCB. Press `E` to inspect a symbol/footprint.

### Power and protection (power sheet)
- 12 V input path J3: **there is no fuse and no reverse-polarity protection on
  `12V-SW`**. F1/F2 are 5 V-side polyfuses; D14/D15 protect only the regulator
  input. Use an external inline fuse, a keyed/polarised lead and a
  current-limited supply on the bench.
- 5 V regulator U8 (LM2940), decoupling, regulator input/output nets.
- `Vdrive` selection JP1: pin 1 = 5 V, pin 3 = 12 V-SW, pin 2 = Vdrive.
  **JP1 must be fitted**; open leaves the gate drivers unpowered.
- GND/PGND partition and the single connection point; do not assume all ground
  symbols are one net.

### Inputs
- Crank/cam conditioning, VR/Hall jumpers JP2/JP3, pull-up jumpers JP4/JP5.
- MAP sensor U5, TPS, CLT, IAT, wideband input.
- ADC clamp/protection U6; the four spare channels are U6 pins 7, 12, 13, 15.
- Every input has a ground reference and a defined range.

### Fuel and ignition
- **Injectors:** U1 and U3 (two dual-channel VNLD5090) drive INJ-1..4. U7/U9
  drive BOOST/IDLE/HC-1/HC-2, **not injectors**.
- **Injector clamp:** D5/D6/D11/D12 are indicator-LED series diodes, not
  flyback diodes. Turn-off energy is absorbed by the VNLD5090 internal clamp.
  The aux outputs have external freewheel diodes D16–D19.
- **Ignition:** U2/U4 IXDN602 gate drivers, 10 Ω series resistors
  R13/R14/R27/R28, outputs IGN-1..4. **No IGBTs, no coil primary path, no coil
  clamp.** The 10 Ω resistor is the only current limit; never connect a coil
  primary. Read `COIL_INJECTOR_OPTIONS.md`.
- Dwell is set by the Mega/Speeduino firmware, not the PCB.
- Low-impedance injectors should not be connected directly; see the coil/
  injector note.

### Outputs and connector
- J4 IDC40: **IGN-3 = pin 16, IGN-4 = pin 15**. Injectors are interleaved with
  aux outputs (INJ-3 = 5/7, INJ-4 = 9/11; pin 6 HC-2, 8 IDLE, 10 HC-1).
- The schematic symbol is counter-clockwise; the footprint is normal odd/even
  IDC. **Audit by pin number, not drawing position.**
- **J4 has no +12 V pin.** Only four GND pins (17/19/23/36), shared with sensor
  returns; do not put high-current returns through the ribbon.
- **J4.21 MAP-EXT is hard-wired to U5** through JP11 (default bridged). For an
  external MAP, cut JP11 A–C and bridge B–C, **or do not fit U5**. Otherwise
  the two outputs fight.
- JP6–JP11 default bridges; JP11 must be changed before external MAP.

### Mechanical
- H1–H3 3.2 mm NPTH mounting holes; no top-right hole.
- Header stack/standoff height; J3 strain relief.
- 100.05 × 100.05 mm outline; confirm the board-house quote uses that size.

## 4. PCB audit

- Confirm the board is **2-layer** (F.Cu/B.Cu).
- Check the GND/PGND pours and return paths. The injector/aux currents and the
  four shared IDC GND pins are a real review item.
- Inspect the 0.25/0.3/0.4/0.5/0.75/1.0 mm tracks; the 1.0 mm tracks carry
  higher current. The project has not done a thermal calculation.
- Check the 0.05 mm minimum via annular ring against the chosen fab; it is tight
  for low-cost processes.
- Inspect the 199 courtyard exclusions in `drc_exclusions.csv`. The three
  header-to-header pairs depend on the chosen header MPN.
- Check the 31 remaining DRC errors (JP6–JP11 jumpers) and the 4 ERC errors
  (U6 spare channels). Confirm they are unused/aux, not engine I/O.
- **143 `lib_footprint_mismatch` warnings remain**: the footprint-library audit
  is not done. This is an H3 item.
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
Re-run `generate.py` after any BOM change.

## 6. Bench audit

Before power:

1. Inspect the bare board under magnification for bridges, missing pads and
   damaged silkscreen.
2. Continuity in diode mode: each VNLD output can read a diode drop from
   GND→OUT (body diode) and open from OUT→GND — **that is normal, not a short**.
   `12V-SW`→GND will not read open because of LED strings/DRV8825; record the
   values rather than expecting infinity.
3. Confirm no jumper is fitted incorrectly; JP1 must be fitted and set to the
   intended 5 V/12 V position.
4. Power from a **current-limited supply with an external fuse and a keyed
   lead**, initially at low current. Reverse polarity is not protected on v1.
5. Measure **VDD (5 V), VDDA (after F1), Vdrive (JP1.2)**. The shield has no
   3.3 V rail; the Mega's 3.3 V is the Mega's.
6. Dummy loads first: injector dummy = resistor matching the injector
   resistance and power rating to an external +12 V (or a real high-Z injector
   with no fuel); IGN dummy = 1 kΩ to GND. **Never a coil primary.**
7. Scope the outputs; verify polarity, idle-low at boot/Mega reset, pulse width
   and dwell. Then one known smart coil/test injector, one channel at a time.

Stop conditions: any smell, smoke, hot component, supply current limit, or
unexpected output. Disconnect power and record the fault.

## 7. Independent review

AI review is not a substitute for a human EE. Your brother declining means the
project’s H4 gate is not satisfied. Options:

- a freelance automotive/electronics engineer with PCB review experience;
- a local makerspace or university electronics lab;
- an online PCB review service (ask for a written findings list);
- a motorsport wiring/ECU shop with an electronics engineer.

Give them `docs/START_HERE_HARDWARE.md`, this guide, `COIL_INJECTOR_OPTIONS.md`,
the KiCad project, verification reports, `HARDWARE_BLOCKERS.md`,
`FABRICATION_NOTES.md` and the assembly pack. Ask for a written pass/fail with
findings.

Questions a human EE must answer include:

1. Is the VNLD5090 internal clamp energy adequate for the chosen injectors at
   maximum RPM?
2. Is the IDC40/ribbon acceptable for injector current and the four shared GND
   returns, and which header/cable part numbers?
3. Is the 10 Ω gate-driver output within the chosen smart coil's input ratings,
   with no back-drive risk?
4. Is an external fuse acceptable instead of on-board 12 V fusing, at what
   rating?
5. Do they accept the ERC exceptions and 199 courtyard exclusions, including
   the three header pairs once the header is chosen?
6. Are the 1.0 mm tracks adequate at 1 oz copper for the bench currents?
7. Is the 0.05 mm via annular ring acceptable to the fab?
8. Which MAP sensor option is approved?
9. Do the key power/connector/polarity footprints match the chosen datasheets?
10. A signed pass/fail for H4.

## 8. Minimum self-audit record

Write down the commit/file hashes, check-script exit codes and counts, each
accepted ERC/DRC exception, BOM/connector choices and EOL alternatives,
mechanical/standoff decisions, bench measurements (supply current, rail
voltages, output waveforms), and anything you could not verify.
