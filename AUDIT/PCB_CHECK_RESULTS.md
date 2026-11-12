# PCB Check Results — Prototype v1 (Speeduino v0.4.4d SMD)

| Field | Value |
|---|---|
| Document ID | AUD-HW-CHK-01 |
| Status | **H2 NOT PASSED.** Checks executed; exceptions open. Not fabrication-approved. |
| Date | 2026-11-13 |
| Workstream | Hardware |
| Design under test | `hardware/prototype-v1/` (electrically Speeduino v0.4.4d SMD) |
| Upstream source | `hardware/upstream-speeduino/v0.4.4d/`, commit `ee6a9242834891877744ff2137663b6ca8d79ed6` |
| Tool | KiCad 9.0.9 (`kicad/kicad@sha256:e638b79b…7729`), Docker Engine 29.9.2 |
| Reproduction | `hardware/tools/check_prototype_v1.sh` |
| Raw evidence | `hardware/prototype-v1/verification/` |

> **Read this before quoting a number.** ERC/DRC pass is evidence of design
> consistency under a particular rule set. It is **not** evidence of automotive
> electrical reliability, thermal adequacy, EMC, or Evo VIII compatibility.
> Several violations below are inherited from upstream and are intentional
> solder-jumper bridges or unused-pin annotations. Others are genuine open items.
> No rule was globally disabled and no DRC exclusion was added.

## 1. Tool and command evidence

```text
KiCad 9.0.9, release build
Image: kicad/kicad@sha256:e638b79b0321f29395a5b783e94bb9f3c73303e8da15da27b8f5cb4b67a37729
Host: Linux 6.8.0-147-generic x86_64
```

Commands (run from the repository root):

```bash
hardware/tools/check_prototype_v1.sh

# equivalent individual commands inside the container:
kicad-cli sch erc --output /work/verification/erc.rpt /work/v0.4.4d.kicad_sch
kicad-cli pcb drc --output /work/verification/drc.rpt /work/v0.4.4d.kicad_pcb
hardware/tools/verify_upstream_baseline.sh
```

Observed exit codes: ERC `0`, DRC `0`, connectivity comparison `0`.
KiCad exit code 0 means the report was written; it does **not** mean zero
violations. Counts must be read from the reports.

## 2. Summary

| Check | Result |
|---|---|
| Schematic ERC | **101 violations** (44 errors, 57 warnings), 0 unparsed crashes |
| PCB DRC | **609 violations** (238 errors, 371 warnings), **0 unconnected items** |
| Schematic ↔ PCB connectivity | **PASS** — 514/514 common nodes identical, 0 split nets |
| Board outline | Closed single outline, 100.05 × 100.05 mm |
| Mounting holes | **None** — no NPTH/M3 standoff holes; see `HW-MECH-02` |
| Copper layers | 2 (F.Cu, B.Cu) |
| Footprints | 297 |
| Tracks/vias | 1233 track objects including 178 vias |
| Zones | 4 (GND/PGND pours) |
| Nets | 185 on PCB; 190 in schematic (difference is auto-named/unused pins) |
| Gerber outputs | 9 layer files + `.gbrjob`, generated successfully |
| Drill outputs | 1 Excellon file + drill map PDF |
| Assembly outputs | position CSV, IPC-D-356 netlist, BOM CSV, schematic PDF, KiCad netlist |
| Unrouted nets | **0** (KiCad DRC "unconnected items") |

The board is **fully routed**. This is the first time the repository has a
routed, firmware-compatible, upstream-derived baseline with actual check
evidence. It is still not a fabrication release.

## 3. ERC — 101 violations and disposition

Raw report: `hardware/prototype-v1/verification/erc.rpt`.

| Category | Count | Severity | Disposition |
|---|---:|---|---|
| `global_label_dangling` | 37 | warning | **Accepted (unused I/O).** Global labels for unused MCU pins (`MCU-D12…D53`, `MCU-A6/A7`, `MISO/MOSI/SCLK`, `ADC-Clamp-7/11/12/14`) are exposed for the prototype area but not connected elsewhere. They are not used by the v0.4.4d circuit. |
| `pin_not_connected` | 26 | error | **Open / accepted with justification.** Unused Arduino shield pins (A11–A15, AREF, SDA/SCL, VIN, 3V3, ICSP 5V/GND/MISO/MOSI/RST/SCK), unused stepper module pins (M0–M2, FLT), unused switch S1 NC/NO/C contacts, unused diagnostic outputs (U7/U9 STATUS1/2), and J2 pin 6. These are genuinely unconnected; they should eventually carry no-connect flags or be documented as reserved. None is a used engine I/O. |
| `multiple_net_names` | 11 | warning | **Accepted (net aliases).** `ADC-Clamp-n` and `MCU-An` are intentionally two names for the same net at the ADC clamp/processor boundary. Netlist uses `ADC-Clamp-n`. |
| `power_pin_not_driven` | 9 | error | **Accepted (no PWR_FLAG).** Power enters through the Arduino shield, the Pololu module and the 12 V input, so several `power_in` pins have no `power_out` partner. This is a KiCad annotation issue, not an open circuit; the DRC reports 0 unconnected items. Adding PWR_FLAGs is a future schematic-quality fix, not an electrical redesign. |
| `pin_not_driven` | 9 | error | **Accepted (unused inputs).** A1 M0–M2, U6 I7/I11/I12/I14 (unused ADC mux inputs), XA1 AREF and MISO. No engine-critical input is undriven. |
| `footprint_link_issues` | 4 | warning | **Open (library packaging).** Upstream references custom libraries `Shields`, `Misc`, `Connector_Automotive` and a hand-solder 0805 footprint variant that are not present in the upstream tree. Footprints are embedded in the PCB, so manufacturing is unaffected; library links need repair for a clean H2. |
| `unconnected_wire_endpoint` | 2 | warning | **Open (schematic hygiene).** Two wire stubs at root-sheet `(193.04, 68.58)` and `(193.04, 78.74)` mm, 8.89 mm long. They do not appear as PCB nets. Remove or terminate them before H3. |
| `lib_symbol_issues` | 2 | warning | **Open (legacy symbol name).** `Device:CP1` is a KiCad 5 symbol name that no longer exists in the KiCad 9 library; the symbols are embedded and render correctly. Update the library link or accept as a legacy exception. |
| `no_connect_dangling` | 1 | warning | **Open (cosmetic).** One no-connect flag at `(213.36, 55.88)` mm is not attached to a pin. Remove or re-place it. |

**ERC conclusion:** no evidence of an open engine-critical net. All engine I/O
nets are present in both schematic and PCB; see §4. The 44 errors are annotation
and unused-pin issues, not proof of a functional fault, but they must be reduced
or individually justified before H2 can be called complete.

## 4. Schematic ↔ PCB connectivity — PASS

The schematic netlist and PCB pad connectivity were compared as net partitions,
not by net name (eeschema uses pin names, pcbnew uses pad numbers).

```text
schematic nodes: 520
pcb nodes:       514
common nodes:    514
schematic nets split on pcb: 0
pcb nets split on schematic: 0
nodes only in schematic: 6
nodes only in pcb:       0
RESULT: PASS
```

The 6 schematic-only nodes are `XA1.5V2`, `XA1.GND4`, `XA1.MISO`, `XA1.MOSI`,
`XA1.RST2`, `XA1.SCK` — the Arduino ICSP header pins, which the
`Arduino_Mega2560_Shield_IO_Only` PCB footprint does not populate. This is
expected and consistent with the unused-pin ERC findings.

Raw evidence: `hardware/prototype-v1/verification/connectivity.txt`.

## 5. DRC — 609 violations and disposition

Raw report: `hardware/prototype-v1/verification/drc.rpt`.

The zones were re-filled with the KiCad 9 filler because legacy fills cannot be
used by KiCad 8+. This removed 155 legacy `clearance` violations (zone fill
0.2488–0.2494 mm vs a 0.25 mm rule) and reduced starved thermals from 6 to 4.
The netlist partition did not change. The resulting DRC uses the upstream
project's `v0.4.4d.kicad_pro` board settings (for example
`min_via_annular_width = 0.05 mm`); the legacy board-local setup used 0.10 mm.
This is a **rule-set migration to the upstream project file**, not a rule
suppression. It is recorded here so a reviewer can reproduce both numbers:
766 before refill, 609 after.

| Category | Count | Severity | Disposition |
|---|---:|---|---|
| `courtyards_overlap` | 199 | error | **Open (mechanical review).** Dense connector/module placement, e.g. J1/J5 and the Arduino shield area. Courtyard overlap does not by itself prove a physical clash, but every overlap must be reviewed against the assembly drawing. Not cleared. |
| `silk_overlap` | 199 | warning | **Open (cosmetic).** Reference designators and outlines overlap on the dense shield/module area. Does not affect copper. Fix before fabrication for assembly legibility. |
| `lib_footprint_mismatch` | 143 | warning | **Accepted (library vintage).** Embedded KiCad 5 footprints differ from the current KiCad 9 library copies. The embedded footprints are the manufacturing authority. A footprint-level audit is still required. |
| `solder_mask_bridge` | 19 | error | **Accepted (intentional jumpers).** All 19 are JP6–JP11, the solder-configurable jumpers whose footprint deliberately bridges pads. Mask openings span the bridge by design. |
| `silk_over_copper` | 19 | warning | **Open (cosmetic).** Silkscreen clipped by mask/copper, mostly U5 and the shield outline. No copper effect. |
| `shorting_items` | 12 | error | **Accepted (intentional jumpers).** All 12 are the copper bridge polygons of `SolderJumper_2_Bridged` / `Jumper_3_Bridged12` on JP6–JP11. The bridge is the intended default configuration. This is a footprint-vs-DRC modelling issue, not an accidental short. |
| `silk_edge_clearance` | 5 | warning | **Open (cosmetic).** Silkscreen clipped by the 100 mm board edge, mostly the Arduino shield outline. |
| `starved_thermal` | 4 | error | **Open (assembly/thermal).** GND pads C5.2, C10.2, C11.2 and U4.3 have only one thermal spoke to the GND pour (rule minimum 2). They are still connected, but solderability/current sharing is not ideal. Re-fill with adjusted thermal settings or add spokes; re-check. |
| `lib_footprint_issues` | 4 | warning | **Open (library packaging).** Missing `Misc`/`Shields` library tables and one hand-solder footprint variant. Embedded footprints are intact. |
| `padstack` | 2 | error | **Accepted (mechanical).** U5 (MPX4250) has two 4.1 mm mechanical mounting tabs modelled as 4.1 mm PTH pads, i.e. zero annular copper. They are screw holes, not electrical pads. A future footprint cleanup should make them NPTH. |
| `annular_width` | 2 | error | **Same U5 mechanical tabs.** `actual -0.0044 mm` annular width is the consequence of pad size = drill size. No electrical function. |
| `track_dangling` | 1 | warning | **Open (review).** A 0.552 mm VDDA track stub at `(162.157, 132.991)` mm has an unconnected end. DRC still reports 0 unconnected *items*, so this is likely a redundant stub; confirm and remove or terminate. |

**DRC conclusion:** the board has **no unrouted nets**. The only "shorting"
violations are intentional solder-jumper bridges. The remaining errors are
mechanical/courtyard, thermal-spoke and two mechanical mounting-tab issues.
H2 is **not** passed until the courtyard overlaps, starved thermals, VDDA stub
and library-link warnings are resolved or formally reviewed.

## 6. Manufacturing export verification

Generated from the native PCB/schematic by
`hardware/tools/check_prototype_v1.sh`:

| Output | File(s) | Result |
|---|---|---|
| Gerbers | 9 files: F/B copper, F/B paste, F/B silkscreen, F/B mask, Edge.Cuts | Generated; `.gbrjob` written |
| Drill | `v0.4.4d.drl` (Excellon, mm) + drill map PDF | Generated |
| Position | `v0.4.4d-pos.csv` (mm, drill origin) | 291 component placements |
| Bare-board net test | `v0.4.4d-ipc-d-356.net` | Generated |
| BOM | `v0.4.4d-bom.csv` | 47 grouped rows; 11 rows lack MPN (blocker HW-BOM-01) |
| Schematic PDF | `v0.4.4d-schematic.pdf` | Generated from native `.kicad_sch` |
| Netlist | `v0.4.4d.net` | Generated; used for connectivity check |

The gerber/drill files are **review artefacts only**. They have not been sent to
a fabricator, DFM-checked by a board house, or manufactured.

## 7. What is still required for H2/H3

1. Fix or formally document the 199 courtyard overlaps with a mechanical
   assembly drawing; do not assume they are harmless.
2. Resolve the 4 starved thermals and the VDDA track stub.
3. Add proper no-connect flags/PWR_FLAGs, or explicitly accept each unused-pin
   ERC error in a reviewed exception list.
4. Repair the 4 missing library links and the 2 `Device:CP1` symbol links.
5. Re-run this script and attach the new raw reports.
6. Independent hardware review (H3/H4). This document cannot approve
   fabrication.
