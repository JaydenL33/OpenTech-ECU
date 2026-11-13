# PCB Check Results — Prototype v1 (Speeduino v0.4.4d SMD)

| Field | Value |
|---|---|
| Document ID | AUD-HW-CHK-01 |
| Status | **H2 NOT PASSED.** Checks re-run after the R-01…R-03 minimal revision; remaining exceptions open. Not fabrication-approved. |
| Date | 2026-11-13 (revision run) |
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
>
> **Revision note.** After the initial conversion, the prototype-v1 PCB received
> three documented fixes (`AUDIT/HARDWARE_DIFF.md` R-01…R-03): four starved
> thermal pads made solid, a duplicate VDDA track stub removed, and two U5
> mounting tabs converted to NPTH. DRC dropped from **609 (238 errors)** to
> **600 (230 errors)** with no new violation category and connectivity still
> PASS. ERC is unchanged at 101 because none of the changes touches the
> schematic.

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
| PCB DRC | **600 violations** (230 errors, 370 warnings), **0 unconnected items** |
| Schematic ↔ PCB connectivity | **PASS** — 514/514 common nodes identical, 0 split nets |
| Board outline | Closed single outline, 100.05 × 100.05 mm |
| Mounting holes | **None** — no NPTH/M3 standoff holes; see `HW-MECH-02` |
| Copper layers | 2 (F.Cu, B.Cu) |
| Footprints | 297 (147 component footprints + 150 unannotated proto-area pads `REF**`, no net) |
| Footprint assignment | 147/147 schematic components have a non-empty footprint; 0 unassigned |
| Tracks/vias | 1233 track objects including 178 vias |
| Zones | 4 (GND/PGND pours) |
| Nets | 185 on PCB; 190 in schematic (difference is auto-named/unused pins) |
| Gerber outputs | 9 layer files + `.gbrjob`, generated successfully |
| Drill outputs | 1 Excellon file + drill map PDF; `MixedPlating` file function with a declared NonPlated/NPTH tool |
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

Re-run after R-01…R-03: identical 514/514 common nodes, 0 splits, RESULT: PASS.
Neither the solid zone connections nor the removed duplicate track changed the
net partition.

## 5. DRC — 600 violations and disposition

Raw report: `hardware/prototype-v1/verification/drc.rpt`.

The zones were re-filled with the KiCad 9 filler because legacy fills cannot be
used by KiCad 8+. This removed 155 legacy `clearance` violations (zone fill
0.2488–0.2494 mm vs a 0.25 mm rule) and reduced starved thermals from 6 to 4.
After the R-01…R-03 revision, the remaining four starved thermals, the VDDA track
stub and the two U5 padstack/annular pairs are also cleared, taking the total to
600 (230 errors, 370 warnings). The netlist partition did not change. The
resulting DRC uses the upstream project's `v0.4.4d.kicad_pro` board settings (for
example `min_via_annular_width = 0.05 mm`); the legacy board-local setup used
0.10 mm. This is a **rule-set migration to the upstream project file**, not a
rule suppression. It is recorded here so a reviewer can reproduce both numbers:
766 before refill, 609 after refill, 600 after R-01…R-03.

| Category | Count | Severity | Disposition |
|---|---:|---|---|
| `courtyards_overlap` | 199 | error | **Open (mechanical review).** Dense connector/module placement, e.g. J1/J5 and the Arduino shield area. Courtyard overlap does not by itself prove a physical clash, but every overlap must be reviewed against the assembly drawing. Not cleared. |
| `silk_overlap` | 199 | warning | **Open (cosmetic).** Reference designators and outlines overlap on the dense shield/module area. Does not affect copper. Fix before fabrication for assembly legibility. |
| `lib_footprint_mismatch` | 143 | warning | **Accepted (library vintage).** Embedded KiCad 5 footprints differ from the current KiCad 9 library copies. The embedded footprints are the manufacturing authority. A footprint-level audit is still required. |
| `solder_mask_bridge` | 19 | error | **Accepted (intentional jumpers).** All 19 are JP6–JP11, the solder-configurable jumpers whose footprint deliberately bridges pads. Mask openings span the bridge by design. |
| `silk_over_copper` | 19 | warning | **Open (cosmetic).** Silkscreen clipped by mask/copper, mostly U5 and the shield outline. No copper effect. |
| `shorting_items` | 12 | error | **Accepted (intentional jumpers).** All 12 are the copper bridge polygons of `SolderJumper_2_Bridged` / `Jumper_3_Bridged12` on JP6–JP11. The bridge is the intended default configuration. This is a footprint-vs-DRC modelling issue, not an accidental short. |
| `silk_edge_clearance` | 5 | warning | **Open (cosmetic).** Silkscreen clipped by the 100 mm board edge, mostly the Arduino shield outline. |
| `lib_footprint_issues` | 4 | warning | **Open (library packaging).** Missing `Misc`/`Shields` library tables and one hand-solder footprint variant. Embedded footprints are intact. |

Violations cleared by the R-01…R-03 revision (previously open above):

| Category | Before | After | Fix |
|---|---:|---:|---|
| `starved_thermal` | 4 | 0 | R-01: C5.2, C10.2, C11.2, U4.3 set to solid GND zone connection |
| `track_dangling` | 1 | 0 | R-02: duplicate 0.552 mm VDDA segment removed |
| `padstack` | 2 | 0 | R-03: U5 mechanical tabs converted to NPTH |
| `annular_width` | 2 | 0 | R-03: same U5 tabs; no plated zero-annulus hole remains |

**DRC conclusion:** the board has **no unrouted nets**. The only "shorting"
violations are intentional solder-jumper bridges. Of the 230 remaining errors,
**199 are `courtyards_overlap`** requiring mechanical review and **31 are the
intentional solder-jumper `solder_mask_bridge`/`shorting_items`** (19 + 12). No
thermal, padstack or dangling-track error remains. H2 is **not** passed until the
courtyard overlaps, ERC exceptions and library-link warnings are resolved or
formally reviewed.

## 6. Manufacturing export verification

Generated from the native PCB/schematic by
`hardware/tools/check_prototype_v1.sh`:

| Output | File(s) | Result |
|---|---|---|
| Gerbers | 9 files: F/B copper, F/B paste, F/B silkscreen, F/B mask, Edge.Cuts | Generated; `.gbrjob` written |
| Drill | `v0.4.4d.drl` (Excellon, mm) + drill map PDF | Generated; `MixedPlating` with `T9C4.100` declared NonPlated/NPTH (U5 tabs) |
| Position | `v0.4.4d-pos.csv` (mm, drill origin) | 141 real placements, 0 `REF**`; JP6–JP11 are hand-solder/config features and are not machine placements |
| Bare-board net test | `v0.4.4d-ipc-d-356.net` | Generated |
| BOM | `v0.4.4d-bom.csv` | 47 grouped rows; 11 rows lack MPN (blocker HW-BOM-01) |
| Schematic PDF | `v0.4.4d-schematic.pdf` | Generated from native `.kicad_sch` |
| Netlist | `v0.4.4d.net` | Generated; used for connectivity check |

The gerber/drill files are **review artefacts only**. They have not been sent to
a fabricator, DFM-checked by a board house, or manufactured.

## 7. What is still required for H2/H3

1. Fix or formally document the 199 courtyard overlaps with a mechanical
   assembly drawing; do not assume they are harmless.
2. Add proper no-connect flags/PWR_FLAGs, or explicitly accept each unused-pin
   ERC error in a reviewed exception list (ERC is unchanged at 101 because
   R-01…R-03 were PCB-only).
3. Repair the 4 missing library links and the 2 `Device:CP1` symbol links.
4. Re-run this script and attach the new raw reports.
5. Independent hardware review (H3/H4). This document cannot approve
   fabrication.
