# Assembly pack (manufacturer-neutral)

Generated from the KiCad BOM and position file. This is an assembly-data
transform only: no circuit, net, footprint or component-value change.

| Item | Count | Notes |
|---|---:|---|
| Machine-placed references | 139 | 14 ordinary through-hole connectors/socket; rest SMT |
| CPL rows | 139 | all Top; every origin inside the outline |
| Rows with a verified MPN | 130 | LCSC blank by design; assembler maps MPN to its library |
| Connector rows needing an EE choice | 9 | listed in bom-unresolved.csv |
| Hand-solder items | 2 | XA1 shield headers and A1 carrier sockets; no machine placement |
| Bare copper jumpers | 6 | JP6-JP11; no parts, no assembly |
| Customer-supplied plug-ins | 2 | Arduino Mega, Pololu DRV8825 carrier |

The previous composite XA1 origin (110.0, -107.8 mm) was outside the board
outline and is no longer in the CPL. XA1 is instead a hand-solder header kit:
one 24-pad row, one 28-pad row and one 2x17 column. The Pololu carrier itself
is not in the BOM; two 1x8 board-side sockets are the hand-solder item.

## Files

- `bom-jlcpcb.csv` — Comment, Designator, Footprint, MPN, LCSC Part #.
- `cpl-jlcpcb.csv` — Designator, Mid X, Mid Y, Layer, Rotation.
- `bom-hand-solder.csv` — XA1/A1-SOCKET hand-solder items and the bare jumpers.
- `bom-unresolved.csv` — connector rows with no verified MPN; EE decision.
- `generate.py` — reproducible generator; no network access.

## Limits

This is not a quote, DFM approval, H3 or H4 sign-off. LCSC mappings, stock,
prices, rotations and pin-1 polarity must be confirmed by the assembler and
the EE. The board remains a bench prototype.
