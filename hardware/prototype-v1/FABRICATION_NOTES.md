# Fabrication and Assembly Notes — prototype-v1 (Speeduino v0.4.4d SMD review copy)

| Field | Value |
|---|---|
| Document ID | HW-FAB-01 |
| Status | **Review package. NOT released for fabrication.** H4 (fabrication approval) requires independent review and explicit human approval. |
| Date | 2026-11-13 |
| Design | `v0.4.4d.kicad_pro` / `.kicad_sch` / `.kicad_pcb` in this directory |
| Revision | Upstream v0.4.4d + OpenTech R-01…R-10 (`../../AUDIT/HARDWARE_DIFF.md`) |
| Drawings | `manufacturing/assembly/v0.4.4d-assembly-top.pdf`, `…-assembly-bottom.pdf`, `manufacturing/drill/v0.4.4d-fab-drawing.pdf`, `manufacturing/drill/v0.4.4d-drl_map.pdf` |

Each statement is marked as **[UPSTREAM]** (read from the preserved upstream
files), **[MEASURED]** (produced by the KiCad 9.0.9 tools in
`../tools/check_prototype_v1.sh`), or **[ASSUMPTION]** (a stated choice that a
reviewer and the board house must confirm). Nothing here is a supplier quote or
a fabrication sign-off.

## 1. Board

| Item | Value | Basis |
|---|---|---|
| Outline | 100.05 × 100.05 mm rectangle, single closed Edge.Cuts outline. KiCad page coordinates (110.475, 36.975)–(210.525, 137.025) mm | [MEASURED] |
| Copper layers | 2 (F.Cu, B.Cu) | [UPSTREAM] layer table; [MEASURED] drill report |
| Finished thickness | 1.6 mm | [UPSTREAM] `(general (thickness 1.6))` in `../upstream-speeduino/v0.4.4d/v0.4.4d.kicad_pcb` |
| Dielectric | FR-4, standard Tg | [ASSUMPTION]. Upstream gives no material. |
| Copper weight | 1 oz (35 µm) finished, both layers | [ASSUMPTION]. The 35 µm in the `.gbrjob` is KiCad's default, not upstream evidence. The 1.0 mm high-current tracks were sized by upstream; their current rating is not verified here. |
| Surface finish | HASL lead-free or ENIG | [ASSUMPTION]. The `.gbrjob` says `"Finish": "None"`. ENIG is flatter for the 0805/SOIC/TO-263 paste print. |
| Solder mask / silkscreen | Both sides; green mask, white silk | [ASSUMPTION]. Colour has no electrical effect. |
| Smallest drill | 0.40 mm (176 vias, 0.8 mm pad) | [MEASURED] drill report |
| Track widths used | 0.25 / 0.3 / 0.4 / 0.5 / 0.75 / 1.0 mm | [MEASURED] |
| Design-rule minima | track 0.20 mm, via drill 0.30 mm, via annular 0.05 mm, hole clearance 0.25 mm | [UPSTREAM] project rules. Annular 0.05 mm is below many low-cost fab defaults. The board-house DFM must confirm. |
| Panelisation, impedance control, fabrication tolerances | None specified | [ASSUMPTION]. Use board-house standard. |

## 2. Drill summary — [MEASURED] `manufacturing/drill/v0.4.4d-drill-report.txt`

| Tool | Ø (mm) | Plating | Hits | Use |
|---|---:|---|---:|---|
| T1 | 0.400 | PTH | 176 | vias |
| T2 | 0.500 | PTH | 2 | vias |
| T3 | 0.800 | PTH | 24 | component leads |
| T4 | 0.840 | PTH | 6 | component leads |
| T5 | 0.850 | PTH | 2 | component leads |
| T6 | 1.000 | PTH | 191 | headers, proto pads |
| T7 | 1.016 | PTH | 126 | Arduino shield headers |
| T8 | 1.300 | PTH | 2 | J3 terminal block |
| T9 | 3.200 | **NPTH** | 3 | mounting holes H1–H3 (R-04) |
| T10 | 4.100 | **NPTH** | 2 | U5 MPX4250 mounting tabs (R-03) |

Totals: 529 plated holes and 5 non-plated holes. KiCad merges both into one
Excellon file (`MixedPlating`). Each NPTH tool is declared `NonPlated,NPTH`.
Origin: KiCad page origin (the aux/drill origin is (0, 0)). The Excellon file
has Y negative.

## 3. Mounting holes and mechanical support (HW-MECH-02)

| Ref | KiCad page XY (mm) | From board lower-left, Y up (mm) | Drill | Keep-out | Measured clearance |
|---|---|---|---|---|---|
| H1 | (113.975, 40.475) | (3.500, 96.550) | 3.2 mm NPTH, no net | radius 3.8 mm, F.Cu + B.Cu: no tracks, vias or copper pour | discrete copper 6.51 mm (U5 tab); fill 3.80 mm; courtyard 17.38 mm |
| H2 | (113.975, 133.525) | (3.500, 3.500) | 3.2 mm NPTH, no net | same | discrete copper 12.09 mm (J7.5); fill 3.80 mm; courtyard 10.46 mm |
| H3 | (207.025, 133.525) | (96.550, 3.500) | 3.2 mm NPTH, no net | same | discrete copper 5.11 mm (GND via); fill 3.80 mm; courtyard 9.76 mm |

The clearances are [MEASURED] in `verification/mounting-holes.txt`. They are
distances from the hole centre to the nearest object edge. Each hole wall is
1.90 mm from the board edges. The hole pattern is 93.05 × 93.05 mm at three
corners.

- **Footprint.** KiCad `MountingHole:MountingHole_3.2mm_M3`: unplated, no
  annular ring, excluded from the BOM and position files, board-only. H1/H2 are
  labelled on the silkscreen. H3 is labelled on F.Fab only, because the upstream
  logo and text fill the silkscreen space there.
- **Keep-out sizing [ASSUMPTION].** The 3.8 mm radius is the 7.0 mm OD of an
  M3 flat washer (DIN 125 / ISO 7089 nominal) plus 0.3 mm. A metal screw head
  or washer therefore cannot touch copper on either side. Soldermask is not
  counted as insulation.
- **No top-right hole [MEASURED].** `verification/mounting-holes.txt` searches
  a grid over x 190–207 mm, y 39.5–52 mm (0.5 mm step). It finds **0** sites
  with ≥ 3.8 mm to copper and ≥ 3.45 mm to courtyards. The least-bad site,
  (199.5, 42.0), has 1.30 mm to the VDD track (3.8 mm required) and 0.99 mm to
  the D14 courtyard (3.45 mm required). The J3 power terminal block (pads at
  x ≈ 205 mm, y ≈ 44 / 49 mm), D14/D15/C17 and the VDD track along the top edge
  fill that area. A hole there would need the VDD route or J3 to move. That is
  an electrical/layout change outside this minimal revision.
- **Defensible alternative for the top-right corner [ASSUMPTION — reviewer
  decision].** The board is an Arduino Mega 2560 shield. Its top-right quadrant
  sits over the Mega's 36-pin double-row header (D22–D53), next to J3. The
  proposed bench strategy:
  1. Mount the Arduino Mega 2560 on its own standoffs through the Mega's
     mounting holes. The Mega hole positions come from the Mega, not this
     board. Measure them on the physical Mega.
  2. Seat the shield on the Mega headers.
  3. Support the shield on M3 standoffs at H1, H2 and H3, outside the Mega
     outline (Mega outline: x 110.0–211.6 mm, y 54.46–107.8 mm. H1 is above it;
     H2/H3 are below it).
  4. Strain-relieve the J3 power cable to the enclosure or base plate, so
     terminal-screw torque and cable pull are not carried by the PCB or the
     Mega headers.

  Standoff length = Mega standoff height + Mega PCB + header mating height.
  It must be measured on the actual Mega and header stack. This document
  does not specify it.
- **Bottom side.** Only JP6–JP10 (copper solder jumpers, no parts) are on
  B.Cu. A standoff and nut at H1–H3 contacts no component. The Mega's
  bottom-side courtyard (USB/power jack, x 103.65–121.43 mm) does not reach
  H1–H3.

## 4. Assembly

| Group | Count | Process intent | Notes |
|---|---:|---|---|
| SMD parts, top side | 123 | Reflow (stencil from F.Paste) or hot-air | 0805 passives, SOD-323/SOD-123F/SMA/SMB diodes, SOIC-8/16, SOT-23, TO-263 (U8), 1812 fuses, 7343/3528 tantalum. **C5.2, C10.2, C11.2 and U4.3 have solid GND-pour connections (R-01) and need reflow or preheat. An iron alone may not wet them.** |
| THT parts, top side | 18 | Hand-solder after reflow | A1 (DRV8825 module socket/headers), IC3 (DIP-8 socket), J1–J8 headers/terminal, JP1–JP5 jumper headers, S1, U5 (MPX4250, "mount flat side up" per silkscreen; two 4.1 mm NPTH tab holes; the fastening method is a reviewer decision), XA1 (Arduino Mega shield headers). |
| Copper solder jumpers | 6 | No assembly. Bridged in copper by default. | JP6–JP10 (bottom) and JP11 (top). Cut a trace to change the configuration. See `AUDIT/PCB_CHECK_RESULTS.md` §5.2. |
| Proto-area pads | 150 | No assembly | Unannotated `REF**` pads, excluded from BOM and position files |
| Mounting holes | 3 | No assembly | H1–H3, see §3 |

- The position file `manufacturing/assembly/v0.4.4d-pos.csv` has 141
  placements: 123 SMD + 18 THT. It has no proto pads, mounting holes or
  solder jumpers. Coordinates use the drill/page origin, in mm.
- Orientation: pin-1/polarity marks are on F.Silkscreen and F.Fab in
  `v0.4.4d-assembly-top.pdf`. The bottom drawing is mirrored (viewed from the
  bottom).
- BOM: `manufacturing/assembly/v0.4.4d-bom.csv`. 11 grouped rows still have no
  MPN (HW-BOM-01). An assembler cannot quote from this BOM until that is
  closed.
- Modules (Arduino Mega 2560, Pololu DRV8825 carrier) are plugged in, not
  soldered to this board.

## 5. Not covered — required before H4

- Board-house DFM report against the 0.05 mm via-annular rule and the
  0.25 mm/0.20 mm geometry.
- Independent mechanical check of H1–H3, the standoff stack and the
  top-right-corner strategy.
- Thermal, current-capacity, creepage and EMC assessment. None has been done.
- Confirmation of every [ASSUMPTION] above by the human approver.
