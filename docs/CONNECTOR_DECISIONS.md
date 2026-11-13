# Connector decisions for EE review

2026-11-13. This is the short decision list for the electronic-engineer review.
It is not a sourcing approval, and no LCSC stock, price or automotive
qualification is claimed. The machine assembly pack is 139 placements; nine
connector/socket references still need an exact ordering code. Two items are a
small hand-solder kit, not machine placements.

## A. Nine machine-placed connector decisions

These are already in `bom-jlcpcb.csv` and `cpl-jlcpcb.csv`; only the MPN is
missing. JLCPCB or PCBWay can place them once an exact part is chosen. Verify
the datasheet body, lead length, hole fit, plating and process.

| Refs | Qty | Current footprint | Candidate family | Decision |
|---|---:|---|---|---|
| J1, J6, J7 | 3 | `PinHeader_1x05_P2.54mm_Vertical` | Samtec HTSW-105-07-L-S (candidate) | Approve exact ordering code, lead length, housing width and end-to-end fit. The three header-to-header courtyard exclusions put end pins exactly 2.54 mm apart; the chosen housings must butt without exceeding that pitch. |
| J5 | 1 | `PinHeader_1x04_P2.54mm_Vertical` | Samtec HTSW-104-07-L-S (candidate) | Same fit decision; J5 also meets J6 at a perpendicular corner. |
| J8 | 1 | `PinHeader_1x03_P2.54mm_Vertical` | Samtec HTSW-103-07-L-S (candidate) | Same fit decision. |
| J2 | 1 | `PinHeader_1x06_P2.54mm_Vertical` | Samtec HTSW-106-07-L-S (candidate) | Approve exact code and mating access. |
| J3 | 1 | `TerminalBlock_Phoenix_PT-1,5-2-5.0-H_1x02_P5.00mm_Horizontal` | Phoenix MKDSN 1,5/2 1729018 (candidate, not proved footprint-equivalent) | Compare body, wire entry, 5 mm pitch, lead dimensions and screwdriver clearance; approve wire gauge/current. |
| J4 | 1 | `IDC-Header_2x20_P2.54mm_Vertical` | Samtec TST family or equivalent keyed 2x20 | Select the exact keyed 40-contact header and matching ribbon socket as a pair. Verify notch, pin 1, body and hole fit. |
| IC3 | 1 | `DIP-8_W7.62mm_Socket` | Samtec IC family or equivalent | Select the 8-pin, 7.62 mm row-spacing socket; verify retention, lead diameter and conditioner height. |

The prior Harwin `M20-999xx45` candidate is **not accepted** because its
drawing states "End Stackable: No." Do not use it without an explicit EE
disposition and a dimensional check.

## B. Two hand-solder items (not in the machine CPL)

These are deliberately outside `cpl-jlcpcb.csv`; they have no single physical
machine placement in the current footprint. Hand-solder them after the
manufacturer returns the board, or have the brother do them. The Arduino Mega
and Pololu DRV8825 carrier themselves remain plug-in modules.

| Item | Detail | Candidate | Decision |
|---|---|---|---|
| XA1 shield headers | One 24-pad row, one 28-pad row, one 2x17 column; no single composite placement | Samtec TSW/HTSW male header family; exact strips/gender/tail length TBD | Choose the strips, gender (male pins to match the Mega's female headers), tail length and stack height against the real Mega. Confirm the 24/28/2x17 grouping against the footprint and assembly drawing. |
| A1 carrier sockets | Two 1x8 sockets, 2.54 mm, for the Pololu carrier; carrier is customer-supplied | Samtec SSQ-108-01-F-S (candidate) or equivalent | Confirm row spacing, orientation and stack height. Hand-solder two sockets; the carrier plugs in. |

## C. Mechanical / fabrication decisions

1. Three M3 NPTH holes (H1–H3) are present; the top-right corner is supported
   through the Mega standoffs and J3 strain relief. Measure the actual Mega and
   header stack and confirm the standoff length.
2. Confirm the header housing fit at the three courtyard exclusions:
   J1.5–J5.1 at (198.255, 71.24)/(198.255, 68.70); J5.4–J6.5 at
   (198.255, 61.08)/(195.715, 61.08); J6.1–J8.3 at (185.555, 61.08)/(183.015,
   61.08) mm. Nominal dimensions alone do not close these.
3. Confirm the 0.05 mm via annular ring against the chosen board house's DFM
   capability, and quote the actual 100.05 mm outline.
4. Confirm material, copper weight, finish and colour assumptions in
   `hardware/prototype-v1/FABRICATION_NOTES.md`.
5. JP6–JP10 are bridged copper features; JP11 is bridged 1–2. JP11 must be cut
   before connecting an external MAP source to IDC40 pin 21.

## D. What "done" means for this decision list

An EE has either approved an exact ordering code for each machine connector or
explicitly removed that connector from the assembly plan. The two hand-solder
items have a chosen strip/socket and a stack-height check. No substitution of
power, analogue, trigger-conditioning or output-driver parts is approved on pin
compatibility alone.
