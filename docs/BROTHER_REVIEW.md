# Brother review packet — OpenTech-ECU prototype-v1

2026-11-13. This is the human engineering review requested before any board is
ordered. It is not a request to certify the design as automotive-safe. The
board is a bench prototype only: no engine, no vehicle, no injectors or coils.

## 1. What the board is

A 100.05 × 100.05 mm, 2-layer **Arduino Mega 2560 shield** based on the
Speeduino v0.4.4d SMD design. The Mega and the Pololu DRV8825 carrier are
customer-supplied plug-in modules. The board is not a plug-in Evo VIII ECU.

## 2. Current evidence

| Item | Current state |
|---|---|
| Schematic/PCB | Native KiCad 9, fully routed, 0 unconnected, connectivity PASS |
| ERC | 49 items (4 errors, 45 warnings), all dispositioned |
| DRC | 397 reported (31 errors, 366 warnings) + 199 per-item courtyard exclusions |
| Remaining DRC errors | JP6–JP11 intentional solder-bridge/mask items |
| Mechanical | 3 × M3 NPTH holes; top-right corner supported via Mega standoffs and J3 strain relief |
| Assembly | 139 machine placements; XA1 shield headers and A1 sockets are a small hand-solder kit |
| BOM | 140 machine rows in the pack; nine connector/socket rows still need an EE ordering code |
| Independent AI review | Focused hardware review ACCEPT WITH FINDINGS; not a human sign-off |

## 3. Decisions requested from you

### A. Nine machine connector choices

Please choose or reject an exact ordering code for:

1. J1, J6, J7 — 3 × 1x05, 2.54 mm
2. J5 — 1x04, 2.54 mm
3. J8 — 1x03, 2.54 mm
4. J2 — 1x06, 2.54 mm
5. J3 — 2-pin, 5.0 mm terminal block
6. J4 — keyed 2x20 IDC40 header + mating ribbon socket
7. IC3 — DIP-8, 7.62 mm row-spacing socket

Candidates and the exact fit questions are in
[`CONNECTOR_DECISIONS.md`](CONNECTOR_DECISIONS.md). The main mechanical issue is
that J1/J5/J6/J8 end pins are exactly 2.54 mm apart; the housings must butt
without exceeding that pitch. The prior Harwin candidate says "End Stackable:
No," so it needs your explicit decision or replacement.

### B. Two hand-solder items

1. **XA1 shield headers** — one 24-pad row, one 28-pad row, one 2x17 column.
   Choose the exact strips, gender and stack height against the real Mega.
2. **A1 carrier sockets** — two 1x8 sockets for the Pololu DRV8825 carrier;
   the carrier plugs in.

These are outside the machine CPL because the current footprint has no single
physical placement. They are the only hand-solder items. If you prefer, the
assembler can quote a header-kitting service once you choose the strips.

### C. Mechanical / DFM

- Measure the actual Mega + header stack and confirm the standoff length.
- Confirm the three-hole support strategy and J3 strain relief.
- Review the 0.05 mm via annular ring against the chosen board house's DFM.
- Confirm the 100.05 mm outline quote (do not trim it silently).
- Confirm material, copper weight, finish and colour assumptions in
  [`FABRICATION_NOTES.md`](../hardware/prototype-v1/FABRICATION_NOTES.md).
- Confirm JP11 must be cut before external MAP is connected to IDC40 pin 21.

## 3A. Sourcing blockers found by the cost model (2026-11-13)

The 5-board assembly estimate is **A$160.79/board expected** (range
A$127.70–196.57), or **A$220.56/board** with the Mega and DRV8825 plug-ins.
That is under the A$250 target, but it is an estimate, not a quote. A turnkey
order cannot be placed as-is:

- **MPX4250AP MAP sensor:** discontinued and zero JLCPCB stock. It is also the
  largest single part cost (~A$33.89/board). Please approve a footprint- and
  transfer-function-compatible alternative, or a consignment/owner-supplied
  route. Do not substitute on pin compatibility alone.
- **Also zero stock:** LH R974-LP-1 LED, CL21B224KOCNNNC 0.22 µF,
  SSM3K357R MOSFET, AYZ0202AGRLC switch. LM2940S-5.0/NOPB has one unit.
- **No stocked JLCPCB match:** 0ZCC0050FF2C fuse, J4 keyed IDC40, IC3 DIP-8
  socket. These need an approved alternative or consignment.
- **Nine machine connector/socket MPNs** remain your decision; see
  `CONNECTOR_DECISIONS.md`.
- The J3 CSV comma/quoting defect is fixed; all pack CSVs are properly quoted.

The cost and stock evidence is in `docs/HARDWARE_COST.md` and
`AUDIT/COST_ESTIMATE.md`. The estimate is ±25%; a real quote needs the chosen
parts and an upload.

## 4. Files to review

- [`hardware/prototype-v1/`](../hardware/prototype-v1/) — KiCad project, schematic, PCB.
- [`PCB_CHECK_RESULTS.md`](../AUDIT/PCB_CHECK_RESULTS.md) — ERC/DRC dispositions.
- [`HARDWARE_BLOCKERS.md`](../HARDWARE_BLOCKERS.md) — remaining H3 blockers.
- [`hardware/prototype-v1/manufacturing/jlcpcb/`](../hardware/prototype-v1/manufacturing/jlcpcb/) — BOM, CPL, hand-solder and unresolved lists.
- [`CONNECTOR_DECISIONS.md`](CONNECTOR_DECISIONS.md) — the decision list above.
- [`JLCPCB_ASSEMBLY.md`](JLCPCB_ASSEMBLY.md) — quote/upload sequence.
- [`FABRICATION_NOTES.md`](../hardware/prototype-v1/FABRICATION_NOTES.md) — assumptions.
- Top/bottom assembly PDFs and fab drawing under `hardware/prototype-v1/manufacturing/`.

## 5. Sign-off

Please record one of:

- **Approved for prototype fabrication** — with any conditions and chosen
  connector ordering codes.
- **Approved with changes** — list the changes; no order until resolved.
- **Not approved** — list the blocking findings.

A signature/date and the exact files/revision reviewed are required. This is the
independent engineering review needed for the project's H4 gate; the owner must
still give explicit approval before fabrication.

## 6. Safety boundary

This board must not be connected to the Evo VIII, an engine, injectors, coils or
a fuel system. First power-up, if approved later, is bench-only with a
current-limited supply, fuses, dummy loads and an emergency disconnect. Passing
KiCad checks and an AI review are not automotive safety certification.
