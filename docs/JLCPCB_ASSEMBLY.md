# PCBA assembly review — JLCPCB / PCBWay

2026-11-13. This is a review package, not a quote, DFM acceptance or H3/H4
approval. The same BOM/CPL/gerber pack can be quoted by JLCPCB, PCBWay or
another assembler.

## What is machine-assembled

- **139 machine placements**, all top-side: 123 SMT parts plus ordinary
  through-hole connectors/socket (J1–J8, IC3). The pack is
  `hardware/prototype-v1/manufacturing/jlcpcb/`.
- **BOM** `bom-jlcpcb.csv`: Comment, Designator, Footprint, MPN, LCSC Part #.
  LCSC is blank for rows that already have an MPN; the assembler's BOM tool is
  expected to map it to its own library. Nine connector/socket rows have no
  verified MPN yet and are listed in `bom-unresolved.csv`.
- **CPL** `cpl-jlcpcb.csv`: Designator, Mid X, Mid Y, Layer, Rotation. Every
  origin is inside the 100.05 mm outline. All placements are Top.

## What is not machine-assembled

- **XA1 shield headers**: one 24-pad row, one 28-pad row and one 2x17 column.
  The composite footprint has no single physical placement and its origin was
  outside the outline. These are a small hand-solder kit; see
  `bom-hand-solder.csv` and `docs/CONNECTOR_DECISIONS.md`.
- **A1 carrier sockets**: two 1x8 sockets for the Pololu DRV8825 carrier; the
  carrier itself is customer-supplied and plugs in.
- **Arduino Mega 2560**: customer-supplied plug-in; not a BOM item.
- **JP6–JP11**: bare copper solder jumpers, no parts and no assembly operation.
  JP11 is bridged 1–2 and must be cut before external MAP is connected.
- **H1–H3 mounting holes and REF\*\* proto pads**: board-only features.

This split is a deliberate, documented compromise so the manufacturer does not
receive a fake composite placement. It is not a circuit change. If the assembler
offers a header-kitting service, XA1 can be moved back into the machine BOM
later with a per-strip breakdown.

## Cost and DFM

No delivered AUD total is claimed. The quote must include PCB fabrication,
parts, setup, stencil, SMT joints, manual/THT labour, feeder loading, freight and
applicable tax. The owner's target is below A$250 per completed board at a
5-board prototype quantity; that is a target, not a quote.

Ask both manufacturers to confirm:

- actual 100.05 × 100.05 mm outline pricing (do not trim the design to assume a
  100 mm promotion);
- DFM against the 0.05 mm via annular ring and 0.20 mm track / 0.30 mm via rules;
- mixed SMT + THT process and selective/manual soldering;
- whether the nine connector choices are in stock, need an alternative, or must
  be consigned;
- assembly preview for pin 1, polarity, rotation and the negative-Y page-origin
  convention against the top assembly drawing.

## Upload sequence

1. EE approves the nine machine connector choices and the two hand-solder
   items in `docs/CONNECTOR_DECISIONS.md`.
2. Update `bom-jlcpcb.csv` MPNs for those nine rows; regenerate and reconcile
   with `generate.py`.
3. Quote with Gerbers/drill + `bom-jlcpcb.csv` + `cpl-jlcpcb.csv` at 5 boards,
   top-side assembly. Mark the Mega and DRV8825 carrier as customer-supplied /
   not assembled. The hand-solder items are not in the CPL.
4. Review the assembly preview and DFM report. Do not approve substitutions for
   power, analogue, trigger or output parts on pin compatibility alone.
5. Only after the EE review is recorded and the owner explicitly approves H4 may
   a board be ordered. No order, purchase or fabrication has occurred here.

## Files

- `hardware/prototype-v1/manufacturing/gerbers/`
- `hardware/prototype-v1/manufacturing/drill/`
- `hardware/prototype-v1/manufacturing/jlcpcb/bom-jlcpcb.csv`
- `hardware/prototype-v1/manufacturing/jlcpcb/cpl-jlcpcb.csv`
- `hardware/prototype-v1/manufacturing/jlcpcb/bom-hand-solder.csv`
- `hardware/prototype-v1/manufacturing/jlcpcb/bom-unresolved.csv`
- `hardware/prototype-v1/manufacturing/assembly/v0.4.4d-assembly-top.pdf`
- `hardware/prototype-v1/FABRICATION_NOTES.md`
- `docs/CONNECTOR_DECISIONS.md`
- `docs/BROTHER_REVIEW.md`
