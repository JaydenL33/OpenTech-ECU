# Hardware cost model — assembled bench prototype (5 boards)

| Field | Value |
|---|---|
| Document ID | AUD-HW-COST-01 |
| Status | **Calculated estimate, not a quotation.** No files uploaded, no order placed, no payment. |
| Date | 2026-11-13 |
| Target | Below **A$250 per completed board**, 5-board prototype batch |
| Result (expected) | **A$160.79 per board / A$803.97 for 5 boards**; range A$127.70–A$196.57 |
| Currency | 1 USD = **1.4618 AUD** (RBA, 12 Nov 2026) |
| Board | Speeduino v0.4.4d SMD, 100.05 × 100.05 mm, 2-layer, 1.6 mm, 1 oz, HASL |
| Assembly pack | 139 machine placements, top side: 311 SMT joints + 99 THT joints |
| Related | `docs/JLCPCB_ASSEMBLY.md`, `docs/CONNECTOR_DECISIONS.md`, `AUDIT/COST_ESTIMATE.md`, `AUDIT/BOM_REVIEW.md` |

> **What this is.** A bottom-up estimate built from dated, published prices and
> a published assembly fee schedule, with every uncertain input labelled. It is
> **not** a manufacturer quote. Only an upload quote (Gerbers + BOM + CPL) can
> become one. Five inputs are estimates: PCB fabrication, freight, the unstocked
> connector/socket and fuse lines, the feeder-loading count, and import tax
> handling. Those are listed in §8.

## 1. Result

| Case | Basis | 5 boards (AUD) | Per board (AUD) |
|---|---|---:|---:|
| **Low** | 18 extended part types, generic hand-solder parts, USD 8 fab, USD 12 freight | 638.51 | **127.70** |
| **Expected** | 33 extended SMT types, Samtec/Phoenix representative parts, USD 18 fab, USD 25 freight | 803.97 | **160.79** |
| **High** | 41 extended types (feeder fee on the seven THT types and a substituted fuse too), parts +15%, USD 35 fab, USD 45 freight | 982.87 | **196.57** |

The expected case is **A$89 below the A$250 target** per board. The high case is
still A$53 below it. The target is met on this model, but the margin is not large
enough to absorb a bad fabrication quote *and* a parts shortage at the same time.

Included in "per board": PCB fabrication, all parts, assembly labour and fees,
stencil, setup, freight to Australia and 10% GST. Excluded: the Arduino Mega
2560 and Pololu DRV8825 carrier (customer-supplied plug-ins), test equipment,
enclosure, harness and the owner's hand-soldering time.

Adding the two plug-ins at verified prices (Pololu USD 15.95; Mega ≈ GBP 16)
gives **A$220.56 per board** — still under target, with A$29 of headroom.

## 2. Scope and method

- 5 boards, one order, one top side, Economic PCBA (JLCPCB's prototype tier).
- Every machine-placed line is priced from the **JLCPCB parts catalogue** on
  2026-11-13, i.e. the price JLCPCB would charge for that part in an order, not
  a distributor list price.
- Joint counts are **derived from the PCB**, not assumed: 311 numbered SMD pads
  and 99 numbered plated through-hole pads across the 139 CPL designators
  (`AUDIT/COST_ESTIMATE.md` §2).
- The nine unresolved connector/socket rows are priced with **representative
  equivalents** and flagged. They are not approved MPNs; they are a cost bracket
  for a decision that is still open.
- The two hand-solder items are priced as parts only; the labour is the owner's
  or the brother's (no cash cost modelled).
- Australian GST at 10% is modelled on goods plus freight. JLCPCB may collect it
  at checkout; otherwise it is payable at import. Neither was verified. All
  figures in this document are **landed** (freight and GST included); the
  manufacturers' list prices exclude both.

## 3. Verified inputs

| Input | Value used | Status | Source (accessed 2026-11-13) |
|---|---|---|---|
| USD/AUD | 1.4618 (RBA: USD 0.6841 per AUD, 12 Nov 2026) | Verified | https://www.rba.gov.au/statistics/frequency/exchange-rates.html |
| Cross-check | 1.4661 (ECB reference, 12 Nov 2026) | Verified | https://api.frankfurter.dev/v1/latest?base=USD&symbols=AUD |
| JLCPCB Economic setup | USD 8.18/order | Verified | https://jlcpcb.com/help/article/pcb-assembly-price |
| JLCPCB stencil | USD 1.53/order | Verified | same |
| JLCPCB SMT joint | USD 0.0016/joint | Verified | same |
| JLCPCB manual (THT) joint | USD 0.0164/joint | Verified | same |
| JLCPCB hand-soldering labour | USD 3.58/order | Verified | same |
| JLCPCB extended-part feeder | USD 3.07/type (Economic) | Verified | same |
| JLCPCB parts-placement confirmation | USD 0.45/order | Verified | same |
| JLCPCB packing | free (Economic) | Verified | same |
| Large-board surcharge | not applicable (5 × 100.1 cm² = 500 cm² < 650 cm²) | Verified | same |
| JLCPCB part prices/stock | per line in §4 | Verified catalogue, 2026-11-13 | `POST https://jlcpcb.com/api/overseas-pcb-order/v1/shoppingCart/smtGood/selectSmtComponentList` |
| JLCPCB 2-layer entry price | "from $2" entry offer; widget defaults to 5 pcs | Advertised; size cap not stated on that page | https://jlcpcb.com/pcb-fabrication/fr4-pcb |
| PCBWay entry price | "from $29 / 20 pcs PCBA", stencil offer, "$5 / 10 pcs" PCB | Advertised, not this board | https://www.pcbway.com/pcb-assembly.html |
| Pololu DRV8825 carrier 2133 | USD 15.95 (1), 14.67 (5), in stock | Verified | https://www.pololu.com/product/2133 |
| Assembled 0.4.4d (US) | USD 139.99 no MAP / 159.99 with 4-bar MAP | Verified listing | https://openlogicefi.com/shop/p/044d |
| Assembled 0.4.4C (UK) | GBP 155, +GBP 19 VR conditioner, +GBP 16 Mega | Verified listing | https://diy-efi.co.uk/product/speeduino_0-4-4_assembled |
| Official store | bare v0.4 PCB USD 10.00; **no assembled 0.4.4d listed** | Verified | https://speeduino.com/shop/12-speeduino-boards |

The JLCPCB fee page states "Last updated on Oct 22, 2026". All dollar amounts on
it are USD as published on the English site.

## 4. Itemised cost at 5 boards

Unit prices are USD from the JLCPCB catalogue unless marked *est.* Rows are
rounded to the cent; subtotals use unrounded unit prices, so a column may differ
from the sum of its rounded rows by a cent or two.

### 4.1 Machine-placed BOM lines (34 catalogue-priced MPN rows, 128 placements)

The 35th machine MPN, the 0ZCC0050FF2C fuse, is priced in §4.3 because it is
not in the JLCPCB library. The nine unresolved connector/socket designators are
in §4.2.

| Line | Qty/board | Unit USD | 5 boards USD | LCSC | Note |
|---|---:|---:|---:|---|---|
| RC0805FR-071KL 1k | 18 | 0.0030 | 0.27 | C95781 | |
| CC0805KRX7R9BB104 0.1uF | 13 | 0.0138 | 0.90 | C49678 | JLCPCB basic part |
| RC0805FR-07100KL 100k | 12 | 0.0038 | 0.23 | C96346 | |
| RC0805FR-072K4L 2.4k | 9 | 0.0086 | 0.39 | C114521 | |
| ERJ-6ENF4700V 470R | 9 | 0.0071 | 0.32 | C413274 | |
| LH R974-LP-1 LED | 8 | 0.1473 | 5.89 | C2899895 | **EOL; stock 0** |
| CL21B224KOCNNNC 0.22uF | 6 | 0.0192 | 0.58 | C296721 | **stock 0** |
| VNLD5090TR-E | 4 | 2.5326 | 50.65 | C222209 | stock 30 |
| S110FA | 4 | 0.5039 | 10.08 | C556397 | stock 119 |
| RC1206FR-0710RL 10R | 4 | 0.0238 | 0.48 | C108080 | |
| 1N4448WX-TP | 4 | 0.0164 | 0.33 | C712353 | |
| SSM3K357R | 3 | 0.0398 | 0.60 | C7558097 | **stock 0** |
| M20-9990345 (JP1–JP3) | 3 | 0.2149 | 3.22 | C3337081 | rejected as connector candidate (not end-stackable); retained as current BOM part for JP1–JP3 |
| CC0805KKX7R7BB105 1uF | 3 | 0.0383 | 0.57 | C94122 | |
| RT0805BRD072K49L 2.49k | 2 | 0.0429 | 0.43 | C375522 | |
| RC0805JR-0710KL 10k | 2 | 0.0053 | 0.05 | C100047 | |
| RC0805FR-07220RL 220R | 2 | 0.0045 | 0.04 | C114519 | |
| MM3Z5V1ST1G | 2 | 0.0284 | 0.28 | C236206 | |
| M20-9990245 (JP4–JP5) | 2 | 0.1293 | 1.29 | C3338413 | candidate only |
| IXDN602SIATR | 2 | 1.4659 | 14.66 | C431679 | stock 1,179 |
| CC0805KRX7R9BB471 470pF | 2 | 0.0094 | 0.09 | C107152 | |
| CC0805KRX7R9BB103 0.01uF | 2 | 0.0074 | 0.07 | C83170 | |
| TAJB476K006RNJ 47uF | 1 | 0.3021 | 1.51 | C7207 | |
| T491D106K050AT 10uF | 1 | 1.4643 | 7.32 | C117038 | |
| SP720ABTG | 1 | 2.2217 | 11.11 | C434726 | stock 479 |
| SMBJ40A-Q | 1 | 0.1425 | 0.71 | C7462772 | |
| RC0805FR-073K9L 3.9k | 1 | 0.0063 | 0.03 | C114221 | |
| RC0805FR-072KL 2k | 1 | 0.0046 | 0.02 | C114572 | |
| RC0805FR-07120RL 120R | 1 | 0.0061 | 0.03 | C114245 | |
| **MPX4250AP** | 1 | 23.1821 | **115.91** | C2802676 | **discontinued; stock 0** |
| LM2940S-5.0/NOPB | 1 | 3.2426 | 16.21 | C133956 | **stock 1** |
| CL21A226KQCLRNC 22uF | 1 | 0.0791 | 0.40 | C3909963 | |
| B130-13-F | 1 | 0.0915 | 0.46 | C134423 | |
| AYZ0202AGRLC | 1 | 1.8127 | 9.06 | C221564 | **stock 0** |
| **Subtotal** | | | **254.21** | | |

### 4.2 Nine unresolved connector/socket rows — representative, not approved

These have no verified MPN (`bom-unresolved.csv`). Prices are for the closest
JLCPCB part (in stock where possible); two lines are estimates. The EE decision
may change them.

| Line | Qty/board | Unit USD | 5 boards USD | LCSC | Note |
|---|---:|---:|---:|---|---|
| HTSW-105-07-G-S (J1/J6/J7) | 3 | 0.6332 | 9.50 | C3337223 | -G-S priced; candidate is -L-S; not approved |
| HTSW-106-07-L-S (J2) | 1 | 0.7444 | 3.72 | C3324216 | not approved |
| HTSW-104-xx-S (J5) | 1 | 0.6119 | 3.06 | C3332127 | -08 tail priced; -07 not quoted |
| HTSW-103-07-L-S (J8) | 1 | 0.3375 | 1.69 | C3333112 | not approved |
| Phoenix 1729018 (J3) | 1 | 0.5612 | 2.81 | C3820334 | same MPN as the candidate |
| IDC 2x20 keyed (J4) | 1 | 1.50 *est.* | 7.50 | — | **no stocked JLCPCB match** |
| DIP-8 7.62 mm socket (IC3) | 1 | 0.20 *est.* | 1.00 | — | **no stocked JLCPCB match** |
| **Subtotal** | | | **29.27** | | |

### 4.3 Fuse and hand-solder parts

| Line | Qty/board | Unit USD | 5 boards USD | LCSC | Note |
|---|---:|---:|---:|---|---|
| 0ZCC0050FF2C polyfuse | 2 | 0.50 *est.* | 5.00 | — | **not in JLCPCB library**; consign or substitute |
| XA1 28-pin TSW-128-07-T-S | 1 | 1.1567 | 5.78 | C3329247 | stock 0 |
| XA1 24-pin TSW-124-07-L-S | 1 | 1.5232 | 7.62 | C3348299 | stock 0 |
| XA1 2x17 TSW-117-07-L-D | 1 | 2.9612 | 14.81 | C3345936 | stock 23 |
| A1 SSQ-108-01-F-S 1x8 socket | 2 | 1.4643 | 14.64 | C7019014 | EE candidate; stock 3 |
| **Subtotal** | | | **47.85** | | |

### 4.4 Assembly fees (JLCPCB Economic, 5 boards)

| Fee | Arithmetic | USD |
|---|---|---:|
| Setup | per order | 8.18 |
| Stencil | per order | 1.53 |
| SMT joints | 311 × USD 0.0016 × 5 | 2.49 |
| Manual/THT joints | 99 × USD 0.0164 × 5 | 8.12 |
| Hand-soldering labour | per order | 3.58 |
| Placement confirmation | per order | 0.45 |
| Feeder loading, extended parts | 33 × USD 3.07 | 101.31 |
| **Subtotal** | | **125.66** |

The 33 figure is the 35 machine MPN types, minus the one JLCPCB **basic** part
(0.1 µF, C49678), minus the fuse, which is not in the JLCPCB library and is
assumed to be consigned and hand-placed rather than feeder-loaded. The seven
through-hole connector types in §4.2 are also hand-placed, so they are not
charged a feeder fee in the expected case. If the fuse is replaced by an
in-library part and JLCPCB charges feeder loading for the seven THT types too,
the count is 41 and the fee is 41 × 3.07 = USD 125.87; that is the high case.

### 4.5 Fabrication, freight and tax

| Line | USD | Status |
|---|---:|---|
| Parts (§4.1–4.3) | 331.33 | Catalogue prices + estimates |
| Assembly (§4.4) | 125.66 | Published fee schedule |
| PCB fabrication, 5 pcs, 100.05 × 100.05 mm, 2-layer, 1.6 mm, 1 oz, HASL | 18.00 | **Estimate** — see §8 |
| Freight to Australia (DHL-class, ~0.5 kg) | 25.00 | **Estimate** |
| Goods + freight subtotal | 499.99 | |
| GST 10% | 50.00 | Assumes collection at sale/import |
| **Total, 5 boards** | **549.99** | **A$803.97** |
| **Per board** | **110.00** | **A$160.79** |

## 5. Arithmetic at 5 boards

```text
Parts, 34 catalogue MPN rows            USD 254.21
Parts, 7 connector representatives      USD  29.27
Parts, fuse + hand-solder items         USD  47.85
                                        -----------
Parts subtotal                          USD 331.33   = USD 66.27/board

Setup/stencil/confirm/hand labour       USD  13.74
Feeder loading (33 × 3.07)              USD 101.31
Joint fees (SMT 2.49 + THT 8.12)        USD  10.61
                                        -----------
Assembly subtotal                       USD 125.66   = USD 25.13/board

PCB fabrication (est.)                  USD  18.00
Freight (est.)                          USD  25.00
                                        -----------
Goods + freight                         USD 499.99
GST 10%                                 USD  50.00
                                        -----------
Total for 5 boards                      USD 549.99   = A$803.97
Per board                               USD 110.00   = A$160.79
```

**Setup amortisation** (excluding GST). The per-order charges — setup 8.18,
stencil 1.53, confirmation 0.45, hand-soldering 3.58 and feeder loading 101.31 —
total USD 115.05, or **A$33.63 per board at 5 boards**. At 1 board they would be
A$168.17 per board; at 10 boards A$16.82. This is why the batch size matters more
than any single component except the MAP sensor.

**Per-type feeder amortisation.** Each extended part type costs USD 3.07 per
order regardless of quantity: A$0.90/board at 5 boards, A$0.45/board at 10.

## 6. Biggest cost drivers (expected case, A$160.79/board)

| Driver | A$/board | Share | Controllable? |
|---|---:|---:|---|
| MPX4250AP MAP sensor | 33.89 | 21% | Yes — substitute, or owner-supply |
| Feeder loading (33 extended types) | 29.62 | 18% | Yes — consolidate to basic parts |
| VNLD5090TR-E ×4 | 14.81 | 9% | No — driver IC; substitution needs EE review |
| Other parts (passives, LEDs, diodes, ICs) | 35.64 | 22% | Partly — basic-part substitution |
| Hand-solder header/socket parts | 12.53 | 8% | Partly — generic strips vs Samtec |
| Assembly fees other than feeder | 7.12 | 4% | No |
| PCB fabrication | 5.26 | 3% | Partly — quote the real outline |
| Freight | 7.31 | 5% | Partly — consolidate orders |
| GST 10% | 14.62 | 9% | No |

The two single largest controllable items are the MAP sensor and the extended-part
count. Together they are ~A$63/board, 39% of the total.

## 7. Ways to stay under A$250/board

The expected case already passes, so these are margin or risk measures, not
rescues. Values are A$ per board at 5 boards, including GST.

| Lever | Effect | Caveat |
|---|---:|---|
| Keep the 5-board batch (do not order 1) | per-order charges fall from A$185 to A$37.00/board | already assumed |
| Map 0805 passives to JLCPCB **basic** parts | up to −19.75 (20 fewer types) | values/tolerance/dielectric must match; EE sign-off |
| Owner supplies the MAP sensor, or an in-production sensor is approved | −37.27 | MPX4250AP is discontinued; a substitute needs a footprint/transfer-function review |
| Generic 2.54 mm strips/sockets instead of Samtec for XA1/A1 | −9 to −11 | stack height and retention must be checked against the real Mega |
| Order 10 boards instead of 5 | −22.68/board | doubles cash to ~A$1,381; only useful if spares are wanted |
| Consolidate to one order | already assumed | splitting fab and assembly adds a second freight and setup |

**Do not** assume the US$2 / US$5 entry prices apply. Entry offers are for
small 2-layer boards (typically 100 × 100 mm or less) and this outline is
100.05 mm, so it needs its own quote.

## 8. Unknowns, estimates and confidence

| Input | Value used | Why it is not verified | Confidence |
|---|---|---|---|
| PCB fabrication | USD 18 (8–35) | 100.05 mm is at or over the usual 100 mm entry-offer size class; both houses quote by upload only | Low |
| Freight | USD 25 (12–45) | no checkout/address entered; depends on weight and service | Low |
| Feeder-loading count | 33 (18–41) | JLCPCB's BOM tool decides which MPNs map to basic vs extended; THT treatment unconfirmed | Medium |
| J4 IDC and IC3 socket | USD 1.50 / 0.20 | no stocked JLCPCB match; exact keyed header not selected | Low |
| Fuse | USD 0.50 each | 0ZCC0050FF2C not in the JLCPCB library | Low |
| Connector representatives | §4.2 | the nine MPNs are an open EE decision | Medium |
| GST collection | 10% added | depends on JLCPCB's AU tax registration and checkout | Medium |
| Part availability | catalogue prices | 5 lines have zero stock (2 EOL) and 4 more have under 500 units | **Low** |

**Stock/EOL facts (2026-11-13, JLCPCB catalogue).** Zero stock: MPX4250AP
(discontinued), LH R974-LP-1 (discontinued), CL21B224KOCNNNC, SSM3K357R,
AYZ0202AGRLC. Near-zero: LM2940S-5.0/NOPB (1), S110FA (119), VNLD5090TR-E (30),
SP720ABTG (479). A turnkey order **cannot be placed as-is**; these lines need
pre-order, consignment or an approved alternative. The model prices them at
catalogue value and therefore understates sourcing risk, not unit price.

**Supply-risk sensitivity.** If the MAP sensor has to be bought at twice the
catalogue price, the high case rises to ~A$234/board — still under A$250, with
A$16 of headroom.

**Overall confidence: ±25%.** Parts and the assembly fee schedule are solid;
fabrication and freight are planning estimates. The result would have to be
~55% worse than expected to breach A$250.

## 9. Comparison

| Path | Basis | Landed total (AUD) | Per board (AUD) | Status |
|---|---|---:|---:|---|
| Bare PCB only, 5 pcs | fab + freight + GST, no parts | 69.14 for 5 | 13.83 | Estimate |
| **JLCPCB assembly, 5 boards** | §5 | **803.97 for 5** | **160.79** | Estimate |
| JLCPCB + Mega + DRV8825 | + Pololu USD 15.95, Mega GBP 16 | 1,102.79 for 5 | 220.56 | Estimate |
| PCBWay assembly, 5 boards | no published fee schedule | not quoted | not quoted | **Unquoted** |
| Assembled 0.4.4d, OpenLogicEFI (US) | USD 139.99 no MAP, + est. freight, + GST | 273.34 for 1 | 273.34 | Listing verified |
| Assembled 0.4.4d + 4-bar MAP | USD 159.99, + est. freight, + GST | 305.51 for 1 | 305.51 | Listing verified |
| Assembled 0.4.4C, DIY-EFI (UK) | GBP 155 + GBP 16 Mega + est. freight + GST | 407.25 for 1 | 407.25 | Listing verified |

The three assembled-board rows are single-unit purchases; the total column is
not a 5-board price and shows one landed board. Freight for those rows is an
estimate (USD 30 / GBP 20), because neither seller publishes an Australia rate
without checkout.

**PCBWay.** Its public pages advertise "$29 for 20 pcs PCBA", a stencil offer
and "$5/10 pcs" fabrication, but publish no per-joint/per-type fee table. A real
comparison needs the same Gerber/BOM/CPL upload. At the expected parts,
fabrication and freight, PCBWay's assembly fees could rise to **USD 403 per
order** — about A$589 before GST, A$648 of the A$1,250 landed budget — before
the result reached A$250/board. JLCPCB's modelled assembly is USD 125.66.
PCBWay would have to be more than three times JLCPCB on assembly to lose on
cost. It may still win on process (e.g. accepting consigned parts or the fuse),
which is a capability question, not a price one.

**Buying instead of building.** A single assembled 0.4.4d lands at ~A$273
(no MAP) to ~A$306 (with MAP) including estimated freight and GST. The 5-board
build is ~A$161/board, so building is ~A$110/board cheaper — but it carries
assembly risk and no functional test, and the comparison board is tested.
Neither seller includes the Mega or the DRV8825 carrier. The official Speeduino
store currently lists only bare boards and the Dropbear, not an assembled
0.4.4d.

## 10. What to ask the manufacturer

1. **Outline price.** Quote the actual 100.05 × 100.05 mm outline, 2-layer,
   1.6 mm, 1 oz, HASL, 5 pcs — confirm it is not priced as the 100 × 100 mm
   promotional size.
2. **DFM.** Confirm the 0.05 mm via annular ring, 0.20 mm track/space and
   0.30 mm via are standard-capability, with no engineering surcharge.
3. **Mixed SMT/THT.** Confirm the Economic/Standard tier accepts 311 SMT joints
   plus 99 through-hole joints (14 connectors, DIP-8 socket, MAP sensor tabs),
   and whether the THT work is manual, pin-in-paste, or both, and at what fee.
4. **Stock and alternates.** Confirm stock/alternates for the nine unresolved
   connectors, the five zero-stock lines (two EOL) and the four low-stock lines;
   ask whether consignment is accepted in the chosen tier, and whether
   0ZCC0050FF2C can be consigned.
5. **Assembly preview.** Review pin 1, polarity and rotation for all 139
   placements, and confirm the CPL's negative-Y convention and Top layer against
   the assembly drawing.
6. **Landed cost.** Confirm freight, insurance, GST treatment and any import
   charges to Australia, and whether the extended-part count is 33 or 41.
7. **Basic-part substitution.** Ask whether the 0805 passives can be mapped to
   the basic library without changing value, tolerance or dielectric.

## 11. Sources and access dates

All accessed **2026-11-13** unless stated. Prices are as published on those
pages; no page was cached or altered.

| # | Source | Used for |
|---|---|---|
| 1 | https://www.rba.gov.au/statistics/frequency/exchange-rates.html | FX, 12 Nov 2026: USD 0.6841/AUD |
| 2 | https://api.frankfurter.dev/v1/latest?base=USD&symbols=AUD | FX cross-check: 1.4661 |
| 3 | https://jlcpcb.com/help/article/pcb-assembly-price | JLCPCB fee schedule (updated 22 Oct 2026) |
| 4 | `POST https://jlcpcb.com/api/overseas-pcb-order/v1/shoppingCart/smtGood/selectSmtComponentList` | Part prices, stock, basic/extended, LCSC codes |
| 5 | https://jlcpcb.com/pcb-fabrication/fr4-pcb | "from $2", 5 pcs, 100 × 100 mm default |
| 6 | https://www.pcbway.com/pcb-assembly.html | "$29/20 pcs", free stencil, "$5/10 pcs" |
| 7 | https://www.pololu.com/product/2133 | DRV8825 carrier USD 15.95 |
| 8 | https://openlogicefi.com/shop/p/044d | Assembled 0.4.4d USD 139.99/159.99 |
| 9 | https://diy-efi.co.uk/product/speeduino_0-4-4_assembled | Assembled 0.4.4C GBP 155 |
| 10 | https://speeduino.com/shop/12-speeduino-boards | Bare v0.4 PCB USD 10; no assembled 0.4.4d |

LCSC part pages were not directly readable (client-side rendering and bot
blocking). The LCSC codes above come from the JLCPCB catalogue record and can be
looked up at `https://www.lcsc.com/search?q=<code>`; the catalogue price is what
a JLCPCB order would charge and is the correct input for this model.

No quote number exists. No order, upload, fabrication or purchase was made.
