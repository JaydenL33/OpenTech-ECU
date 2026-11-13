# Hardware Diff — Prototype v1 vs Upstream Speeduino v0.4.4d

| Field | Value |
|---|---|
| Document ID | AUD-HW-DIFF-01 |
| Status | Option B minimal revision: format conversion + R-01…R-03 PCB fixes + HW-H2-01 R-04…R-10 (mounting holes, library links, ERC annotation, DRC exclusions, fab-drawing annotations). Netlist partition unchanged. |
| Date | 2026-11-13 |
| Upstream | `hardware/upstream-speeduino/v0.4.4d/`, commit `ee6a9242834891877744ff2137663b6ca8d79ed6` |
| Derived | `hardware/prototype-v1/` |
| Related | `AUDIT/UPSTREAM_PROVENANCE.md`, `AUDIT/PCB_CHECK_RESULTS.md` |

## 1. Summary of changes

| # | Change | Type | Electrical effect | Evidence |
|---|---|---|---|---|
| D-01 | Legacy `.sch` → native `.kicad_sch` | File format | None intended; KiCad 9 save | 7 sheets converted; netlist exported; connectivity PASS |
| D-02 | Legacy `.kicad_pcb` → native `.kicad_pcb` | File format | None intended; KiCad 9 save | PCB netlist partition identical |
| D-03 | Re-fill copper zones with KiCad 9 filler | Copper fill only | Same nets; updated clearances; removed 155 legacy clearance violations; starved thermals 6→4 | DRC before 766 / after 609; connectivity PASS |
| D-04 | Enable bundled `v0.4.4c-rescue.lib` in `sym-lib-table` | Project library config | None | ERC library warnings 111→101 |
| D-05 | Copy `fp-lib-table` for reference | Project library config | None | Custom libraries still missing; ERC warning recorded |
| D-06 | Mark 150 unannotated proto-area pads `exclude from position files` / `exclude from BOM` | Assembly metadata | None | Position file now 141 real placements, 0 `REF**`; DRC unchanged at that time (609) |
| R-01 | Four starved-thermal GND pads (C5.2, C10.2, C11.2, U4.3) set to **solid** zone connection | Copper/zone connection | Same net; lower-impedance GND at those pads | DRC `starved_thermal` 4 → 0; connectivity PASS |
| R-02 | Delete redundant 0.552 mm VDDA track segment at (162.157, 132.991)→(162.709, 132.991) mm | Track removal (duplicate) | None: segment lies exactly on the through VDDA track and had one unconnected end | DRC `track_dangling` 1 → 0; connectivity PASS |
| R-03 | Convert U5 MAP-sensor 4.1 mm mechanical mounting tabs from PTH pads to **NPTH** holes | Pad attribute | None: both tabs had no net and zero annular copper | DRC `padstack` 2→0, `annular_width` 2→0; drill file marks the 4.1 mm tool NonPlated/NPTH |
| R-04 | Add H1–H3: 3.2 mm NPTH M3 corner mounting holes (3.5 mm inset) with board-level keep-outs of radius 3.8 mm on F.Cu+B.Cu (no tracks/vias/pour) | Mechanical + copper-pour clearance | None on nets; GND pour pulled back to 3.80 mm around each hole | `verification/mounting-holes.txt` PASS; DRC adds 0 violations; connectivity PASS |
| R-05 | Project-local footprint libraries `Shields`, `Misc`, `Speeduino_Legacy` extracted from the board; C19 relinked to `Speeduino_Legacy:`; dead library-table entries removed | Library links | None: geometry is the embedded footprint | ERC `footprint_link_issues` 4→0; DRC `lib_footprint_issues` 4→0; no new `lib_footprint_mismatch` |
| R-06 | `Device:CP1` → `Device:C_Polarized_US` (C17, C21) | Symbol link | None: identical pins/graphics | ERC `lib_symbol_issues` 2→0; netlist partition identical |
| R-07 | 5 × PWR_FLAG (GND, 12V-SW at J3; VRegIn after D14; Vdrive after JP1; VDDA after F1) | Schematic annotation | None: PWR_FLAG has no footprint | ERC `power_pin_not_driven` 9→0 |
| R-08 | 29 no-connect flags on unused pins; remove ICSP labels MISO/MOSI/SCLK + 3 × 1.27 mm wires, 2 × 8.89 mm dangling wire stubs, 1 dangling no-connect | Schematic annotation/hygiene | None on PCB: affected nets are single-node; 3 schematic-only nets renamed | ERC `pin_not_connected` 26→0, `pin_not_driven` 9→4, `unconnected_wire_endpoint` 2→0, `no_connect_dangling` 1→0, `global_label_dangling` 37→34 |
| R-09 | 199 per-item DRC exclusions (`courtyards_overlap` only), each with a justification comment | DRC exception record | None | DRC reported 596→397; excluded items listed in `verification/drc-with-exclusions.rpt` and `drc_exclusions.csv` |
| R-10 | Fab-drawing dimensions (board 100.05 mm ×2, hole pitch 93.05 mm ×2) and note text on User.Drawings | Documentation layer | None: User.Drawings is not in the Gerber set | Fab drawing `manufacturing/drill/v0.4.4d-fab-drawing.pdf` |

Reproduction for R-01…R-03 (idempotent; prints before/after SHA-256 and each
change):

```bash
docker run --rm --user "$(id -u):$(id -g)" \
  -v "$PWD/hardware/prototype-v1:/work" \
  -v "$PWD/hardware/tools:/tools:ro" \
  kicad/kicad@sha256:e638b79b0321f29395a5b783e94bb9f3c73303e8da15da27b8f5cb4b67a37729 \
  python3 /tools/apply_v1_minimal_revision.py /work
```

Observed first run (2026-11-13): 7 changes applied; board SHA-256
`22d4c905…95b2169` → `dffe0337…e92d2f2f`. Second run: 0 changes, identical hash
(idempotent). Raw check evidence: `hardware/prototype-v1/verification/`.

**No component value, net assignment, track topology, via or board outline edit
was made.** R-02 removes duplicate copper on an existing net. R-03 changes only
the plating attribute of two no-net mechanical holes. R-04 adds three no-net
NPTH holes and copper keep-outs. R-05 changes only the footprint *library link*
of C19 (pad geometry is byte-identical). R-06…R-08 are schematic annotations.
The netlist partition is unchanged (190 schematic nets before/after; 514/514
schematic↔PCB nodes).

Reproduction for R-04…R-10 (HW-H2-01). Each step is idempotent and prints
before/after SHA-256:

```bash
IMG=kicad/kicad@sha256:e638b79b0321f29395a5b783e94bb9f3c73303e8da15da27b8f5cb4b67a37729
RUN="docker run --rm --user $(id -u):$(id -g) -v $PWD/hardware/prototype-v1:/work -v $PWD/hardware/tools:/tools:ro $IMG"
$RUN python3 /tools/apply_hw_h2_01_revision.py sch /work          # R-05 (sch part), R-06, R-07, R-08
$RUN python3 /tools/apply_hw_h2_01_revision.py pcb /work          # R-04, R-05 (pcb/libs), R-10
$RUN kicad-cli pcb drc --format json --severity-all --severity-exclusions \
     --output /work/verification/drc-for-exclusions.json /work/v0.4.4d.kicad_pcb
$RUN python3 /tools/apply_hw_h2_01_revision.py exclusions /work   # R-09
hardware/tools/check_prototype_v1.sh
```

Observed (2026-11-13, logs in `hardware/prototype-v1/verification/revision-logs/`):
schematic step 47 changes, PCB step 16 changes (board `dffe0337…e92d2f2f` →
`05daa61a…36252452`). A keep-out geometry correction followed: the first
32-gon was inscribed in r = 3.8 mm, which let the fill reach 3.782 mm. Six
more changes gave `b2f6cd3e…facb82b8`. Second runs: 0 changes, identical
hashes. Exclusion step: 199 written (174 class P, 22 class H, 3 class C).

Clean-room reproduction: the same sequence on `git archive HEAD` of the
pre-revision tree gave byte-identical schematics, `.kicad_pro`, library tables
and library footprints. It also gave identical ERC (49) and DRC (397 reported +
199 excluded) category counts. The `.kicad_pcb` differs only by footprint order
and random KiCad object UUIDs for the new objects (96 differing lines after
masking UUIDs; the sorted-line multisets are identical, so every difference
is a reordering of the new hole, keep-out and dimension blocks).

## 2. Revision change records (R-01…R-10)

### 2.1 Change record for R-01 (thermal connection)

| Item | Detail |
|---|---|
| What changed | `zone_connect` of C5.2, C10.2, C11.2 and U4.3 changed from thermal relief (inherit) to solid on the F.Cu GND pour. |
| Why | KiCad DRC reported these four pads with only one thermal spoke against a required minimum of two. Connectivity existed, but solderability/current sharing was marginal. |
| Electrical effect | Direct copper connection from each pad to the GND pour. Lower connection inductance/resistance; no net or voltage change. |
| Ratings | C5/C10/C11 are 0805 ceramic decoupling capacitors on GND; U4.3 is an SOIC-8 GND pin. No current rating is approached by the plane connection. |
| Operating limits | None added; the pads were already GND. Thermal relief no longer limits soldering heat. |
| Failure behaviour | A solid connection cannot "open" a spoke; it removes a partial-connection failure mode. If the plane is damaged, behaviour is the same as any plane-connected pad. |
| Tests performed | KiCad 9.0.9 DRC: 4 starved thermal errors → 0. Connectivity comparison PASS (514/514 common nodes, 0 splits). |
| Remaining review | Reviewer to confirm reflow/hot-air assembly for these four pads; an iron-only hand build may need preheat. |

### 2.2 Change record for R-02 (VDDA stub)

| Item | Detail |
|---|---|
| What changed | Removed one `PCB_TRACK` on net VDDA, length 0.552 mm, width 0.5 mm, F.Cu, from (162.157, 132.991) to (162.709, 132.991) mm. |
| Why | DRC `track_dangling` reported an unconnected end at (162.157, 132.991). Inspection shows the segment is exactly collinear with, and entirely inside, the through track (149.047374, 132.991)→(162.709, 132.991). It is duplicate copper. |
| Electrical effect | None: the remaining track covers the same path and still connects the same VDDA nodes. |
| Ratings | Track width unchanged at 0.5 mm on the remaining segment. |
| Operating limits | None. |
| Failure behaviour | Removing duplicate copper cannot disconnect the net; verified by connectivity comparison. |
| Tests performed | DRC `track_dangling` 1 → 0; `Found 0 unconnected items`; connectivity PASS. |
| Remaining review | Reviewer to confirm in the PCB editor that VDDA still reaches every pad; netlist evidence is attached. |

### 2.3 Change record for R-03 (U5 mechanical tabs)

| Item | Detail |
|---|---|
| What changed | Two 4.1 mm holes on U5 (MPX4250 MAP sensor) changed from plated through-hole pads with zero annular ring to non-plated through holes (NPTH). |
| Why | These are mechanical mounting tabs, not electrical pads. Modelling them as PTH produced `padstack` and `annular_width` errors and would ask a fabricator to plate a zero-annulus hole. |
| Electrical effect | None: both pads had no net. |
| Ratings | 4.1 mm NPTH suits the sensor's mounting screws/tabs; plating is not required mechanically. |
| Operating limits | None electrical. Confirm screw head/tab clearance during mechanical assembly review. |
| Failure behaviour | Non-plated holes cannot create a short or a plated barrel failure. |
| Tests performed | DRC `padstack` 2 → 0, `annular_width` 2 → 0. Excellon drill file now declares `T9C4.100` as `NonPlated,NPTH,ComponentDrill` (file function `MixedPlating`). |
| Remaining review | Mechanical reviewer to confirm the U5 mounting method and screw/tab stack. |

### 2.4 Structural check: only the intended objects changed

Board statistics were produced with `pcbnew` for the preserved upstream KiCad 5
board and the revised native board:

| Object | Upstream v0.4.4d | Revised prototype-v1 | Expected difference |
|---|---:|---:|---|
| Footprints | 297 | 297 | none |
| Pads | 677 | 677 | none |
| Nets | 185 | 185 | none |
| Zones | 4 | 4 | none |
| Track/via objects | 1233 | 1232 | −1 (R-02 stub) |
| Track segments | 1055 | 1054 | −1 |
| Vias | 178 | 178 | none |
| NPTH pads | 0 | 2 | +2 (R-03 U5 tabs) |

After HW-H2-01 (R-04…R-10), measured with `pcbnew` against the R-03 board:

| Object | After R-03 | After R-10 | Expected difference |
|---|---:|---:|---|
| Footprints | 297 | 300 | +3 (H1–H3) |
| Pads | 677 | 680 | +3 (H1–H3, all NPTH) |
| NPTH pads | 2 | 5 | +3 |
| Nets (excluding net 0) | 184 | 184 | none |
| Zones (incl. rule areas) | 4 (2 rule areas) | 7 (5 rule areas) | +3 keep-outs |
| Track/via objects | 1232 | 1232 | none |
| Vias | 178 | 178 | none |
| Board drawings | 189 | 194 | +5 (R-10 on User.Drawings) |

This confirms that R-01 changed only zone-connection attributes, R-02 removed
exactly one track object, and R-03 changed exactly two pad attributes. No
footprint, pad count, net count or zone outline changed.

### 2.5 Change record for R-04 (mounting holes H1–H3)

| Item | Detail |
|---|---|
| What changed | Added three `MountingHole:MountingHole_3.2mm_M3` footprints (unchanged KiCad 9 library copies): H1 (113.975, 40.475), H2 (113.975, 133.525), H3 (207.025, 133.525) mm (page coordinates). Each is 3.2 mm NPTH, no net, board-only, excluded from BOM and position files. Added three board-level rule areas `H1_keepout`…`H3_keepout`: 32-gon circumscribing r = 3.8 mm, on F.Cu+B.Cu, forbidding tracks, vias and copper pour. Footprints/pads are not forbidden, so the hole's own NPTH pad is not flagged. H3's reference text is on F.Fab, because the silkscreen near H3 holds the upstream logo/text. |
| Why | HW-MECH-02. The upstream board had no mounting holes, so a bench fixture or enclosure had nothing to fasten to. |
| Placement analysis | Exploratory clearance sweep of the four 3.5–5.0 mm corner insets on the unmodified board: TL/BL/BR clear. TR blocked: J3 courtyard 0.03 mm, VDD track 0.28 mm at a 4.0 mm inset. A 4.0 mm inset put the upstream logo inside H3's drill (logo edge 1.15 mm from centre), so all holes use a 3.5 mm inset. Logo-to-hole-centre is then 1.66 mm, outside the 1.6 mm drill radius. The final board is measured by `hardware/tools/check_mounting_holes.py` → `verification/mounting-holes.txt`. Nearest discrete copper: H1 6.51 mm (U5 NPTH tab), H2 12.09 mm (J7.5), H3 5.11 mm (GND via). Zone fill: 3.80 mm at all three. Courtyards ≥ 9.76 mm. TR grid search: 0 feasible sites. |
| Electrical effect | None on any net. The GND pour on F.Cu/B.Cu is cleared within 3.8 mm of each hole. DRC reports no isolated copper and 0 unconnected items, and connectivity is unchanged. |
| Ratings | M3 clearance hole, 3.2 mm. Keep-out sized for a 7.0 mm washer (DIN 125 / ISO 7089 nominal) + 0.3 mm. Hole wall 1.9 mm from both edges. |
| Operating limits | Mechanical only. The board is supported at three corners. The top-right corner (J3 power terminal) has no hole. See `hardware/prototype-v1/FABRICATION_NOTES.md` §3 for the Mega-standoff + J3 strain-relief strategy. |
| Failure behaviour | Unplated, unconnected holes cannot short a net. A metal screw/washer stays ≥ 3.8 mm from copper. A missing or loose standoff leaves the board on the Mega headers, the same as upstream. |
| Tests performed | KiCad 9.0.9 DRC: no `items_not_allowed`, courtyard, clearance, `hole_clearance` or silk violation from H1–H3. Mounting-hole check PASS. Drill file: tool `T9C3.200` NonPlated/NPTH, 3 hits. Position file unchanged at 141 rows. BOM has no H rows. |
| Remaining review | Mechanical reviewer: accept the 3-hole pattern and the top-right strategy, and measure standoff length on the real Mega + header stack. Optional future revision: reroute the VDD top-edge track to free a fourth hole (not done; it would be a layout change). |

### 2.6 Change record for R-05 (library links)

| Item | Detail |
|---|---|
| What changed | Wrote project-local footprint libraries from the footprints embedded in the board (normalised to origin/0°, nets cleared): `lib/Shields/Shields.pretty/Arduino_Mega2560_Shield_IO_Only`, `lib/Misc/Misc.pretty/MPX4250` (includes the R-03 NPTH tabs), `lib/Misc/Misc.pretty/AYZ0202AGRLC`, `lib/Speeduino_Legacy/Speeduino_Legacy.pretty/C_0805_2012Metric_Pad1.15x1.40mm_HandSolder`. Relinked C19 from `Capacitor_SMD:` to `Speeduino_Legacy:` in the schematic and PCB. `fp-lib-table` now lists Shields, Misc and Speeduino_Legacy. The unused `Connector_Automotive` entry is dropped. `sym-lib-table` now lists only `v0.4.4c-rescue`; it holds every custom symbol. The dropped entries `Arduino`, `Connector`, `Misc` and `IC_Automotive` pointed at files that never existed upstream, and no `lib_id` referenced them. |
| Why | HW-ERC-02. Missing library links produced ERC/DRC library warnings, and a reviewer could not open the libraries. |
| Substitution note | KiCad 9 has `C_0805_2012Metric_Pad1.18x1.45mm_HandSolder`, but not the KiCad 5 `Pad1.15x1.40mm` variant. The legacy pad geometry is kept, not swapped to the new size, so copper/paste are unchanged. |
| Electrical effect | None: no pad geometry or net change. DRC finds no new `lib_footprint_mismatch` for the four extracted footprints. |
| Tests performed | ERC `footprint_link_issues` 4→0. DRC `lib_footprint_issues` 4→0. BOM row for C19 shows the new library nickname only. |
| Remaining review | The 143 `lib_footprint_mismatch` warnings for standard KiCad footprints (embedded KiCad 5 copies vs KiCad 9 library) remain. That footprint-by-footprint audit is still open. |

### 2.7 Change record for R-06 (`Device:CP1`)

| Item | Detail |
|---|---|
| What changed | The embedded `Device:CP1` lib symbol was replaced by the KiCad 9 `Device:C_Polarized_US` definition, and C17/C21 `lib_id` updated. Values and footprints unchanged. |
| Why | KiCad 6 renamed `CP1` to `C_Polarized_US`. The old link produced `lib_symbol_issues`. |
| Electrical effect | None. Pin 1 at (0, 3.81) length 2.794, pin 2 at (0, −3.81) length 3.302, both passive, with identical graphics, checked against `/usr/share/kicad/symbols/Device.kicad_sym` in the pinned image. |
| Tests performed | ERC `lib_symbol_issues` 2→0. Schematic netlist partition identical (190 nets). |

### 2.8 Change record for R-07 (PWR_FLAG)

| Item | Detail |
|---|---|
| What changed | `#FLG01` GND at J3.1 and `#FLG02` 12V-SW at J3.2 (Connector sheet). `#FLG03` VRegIn at (139.70, 62.23), after D14 (Power sheet). `#FLG04` VDDA, after F1, via a 5.08 mm wire (Power sheet). `#FLG05` Vdrive, after JP1, via a 5.08 mm wire (Fuel & Ignition sheet). |
| Why | Each of these nets receives power through a connector, a diode, a jumper or a polyfuse, all passive pins. KiCad therefore could not see a driver, which caused 9 `power_pin_not_driven` errors. |
| Electrical effect | None. PWR_FLAG is an ERC-only symbol with no footprint. The ERC now assumes these nets are powered. A real loss of supply on them would not be caught by ERC. That is the normal meaning of PWR_FLAG and is limited to these five entry points. |
| Tests performed | ERC `power_pin_not_driven` 9→0. No flag produced a `pin_not_connected`, so each is attached to its net. |

### 2.9 Change record for R-08 (no-connects and schematic hygiene)

| Item | Detail |
|---|---|
| What changed | No-connect flags on: XA1 SDA, SCL, A11–A15, AREF, IOREF, 3V3, VIN and the six ICSP pins (MISO, MOSI, SCK, 5V, GND, RESET). S1 second pole (NC_2, C_2, NO_2). U7 and U9 STATUS1/STATUS2. J2 pin 6. A1 FLT, M0, M1, M2. That is 29 pins. Removed: dangling global labels MISO/MOSI/SCLK and their 1.27 mm wires; Power-sheet wire stubs (184.15→193.04, 68.58) and (184.15→193.04, 78.74); the dangling no-connect at (213.36, 55.88). |
| Why | HW-ERC-01. Every pin had no connection on the PCB, but ERC could not tell "intentionally unused" from "forgotten". |
| Justification per pin | No flagged pin has any PCB connection: each is a single-node net (`unconnected-…`) or has no pad. The six ICSP pins have no pad in `Arduino_Mega2560_Shield_IO_Only` (see connectivity "schematic-only nodes"). SDA/SCL: the shield pads are unused, and the same Mega signals are used as D20/D21 through J5. A1 M0–M2: microstep selects, left open as upstream (full-step). The module's internal pull-downs are per the Pololu DRV8825 carrier documentation; reviewer to confirm. VNLD5090 STATUS outputs are unused upstream, and U1/U3 already had no-connects. No engine-critical or used pin was flagged. |
| Electrical effect | None on the PCB. Three schematic-only nets were renamed (`MISO`/`MOSI`/`SCLK` → `unconnected-(XA1-…)`). They have no PCB pads. The 190-net partition is identical. |
| Tests performed | ERC `pin_not_connected` 26→0; `pin_not_driven` 9→4 (U6 spare channels remain, documented); `unconnected_wire_endpoint` 2→0; `no_connect_dangling` 1→0; `global_label_dangling` 37→34. Connectivity PASS 514/514. |
| Remaining review | Reviewer to accept the 4 U6 spare-channel `pin_not_driven` exceptions (`AUDIT/PCB_CHECK_RESULTS.md` §3). |

### 2.10 Change record for R-09 (courtyard DRC exclusions)

| Item | Detail |
|---|---|
| What changed | 199 entries in `v0.4.4d.kicad_pro` `drc_exclusions`. Each is keyed to one marker (`courtyards_overlap\|x\|y\|uuidA\|uuidB`; marker position from `SHAPE_POLY_SET.Collide`) and has a comment. No rule severity was changed, and no other category was excluded. |
| Classes | **P (174):** proto pad ↔ proto pad. **H (22):** proto pad ↔ J1/J5/J6/J8. **C (3):** J1↔J5, J5↔J6, J6↔J8, headers on one continuous 2.54 mm pitch whose housings butt with zero gap. |
| Why | HW-DRC-01. None of the 199 is a physical clash. Proto pads hold no part, and the 3 header pairs touch only at the courtyard margin. |
| Electrical effect | None. |
| Failure behaviour | An exclusion is tied to item UUIDs and the marker position. If any listed footprint moves, the overlap reappears in DRC; it cannot hide a new clash. |
| Tests performed | DRC reported 596 → 397. `--severity-exclusions` report still lists all 199 with comments. `verification/drc_exclusions.csv` lists every item. |
| Remaining review | Reviewer to accept the 3 class-C header pairs against the chosen header MPN (HW-BOM-01). |

### 2.11 Change record for R-10 (fab-drawing annotations)

| Item | Detail |
|---|---|
| What changed | On User.Drawings: four aligned dimensions (board 100.05 × 100.05 mm; H1–H2 and H2–H3 pitch 93.05 mm) and one note block that points to `FABRICATION_NOTES.md`. |
| Electrical effect | None. User.Drawings is not in the exported Gerber layer list. |
| Tests performed | Gerber layer set unchanged. Fab drawing exported. |

## 3. Why the zone re-fill is not a circuit change

KiCad 8+ cannot use the legacy zone-fill strategy. Opening and saving the
upstream board without re-filling leaves stale fill polygons that fail modern
clearance checks (155 violations at 0.2488–0.2494 mm against a 0.25 mm zone
rule). The KiCad 9 zone filler recomputes the same GND/PGND pours from the same
zone outlines, nets and design rules. It does not add or remove nets.

Verification:

- DRC after re-fill: **0 unconnected items**.
- Schematic↔PCB connectivity: **514/514 common nodes identical, 0 split nets**.
- The 12 `shorting_items` are unchanged in count and remain the intentional
  JP6–JP11 solder-jumper bridges.

## 4. Rule-set note

The upstream project file (`v0.4.4d.kicad_pro`) is preserved in spirit: its
board rules are identical after the KiCad 9 save (for example
`min_via_annular_width = 0.05 mm`). The legacy board-local setup used 0.10 mm
annular minimum, so the first DRC run reported 766 violations and the post-save
run reports 609. This is a migration from the legacy board-local rule to the
upstream project rule, not a rule suppression. No rule severity has been changed.
After R-01…R-03 the DRC total was **600 (230 errors, 370 warnings)**. After
R-04…R-10 it is **397 reported (31 errors, 366 warnings) + 199 excluded**
(`courtyards_overlap`, per item, with comments). There are 0 unconnected items.
The 31 remaining errors are the JP6–JP11 jumper exceptions. They are deliberately
**not** excluded.

Project-file side effect of the R-04/R-10 `pcbnew` save: `v0.4.4d.kicad_pro`
gained 47 keys that KiCad 9.0.9 writes with their in-memory defaults. Examples:
`rule_severities` entries such as `creepage`, `isolated_copper` and
`starved_thermal`; `rules/min_resolved_spokes = 2`; teardrop and tuning-pattern
defaults; `apply_defaults_to_fp_* = false`. Four line-width floats were
re-serialised (`0.049999999999999996` → `0.05`). A key-by-key comparison with
the previous file found **no existing value changed or removed**, apart from
`drc_exclusions`. The added keys were already the effective defaults when the
earlier DRC runs used this KiCad build.

## 5. Review checklist for a human reviewer

1. Confirm the preserved upstream tree matches `SHA256SUMS`.
2. Open `prototype-v1/v0.4.4d.kicad_pro` and inspect the six hierarchical
   sheets and PCB; confirm no value/net/footprint changes are visible.
3. Re-run `hardware/tools/check_prototype_v1.sh`.
4. Inspect the re-filled zones on F.Cu/B.Cu; confirm GND/PGND pours are
   continuous and no new isolation or short exists.
5. Review the exceptions in `AUDIT/PCB_CHECK_RESULTS.md` §3/§5, the 199 DRC
   exclusions in `verification/drc_exclusions.csv`, and the mounting-hole
   strategy in `prototype-v1/FABRICATION_NOTES.md` §3.
6. Do not approve fabrication until the H2/H3 blockers in
   `HARDWARE_BLOCKERS.md` are closed or explicitly accepted.
