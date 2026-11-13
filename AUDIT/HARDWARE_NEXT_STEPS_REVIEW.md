# Hardware next-steps review — prototype-v1 output stage, coils/injectors, self-audit, what is left

| Field | Value |
|---|---|
| Document ID | AUD-HW-NEXT-01 |
| Reviewer | @hardware-engineer (AI review agent). **This is not an independent human EE review and grants no gate.** |
| Date | 2026-11-13 |
| Base commit | `2b5d4d4d` (master) |
| Scope | Review/decision only. No schematic, PCB, netlist, BOM or manufacturing file was changed. |
| Evidence | `hardware/prototype-v1/manufacturing/assembly/v0.4.4d.net` (KiCad 9.0.9 netlist), `…/v0.4.4d-bom.csv`, `v0.4.4d.kicad_pcb` (J4 pad geometry), `hardware/bench-interface/idc40_pinout.csv`, independent ERC/DRC re-run (§3.1) |

Confidence labels: **[NETLIST]** traced in the netlist/PCB in this review;
**[RE-RUN]** reproduced with kicad-cli here; **[DATASHEET-UNVERIFIED]** a
device parameter the reviewer could not confirm because the ST and Littelfuse
datasheet pages failed to load in this session (timeouts / HTTP 403). Treat
those as questions, not facts.

---

## 1. Verdict on the output-stage characterisation

**Broadly right in conclusion (v1 is not a coil driver; injectors are protected
low-side), but the existing docs contain three factual errors that matter for
a novice auditor.** Corrected picture:

### 1.1 Injector and auxiliary low-side outputs — VNLD5090 (dual-channel, SOIC-8)

| Driver | Channel | MCU pin [NETLIST] | Output net | J4 pins | External diode on output |
|---|---|---|---|---|---|
| U1 | IN1/DRAIN1 | D8 (R5) | `INJ-1-OUT` | 1, 2 | none to 12 V; D5 = indicator path |
| U3 | IN1/DRAIN1 | D9 (R17) | `INJ-2-OUT` | 3, 4 | none to 12 V; D11 = indicator path |
| U3 | IN2/DRAIN2 | D10 (R18) | `INJ-3-OUT` | 5, 7 | none to 12 V; D12 = indicator path |
| U1 | IN2/DRAIN2 | D11 (R6) | `INJ-4-OUT` | 9, 11 | none to 12 V; D6 = indicator path |
| U7 | DRAIN1 | (aux) | `BOOST-OUT` | 12 | D17 freewheel → `12V-SW` |
| U7 | DRAIN2 | (aux) | `HC-1-OUT` | 10 | D18 freewheel → `12V-SW` |
| U9 | DRAIN1 | (aux) | `IDLE-OUT` | 8 | D19 freewheel → `12V-SW` |
| U9 | DRAIN2 | (aux) | `HC-2-OUT` | 6 | D16 freewheel → `12V-SW` |

Findings:

- **F-1 (correction).** Only **U1 and U3** drive the four injectors. **U7 and
  U9 drive BOOST / IDLE / HC-1 / HC-2**, not injectors.
  `COIL_INJECTOR_OPTIONS.md` §1 says "4 injector outputs — U1/U3/U7/U9".
- **F-2 (correction).** **D5/D6/D11/D12 are not flyback diodes.** They are
  1N4448WX signal diodes (BOM) in the injector indicator-LED paths:
  `12V-SW → R9 (2.4 k) → D3 (LED) → D5 → INJ-1-OUT`, and the same for the other
  three channels. **INJ-1..4 have no external clamp to 12 V.** At turn-off the
  injector's inductive energy is absorbed by the VNLD5090's internal
  drain–source clamp. That is the usual fast-closing injector arrangement, so
  it is not a defect, but the docs describe it wrongly. The clamp voltage and
  the repetitive clamp energy (4 injectors × RPM) must be checked against the
  datasheet [DATASHEET-UNVERIFIED]. Only the aux outputs have freewheel
  diodes (D16–D19, S110FA, cathode on `12V-SW`).
- STATUS1/STATUS2 diagnostics on all four VNLD5090s are unconnected
  [NETLIST]. There is no fault feedback to the MCU.
- MCU drive pins D8/D9/D10/D11 have 100 k pull-downs (R1/R15/R16/R2), so the
  injectors are off while the Mega is in reset or unplugged [NETLIST]. That is
  a good default state.
- **Limits.** The BOM says "13 A class". ST's own data was not retrieved here,
  so the R_DS(on), current limit, clamp voltage and E_AS are
  [DATASHEET-UNVERIFIED]. The binding limit for injectors on v1 is probably
  **not the driver but the path**: SOIC-8 dual-channel thermal, and the
  **IDC40/ribbon** carrying injector current on two contacts per injector
  (§5).

### 1.2 Ignition outputs — IXDN602 (dual gate driver)

| MCU pin | Driver input | Driver output | Series R | Net / J4 pin | Indicator |
|---|---|---|---|---|---|
| D40 (R8, 100 k pull-down R4) | U2 IN_B | U2 OUT_B | R14 10 Ω 1206 | `IGN-1-OUT` / **13** | R12 + D2 |
| D38 (R7, R3) | U2 IN_A | U2 OUT_A | R13 | `IGN-2-OUT` / **14** | R11 + D1 |
| D52 (R22, R20) | U4 IN_B | U4 OUT_B | R28 | `IGN-3-OUT` / **16** | R26 + D10 |
| D50 (R21, R19) | U4 IN_A | U4 OUT_A | R27 | `IGN-4-OUT` / **15** | R25 + D9 |

- **Confirmed [NETLIST]: there are no IGBTs, no coil-primary current path and
  no coil clamp on the board.** Each IGN-x-OUT net contains only the J4 pin and
  the 10 Ω resistor. The board drives push-pull gate drive or logic level only.
- `Vdrive` (U2.6, U4.6, C24–C27) comes only from **JP1.2**. JP1.1 = `VDD`
  (on-board 5 V from LM2940 U8). JP1.3 = `12V-SW`. Confirmed.
  - JP1 1–2 → IGN outputs swing 0–5 V. This suits a smart-coil logic input
    **if** that coil's input is 5 V-compatible and draws little current.
  - JP1 2–3 → IGN outputs swing 0–~12 V. This is for an external igniter or
    IGBT gate that needs it, and **can overstress a 5 V-only coil input**.
  - JP1 **open** → IXDN602 unpowered and outputs undefined. JP1 must be fitted.
- **F-3 (new risk) — no output current limit except 10 Ω.** A short, a
  mis-wired dumb coil, or a low-impedance igniter input on IGN-x-OUT gives
  ≈0.5 A at 5 V (2.5 W in a 1206 resistor) or ≈1.2 A at 12 V (~14 W). R13/R14/
  R27/R28 will overheat, and in the 5 V position the current is drawn from the
  LM2940 rail that also feeds the Mega. **Never connect a coil primary to
  IGN-x-OUT.**
- **F-4 (new risk) — back-drive.** Some coil/igniter inputs have an internal
  pull-up. A pull-up to 12 V on an IGN pin with JP1 at 5 V would push current
  through the IXDN602 output into the 5 V rail [DATASHEET-UNVERIFIED for the
  IXDN602 internal structure; the risk is generic to CMOS push-pull outputs].
  Measure this first (§2.3).
- IXDN602 polarity: "IXDN" is understood to be the non-inverting variant
  [DATASHEET-UNVERIFIED]. The firmware "ignition output polarity" setting must
  be checked on a scope before any coil is connected.

### 1.3 Supply protection — correction relevant to bench safety

- **F-5 (correction).** `SELF_AUDIT_GUIDE.md` §3 lists "12 V input path, fuse
  F1/F2". **F1 and F2 are 5 V-side polyfuses** (0ZCC0050FF2C): F1 `VDD → VDDA`
  (sensor 5 V, J4.25/26) and F2 `VDD → J1`. **There is no fuse on `12V-SW`.**
- D14 (B130, 1 A Schottky) and D15 (SMBJ40A TVS) protect only `VRegIn`, the
  regulator input [NETLIST]. `12V-SW` goes straight from J3.2 to A1 VMOT, JP1.3,
  the LED resistors, J8 and the cathodes of D16–D19.
- **Consequence:** reverse polarity on J3 is **not** survivable by design. A
  low-impedance path exists: GND → U7/U9 body diodes → D16–D19 → `12V-SW`. The
  DRV8825 VMOT and (with JP1 at 12 V) the IXDN602s are also exposed. For the
  bench: use an **external inline fuse, a keyed/polarised supply lead and a
  current-limited supply**. This is a v2 design input, not a v1 change.

### 1.4 Connector (J4 IDC40) pinout traps [NETLIST + PCB]

1. **IGN-3 is pin 16 and IGN-4 is pin 15** (not 15/16).
2. Injectors are not on consecutive pins: INJ-3 = 5 + 7 and INJ-4 = 9 + 11.
   Pin 6 is HC-2, pin 8 IDLE and pin 10 HC-1, between the injector pins.
3. **The schematic symbol is `Conn_02x20_Counter_Clockwise` but the footprint
   uses standard odd/even IDC numbering.** Pad 1 is at (0, 0) and pad 2 at
   (2.54, 0) in the J4 footprint. Pin numbers match electrically, so ribbon
   wire *n* = pad *n*. The schematic drawing, however, puts pins 1–20 on one
   side and 21–40 on the other. **Audit pins by number, never by their
   position in the schematic drawing.**
4. **J4 has no +12 V pin.** Injector, coil and igniter supply must come from
   outside. There are only four GND pins (17/19/23/36), and they are shared
   with sensor returns. High-current returns must not go through the ribbon.
5. **J4.21 `MAP-EXT` is hard-wired to the on-board MAP sensor output U5.1**
   through JP11 A–C (default bridged), then R34 → A3. An external MAP sensor
   on pin 21 with U5 fitted gives two outputs fighting. Either cut JP11 A–C
   and bridge B–C (U5 then goes through R55 to A5), or **do not fit U5**.

---

## 2. Smart vs dumb coils

### 2.1 Review of `COIL_INJECTOR_OPTIONS.md` §2

The direction is **technically correct and appropriately cautious**: a smart
coil gets a logic dwell signal, a dumb coil needs a power stage that v1 does
not have, and the doc says to test one channel first. Corrections:

- §1 table and §3: fix F-1/F-2 (U7/U9 are aux; D5/D6/D11/D12 are indicator
  diodes; injector turn-off is clamped inside the driver).
- §2A: add F-3 (10 Ω-only current limit) and F-4 (input pull-up back-drive).
  Add "JP1 2–3 can overstress a 5 V-only input".
- §2A: "Examples people commonly mean by R35, Yaris, Audi/VW: a coil with a
  built-in igniter" reads as a classification. Reword so that the
  classification is **per part number, from a datasheet or OEM manual**.
- §2B: a v2/daughter board should take the **5 V logic** IGN-x-OUT and
  generate gate drive **next to** the IGBT. Driving an IGBT gate at 12 V down a
  ribbon cable is poor practice (gate ringing, ground bounce).
- §7: add "never fire a coil with an open secondary; use a grounded spark gap
  or plug."

### 2.2 Decision table (no part is approved by this review)

| Family as commonly named | Commonly reported type | Evidence in repo | Status | Usable on v1? |
|---|---|---|---|---|
| Nissan R35 / VR38-style coil | Integrated igniter (smart) | **None** | UNVERIFIED | Only after §2.3 passes for the exact part number |
| Toyota Yaris / NZ/ZZ-family coil | Integrated igniter; some have an IGF feedback pin | **None** | UNVERIFIED | Only after §2.3; IGF is not used by v1 |
| Audi/VW 4-pin "pencil" coil (various 06x/07x numbers) | Integrated igniter | **None** | UNVERIFIED | Only after §2.3 |
| Evo VIII OEM coil packs | **Unknown in this repo**: may need an external igniter | None | UNVERIFIED | Do not assume; service-manual evidence needed |
| Any 2-terminal primary coil (e.g. older distributor/canister/wasted-spark) | Dumb | n/a | — | **No.** Needs v2/daughter board |
| Aftermarket "replacement" or clone of any of the above | Unknown; internals may differ from OEM | none | UNVERIFIED | Treat as a separate part |

"Commonly reported" means aftermarket-ECU community usage. It is **not** a
datasheet basis. Classify a coil from the **exact part number** printed on the
coil, plus a datasheet or OEM workshop manual.

### 2.3 Per-coil questions — answer and record before connecting to IGN-x-OUT

1. Exact part number (photo of the label) and source document.
2. Pin functions: +12 V, power GND, signal GND (if separate), trigger (IGT),
   feedback (IGF), if any.
3. Is the trigger **active-high** (dwell while high)? Idle level?
4. Trigger input voltage rating: is 5 V enough, and is 12 V allowed? This sets
   JP1.
5. **Trigger input current at 5 V.** Measure with the coil powered from a
   current-limited supply and the trigger fed through a known resistor and an
   ammeter, not from the ECU. Accept only a few mA. Anything near 100 mA or
   more means "do not connect".
6. **Does the trigger input have an internal pull-up, and to what?** Measure
   the open-circuit trigger-pin voltage with the coil powered. If it is above
   5 V, it must not go on JP1 = 5 V.
7. Recommended dwell vs battery voltage, maximum dwell, and whether there is
   an internal over-dwell/thermal shutdown.
8. Secondary path: plug or spark gap grounded to the coil's ground, never an
   open secondary.
9. Bench sequence: the IGN-x-OUT waveform into a 1 kΩ resistive dummy on a
   scope (polarity, idle low at boot, dwell, no glitch at Mega reset) → one
   coil → one channel at a time. Measure the coil supply current with a
   current probe.

### 2.4 Minimum v2 / daughter-board scope for dumb coils — a scope, **not a design**

Recommended form: **a separate coil-driver daughter board fed by v1 IGN-x-OUT
at 5 V logic (JP1 1–2)**. This leaves v1 unchanged. Each channel needs:

- An ignition-rated IGBT or ignition driver IC with a collector clamp,
  integrated or external, rated for the coil's primary flyback voltage and
  peak current.
- Local gate drive or a logic-level gate. Gate resistor and gate–emitter
  pull-down so the IGBT stays off when the input floats or the ribbon is
  unplugged.
- A **hardware over-dwell / current limit** that does not depend on firmware.
- Input conditioning: series R, pull-down, ESD clamp. A defined off state when
  v1 is unpowered.

For the whole board:

- Coil supply bulk plus ceramic decoupling near the coils' +12 V.
- **A separate high-current coil ground** returning to the supply star point,
  not through the IDC40.
- A fused coil supply and a power connector rated for the current (not the
  IDC40).
- Thermal sizing: IGBT dissipation at maximum RPM × dwell, copper area,
  temperature rise.
- Creepage and clearance around collector nodes for the clamp voltage.
  Collector test points that are HV-safe or isolated.
- Scope test points on the logic input, gate and current-sense.
- Its own ERC/DRC, BOM and **its own independent review**.

Injector-side v2 items (only if low-Z is required): peak-and-hold drivers with
a matching injector connector rated for the current. Main-board v2 inputs from
this review: a fuse and reverse-polarity protection on `12V-SW`, and VNLD
STATUS feedback to the MCU.

---

## 3. Self-audit guide (`SELF_AUDIT_GUIDE.md`)

### 3.1 Counts reproduced independently [RE-RUN]

The project was copied to a scratch directory, so no tracked file was
rewritten. The checks were run with the pinned image
`kicad/kicad@sha256:e638b79b0321f29395a5b783e94bb9f3c73303e8da15da27b8f5cb4b67a37729`
(`kicad-cli --version` → `9.0.9`):

```text
kicad-cli sch erc --output erc.rpt v0.4.4d.kicad_sch                         rc=0
kicad-cli pcb drc --output drc.rpt v0.4.4d.kicad_pcb                          rc=0
kicad-cli pcb drc --severity-all --severity-exclusions --output drcx.rpt …    rc=0
```

| Item | Claimed | Re-run |
|---|---|---|
| ERC | 49 = 4 errors + 45 warnings | **49 = 4 + 45** ✔ (34 `global_label_dangling`, 11 `multiple_net_names`, 4 `pin_not_driven`) |
| 4 ERC errors | U6 spare SP720 channels pins 7, 12, 13, 15 | ✔ `pin_not_driven` on U6 only. `ADC-Clamp-7/11/12/14` are single-pad nets |
| DRC reported | 397 = 31 errors + 366 warnings | **397 = 31 + 366** ✔ |
| 31 DRC errors | JP6–JP11 | ✔ 19 `solder_mask_bridge` + 12 `shorting_items` |
| Exclusions | 199 courtyard | ✔ 596 with exclusions − 397 = **199, all `courtyards_overlap`** |
| Unconnected | 0 | ✔ 0 unconnected pads |

kicad-cli exit code 0 does **not** mean "no violations". Always read the
counts.

### 3.2 Corrections and novice traps

| # | Location | Issue | Correction |
|---|---|---|---|
| S-1 | §3 Power | "12 V input path, fuse F1/F2" | F1/F2 are 5 V-side polyfuses (VDDA, J1). There is no fuse on `12V-SW`. D14/D15 protect only the regulator input (F-5). |
| S-2 | §3 Fuel & ignition | "D5/D6/D11/D12 flyback diodes" | Indicator-LED series diodes. The injector clamp is inside the VNLD5090. Injectors are U1/U3; U7/U9 are aux (F-1/F-2). |
| S-3 | §3 Outputs/connector | J4 audit | Add the counter-clockwise-symbol vs odd/even-footprint trap, IGN-3/4 on 16/15, no +12 V on J4 (§1.4). |
| S-4 | §3 JP11 | "cut before external MAP" | Spell out the cases: cut A–C and bridge B–C, **or** do not fit U5 (also a way around the MPX4250AP EOL). |
| S-5 | §3 Power | JP1 | Add: JP1 must be fitted. At 12 V the IGN outputs swing to ~12 V. |
| S-6 | §1 | Running `check_prototype_v1.sh` | It **rewrites tracked files** in `verification/` and `manufacturing/`. Run `git status` / `git diff --stat` afterwards. Any non-timestamp change in Gerbers, drill, BOM or position files is a finding. Needs Docker and the ~2 GB pinned image. |
| S-7 | §4 | Missing item | 143 `lib_footprint_mismatch` warnings: the footprint audit is not done (an H3 item). The guide does not mention it. |
| S-8 | §4 | "four copper layers? No:" | Reword: "Confirm the board is 2-layer (F.Cu/B.Cu)." |
| S-9 | §3 | "Highlight Net (right-click)" | Menu location varies by KiCad version. The default hotkey is `` ` `` (backtick) in both editors. Cross-probe by selecting in one editor. |
| S-10 | §6 step 2 | Continuity checks | In diode mode, each VNLD output reads a diode drop from GND→OUT (body diode) and open from OUT→GND. **That is normal, not a short.** `12V-SW`→GND will not read open (LED strings, DRV8825). Record the values; don't expect infinity. |
| S-11 | §6 step 5 | "3.3 V (if present)" | The shield has no 3.3 V rail. Measure `VDD` (5 V), `VDDA` (after F1), `Vdrive` (JP1.2). The Mega's 3.3 V is the Mega's. |
| S-12 | §6 | Bench supply | Add: external fuse plus keyed lead on J3. Reverse polarity is not protected (F-5). |
| S-13 | §6 step 6 | Dummy loads | Specify: injector dummy = resistor of matching resistance and power rating to an external +12 V (or a real high-Z injector with no fuel). IGN dummy = 1 kΩ to GND. Never a coil primary. |

`START_HERE_HARDWARE.md`: no factual error found. Optional: mention the four
aux outputs (U7/U9) next to "4 injector channels".

This review **records** the corrections. The three docs are left unchanged
because they belong to another workstream. Apply S-1…S-13 and §2.1 in a
follow-up doc commit.

---

## 4. Injector decision (v1)

| Injector | Decision for v1 | Basis |
|---|---|---|
| **High-Z (measured ≥ ~10 Ω cold)** | **Use.** ~1 A per channel at 14 V. Two J4 contacts per injector. Driver internal clamp. | [NETLIST] path. Driver margins are [DATASHEET-UNVERIFIED] but ~1 A is far below a "13 A class" device. |
| **Low-Z (≈2–4 Ω)** direct | **No.** | Several amps per channel through IDC/ribbon contacts. Two channels in one SOIC-8 dissipate together. Higher clamp energy. Possible interaction with the current limit. There is no peak-and-hold. |
| Low-Z + **external ballast resistor pack** | Possible bench option **only with EE sign-off**. Size the pack so total resistance ≈ high-Z, and keep it off the PCB and off the ribbon's current budget. | Changes the injector's opening time. Needs dead-time recalibration. |
| Low-Z peak-and-hold | v2/daughter board | Not on v1. |

"Subaru injector" is not a resistance class. Measure each injector, and
record the part number and resistance at room temperature. The impedance of
the Evo VIII OEM injectors is **not established in this repo**.

---

## 5. What is left — prioritised

### A. Must do before ordering (H3 → H4)

1. **Independent human EE review (HW-REVIEW-01).** This is the gate. Give the
   reviewer this doc plus §7's questions. The brother declined, so engage a
   paid or freelance reviewer and get a written pass/fail (see
   `SELF_AUDIT_GUIDE.md` §7).
2. **Decide the MPX4250AP path (EOL, stock 0, ~21 % of cost).** Options:
   (a) owner-supply a genuine part; (b) **DNP U5 and use an external MAP sensor
   on J4.21** for the bench (no PCB change; JP11 can stay bridged if U5 is
   absent; the firmware MAP calibration must match the external sensor);
   (c) an in-production substitute, which needs a footprint and
   transfer-function review. Choosing (b) also removes the J4.21 fight risk.
3. **Close the 9 connector/socket MPNs** (IC3, J1–J8 incl. J3 terminal block
   and **keyed** J4 IDC box header plus the mating cable socket). Also XA1
   header strips and the A1 sockets (hand-solder). The 3 header-pair courtyard
   exclusions depend on the chosen header MPN.
4. **Resolve the zero/low-stock lines**: LH R974-LP-1 LEDs (EOL),
   CL21B224KOCNNNC, SSM3K357R, AYZ0202AGRLC (zero stock); LM2940S-5.0 (1),
   VNLD5090TR-E (30), S110FA, SP720 (low). For each: alternate, consign or
   pre-order. The polyfuse 0ZCC0050FF2C is not in the JLCPCB library. Indicator
   LEDs can be DNP without functional loss; record it.
5. **Regenerate the assembly pack** after steps 2–4 (`generate.py`). Re-check
   that every BOM row has an MPN, that BOM and CPL both list 139 placements,
   and that the hand-solder split (XA1, A1 sockets, THT parts) and JP6–JP11
   "no part" are right. Get a **real quote** (JLCPCB/PCBWay) with DFM.
6. **DFM against the chosen fab**: 0.05 mm via annular ring (`min_via_annular_width`;
   below many low-cost defaults), 0.20 mm track, 0.25 mm hole clearance, and
   the **100.05 × 100.05 mm** outline (some fabs price above 100 × 100 mm
   differently; confirm the quote uses the real size).
7. **Header and mechanical fit**: measure the actual Mega and header stack;
   standoff length; J3 strain relief; the top-right corner strategy.
8. **Human acceptance of the H2 exceptions**: 4 U6 ERC, 31 JP6–JP11, 199
   courtyard exclusions, and a footprint-library spot-check of the 143
   mismatches, at minimum for every power, connector and polarity-sensitive
   part.
9. **Apply the doc corrections in §3.2** so the self-audit is correct before
   anyone uses it on real hardware.

### B. Can wait until after the first bench board (H5/H6)

1. Write the bench test plan as a procedure using S-10…S-13: fused,
   current-limited first power; rail checks; dummy loads; scope captures.
   Update `AUDIT/EVO8_BENCH_TEST_PLAN.md`.
2. Smart-coil trial on one channel after §2.3 passes for an exact part number.
3. Measured injector and driver temperatures at representative duty cycles.
4. Footprint-library audit of all 143 mismatches. Cosmetic silk items.
5. Coil-driver daughter-board scope (§2.4) → design → separate review.
6. v2 main-board inputs: 12 V fuse and reverse-polarity protection, VNLD
   STATUS feedback, a real power/injector connector, Nexus/Link expansion
   (HW-EXP-01).

### C. Must not be connected (any stage until separately approved)

- The Evo VIII or any vehicle harness. A running engine, a fuel system, or a
  pressurised injector.
- **Any coil primary to IGN-x-OUT.** Any coil with an open secondary.
- Low-Z injectors direct to INJ-x-OUT.
- An external MAP sensor on J4.21 while U5 is fitted and JP11 A–C is bridged.
- A supply without an external fuse and current limit, and any reverse-able
  supply lead.
- A coil input with a pull-up above 5 V while JP1 is at 5 V.
- Scope/laptop ground or inputs on any coil primary or secondary node.

---

## 6. Proposed v2 / daughter-board scope

See §2.4 for the coil-driver scope and §5.B.6 for main-board inputs. This is
**explicitly a scope, not a design**. No part is selected, no rating is
asserted and nothing here authorises a PCB change. The order follows the
project directive: build and bench v1 unchanged first, then design the coil
daughter board as a separate hardware item with its own review.

---

## 7. Minimum questions for an independent human EE

Only a human EE can answer these, ideally with datasheets and measurements:

1. Do you accept that **INJ-1..4 rely only on the VNLD5090 internal clamp**?
   Is its clamp energy rating adequate for the chosen injectors (inductance,
   current) at maximum RPM, with 4 channels in 2 packages?
2. Is the **IDC40 + ribbon** acceptable for injector current (two contacts per
   injector) and for returns on four shared GND pins, for high-Z injectors on
   the bench? Which IDC header and cable MPN, and what are its contact ratings?
3. For the chosen smart coil (exact part number): is the IXDN602 + 10 Ω output
   at 5 V (or 12 V) within the coil input's ratings? Is there any back-drive
   risk (F-4)?
4. Do you accept the lack of 12 V fusing and reverse-polarity protection on
   `12V-SW` for a bench board with an external fuse (F-5)? What external fuse
   rating?
5. Do you accept the four U6 spare-channel ERC exceptions, the 31 JP6–JP11
   jumper exceptions and the 199 courtyard exclusions, including the three
   header-pair cases with the chosen header MPN?
6. Are the 1.0 mm tracks carrying injector and ground return current adequate
   at 1 oz copper for the expected bench currents?
7. Is the 0.05 mm via annular ring acceptable to the chosen fab, or should the
   fab's DFM override it?
8. MPX4250AP: approve DNP plus external MAP, owner-supplied, or a named
   substitute?
9. Footprint spot-check: do the embedded footprints for U1/U3/U7/U9, U2/U4,
   U8 (TO-263), D14/D15, J3 and J4 match the chosen MPN datasheets, including
   pin 1 and polarity?
10. A signed pass/fail with findings for H4. AI review cannot provide this.
