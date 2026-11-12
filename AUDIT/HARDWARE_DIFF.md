# Hardware Diff — Prototype v1 vs Upstream Speeduino v0.4.4d

| Field | Value |
|---|---|
| Document ID | AUD-HW-DIFF-01 |
| Status | Format-only conversion; no intentional electrical change |
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
| D-06 | Mark 150 unannotated proto-area pads `exclude from position files` / `exclude from BOM` | Assembly metadata | None | Position file now 141 real placements, 0 `REF**`; DRC unchanged 609 |

**No component value, net assignment, footprint assignment, track, via, board
outline or mounting-hole edit was made.**

## 2. Why the zone re-fill is not a circuit change

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

## 3. Rule-set note

The upstream project file (`v0.4.4d.kicad_pro`) is preserved in spirit: its
board rules are identical after the KiCad 9 save (for example
`min_via_annular_width = 0.05 mm`). The legacy board-local setup used 0.10 mm
annular minimum, so the first DRC run reported 766 violations and the post-save
run reports 609. This is a migration from the legacy board-local rule to the
upstream project rule, not a rule suppression. No DRC exclusion was added.

## 4. Review checklist for a human reviewer

1. Confirm the preserved upstream tree matches `SHA256SUMS`.
2. Open `prototype-v1/v0.4.4d.kicad_pro` and inspect the six hierarchical
   sheets and PCB; confirm no value/net/footprint changes are visible.
3. Re-run `hardware/tools/check_prototype_v1.sh`.
4. Inspect the re-filled zones on F.Cu/B.Cu; confirm GND/PGND pours are
   continuous and no new isolation or short exists.
5. Review the exceptions in `AUDIT/PCB_CHECK_RESULTS.md` §3/§5.
6. Do not approve fabrication until the H2/H3 blockers in
   `HARDWARE_BLOCKERS.md` are closed or explicitly accepted.
