# Prototype v1 — Speeduino v0.4.4d SMD review copy

This is the **native KiCad 9 review copy** of the selected Speeduino v0.4.4d SMD
design. The electrical design is upstream. OpenTech-ECU saved it to KiCad 9
format, re-filled the copper zones and applied documented minimal revisions:
R-01…R-03 (PCB fixes) and R-04…R-10 (HW-H2-01: mounting holes, library links,
ERC annotation, DRC exclusions, fab drawing). The netlist is unchanged. It is
**not fabrication-approved**.

| Field | Value |
|---|---|
| Base design | Speeduino v0.4.4d SMD |
| Upstream commit | `ee6a9242834891877744ff2137663b6ca8d79ed6` |
| Preserved source | `../upstream-speeduino/v0.4.4d/` |
| KiCad | 9.0.9 |
| Board | 100.05 × 100.05 mm, 2-layer, 1.6 mm, 4 injector / 4 ignition, Arduino Mega 2560 shield, IDC40 |
| Mounting | H1–H3: 3.2 mm NPTH (M3) at three corners, 3.5 mm inset, 3.8 mm copper keep-out; no top-right hole (see `FABRICATION_NOTES.md` §3) |
| Routing | Complete: 0 unconnected items |
| ERC | 49 (4 errors, 45 warnings), all dispositioned — `verification/erc.rpt`, `AUDIT/PCB_CHECK_RESULTS.md` §3 |
| DRC | 397 reported (31 errors = JP6–JP11 jumpers, 366 warnings) + 199 per-item courtyard exclusions; 0 unconnected — `verification/drc.rpt`, `drc-with-exclusions.rpt` |
| Connectivity | PASS, 514/514 common nodes |
| Status | **H2 complete with documented exceptions, pending independent acceptance (not a pass); H3 not ready; not fabrication-approved** |

## Open the project

Open `v0.4.4d.kicad_pro` in KiCad 9.0.x. The root schematic
`v0.4.4d.kicad_sch` references the six hierarchical sheets in this directory.
The PCB is `v0.4.4d.kicad_pcb`.

`sym-lib-table` enables the bundled `v0.4.4c-rescue.lib`, which holds every
custom symbol. `fp-lib-table` enables three project-local footprint libraries
under `lib/`: `Shields`, `Misc` and `Speeduino_Legacy`. They were extracted
from the footprints embedded in the routed board (R-05), so library and board
copies match. The upstream `Connector_Automotive`/`IC_Automotive` libraries
were never in the upstream tree, nothing referenced them, and their table
entries have been removed.

## Reproduce the checks and exports

From the repository root:

```bash
hardware/tools/check_prototype_v1.sh
```

This runs, inside the pinned `kicad/kicad@sha256:e638b79b…7729` container:

- ERC on `v0.4.4d.kicad_sch`;
- DRC on `v0.4.4d.kicad_pcb`;
- DRC again with `--severity-exclusions` (excluded items + their comments);
- Gerber, drill (+ map + report), position, IPC-D-356, BOM, netlist and
  schematic PDF export;
- top/bottom assembly drawings, fab drawing, top/bottom 3D renders;
- schematic-to-PCB connectivity comparison;
- mounting-hole clearance check (`../tools/check_mounting_holes.py`).

The HW-H2-01 design edits are reproducible with
`../tools/apply_hw_h2_01_revision.py` (steps `sch`, `pcb`, `exclusions`). The
commands are in `../../AUDIT/HARDWARE_DIFF.md` §1.

Outputs are written to `verification/` and `manufacturing/`.

## Manufacturing package

| Directory | Contents |
|---|---|
| `manufacturing/gerbers/` | 9 Gerber layers + `.gbrjob` |
| `manufacturing/drill/` | Excellon drill file, drill map PDF, drill report, **fabrication drawing PDF** |
| `manufacturing/assembly/` | BOM CSV, position CSV (141 real placements, no proto pads/holes), IPC-D-356 net test, KiCad netlist, schematic PDF, **top/bottom assembly drawings** |
| `FABRICATION_NOTES.md` | Stack-up (upstream 2-layer 1.6 mm), stated assumptions, hole coordinates, drill summary, reflow vs hand-solder intent |

These are **review artefacts**. No board house has DFM-checked them, and
nothing has been manufactured. The BOM has 11 rows without MPNs; see
`HW-BOM-01`.

## What changed from upstream

1. Legacy `.sch` → native `.kicad_sch` (KiCad 9 save).
2. Legacy `.kicad_pcb` → native `.kicad_pcb` (KiCad 9 save).
3. Copper zones re-filled by the KiCad 9 zone filler (required for the format;
   removed 155 legacy clearance violations; netlist unchanged).
4. Bundled rescue symbol library enabled via `sym-lib-table` and `fp-lib-table`
   copied for reference.
5. The 150 unannotated proto-area pads are marked exclude-from-BOM and
   exclude-from-position-files. This is assembly metadata only; no copper
   change.
6. R-01: four starved-thermal GND pads (C5.2, C10.2, C11.2, U4.3) set to a
   solid zone connection.
7. R-02: duplicate 0.552 mm VDDA track segment removed.
8. R-03: two U5 MAP-sensor mechanical tabs converted from PTH to NPTH.
9. R-04: H1–H3 M3 NPTH mounting holes + 3.8 mm copper keep-outs.
10. R-05: project-local footprint libraries; C19 relinked to
    `Speeduino_Legacy:` (same pads).
11. R-06: `Device:CP1` → `Device:C_Polarized_US` (identical symbol).
12. R-07: 5 PWR_FLAGs at real power entry points.
13. R-08: 29 no-connects on unused pins; dangling ICSP labels, 2 wire stubs
    and 1 stray no-connect removed.
14. R-09: 199 per-item courtyard DRC exclusions with comments.
15. R-10: fab-drawing dimensions/notes on User.Drawings.

No component value, net, routing topology or outline edit was made. The
schematic netlist partition is identical to before. R-01…R-03 are reproducible
with `../tools/apply_v1_minimal_revision.py`, and R-04…R-10 with
`../tools/apply_hw_h2_01_revision.py`. All are recorded in
`../../AUDIT/HARDWARE_DIFF.md` §2. Full provenance:
`../../AUDIT/UPSTREAM_PROVENANCE.md`. Full check disposition:
`../../AUDIT/PCB_CHECK_RESULTS.md`.

## Do not

- Do not fabricate this board without independent review and explicit human H4
  approval.
- Do not connect it to an Evo VIII; the vehicle interface is unverified.
- Do not treat ERC/DRC as automotive safety evidence.
