# Prototype v1 — Speeduino v0.4.4d SMD review copy

This is the **native KiCad 9 review copy** of the selected Speeduino v0.4.4d SMD
design. The electrical design is upstream; OpenTech-ECU saved it to KiCad 9
format, re-filled the copper zones and applied three documented PCB fixes
(R-01…R-03). It is **not fabrication-approved**.

| Field | Value |
|---|---|
| Base design | Speeduino v0.4.4d SMD |
| Upstream commit | `ee6a9242834891877744ff2137663b6ca8d79ed6` |
| Preserved source | `../upstream-speeduino/v0.4.4d/` |
| KiCad | 9.0.9 |
| Board | 100.05 × 100.05 mm, 2-layer, 4 injector / 4 ignition, Arduino Mega 2560 shield, IDC40 |
| Routing | Complete: 0 unconnected items |
| ERC | 101 violations (44 errors, 57 warnings) — see `verification/erc.rpt` |
| DRC | 600 violations (230 errors, 370 warnings), 0 unconnected — see `verification/drc.rpt` |
| Connectivity | PASS, 514/514 common nodes |
| Status | **H2 not passed; not fabrication-approved** |

## Open the project

Open `v0.4.4d.kicad_pro` in KiCad 9.0.x. The root schematic
`v0.4.4d.kicad_sch` references the six hierarchical sheets in this directory.
The PCB is `v0.4.4d.kicad_pcb`.

The `sym-lib-table` enables the bundled `v0.4.4c-rescue.lib`. The custom
`Shields`, `Misc`, `Connector_Automotive` and `IC_Automotive` libraries are
**not** included in the upstream tree; their symbols/footprints are embedded in
the design, but the library links produce ERC warnings. See
`../HARDWARE_BLOCKERS.md` `HW-ERC-02`.

## Reproduce the checks and exports

From the repository root:

```bash
hardware/tools/check_prototype_v1.sh
```

This runs, inside the pinned `kicad/kicad@sha256:e638b79b…7729` container:

- ERC on `v0.4.4d.kicad_sch`;
- DRC on `v0.4.4d.kicad_pcb`;
- Gerber, drill, position, IPC-D-356, BOM, netlist and schematic PDF export;
- schematic-to-PCB connectivity comparison.

Outputs are written to `verification/` and `manufacturing/`.

## Manufacturing package

| Directory | Contents |
|---|---|
| `manufacturing/gerbers/` | 9 Gerber layers + `.gbrjob` |
| `manufacturing/drill/` | Excellon drill file + drill map PDF |
| `manufacturing/assembly/` | BOM CSV, position CSV (141 real placements, no proto pads), IPC-D-356 net test, KiCad netlist, schematic PDF |

These are **review artefacts**. They have not been DFM-checked by a board house
or manufactured. The BOM has 11 rows without MPNs; see `HW-BOM-01`.

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

No component value, net, footprint assignment, routing topology, outline or
mounting-hole edit was made. R-01…R-03 are reproducible with
`../tools/apply_v1_minimal_revision.py` and recorded in
`../../AUDIT/HARDWARE_DIFF.md` §2. Full provenance:
`../../AUDIT/UPSTREAM_PROVENANCE.md`. Full check disposition:
`../../AUDIT/PCB_CHECK_RESULTS.md`.

## Do not

- Do not fabricate this board without independent review and explicit human H4
  approval.
- Do not connect it to an Evo VIII; the vehicle interface is unverified.
- Do not treat ERC/DRC as automotive safety evidence.
