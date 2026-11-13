# Hardware Diff — Prototype v1 vs Upstream Speeduino v0.4.4d

| Field | Value |
|---|---|
| Document ID | AUD-HW-DIFF-01 |
| Status | Option B minimal revision: format conversion + 3 PCB/manufacturing fixes (R-01…R-03); no net or topology change |
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
| R-03 | Convert U5 MAP-sensor 4.1 mm mechanical mounting tabs from PTH pads to **NPTH** holes | Pad attribute | None: both tabs had no net and zero annular copper | DRC `padstack` 2→0, `annular_width` 2→0; drill file marks `T9C4.100` NonPlated/NPTH |

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

**No component value, net assignment, footprint assignment, track topology, via,
board outline or mounting-hole edit was made.** R-02 removes duplicate copper on
an existing net; R-03 changes only the plating attribute of two no-net mechanical
holes. The netlist partition is unchanged.

## 2. Revision change records (R-01…R-03)

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

This confirms that R-01 changed only zone-connection attributes, R-02 removed
exactly one track object, and R-03 changed exactly two pad attributes. No
footprint, pad count, net count or zone outline changed.

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
upstream project rule, not a rule suppression. No DRC exclusion was added. After
R-01…R-03 the DRC total is **600 (230 errors, 370 warnings)** with 0 unconnected
items.

## 5. Review checklist for a human reviewer

1. Confirm the preserved upstream tree matches `SHA256SUMS`.
2. Open `prototype-v1/v0.4.4d.kicad_pro` and inspect the six hierarchical
   sheets and PCB; confirm no value/net/footprint changes are visible.
3. Re-run `hardware/tools/check_prototype_v1.sh`.
4. Inspect the re-filled zones on F.Cu/B.Cu; confirm GND/PGND pours are
   continuous and no new isolation or short exists.
5. Review the exceptions in `AUDIT/PCB_CHECK_RESULTS.md` §3/§5.
6. Do not approve fabrication until the H2/H3 blockers in
   `HARDWARE_BLOCKERS.md` are closed or explicitly accepted.
