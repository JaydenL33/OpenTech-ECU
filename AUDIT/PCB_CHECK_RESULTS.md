# PCB Check Results — Prototype v1 (Speeduino v0.4.4d SMD)

| Field | Value |
|---|---|
| Document ID | AUD-HW-CHK-01 |
| Status | **H2: checks complete with documented exceptions. Focused independent review ACCEPT WITH FINDINGS. NOT a pass.** H3 not ready. Not fabrication-approved. |
| Date | 2026-11-13 (HW-H2-01 run) |
| Workstream | Hardware |
| Design under test | `hardware/prototype-v1/` (Speeduino v0.4.4d SMD + R-01…R-10) |
| Upstream source | `hardware/upstream-speeduino/v0.4.4d/`, commit `ee6a9242834891877744ff2137663b6ca8d79ed6` |
| Tool | KiCad 9.0.9 (`kicad/kicad@sha256:e638b79b…7729`), Docker |
| Reproduction | `hardware/tools/check_prototype_v1.sh` |
| Raw evidence | `hardware/prototype-v1/verification/` |

> **Read this before quoting a number.** An ERC/DRC result shows design
> consistency under one rule set. It is **not** evidence of automotive
> electrical reliability, thermal adequacy, EMC, or Evo VIII compatibility.
> No rule severity was changed. 199 DRC items are excluded **per item**, each
> with a written justification. Excluded items still appear in
> `drc-with-exclusions.rpt` and `drc_exclusions.csv`. The 31 JP6–JP11
> jumper errors are deliberately **not** excluded.

## 1. Tool and command evidence

```text
KiCad 9.0.9, release build
Image: kicad/kicad@sha256:e638b79b0321f29395a5b783e94bb9f3c73303e8da15da27b8f5cb4b67a37729
Host: Linux 6.8.0-147-generic x86_64
```

```bash
hardware/tools/check_prototype_v1.sh          # exit 0
hardware/tools/verify_upstream_baseline.sh    # exit 0, 32/32 files OK

# individual commands (inside the container, /work = hardware/prototype-v1):
kicad-cli sch erc --output /work/verification/erc.rpt /work/v0.4.4d.kicad_sch
kicad-cli pcb drc --output /work/verification/drc.rpt /work/v0.4.4d.kicad_pcb
kicad-cli pcb drc --severity-all --severity-exclusions \
  --output /work/verification/drc-with-exclusions.rpt /work/v0.4.4d.kicad_pcb
python3 /tools/connectivity_check.py /work
python3 /tools/check_mounting_holes.py /work
```

Observed exit codes (`verification/run-context.txt`): ERC 0, DRC 0, DRC with
exclusions 0, connectivity 0, mounting holes 0. Exit code 0 from kicad-cli means
the report was written. It does **not** mean zero violations. Counts come from
the reports.

## 2. Summary

| Check | Before HW-H2-01 | After HW-H2-01 |
|---|---|---|
| Schematic ERC | 101 (44 errors, 57 warnings) | **49 (4 errors, 45 warnings)**; every item dispositioned in §3 |
| PCB DRC (reported) | 600 (230 errors, 370 warnings) | **397 (31 errors, 366 warnings)** |
| PCB DRC (excluded, per item) | 0 | **199** `courtyards_overlap`, each commented (§5.1) |
| Unconnected items | 0 | **0** |
| Schematic ↔ PCB connectivity | PASS 514/514 | **PASS 514/514**, 0 split nets; schematic net partition identical to before (190 nets) |
| Mounting holes | none | **3 × 3.2 mm NPTH** (H1–H3), clearance check PASS (§6) |
| Library links | 4 footprint + 2 symbol warnings | **0** |
| Board outline | 100.05 × 100.05 mm closed | unchanged |
| Copper layers / thickness | 2 / 1.6 mm | unchanged (1.6 mm from upstream) |
| Footprints | 297 | 300 (+H1–H3) |
| Tracks / vias | 1232 / 178 | unchanged |

## 3. ERC — 49 remaining items, line by line

Raw report: `hardware/prototype-v1/verification/erc.rpt`.

| Category | Before | After | Fixed by |
|---|---:|---:|---|
| `pin_not_connected` (error) | 26 | 0 | R-08 no-connect flags on 26 genuinely unused pins |
| `power_pin_not_driven` (error) | 9 | 0 | R-07 five PWR_FLAGs at real power entry points; R-08 NC on unused XA1 power pins |
| `pin_not_driven` (error) | 9 | **4** | R-08 NC on A1 M0–M2, XA1 AREF, XA1 MISO; **4 U6 spare channels remain** |
| `footprint_link_issues` (warning) | 4 | 0 | R-05 local libraries |
| `lib_symbol_issues` (warning) | 2 | 0 | R-06 `Device:C_Polarized_US` |
| `unconnected_wire_endpoint` (warning) | 2 | 0 | R-08 stubs removed |
| `no_connect_dangling` (warning) | 1 | 0 | R-08 removed |
| `global_label_dangling` (warning) | 37 | **34** | R-08 removed ICSP labels MISO/MOSI/SCLK; 34 accepted below |
| `multiple_net_names` (warning) | 11 | **11** | accepted aliases below |

**Remaining errors: 4.** All are `pin_not_driven` on spare SP720 clamp
channels (U6 pins 7, 12, 13, 15). Each net has a single pad and no track.
They are kept visible as review exceptions **E-ERC-I7/I11/I12/I14**. No engine
I/O is involved. **0 unexplained ERC errors.**

The report prints every `global_label_dangling` item under sheet `/`, even
when the label lives in a sub-sheet (for example the `ADC-Clamp-*` labels are
in `inputs.kicad_sch`). The "Sheet" column repeats the report as printed.

| # | Sheet | Category | Sev | Location (mm) | Object | Disposition |
|---:|---|---|---|---|---|---|
| 1 | `/` | `global_label_dangling` | warning | (114.30, 46.99) | Global Label 'MCU-D15' | Accepted: `MCU-D15` is an unused Arduino digital pin, labelled upstream for proto/expansion use. The PCB pad exists on the shield header; nothing else on the board uses it. |
| 2 | `/` | `global_label_dangling` | warning | (114.30, 49.53) | Global Label 'MCU-D14' | Accepted: `MCU-D14` is an unused Arduino digital pin, labelled upstream for proto/expansion use. The PCB pad exists on the shield header; nothing else on the board uses it. |
| 3 | `/` | `global_label_dangling` | warning | (114.30, 57.15) | Global Label 'MCU-D2' | Accepted, inert leftover: the upstream `MCU-D2` label is attached to no wire or pin (the netlist has no `MCU-D2` net). Arduino D2 **is used**: XA1.D2 is on net `ADC-Clamp-10` (Flex input, U6.11, JP6.1). The label changes nothing. Removing it is a cosmetic upstream edit, deferred. |
| 4 | `/` | `global_label_dangling` | warning | (114.30, 80.01) | Global Label 'MCU-A6' | Accepted: `MCU-A6` is an unused Arduino analog pin, labelled upstream for proto use. The shield has no other connection to it. |
| 5 | `/` | `global_label_dangling` | warning | (114.30, 82.55) | Global Label 'MCU-A7' | Accepted: `MCU-A7` is an unused Arduino analog pin, labelled upstream for proto use. The shield has no other connection to it. |
| 6 | `/` | `global_label_dangling` | warning | (189.23, 57.15) | Global Label 'MCU-D12' | Accepted: `MCU-D12` is an unused Arduino digital pin, labelled upstream for proto/expansion use. The PCB pad exists on the shield header; nothing else on the board uses it. |
| 7 | `/` | `global_label_dangling` | warning | (189.23, 59.69) | Global Label 'MCU-D13' | Accepted: `MCU-D13` is an unused Arduino digital pin, labelled upstream for proto/expansion use. The PCB pad exists on the shield header; nothing else on the board uses it. |
| 8 | `/` | `global_label_dangling` | warning | (189.23, 62.23) | Global Label 'MCU-D22' | Accepted: `MCU-D22` is an unused Arduino digital pin, labelled upstream for proto/expansion use. The PCB pad exists on the shield header; nothing else on the board uses it. |
| 9 | `/` | `global_label_dangling` | warning | (189.23, 64.77) | Global Label 'MCU-D23' | Accepted: `MCU-D23` is an unused Arduino digital pin, labelled upstream for proto/expansion use. The PCB pad exists on the shield header; nothing else on the board uses it. |
| 10 | `/` | `global_label_dangling` | warning | (189.23, 69.85) | Global Label 'MCU-D25' | Accepted: `MCU-D25` is an unused Arduino digital pin, labelled upstream for proto/expansion use. The PCB pad exists on the shield header; nothing else on the board uses it. |
| 11 | `/` | `global_label_dangling` | warning | (189.23, 72.39) | Global Label 'MCU-D26' | Accepted: `MCU-D26` is an unused Arduino digital pin, labelled upstream for proto/expansion use. The PCB pad exists on the shield header; nothing else on the board uses it. |
| 12 | `/` | `global_label_dangling` | warning | (189.23, 74.93) | Global Label 'MCU-D27' | Accepted: `MCU-D27` is an unused Arduino digital pin, labelled upstream for proto/expansion use. The PCB pad exists on the shield header; nothing else on the board uses it. |
| 13 | `/` | `global_label_dangling` | warning | (189.23, 77.47) | Global Label 'MCU-D28' | Accepted: `MCU-D28` is an unused Arduino digital pin, labelled upstream for proto/expansion use. The PCB pad exists on the shield header; nothing else on the board uses it. |
| 14 | `/` | `global_label_dangling` | warning | (189.23, 80.01) | Global Label 'MCU-D29' | Accepted: `MCU-D29` is an unused Arduino digital pin, labelled upstream for proto/expansion use. The PCB pad exists on the shield header; nothing else on the board uses it. |
| 15 | `/` | `global_label_dangling` | warning | (189.23, 82.55) | Global Label 'MCU-D30' | Accepted: `MCU-D30` is an unused Arduino digital pin, labelled upstream for proto/expansion use. The PCB pad exists on the shield header; nothing else on the board uses it. |
| 16 | `/` | `global_label_dangling` | warning | (189.23, 85.09) | Global Label 'MCU-D31' | Accepted: `MCU-D31` is an unused Arduino digital pin, labelled upstream for proto/expansion use. The PCB pad exists on the shield header; nothing else on the board uses it. |
| 17 | `/` | `global_label_dangling` | warning | (189.23, 87.63) | Global Label 'MCU-D32' | Accepted: `MCU-D32` is an unused Arduino digital pin, labelled upstream for proto/expansion use. The PCB pad exists on the shield header; nothing else on the board uses it. |
| 18 | `/` | `global_label_dangling` | warning | (189.23, 90.17) | Global Label 'MCU-D33' | Accepted: `MCU-D33` is an unused Arduino digital pin, labelled upstream for proto/expansion use. The PCB pad exists on the shield header; nothing else on the board uses it. |
| 19 | `/` | `global_label_dangling` | warning | (189.23, 92.71) | Global Label 'MCU-D34' | Accepted: `MCU-D34` is an unused Arduino digital pin, labelled upstream for proto/expansion use. The PCB pad exists on the shield header; nothing else on the board uses it. |
| 20 | `/` | `global_label_dangling` | warning | (189.23, 95.25) | Global Label 'MCU-D35' | Accepted: `MCU-D35` is an unused Arduino digital pin, labelled upstream for proto/expansion use. The PCB pad exists on the shield header; nothing else on the board uses it. |
| 21 | `/` | `global_label_dangling` | warning | (189.23, 97.79) | Global Label 'MCU-D36' | Accepted: `MCU-D36` is an unused Arduino digital pin, labelled upstream for proto/expansion use. The PCB pad exists on the shield header; nothing else on the board uses it. |
| 22 | `/` | `global_label_dangling` | warning | (189.23, 100.33) | Global Label 'MCU-D37' | Accepted: `MCU-D37` is an unused Arduino digital pin, labelled upstream for proto/expansion use. The PCB pad exists on the shield header; nothing else on the board uses it. |
| 23 | `/` | `global_label_dangling` | warning | (189.23, 105.41) | Global Label 'MCU-D39' | Accepted: `MCU-D39` is an unused Arduino digital pin, labelled upstream for proto/expansion use. The PCB pad exists on the shield header; nothing else on the board uses it. |
| 24 | `/` | `global_label_dangling` | warning | (189.23, 110.49) | Global Label 'MCU-D41' | Accepted: `MCU-D41` is an unused Arduino digital pin, labelled upstream for proto/expansion use. The PCB pad exists on the shield header; nothing else on the board uses it. |
| 25 | `/` | `global_label_dangling` | warning | (189.23, 113.03) | Global Label 'MCU-D42' | Accepted: `MCU-D42` is an unused Arduino digital pin, labelled upstream for proto/expansion use. The PCB pad exists on the shield header; nothing else on the board uses it. |
| 26 | `/` | `global_label_dangling` | warning | (189.23, 115.57) | Global Label 'MCU-D43' | Accepted: `MCU-D43` is an unused Arduino digital pin, labelled upstream for proto/expansion use. The PCB pad exists on the shield header; nothing else on the board uses it. |
| 27 | `/` | `global_label_dangling` | warning | (189.23, 118.11) | Global Label 'MCU-D44' | Accepted: `MCU-D44` is an unused Arduino digital pin, labelled upstream for proto/expansion use. The PCB pad exists on the shield header; nothing else on the board uses it. |
| 28 | `/` | `global_label_dangling` | warning | (189.23, 123.19) | Global Label 'MCU-D46' | Accepted: `MCU-D46` is an unused Arduino digital pin, labelled upstream for proto/expansion use. The PCB pad exists on the shield header; nothing else on the board uses it. |
| 29 | `/` | `global_label_dangling` | warning | (189.23, 128.27) | Global Label 'MCU-D48' | Accepted: `MCU-D48` is an unused Arduino digital pin, labelled upstream for proto/expansion use. The PCB pad exists on the shield header; nothing else on the board uses it. |
| 30 | `/` | `global_label_dangling` | warning | (189.23, 140.97) | Global Label 'MCU-D53' | Accepted: `MCU-D53` is an unused Arduino digital pin, labelled upstream for proto/expansion use. The PCB pad exists on the shield header; nothing else on the board uses it. |
| 31 | `/` | `global_label_dangling` | warning | (212.09, 46.99) | Global Label 'ADC-Clamp-7' | Accepted: label `ADC-Clamp-7` names the spare U6 channel; nothing else on the board uses it (see the matching pin_not_driven row). |
| 32 | `/` | `global_label_dangling` | warning | (240.03, 31.75) | Global Label 'ADC-Clamp-14' | Accepted: label `ADC-Clamp-14` names the spare U6 channel; nothing else on the board uses it (see the matching pin_not_driven row). |
| 33 | `/` | `global_label_dangling` | warning | (240.03, 36.83) | Global Label 'ADC-Clamp-12' | Accepted: label `ADC-Clamp-12` names the spare U6 channel; nothing else on the board uses it (see the matching pin_not_driven row). |
| 34 | `/` | `global_label_dangling` | warning | (240.03, 39.37) | Global Label 'ADC-Clamp-11' | Accepted: label `ADC-Clamp-11` names the spare U6 channel; nothing else on the board uses it (see the matching pin_not_driven row). |
| 35 | `/Inputs/` | `pin_not_driven` | error | (215.90, 46.99) | Symbol U6 Pin 7 [I7, Input, Line] | Exception E-ERC-I7: spare SP720 clamp channel I7, net `ADC-Clamp-7` has only this pin on the PCB (single-pad net, no track). Clamp-array input with nothing to protect; upstream design leaves it open. Not engine I/O. Kept visible, not suppressed; reviewer to accept or request NC/tie-off. |
| 36 | `/Inputs/` | `pin_not_driven` | error | (236.22, 31.75) | Symbol U6 Pin 15 [I14, Input, Line] | Exception E-ERC-I14: spare SP720 clamp channel I14, net `ADC-Clamp-14` has only this pin on the PCB (single-pad net, no track). Clamp-array input with nothing to protect; upstream design leaves it open. Not engine I/O. Kept visible, not suppressed; reviewer to accept or request NC/tie-off. |
| 37 | `/Inputs/` | `pin_not_driven` | error | (236.22, 36.83) | Symbol U6 Pin 13 [I12, Input, Line] | Exception E-ERC-I12: spare SP720 clamp channel I12, net `ADC-Clamp-12` has only this pin on the PCB (single-pad net, no track). Clamp-array input with nothing to protect; upstream design leaves it open. Not engine I/O. Kept visible, not suppressed; reviewer to accept or request NC/tie-off. |
| 38 | `/Inputs/` | `pin_not_driven` | error | (236.22, 39.37) | Symbol U6 Pin 12 [I11, Input, Line] | Exception E-ERC-I11: spare SP720 clamp channel I11, net `ADC-Clamp-11` has only this pin on the PCB (single-pad net, no track). Clamp-array input with nothing to protect; upstream design leaves it open. Not engine I/O. Kept visible, not suppressed; reviewer to accept or request NC/tie-off. |
| 39 | `/Inputs/` | `multiple_net_names` | warning | (88.90, 53.34); (88.90, 48.26) | Global Label 'ADC-Clamp-2'; Global Label 'MCU-A3' | Accepted alias (upstream intent): `ADC-Clamp-2` and `MCU-A3` name the same node. KiCad uses `ADC-Clamp-2`; the netlist partition is unchanged. |
| 40 | `/Inputs/` | `multiple_net_names` | warning | (62.23, 77.47); (72.39, 85.09) | Global Label 'ADC-Clamp-6'; Global Label 'MCU-A8' | Accepted alias (upstream intent): `ADC-Clamp-6` and `MCU-A8` name the same node. KiCad uses `ADC-Clamp-6`; the netlist partition is unchanged. |
| 41 | `/Inputs/` | `multiple_net_names` | warning | (63.50, 120.65); (73.66, 128.27) | Global Label 'ADC-Clamp-1'; Global Label 'MCU-A4' | Accepted alias (upstream intent): `ADC-Clamp-1` and `MCU-A4` name the same node. KiCad uses `ADC-Clamp-1`; the netlist partition is unchanged. |
| 42 | `/Inputs/` | `multiple_net_names` | warning | (151.13, 25.40); (161.29, 33.02) | Global Label 'ADC-Clamp-4'; Global Label 'MCU-A2' | Accepted alias (upstream intent): `ADC-Clamp-4` and `MCU-A2` name the same node. KiCad uses `ADC-Clamp-4`; the netlist partition is unchanged. |
| 43 | `/Inputs/` | `multiple_net_names` | warning | (151.13, 71.12); (161.29, 78.74) | Global Label 'ADC-Clamp-5'; Global Label 'MCU-A0' | Accepted alias (upstream intent): `ADC-Clamp-5` and `MCU-A0` name the same node. KiCad uses `ADC-Clamp-5`; the netlist partition is unchanged. |
| 44 | `/Inputs/` | `multiple_net_names` | warning | (49.53, 171.45); (49.53, 180.34) | Global Label 'ADC-Clamp-10'; Global Label 'Flex_Sensor' | Accepted alias (upstream intent): `ADC-Clamp-10` and `Flex_Sensor` name the same node. KiCad uses `ADC-Clamp-10`; the netlist partition is unchanged. |
| 45 | `/Inputs/` | `multiple_net_names` | warning | (152.40, 118.11); (162.56, 125.73) | Global Label 'ADC-Clamp-3'; Global Label 'MCU-A1' | Accepted alias (upstream intent): `ADC-Clamp-3` and `MCU-A1` name the same node. KiCad uses `ADC-Clamp-3`; the netlist partition is unchanged. |
| 46 | `/Inputs/` | `multiple_net_names` | warning | (156.21, 160.02); (156.21, 163.83) | Global Label 'ADC-Clamp-13'; Global Label 'MCU-D51' | Accepted alias (upstream intent): `ADC-Clamp-13` and `MCU-D51` name the same node. KiCad uses `ADC-Clamp-13`; the netlist partition is unchanged. |
| 47 | `/Inputs/` | `multiple_net_names` | warning | (246.38, 82.55); (255.27, 99.06) | Global Label 'ADC-Clamp-9'; Global Label 'MCU-D19' | Accepted alias (upstream intent): `ADC-Clamp-9` and `MCU-D19` name the same node. KiCad uses `ADC-Clamp-9`; the netlist partition is unchanged. |
| 48 | `/Inputs/` | `multiple_net_names` | warning | (240.03, 46.99); (255.27, 105.41) | Global Label 'ADC-Clamp-8'; Global Label 'MCU-D18' | Accepted alias (upstream intent): `ADC-Clamp-8` and `MCU-D18` name the same node. KiCad uses `ADC-Clamp-8`; the netlist partition is unchanged. |
| 49 | `/Proto` | `multiple_net_names` | warning | (191.77, 100.33); (151.13, 93.98) | Global Label 'MCU-D20'; Global Label 'VSS' | Accepted alias (upstream intent): `MCU-D20` and `VSS` name the same node. KiCad uses `MCU-D20`; the netlist partition is unchanged. |

## 4. Schematic ↔ PCB connectivity — PASS

```text
schematic nodes: 520
pcb nodes:       514
common nodes:    514
schematic nets split on pcb: 0
pcb nets split on schematic: 0
nodes only in schematic: 6   (XA1 ICSP: 5V2, GND4, MISO, MOSI, RST2, SCK — no pads in the IO_Only footprint)
nodes only in pcb:       0
RESULT: PASS
```

The exported schematic netlist was also compared with the pre-HW-H2-01
netlist. Both have 190 nets with identical node partitions. Only three
single-node, schematic-only nets were renamed (`MISO`, `MOSI`, `SCLK` →
`unconnected-(XA1-SPI_*)`). PWR_FLAGs and no-connects add no netlist nodes.

## 5. DRC — 397 reported + 199 excluded

Raw reports: `verification/drc.rpt` (reported items),
`verification/drc-with-exclusions.rpt` (all items, exclusions marked
`(excluded)` with their comment).

| Category | Count | Severity | Disposition |
|---|---:|---|---|
| `courtyards_overlap` | 0 reported / **199 excluded** | error | §5.1. Per-item exclusions. No physical clash found. |
| `solder_mask_bridge` | 19 | error | §5.2. **Documented exception, not excluded.** Intentional JP6–JP11 jumpers. |
| `shorting_items` | 12 | error | §5.2. **Documented exception, not excluded.** Intentional JP6–JP11 copper bridges. |
| `silk_overlap` | 199 | warning | **Open, cosmetic.** Mostly proto-pad silk circles touching each other on the 2.54 mm grid, plus dense reference text. No copper effect. Legibility is reviewed against the assembly drawings. Unchanged from upstream. |
| `lib_footprint_mismatch` | 143 | warning | **Open, library vintage.** Embedded KiCad 5 footprints differ from KiCad 9 library copies. The embedded footprints are the manufacturing authority. The footprint-by-footprint audit is not done. Unchanged; the R-05 local libraries added no mismatches. |
| `silk_over_copper` | 19 | warning | **Open, cosmetic.** Silk clipped by mask openings (U5, A1, J3, XA1 outline, logo). The fab clips the silk. Unchanged from upstream. |
| `silk_edge_clearance` | 5 | warning | **Open, cosmetic.** Shield-outline silk at the board edge. Unchanged from upstream. |
| `lib_footprint_issues` | 0 (was 4) | warning | Fixed by R-05. |

The second item KiCad prints for some silk warnings ("Rule Area on F.Cu and
B.Cu") is a report-formatting artefact. Those warnings are identical,
item for item, to the pre-revision run, so the new keep-outs did not cause
them. Before/after comparison of all `silk_*` items: no new and no removed
items.

### 5.1 Courtyard overlaps (HW-DRC-01) — 199 excluded, per item

All 199 were classified from the pre-exclusion DRC JSON
(`verification/drc-for-exclusions.json`). The exclusion keys were generated
by `hardware/tools/apply_hw_h2_01_revision.py exclusions`. Full list with
coordinates, UUIDs and justification: `verification/drc_exclusions.csv`.

| Class | Count | Items | Physical clash? | Justification |
|---|---:|---|---|---|
| P | 174 | proto pad `REF**` ↔ proto pad `REF**` | No | Unpopulated `Connector_Pin:Pin_D1.0mm_L10.0mm` pads on a 2.54 mm grid. No net, no part, not in BOM or position file. The 3.09 mm courtyard is wider than the pitch. Copper clearance passes. |
| H | 22 | proto pad ↔ J1 (7), J6 (6), J8 (5), J5 (4) | No | The headers sit on the same 2.54 mm proto grid. Pad centres are 2.54 mm apart. Only the header housing is a physical body. |
| C | 3 | J1↔J5, J5↔J6, J6↔J8 | **No (zero gap)** | End pins are exactly 2.54 mm apart (J1.5 (198.255, 71.24) ↔ J5.1 (198.255, 68.70); J5.4 (198.255, 61.08) ↔ J6.5 (195.715, 61.08); J6.1 (185.555, 61.08) ↔ J8.3 (183.015, 61.08)). The housings butt like a broken strip. The courtyards overlap by ≈1.14 mm. **Reviewer must confirm the header MPN housing ≤ 2.54 mm per position (HW-BOM-01).** |

No overlap involved an SMD part, an IC, a module or the mounting holes. No real
clash needed relocation. An exclusion is tied to item UUIDs and the marker
position. If a listed footprint moves, the overlap is reported again.

### 5.2 JP6–JP11 solder-jumper exceptions (HW-DRC-04) — 31 items, not excluded

Every item was checked against the jumper footprints and the schematic/netlist.
All 31 lie on JP6–JP11 pads or their bridge polygons. JP6 (140.00, 111.20),
JP7 (140.00, 113.80), JP8 (140.00, 116.40), JP9 (140.00, 119.00) and JP10
(140.00, 121.60) are on B.Cu. JP11 (157.99, 116.33) is on F.Cu. No item is
anywhere else. The defaults match the schematic symbols
`Jumper:SolderJumper_2_Bridged` (JP6–JP10) and `Jumper:Jumper_3_Bridged12`
(JP11), and the bottom silkscreen note "SEVER ASSIGNED JUMPER WHEN USING PROTO
CONNECTOR LINK".

| # | Category | Jumper | Side | Item locations (mm) | Default configuration (schematic/netlist) |
|---:|---|---|---|---|---|
| 1 | `shorting_items` | JP11 | F.Cu | (157.0880, 116.0320) Polygon [(no net)]; (156.6880, 116.3320) Pad 1 [MAP-EXT] | bridged A–C (pads 1–2): `MAP-EXT` (J4.21, R34) ↔ on-board MAP sensor output U5.1; pad 3 (B, R55) open |
| 2 | `shorting_items` | JP11 | F.Cu | (157.0880, 116.0320) Polygon [(no net)]; (157.9880, 116.3320) Pad 2 [Net-(JP11-Pad2)] | bridged A–C (pads 1–2): `MAP-EXT` (J4.21, R34) ↔ on-board MAP sensor output U5.1; pad 3 (B, R55) open |
| 3 | `shorting_items` | JP8 | B.Cu | (140.6500, 116.4000) Pad 1 [FUELPUMP-OUT]; (140.2500, 116.1000) Polygon [(no net)] | bridged 1–2: `FUELPUMP-OUT` (Q3 drain) → `PROTO-3` (J4.31, J7.3) |
| 4 | `shorting_items` | JP8 | B.Cu | (140.2500, 116.1000) Polygon [(no net)]; (139.3500, 116.4000) Pad 2 [PROTO-3] | bridged 1–2: `FUELPUMP-OUT` (Q3 drain) → `PROTO-3` (J4.31, J7.3) |
| 5 | `shorting_items` | JP6 | B.Cu | (140.6500, 111.2000) Pad 1 [ADC-Clamp-10]; (140.2500, 110.9000) Polygon [(no net)] | bridged 1–2: Flex input `ADC-Clamp-10` (XA1.D2) → `PROTO-1` (J4.27, J7.1) |
| 6 | `shorting_items` | JP6 | B.Cu | (139.3500, 111.2000) Pad 2 [PROTO-1]; (140.2500, 110.9000) Polygon [(no net)] | bridged 1–2: Flex input `ADC-Clamp-10` (XA1.D2) → `PROTO-1` (J4.27, J7.1) |
| 7 | `shorting_items` | JP7 | B.Cu | (140.2500, 113.5000) Polygon [(no net)]; (140.6500, 113.8000) Pad 1 [FAN-OUT] | bridged 1–2: `FAN-OUT` (Q2 drain) → `PROTO-2` (J4.29, J7.2) |
| 8 | `shorting_items` | JP7 | B.Cu | (139.3500, 113.8000) Pad 2 [PROTO-2]; (140.2500, 113.5000) Polygon [(no net)] | bridged 1–2: `FAN-OUT` (Q2 drain) → `PROTO-2` (J4.29, J7.2) |
| 9 | `shorting_items` | JP9 | B.Cu | (140.6500, 119.0000) Pad 1 [TACHO-OUT]; (140.2500, 118.7000) Polygon [(no net)] | bridged 1–2: `TACHO-OUT` (Q1 drain) → `PROTO-4` (J4.33, J7.4) |
| 10 | `shorting_items` | JP9 | B.Cu | (139.3500, 119.0000) Pad 2 [PROTO-4]; (140.2500, 118.7000) Polygon [(no net)] | bridged 1–2: `TACHO-OUT` (Q1 drain) → `PROTO-4` (J4.33, J7.4) |
| 11 | `shorting_items` | JP10 | B.Cu | (140.6500, 121.6000) Pad 1 [Clutch_in]; (140.2500, 121.3000) Polygon [(no net)] | bridged 1–2: `Clutch_in` (R37) → `PROTO-5` (J4.35, J7.5) |
| 12 | `shorting_items` | JP10 | B.Cu | (139.3500, 121.6000) Pad 2 [PROTO-5]; (140.2500, 121.3000) Polygon [(no net)] | bridged 1–2: `Clutch_in` (R37) → `PROTO-5` (J4.35, J7.5) |
| 13 | `solder_mask_bridge` | JP11 | F.Cu | (156.6880, 116.3320) Pad 1 [MAP-EXT]; (157.9880, 116.3320) Pad 2 [Net-(JP11-Pad2)] | bridged A–C (pads 1–2): `MAP-EXT` (J4.21, R34) ↔ on-board MAP sensor output U5.1; pad 3 (B, R55) open |
| 14 | `solder_mask_bridge` | JP11 | F.Cu | (157.9880, 116.3320) Pad 2 [Net-(JP11-Pad2)]; (159.2880, 116.3320) Pad 3 [Net-(JP11-Pad3)] | bridged A–C (pads 1–2): `MAP-EXT` (J4.21, R34) ↔ on-board MAP sensor output U5.1; pad 3 (B, R55) open |
| 15 | `solder_mask_bridge` | JP11 | F.Cu | (157.0880, 116.0320) Polygon [(no net)]; (156.6880, 116.3320) Pad 1 [MAP-EXT] | bridged A–C (pads 1–2): `MAP-EXT` (J4.21, R34) ↔ on-board MAP sensor output U5.1; pad 3 (B, R55) open |
| 16 | `solder_mask_bridge` | JP11 | F.Cu | (157.0880, 116.0320) Polygon [(no net)]; (157.9880, 116.3320) Pad 2 [Net-(JP11-Pad2)] | bridged A–C (pads 1–2): `MAP-EXT` (J4.21, R34) ↔ on-board MAP sensor output U5.1; pad 3 (B, R55) open |
| 17 | `solder_mask_bridge` | JP8 | B.Cu | (140.6500, 116.4000) Pad 1 [FUELPUMP-OUT]; (139.3500, 116.4000) Pad 2 [PROTO-3] | bridged 1–2: `FUELPUMP-OUT` (Q3 drain) → `PROTO-3` (J4.31, J7.3) |
| 18 | `solder_mask_bridge` | JP8 | B.Cu | (140.2500, 116.1000) Polygon [(no net)]; (140.6500, 116.4000) Pad 1 [FUELPUMP-OUT] | bridged 1–2: `FUELPUMP-OUT` (Q3 drain) → `PROTO-3` (J4.31, J7.3) |
| 19 | `solder_mask_bridge` | JP8 | B.Cu | (140.2500, 116.1000) Polygon [(no net)]; (139.3500, 116.4000) Pad 2 [PROTO-3] | bridged 1–2: `FUELPUMP-OUT` (Q3 drain) → `PROTO-3` (J4.31, J7.3) |
| 20 | `solder_mask_bridge` | JP6 | B.Cu | (140.6500, 111.2000) Pad 1 [ADC-Clamp-10]; (139.3500, 111.2000) Pad 2 [PROTO-1] | bridged 1–2: Flex input `ADC-Clamp-10` (XA1.D2) → `PROTO-1` (J4.27, J7.1) |
| 21 | `solder_mask_bridge` | JP6 | B.Cu | (140.2500, 110.9000) Polygon [(no net)]; (140.6500, 111.2000) Pad 1 [ADC-Clamp-10] | bridged 1–2: Flex input `ADC-Clamp-10` (XA1.D2) → `PROTO-1` (J4.27, J7.1) |
| 22 | `solder_mask_bridge` | JP6 | B.Cu | (140.2500, 110.9000) Polygon [(no net)]; (139.3500, 111.2000) Pad 2 [PROTO-1] | bridged 1–2: Flex input `ADC-Clamp-10` (XA1.D2) → `PROTO-1` (J4.27, J7.1) |
| 23 | `solder_mask_bridge` | JP7 | B.Cu | (140.6500, 113.8000) Pad 1 [FAN-OUT]; (139.3500, 113.8000) Pad 2 [PROTO-2] | bridged 1–2: `FAN-OUT` (Q2 drain) → `PROTO-2` (J4.29, J7.2) |
| 24 | `solder_mask_bridge` | JP7 | B.Cu | (140.2500, 113.5000) Polygon [(no net)]; (140.6500, 113.8000) Pad 1 [FAN-OUT] | bridged 1–2: `FAN-OUT` (Q2 drain) → `PROTO-2` (J4.29, J7.2) |
| 25 | `solder_mask_bridge` | JP7 | B.Cu | (140.2500, 113.5000) Polygon [(no net)]; (139.3500, 113.8000) Pad 2 [PROTO-2] | bridged 1–2: `FAN-OUT` (Q2 drain) → `PROTO-2` (J4.29, J7.2) |
| 26 | `solder_mask_bridge` | JP9 | B.Cu | (140.6500, 119.0000) Pad 1 [TACHO-OUT]; (139.3500, 119.0000) Pad 2 [PROTO-4] | bridged 1–2: `TACHO-OUT` (Q1 drain) → `PROTO-4` (J4.33, J7.4) |
| 27 | `solder_mask_bridge` | JP9 | B.Cu | (140.2500, 118.7000) Polygon [(no net)]; (140.6500, 119.0000) Pad 1 [TACHO-OUT] | bridged 1–2: `TACHO-OUT` (Q1 drain) → `PROTO-4` (J4.33, J7.4) |
| 28 | `solder_mask_bridge` | JP9 | B.Cu | (140.2500, 118.7000) Polygon [(no net)]; (139.3500, 119.0000) Pad 2 [PROTO-4] | bridged 1–2: `TACHO-OUT` (Q1 drain) → `PROTO-4` (J4.33, J7.4) |
| 29 | `solder_mask_bridge` | JP10 | B.Cu | (140.6500, 121.6000) Pad 1 [Clutch_in]; (139.3500, 121.6000) Pad 2 [PROTO-5] | bridged 1–2: `Clutch_in` (R37) → `PROTO-5` (J4.35, J7.5) |
| 30 | `solder_mask_bridge` | JP10 | B.Cu | (140.2500, 121.3000) Polygon [(no net)]; (140.6500, 121.6000) Pad 1 [Clutch_in] | bridged 1–2: `Clutch_in` (R37) → `PROTO-5` (J4.35, J7.5) |
| 31 | `solder_mask_bridge` | JP10 | B.Cu | (140.2500, 121.3000) Polygon [(no net)]; (139.3500, 121.6000) Pad 2 [PROTO-5] | bridged 1–2: `Clutch_in` (R37) → `PROTO-5` (J4.35, J7.5) |

Notes for the reviewer:

- JP11 has **no** copper bridge between pads 2 and 3. Row 14 is a shared mask
  opening only, as the bridged-1-2 footprint is designed.
- With JP11 as delivered, the on-board MAP sensor output (U5.1) is also present
  on IDC40 pin 21 (`MAP-EXT`). An external MAP source on pin 21 **requires
  cutting JP11 1–2 first**, or the two sources fight.
- With JP6–JP10 as delivered, `PROTO-1…5` on IDC40 pins 27/29/31/33/35 and J7
  carry the Flex, Fan, Fuel-pump, Tacho and Clutch functions. Any other use of
  those pins requires cutting the matching jumper.
- These are kept as visible DRC errors, not exclusions. Remodelling them as
  net-tie footprints would hide a real short if a jumper were wrongly specified.

## 6. Mounting holes and mechanical package (HW-MECH-01/02)

`verification/mounting-holes.txt` (measured on the final board):

| Hole | Centre (mm) | Drill | Nearest discrete copper | Nearest zone fill | Nearest courtyard | Result |
|---|---|---|---|---|---|---|
| H1 | (113.975, 40.475) | 3.2 NPTH | 6.51 (U5 NPTH tab) | 3.80 | 17.38 (A1) | PASS |
| H2 | (113.975, 133.525) | 3.2 NPTH | 12.09 (J7.5) | 3.80 | 10.46 (J7) | PASS |
| H3 | (207.025, 133.525) | 3.2 NPTH | 5.11 (GND via) | 3.80 | 9.76 (IC3) | PASS |

Top-right corner: a grid search found 0 sites meeting ≥ 3.8 mm copper and
≥ 3.45 mm courtyard clearance (J3 + VDD edge track). The support strategy and
the stated assumptions are in `hardware/prototype-v1/FABRICATION_NOTES.md` §3.

## 7. Manufacturing export verification

| Output | File(s) | Result |
|---|---|---|
| Gerbers | 9 layers + `.gbrjob` | Generated. Zone fills were XOR-compared with the pre-revision board on F.Cu and B.Cu. Copper is removed only inside the three keep-outs (29.45 mm² per hole per layer, bboxes at the H1/H2/H3 corners). No copper is added anywhere. Silk gains the H1/H2 labels. Mask gains the three NPTH openings. Edge.Cuts changes are timestamps only. |
| Drill | `v0.4.4d.drl`, `v0.4.4d-drl_map.pdf`, `v0.4.4d-drill-report.txt` | 529 PTH + 5 NPTH (3 × 3.2 mm H1–H3, 2 × 4.1 mm U5). NPTH tools declared NonPlated. |
| Fabrication drawing | `drill/v0.4.4d-fab-drawing.pdf` | Outline, actual-size drills, 100.05 mm and 93.05 mm dimensions, fab notes |
| Assembly drawings | `assembly/v0.4.4d-assembly-top.pdf`, `…-bottom.pdf` (mirrored) | F/B Fab + Silkscreen + outline: references, pin-1/polarity marks, connectors, jumpers, proto area, holes |
| 3D renders | `verification/board-top.png`, `board-bottom.png` | Visual orientation only. Not a fit check. |
| Position | `v0.4.4d-pos.csv` | 141 placements (123 SMD + 18 THT). No `REF**`, H1–H3 or jumpers. |
| Bare-board net test | `v0.4.4d-ipc-d-356.net` | Generated. H1–H3 appear as `N/C` NPTH records. |
| BOM | `v0.4.4d-bom.csv` | 47 grouped rows. C19 library nickname changed only. 11 rows still lack MPN (HW-BOM-01). |
| Schematic PDF / netlist | `v0.4.4d-schematic.pdf`, `v0.4.4d.net` | Generated from the revised schematic |
| Fabrication/assembly note | `hardware/prototype-v1/FABRICATION_NOTES.md` | Stack-up, assumptions, hole coordinates, drill summary, hand-solder vs reflow |

The Gerber and drill files are **review artefacts**. No board house has
DFM-checked them, and nothing has been manufactured.

## 8. H2/H3 status and what a human must decide

- **H2 (engineering checks):** all checks run and reproducible. Remaining
  errors: ERC 4 (U6 spare channels) and DRC 31 (JP6–JP11). Both are documented
  exceptions. 199 courtyard items are excluded with justification. **H2 is
  recorded as "complete with exceptions, pending independent acceptance", not
  as a pass.** An AI-produced disposition is not acceptance.
- **H3 (independent fabrication-review candidate):** **not ready.** Still open:
  - HW-BOM-01: 11 BOM rows have no MPN.
  - Footprint-library audit: 143 `lib_footprint_mismatch` warnings.
  - The cosmetic silk warnings.
  - Independent review (HW-REVIEW-01).
- Reviewer decisions needed:
  1. Accept or reject E-ERC-I7/I11/I12/I14.
  2. Accept the 3 class-C header courtyard exclusions against the chosen header
     part.
  3. Accept the JP6–JP11 default bridges.
  4. Accept the 3-hole mounting pattern and the top-right strategy.
  5. Confirm the [ASSUMPTION] items in `FABRICATION_NOTES.md` (material, copper
     weight, finish, colours).
  6. Board-house DFM against the 0.05 mm via-annular rule.
- H4 (fabrication approval) is **human-only**.
