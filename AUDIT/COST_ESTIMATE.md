# Cost estimate working papers — prototype v1, 5 boards

| Field | Value |
|---|---|
| Document ID | AUD-HW-COST-02 |
| Status | Working papers for `docs/HARDWARE_COST.md`. **Estimate, not a quote.** |
| Date | 2026-11-13 (access window 03:34–03:45 UTC) |
| Scope | Read-only web research and local file parsing. No upload, order, payment or fabrication. |
| Result | A$127.70 / **A$160.79** / A$196.57 per board (low / expected / high) |
| Related | `docs/HARDWARE_COST.md`, `AUDIT/BOM_REVIEW.md`, `docs/JLCPCB_ASSEMBLY.md` |

This annex records how each number was produced, so a reviewer can repeat or
challenge it. The summary and the itemised cost table are in
`docs/HARDWARE_COST.md`; this file holds the raw evidence, the derivations and
the sensitivity arithmetic.

## 1. What could and could not be accessed

| Source | Result |
|---|---|
| JLCPCB assembly fee page | Readable, full fee table, "Last updated on Oct 22, 2026" |
| JLCPCB part catalogue API | Readable JSON: price tiers, stock, basic/extended flag, LCSC code |
| RBA exchange rates | Readable, daily rates to 12 Nov 2026 |
| Frankfurter (ECB) FX API | Readable JSON |
| Pololu product page | Readable, with embedded structured price |
| OpenLogicEFI product page | Readable, including variant JSON and stock |
| DIY-EFI product page | Readable |
| Speeduino official store | Readable |
| PCBWay assembly/PCB pages | Readable marketing pages; no fee table; calculator is JS/file-driven |
| LCSC product pages | Not accessible (client-side rendering; search API returns 404/403) |
| Digi-Key, Mouser, Element14, Octopart | Blocked (HTTP 403 or access-denied page) |
| JLCPCB/PCBWay instant quote | Not obtainable without a Gerber/BOM upload and login |
| In-app browser | Unavailable in this session (`browser.disconnected`) |

Consequence: no manufacturer quotation exists. Distributor cross-checks could
not be made, so every part price is the **JLCPCB catalogue price** — the price
that would actually be charged on a JLCPCB turnkey order — not a distributor
list price.

## 2. Joint count derivation

The assembly fee is per joint, so the joint count was derived from the board
rather than estimated from the BOM.

Method: parse `hardware/prototype-v1/v0.4.4d.kicad_pcb` as s-expressions, take
each top-level `(footprint ...)` block, read its `Reference` property, count
SMD pads and numbered plated through-hole pads; ignore `np_thru_hole` and
unnumbered through-hole pads. Restrict to the 139 designators in
`hardware/prototype-v1/manufacturing/jlcpcb/cpl-jlcpcb.csv`.

Result:

| Metric | Count |
|---|---:|
| CPL designators parsed | 139 |
| Missing from PCB parse | 0 |
| Numbered SMD pads (SMT joints) | **311** |
| Numbered plated THT pads (manual joints) | **99** |
| Total joints | **410** |

Through-hole breakdown: J4 40, IC3 8, J1 5, J2 6, J3 2, J5 4, J6 5, J7 5,
J8 3, JP1–JP3 3 each (9), JP4–JP5 2 each (4), S1 2, U5 6.

Two footprint quirks that a reviewer should know:

- U8 (`LM2940S`, TO-263) has 6 SMD pads plus 2 plated pads named MH1/MH2, so the
  board presents 8 solderable pads where the package has 3 leads plus a tab.
  JLCPCB's own pad count may differ by a few joints.
- S1 (`AYZ0202AGRLC`) has 4 unnamed SMD pads in addition to its numbered pads.

The effect on cost is under USD 0.25 per order either way, because the joint
fees are USD 0.0016 and USD 0.0164.

The parser is a throwaway script at `/tmp/opencode/joint_count.py` (read-only,
not committed); the method above is enough to reproduce it. Output
(2026-11-13):

```text
CPL designators: 139; missing from PCB parse: 0 []
SMT joints (numbered SMD pads): 311
THT joints (numbered plated THT pads): 99
Total joints: 410
```

## 3. Part price and stock evidence

Query (repeated once per MPN; JSON POST):

```bash
curl -s 'https://jlcpcb.com/api/overseas-pcb-order/v1/shoppingCart/smtGood/selectSmtComponentList' \
  -H 'content-type: application/json' \
  --data '{"keyword":"MPX4250AP","currentPage":1,"pageSize":5}'
```

Response fields used: `componentCode` (LCSC), `componentModelEn` (matched MPN),
`componentLibraryType` (`base`/`expand`), `stockCount`, `componentPrices`
(price at quantity 5), `noBuyReason`.

| MPN queried | Matched part | LCSC | Library | Stock | Unit USD @5 | Flag |
|---|---|---|---:|---:|---:|---|
| RC0805FR-071KL | RC0805FR-071KL | C95781 | expand | 5,710,970 | 0.0030 | |
| CC0805KRX7R9BB104 | CC0805KRX7R9BB104 | C49678 | **base** | 24,602,972 | 0.0138 | only basic part |
| RC0805FR-07100KL | RC0805FR-07100KL | C96346 | expand | 2,470,593 | 0.0038 | |
| RC0805FR-072K4L | RC0805FR-072K4L | C114521 | expand | 276,740 | 0.0086 | |
| ERJ-6ENF4700V | ERJ6ENF4700V | C413274 | expand | 4,243 | 0.0071 | |
| LH R974-LP-1 | LH R974-LP-1-0-20-R18 | C2899895 | expand | **0** | 0.1473 | **EOL** |
| CL21B224KOCNNNC | CL21B224KOCNNNC | C296721 | expand | **0** | 0.0192 | |
| VNLD5090-E | *(no match)* | — | — | — | — | exact code not stocked |
| VNLD5090TR-E | VNLD5090TR-E | C222209 | expand | 30 | 2.5326 | used in model |
| S110FA | S110FA | C556397 | expand | 119 | 0.5039 | |
| RC1206FR-0710RL | RC1206FR-0710RL | C108080 | expand | 1,121,224 | 0.0238 | |
| 1N4448WX-TP | 1N4448WX-TP | C712353 | expand | 24,627 | 0.0164 | |
| SSM3K357R | SSM3K357R | C7558097 | expand | **0** | 0.0398 | |
| M20-9990345 | M20-9990345 | C3337081 | expand | 4,187 | 0.2149 | candidate rejected |
| CC0805KKX7R7BB105 | CC0805KKX7R7BB105 | C94122 | expand | 208,253 | 0.0383 | |
| RT0805BRD072K49L | RT0805BRD072K49L | C375522 | expand | 4,331 | 0.0429 | |
| RC0805JR-0710KL | RC0805JR-0710KL | C100047 | expand | 2,533,145 | 0.0053 | |
| RC0805FR-07220RL | RC0805FR-07220RL | C114519 | expand | 363,728 | 0.0045 | |
| MM3Z5V1ST1G | MM3Z5V1ST1G | C236206 | expand | 36,705 | 0.0284 | |
| M20-9990245 | M20-9990245 | C3338413 | expand | 1,605 | 0.1293 | |
| IXDN602SIATR | IXDN602SIATR | C431679 | expand | 1,179 | 1.4659 | |
| CC0805KRX7R9BB471 | CC0805KRX7R9BB471 | C107152 | expand | 57,644 | 0.0094 | |
| CC0805KRX7R9BB103 | CC0805KRX7R9BB103 | C83170 | expand | 1,566,649 | 0.0074 | |
| 0ZCC0050FF2C | *(no match)* | — | — | — | — | not in library |
| TAJB476K006RNJ | TAJB476K006RNJ | C7207 | expand | 13,632 | 0.3021 | |
| T491D106K050AT | T491D106K050AT | C117038 | expand | 3,190 | 1.4643 | |
| SP720ABTG | SP720ABTG | C434726 | expand | 479 | 2.2217 | |
| SMBJ40A-Q | SMBJ40AQ | C7462772 | expand | 1,562 | 0.1425 | suffix variant |
| RC0805FR-073K9L | RC0805FR-073K9L | C114221 | expand | 74,331 | 0.0063 | |
| RC0805FR-072KL | RC0805FR-072KL | C114572 | expand | 254,483 | 0.0046 | |
| RC0805FR-07120RL | RC0805FR-07120RL | C114245 | expand | 163,730 | 0.0061 | |
| MPX4250AP | MPX4250AP | C2802676 | expand | **0** | 23.1821 | **EOL** |
| LM2940S-5.0/NOPB | LM2940S-5.0/NOPB | C133956 | expand | **1** | 3.2426 | |
| CL21A226KQCLRNC | CL21A226KQCLRNC | C3909963 | expand | 941 | 0.0791 | |
| B130-13-F | B130-13-F | C134423 | expand | 47,696 | 0.0915 | |
| AYZ0202AGRLC | AYZ0202AGRLC | C221564 | expand | **0** | 1.8127 | |

Representative connector/socket prices (not approved MPNs):

| Query | Part | LCSC | Stock | Unit USD @1 |
|---|---|---|---:|---:|
| HTSW-105 | HTSW-105-07-G-S | C3337223 | 16 | 0.6332 |
| HTSW-106 | HTSW-106-07-L-S | C3324216 | 190 | 0.7444 |
| HTSW-104 | HTSW-104-08-S-S | C3332127 | 88 | 0.6119 |
| HTSW-103 | HTSW-103-07-L-S | C3333112 | 23 | 0.3375 |
| 1729018 | Phoenix 1729018 | C3820334 | 81 | 0.5612 |
| SSQ-108 | SSQ-108-01-F-S | C7019014 | 3 | 1.4643 |
| TSW-128 | TSW-128-07-T-S | C3329247 | 0 | 1.1567 |
| TSW-124 | TSW-124-07-L-S | C3348299 | 0 | 1.5232 |
| TSW-117 | TSW-117-07-L-D | C3345936 | 23 | 2.9612 |

No stocked JLCPCB match was found for a keyed 2×20 IDC header, a 7.62 mm DIP-8
socket (the generic hit is marked "no longer manufactured") or the
0ZCC0050FF2C fuse. These are modelled as estimates in the main document.

**BOM file defect observed.** The J3 row in `bom-jlcpcb.csv` has an unquoted
comma inside the footprint name
(`TerminalBlock_Phoenix_PT-1,5-2-5.0-H_1x02_P5.00mm_Horizontal`). A naive CSV
reader therefore sees six fields, leaves the MPN column blank and puts the
footprint tail (`5-2-5.0-H_1x02_P5.00mm_Horizontal`) where the MPN should be.
The row is correctly understood as an unresolved connector, but it must be
quoted or the footprint text changed before any assembler upload, or the
assembler's BOM parser may treat that fragment as a part number. This file was
not edited in this round (BOM data is outside this task's ownership).

**Quantity treatment.** Parts are priced at 5 × placements, with no reel
minimum and no attrition. JLCPCB normally charges only the parts consumed, but
pre-order/preferred parts can carry minimum quantities. The high case applies a
+15% parts uplift to cover this.

## 4. Assembly fee derivation

From https://jlcpcb.com/help/article/pcb-assembly-price (updated 2026-10-22),
Economic column:

| Fee | Rate | Applied as | USD |
|---|---|---|---:|
| Setup | 8.18/order | once | 8.18 |
| Stencil | 1.53/order | once | 1.53 |
| SMT assembly | 0.0016/joint | 311 × 5 | 2.488 |
| Manual assembly | 0.0164/joint | 99 × 5 | 8.118 |
| Hand-soldering labour | 3.58/order | once | 3.58 |
| Placement confirmation | 0.45/order | once | 0.45 |
| Feeder loading | 3.07/extended type | 33 types | 101.31 |
| Packing | free | — | 0 |
| **Total** | | | **125.656** |

Checks against the published schedule:

- Large-size surcharge (USD 57.46) starts above 650 cm². Five boards at
  100.05 × 100.05 mm = 5 × 100.1 cm² = 500.5 cm², so it does not apply.
- X-ray inspection applies to BGA/QFN/leadless parts. There are none, so it does
  not apply.
- The Standard-tier handling fee, fixture fee and storage fees do not apply to
  Economic. Standard would add USD 25.56 setup, USD 8.21 stencil, USD 14.93
  handling minimum, USD 16.42 fixtures and USD 0.48/board, so Economic is the
  cheaper tier at this quantity.

Extended-type count: 35 unique machine MPNs, of which exactly one (C49678,
0.1 µF) is a JLCPCB basic part and one (0ZCC0050FF2C) is absent from the
library, leaving 33 extended catalogue types. The seven through-hole connector
types are assumed to be hand-placed and therefore not feeder-loaded.
If the fuse is substituted by an in-library part and JLCPCB also charges feeder
loading for the seven THT types, the count is 41 and the fee is USD 125.87
(+USD 24.56 per order, +A$6.28 per board).

## 5. FX evidence

| Source | Date | Rate | AUD/USD |
|---|---|---|---|
| RBA daily rates (USD 0.6841 per AUD) | 2026-11-12 | 0.6841 | **1.4618** |
| Frankfurter / ECB reference | 2026-11-12 | 1.4661 | 1.4661 |
| Difference | | | 0.29% |

The RBA rate is used because the budget currency is AUD and the RBA is the
domestic reference. At 1.4661 the expected result would be A$161.26/board, a
A$0.47 difference — immaterial.

GBP for the DIY-EFI comparison: RBA 2026-11-12 GBP 0.5159 per AUD → AUD/GBP
1.9384; GBP 155 = A$300.45, GBP 171 = A$331.46.

## 6. Sensitivity arithmetic

All at 5 boards, including 10% GST, FX 1.4618.

| Case | Parts USD | Assembly USD | Fab USD | Freight USD | Total USD | A$/board |
|---|---:|---:|---:|---:|---:|---:|
| Low (18 extended types, generic hand-solder, USD 8 fab, USD 12 freight) | 297.48 | 79.61 | 8 | 12 | 436.80 | 127.70 |
| **Expected (33 types, Samtec/Phoenix reps, USD 18 fab, USD 25 freight)** | **331.33** | **125.66** | **18** | **25** | **549.99** | **160.79** |
| High (41 types, parts +15%, USD 35 fab, USD 45 freight) | 381.03 | 150.22 | 35 | 45 | 672.37 | 196.57 |
| Expected, owner supplies MAP sensor | 331.33 − 115.91 | 125.66 | 18 | 25 | 422.48 | 123.52 |
| High + MAP at 2× catalogue | 381.03 + 115.91 | 150.22 | 35 | 45 | 799.87 | 233.85 |
| 10 boards (same 33 types, USD 30 fab, USD 30 freight) | 662.66 | 136.26 | 30 | 30 | 944.81 | 138.11 |

Derived thresholds:

- **A$250/board budget at 5 boards** = A$1,250 landed = USD 855.11 including
  GST, or USD 777.37 of goods + freight before GST.
- **Assembly headroom** = 777.37 − 331.33 (parts) − 18 (fab) − 25 (freight) =
  **USD 403.04** per order. JLCPCB's modelled assembly is USD 125.66, leaving
  USD 277 of headroom before the target is breached.
- **Breach sensitivity**: the total would need to rise 55.5% above the expected
  case to reach A$250/board.

## 7. Comparison arithmetic

| Path | Arithmetic | AUD |
|---|---|---:|
| Bare PCB, 5 pcs | (18 + 25) × 1.10 × 1.4618 | 69.14 |
| JLCPCB assembled, 5 | (331.33 + 125.66 + 18 + 25) × 1.10 × 1.4618 | 803.97 |
| JLCPCB + plug-ins | 803.97 + (15.95 + 31.01/1.4618) × 5 × 1.10 × 1.4618 | 1,102.79 |
| OpenLogicEFI 0.4.4d, no MAP | (139.99 + 30 est. freight) × 1.10 × 1.4618 | 273.34 |
| OpenLogicEFI 0.4.4d, 4-bar MAP | (159.99 + 30) × 1.10 × 1.4618 | 305.51 |
| DIY-EFI 0.4.4C + Mega | (171 + 20 est. freight) / 0.5159 × 1.10 | 407.25 |

The GBP 16 Mega option and the GBP 155 board price are from the DIY-EFI listing.
The GBP 20 freight and USD 30 freight are estimates, not quotes. The sellers'
pages do not show an Australia shipping rate without checkout.

## 8. Reproducibility and limits

Commands (read-only):

```bash
# FX
curl -s 'https://api.frankfurter.dev/v1/latest?base=USD&symbols=AUD'

# assembly fees
curl -s 'https://jlcpcb.com/help/article/pcb-assembly-price'

# one part (repeat per MPN)
curl -s 'https://jlcpcb.com/api/overseas-pcb-order/v1/shoppingCart/smtGood/selectSmtComponentList' \
  -H 'content-type: application/json' \
  --data '{"keyword":"MPX4250AP","currentPage":1,"pageSize":5}'

# joint counts
python3 /tmp/opencode/joint_count.py   # script listed in §2; not committed
```

Limits that affect confidence:

1. No Gerber/BOM/CPL upload was made, so no manufacturer quote exists.
2. The 100.05 mm outline is at or over the usual 100 mm entry-offer class;
   fabrication is the weakest input (USD 8–35 assumed).
3. Freight is assumed, not quoted.
4. Five machine parts and two hand-solder parts have zero stock; two machine
   parts are EOL. The model does not include broker premiums or expedite fees.
5. The nine connector/socket MPNs are unresolved; representative prices are used.
6. The fuse is not in the JLCPCB library; consignment or substitution is
   required, and the USD 0.50 unit price is an estimate.
7. Distributor cross-checks were blocked, so no independent price triangulation
   was possible.
8. JLCPCB's basic/extended mapping is the catalogue flag; their BOM tool may
   remap a line and change the feeder count.

## 9. What would convert this into a quote

1. EE signs off the nine connector/socket MPNs and the two hand-solder items.
2. `bom-jlcpcb.csv` is updated with LCSC codes for all machine lines and
   regenerated; `cpl-jlcpcb.csv` is reconciled.
3. Gerbers + drill + BOM + CPL are uploaded to JLCPCB (and separately to
   PCBWay) at 5 boards, top-side assembly, 100.05 mm outline.
4. The DFM report and assembly preview are accepted, including 0.05 mm annular
   ring and the THT process.
5. Checkout is taken to the shipping page (without paying) to capture the real
   freight and GST.

Only step 3–5 produce a quotation. Nothing in this annex is one.
