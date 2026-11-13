# BOM Review — Prototype v1 (Speeduino v0.4.4d SMD)

| Field | Value |
|---|---|
| Document ID | AUD-HW-BOM-01 |
| Status | **Incomplete for release.** No prices, 11 rows lack MPN. |
| Date | 2026-11-13 |
| Source BOM | `hardware/upstream-speeduino/v0.4.4d/v0.4.4d BOM.csv` |
| Generated BOM | `hardware/prototype-v1/manufacturing/assembly/v0.4.4d-bom.csv` |
| Underlying design | Speeduino v0.4.4d SMD, upstream commit `ee6a9242834891877744ff2137663b6ca8d79ed6` |
| Related | `HARDWARE_BLOCKERS.md` `HW-BOM-01`, `HW-BOM-02` |

> **Status.** The BOM is real in the sense that it is generated from the native
> schematic and carries manufacturer part numbers for most active/passive parts.
> It is **not** a release BOM: 11 grouped rows (17 placements) have no MPN, no
> supplier prices have been collected in AUD, and no lifecycle/derating review
> has been performed. Nothing was purchased.

## 1. What was generated

```bash
kicad-cli sch export bom \
  --fields 'Reference,Value,Footprint,${QUANTITY},${DNP},Manufacturer_Name,Manufacturer_Part_Number' \
  --labels 'Refs,Value,Footprint,Qty,DNP,Manufacturer,MPN' \
  --group-by 'Value,Footprint,Manufacturer,MPN' \
  --output v0.4.4d-bom.csv v0.4.4d.kicad_sch
```

Observed result:

| Metric | Value |
|---|---:|
| Grouped BOM rows | 47 |
| Total populated placements | 147 |
| Rows with manufacturer + MPN | 36 |
| Rows without MPN | **11** |
| Placements without MPN | **17** |
| Unannotated proto-area pads (`REF**`, `Pin_D1.0mm_L10.0mm`, no net) | 150 |
| Board footprints total | 297 |

The 150 `REF**` footprints are a 1 mm drill / 2 mm copper prototyping grid with
no net. They are intentionally not BOM items, but they **are** unannotated
footprints on the PCB and must be explained in the assembly drawing. They are
not counted in the 147 populated placements.

## 2. Rows requiring an MPN

| Refs | Value | Qty | Why it matters |
|---|---|---:|---|
| A1 | `Pololu_Breakout_DRV8825` | 1 | Stepper-driver module (idle/stepper function); a specific Pololu or compatible module must be pinned. |
| IC3 | `Trig Cond Socket` | 1 | DIP-8 socket for the trigger-conditioning option; part must match the installed IC. |
| J1, J6, J7 | `Conn_01x05` | 3 | 5-pin headers; pitch/current/terminal type must be fixed. |
| J2 | `Conn_01x06` | 1 | 6-pin header. |
| J3 | `Conn_01x02` | 1 | Phoenix-style 2-pin terminal block; exact series/current rating required. |
| J4 | `Conn_02x20_Counter_Clockwise` | 1 | **IDC40** interface; keying, strain relief and mating connector must be specified. |
| J5 | `Conn_01x04` | 1 | 4-pin header. |
| J8 | `Conn_01x03` | 1 | 3-pin header. |
| JP6–JP10 | `SolderJumper_2_Bridged` | 5 | Solder-jumper footprints; generic PCB feature, not a purchased part, but the BOM should state "PCB feature / no order". |
| JP11 | `Jumper_3_Bridged12` | 1 | Same as above. |
| XA1 | `Arduino_Mega2560_Shield` | 1 | The Mega 2560 shield itself (or the mating header set) must be specified; this is the firmware platform. |

**Minimum to close `HW-BOM-01`:** assign real MPNs for A1, IC3, J1–J8, XA1; mark
JP6–JP11 explicitly as PCB features rather than purchased parts (or assign the
appropriate solder-jumper part if they are to be populated).

## 3. Sourcing constraints (preliminary)

These are review rules, not final substitutions:

- Do not substitute the injector/ignition driver ICs, VR conditioner, MAP sensor
  or power-protection devices solely on pin compatibility. The upstream design
  depends on their electrical behaviour (clamp voltage, drive strength, input
  threshold, flyback).
- The IDC40 (J4) is part of the bench interface contract; a different connector
  series changes the breakout and harness design.
- The Arduino Mega 2560 shield is the firmware target. A different MCU board
  invalidates the `megaatmega2560` firmware path and the pin mapping.
- Solder jumpers JP6–JP11 are configuration features; populating them as
  permanent bridges changes the default configuration. Record the intended
  default before assembly.
- The 150 proto-area pads are not components. Do not add them to a purchase
  order.

## 4. What is still missing

1. **MPNs** for the 11 rows above (`HW-BOM-01`).
2. **Dated AUD prices** and supplier/alternate supplier for every row, including
   shipping/tax/assembly caveats (`HW-BOM-02`).
3. **Lifecycle/availability** check for each MPN.
4. **Derating/thermal review** for power, driver and clamp parts.
5. **Assembly method and risk class** per row (SMT, hand-solder, module,
   connector, PCB feature).
6. **JP6–JP11 assembly intent**: the position file now contains 141 real
   placements and no `REF**` proto pads; the six solder jumpers are config
   features and are not machine placements. Confirm whether they should be
   bridged at assembly or left for the builder.
7. **Purchase-vs-build comparison** (assembled v0.4.4d board vs bare PCB +
   parts), under the A$250 target if credible.

No quoted price is stated in this document because none has been obtained. This
is deliberate: inventing a price would violate the project's provenance rules.
The catalogue prices in the appended sourcing note are not quotations.

## HW-BOM-01/02 sourcing follow-up — 2026-11-13

Still OPEN. See docs/JLCPCB_ASSEMBLY.md and docs/HARDWARE_COST.md. No verified LCSC mappings or completed AUD quote; no parts substituted. Harwin M20-9990545 (J1/J6/J7), M20-9990645 (J2), M20-9990445 (J5), M20-9990345 (J8) are candidates only. Manufacturer product information reproduced at https://www.farnell.com/datasheets/4376834.pdf gives 3-position nominal length 7.62 mm but explicitly says End Stackable: No. Drawing/tolerance confirmation remains required before accepting header courtyard exclusions. No approved alternatives. A1 Pololu item 2133 is a customer plug-in; board-side sockets remain unresolved. JP6–JP11 are copper features, not purchased placements. IC3, J3, keyed J4 and mating cable socket, XA1 header set remain unsourced. H3/H4 unchanged.

## Sourcing facts verified — 2026-11-13 (read-only catalogue check)

Query: JLCPCB parts catalogue API, one lookup per BOM MPN
(`POST https://jlcpcb.com/api/overseas-pcb-order/v1/shoppingCart/smtGood/selectSmtComponentList`).
These are **catalogue prices and stock at that timestamp**, not quotations, and
not approval of any part. Full table: `AUDIT/COST_ESTIMATE.md` §3.

| Fact | Detail |
|---|---|
| Catalogue mapping | 34 of the 35 unique MPN values resolve to a catalogue part with an LCSC code. |
| `VNLD5090-E` | The exact order code is not matched; `VNLD5090TR-E` (C222209) is, at USD 2.5326, stock 30. Tape/reel suffix difference needs EE confirmation. |
| End-of-life | `MPX4250AP` (C2802676, USD 23.1821, stock 0) and `LH R974-LP-1` (C2899895, stock 0) are marked "no longer manufactured". |
| Zero stock | `CL21B224KOCNNNC` (C296721), `SSM3K357R` (C7558097), `AYZ0202AGRLC` (C221564). |
| Near-zero stock | `LM2940S-5.0/NOPB` (C133956) 1 unit; `S110FA` (C556397) 119; `VNLD5090TR-E` 30; `SP720ABTG` (C434726) 479. |
| Not in library | `0ZCC0050FF2C` fuse: no exact match; the nearest `0ZCC…` parts all show stock 0. A keyed 2×20 IDC header and a 7.62 mm DIP-8 socket also have no stocked match (the generic DIP-8 hit is marked "no longer manufactured"). |
| Basic library | Of the 35 machine MPNs, 33 are extended catalogue parts, one is **basic** (`CC0805KRX7R9BB104`, C49678) and one (the fuse) is absent from the library. |
| Connector representatives | `Phoenix 1729018` C3820334 USD 0.5612, stock 81; `HTSW-105-07-G-S` C3337223 USD 0.6332, stock 16; `SSQ-108-01-F-S` C7019014 USD 1.4643, stock 3. These are cost representatives; the nine MPNs remain unapproved. |
| Distributor cross-check | Not possible this round: Digi-Key, Mouser, Element14 and Octopart returned 403/access-denied; LCSC pages are client-rendered. |
| File-format defect | The `J3` row has an unquoted comma in its footprint field, so a naive CSV parser shifts columns and reads `5-2-5.0-H_1x02_P5.00mm_Horizontal` as the MPN. Quote the field before any assembler upload. The row is still correctly an unresolved connector. |

Consequence for `HW-BOM-01`: the BOM is still not releasable. A turnkey order
cannot be placed as-is because five lines have zero stock and one line (the fuse)
is absent from the assembler's library. Those need an approved alternative,
pre-order or consignment before a quote. Cost consequence is in
`docs/HARDWARE_COST.md`; no price here is a supplier commitment.
