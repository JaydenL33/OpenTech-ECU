# Hardware Baseline Selection — Speeduino v0.4.4d SMD

| Field | Value |
|---|---|
| Document ID | AUD-HW-BASE-01 |
| Status | Candidate baseline selected (H0 pass; H1 partial; H2 open); **not fabrication-approved** |
| Date | 2026-11-13 |
| Workstream | Hardware |
| Decision | Adopt **Speeduino v0.4.4d SMD** as prototype-v1 electrical baseline; Option B minimal revision (format save + 3 PCB/manufacturing fixes, no net or topology change) |
| Upstream commit | `ee6a9242834891877744ff2137663b6ca8d79ed6` |
| Preserved source | `hardware/upstream-speeduino/v0.4.4d/` |
| Native review copy | `hardware/prototype-v1/` |
| Related | `AUDIT/UPSTREAM_PROVENANCE.md`, `AUDIT/PCB_CHECK_RESULTS.md`, `HARDWARE_BLOCKERS.md` |

## 1. Evaluation method

The official `speeduino/Hardware` repository was cloned and inspected at the
pinned commit. Candidate boards were assessed from their **actual source files**
and by running KiCad 9.0.9 checks, not from README claims. All numbers below
were produced on 2026-11-12/13 with:

```bash
kicad-cli pcb drc --output <report> <board>.kicad_pcb
```

The same pinned container was used for every board:

```text
kicad/kicad@sha256:e638b79b0321f29395a5b783e94bb9f3c73303e8da15da27b8f5cb4b67a37729
KiCad 9.0.9, release build
```

DRC counts are **not** a quality score. Upstream boards use different footprint
libraries and have different densities; the category breakdown matters more than
the total. The raw reports are retained in `/tmp` during this round and the
selected board's report is committed under
`hardware/prototype-v1/verification/drc.rpt`.

## 2. Candidates and measured facts

| Criterion | Speeduino v0.4.4d SMD | Speeduino v0.4.3d THT | DropBear V2 (v2.0.2) | Speeduino v0.3.7 |
|---|---|---|---|---|
| Upstream path | `v0.4/SMD/Latest/` | `v0.4/THT/Latest/` | `DropBear/V2/` | `v0.3/Latest/` |
| Native KiCad source | Yes (KiCad 5, convertible) | Yes (KiCad 5, convertible) | Yes (KiCad 6/7 native) | **No** — Fritzing `.fzz` only |
| Schematic source | 7 legacy `.sch` sheets | 7 legacy `.sch` sheets | Native `.kicad_sch` sheets | Fritzing |
| Routed PCB | Yes, 1233 track/via objects (178 vias) | Yes, 1113 track/via objects (124 vias) | Yes, 2273 track/via objects (400 vias) | Gerbers only; no editable KiCad PCB |
| Unconnected items (KiCad 9 DRC) | **0** | **0** | **0** | Not measurable (no KiCad PCB) |
| DRC violations (KiCad 9, default rules) | 766 raw; **609 after KiCad 9 zone refill**, **600 after R-01…R-03** (same netlist) | 274 | 268 | Not measurable |
| Shorting-item violations | 12, all solder jumpers JP6–JP11 | 12, all solder jumpers | 0 | Not measurable |
| Board size | 100.05 × 100.05 mm | 100.05 × 100.05 mm | 116.13 × 98.05 mm | n/a |
| Copper layers | 2 | 2 | 2 | n/a |
| Footprints | 297 (147 components + 150 proto-area pads) | 134 | 201 | n/a |
| Nets | 185 | 163 | 248 | n/a |
| MCU platform | Arduino Mega 2560 shield | Arduino Mega 2560 shield | **Teensy 4.1** | Arduino Mega 2560 shield |
| Injector / ignition channels | 4 / 4 | 4 / 4 | 4 / 4 | 4 / 4 |
| Interface connector | **IDC40** + screw terminals | **IDC40** + screw terminals | 2× Delphi Sicma 24 | Screw terminals/headers |
| BOM form | CSV, 39 component groups, MPNs present for most | XLSX, THT parts | CSV, MPNs present | XLSX |
| Assembly data | Paste layers + position file | Paste layers (THT); position file | Paste + position | Gerbers only |
| Hand-assembly difficulty | 0805/SOIC; reflow or JLCPCB | **Through-hole; easiest by hand** | Fine-pitch SMD + Teensy module | Through-hole |
| Firmware path | **Upstream Speeduino Mega2560; build verified** (`AUDIT/FIRMWARE_TEST_EVIDENCE.md`) | Upstream Speeduino Mega2560; not built | Different MCU/toolchain; requires separate validation | Upstream Speeduino Mega2560; not built |
| Bench breakout suitability | **High** — IDC40 is a natural breakout | High | Medium — automotive connectors, no IDC40 | Medium |
| Evo VIII adapter | None verified | None verified | None verified | None verified |

## 3. Decision and rationale

**Selected: Speeduino v0.4.4d SMD, reused with a minimal PCB revision (Option A + B).**

Reasons, in priority order:

1. **It is the latest v0.4 revision and is fully routed.** KiCad 9 reports
   0 unconnected items both before and after the R-01…R-03 revision. This
   directly satisfies the directive's requirement not to end with another
   unrouted board.
2. **It matches the proven Speeduino firmware platform.** The design is an
   Arduino Mega 2560 shield, the primary upstream Speeduino target. Choosing a
   Teensy 4.1 board would add an unvalidated MCU/toolchain port before the first
   bench prototype exists.
3. **The IDC40 interface is the best bench connector of the candidates.** It
   exposes the ECU I/O to a breakout/terminal block without committing to an
   unverified Evo VIII connector. This supports the required architecture:
   Speeduino core → bench breakout → future Evo adapter.
4. **It already has four injector and four ignition channels**, matching the
   4G63 four-cylinder target without adding channels.
5. **It has a real BOM with manufacturer part numbers and assembly data**
   (paste layers and a position file). The THT variant's BOM is XLSX-only and
   its revision is older.
6. **The SMD assembly risk is manageable and lower than hand-building every joint.**
   147 component placements (mostly 0805/SOIC/QSOP) can be assembled by a
   low-cost SMT service or by an experienced hand assembler. The THT board
   remains a documented fallback for fully manual assembly.

The THT variant is **not rejected on electrical grounds**; it is a viable
hand-assembly fallback. It was not selected because it is an older revision,
has no CSV BOM with MPNs in the upstream tree, and would still need the same
ERC/DRC exception review.

DropBear V2 is a credible future candidate for a smaller/vehicle-oriented
revision, but its Teensy 4.1 platform and Delphi Sicma connectors change the
firmware and harness problem. It is recorded as a **future expansion option**,
not the first bench baseline.

v0.3.7 is disqualified for this milestone because it has **no editable KiCad
PCB or schematic source** in the upstream tree. Only Fritzing files and gerbers
exist; the directive requires native KiCad source and validated routing.

## 4. What "reused" means here

The electrical design is upstream v0.4.4d. OpenTech-ECU first made **format-only**
changes to produce the review copy, then a documented **Option B minimal
revision** to clear non-jumper DRC errors:

- legacy `.sch` → native `.kicad_sch` (KiCad 9 save);
- legacy `.kicad_pcb` → native `.kicad_pcb` (KiCad 9 save);
- copper zones re-filled with the KiCad 9 zone filler. This is required because
  KiCad 8+ cannot use the legacy zone-fill strategy; it changed only the copper
  fill polygons, not pads, tracks, nets or outline. It removed 155 legacy
  `clearance` violations (0.2488–0.2494 mm vs a 0.25 mm zone rule) and reduced
  starved thermals from 6 to 4. The netlist partition is unchanged;
- added project library table entries and the upstream rescue library so the
  schematic opens without missing-library errors;
- **R-01…R-03 (Option B, `cc2ed0fd`)** — four starved-thermal GND pads set to
  solid connection; one duplicate 0.552 mm VDDA track segment removed; two U5
  mechanical mounting tabs converted from PTH to NPTH. No net, component value,
  footprint assignment or routing topology changed. DRC 609 → 600 (238 → 230
  errors); detail and per-change review in `AUDIT/HARDWARE_DIFF.md` §2;
- no value, net, footprint or outline edit; no mounting holes added yet.

Connectivity equivalence was re-verified after the revision: 514 of 514 common
`(ref, pin)` nodes have identical net partitions; 0 splits in either direction.
See `hardware/prototype-v1/verification/connectivity.txt`.

## 5. Baseline capability limits (v1 truth)

The v0.4.4d schematic provides:

- 4 injector low-side outputs (`INJ-1-OUT` … `INJ-4-OUT`);
- 4 logic-level ignition outputs (`IGN-1-OUT` … `IGN-4-OUT`);
- idle, boost, fan, fuel-pump and tacho outputs;
- 2 high-current auxiliary outputs (`HC-1-OUT`, `HC-2-OUT`);
- MAP, TPS, CLT, IAT and other analog inputs through the Mega shield;
- VR/Hall crank and cam conditioning;
- an IDC40 interface and screw terminals.

It does **not** provide, on this revision:

- a knock sensor interface;
- a CAN transceiver;
- VVT or drive-by-wire outputs;
- flex-fuel sensing;
- an onboard wideband controller;
- onboard data-logging storage;
- the I/O count needed for full Nexus/Link parity.

Those are **hardware-expansion requirements**, not v1 claims. They must not be
used to block the bench prototype, and they must not be described as supported
merely because a tuning screen exposes a control. See `docs/FEATURE-MATRIX.md`
and `HARDWARE_BLOCKERS.md` `HW-EXP-01`.

## 6. What this baseline does not establish

- It is not an Evo VIII plug-in ECU. No factory connector, pinout, trigger
  pattern, injector impedance or coil topology has been verified.
- It is not automotive-qualified. The board has no conformal coating, no
  validated load-dump/transient protection review, no thermal/EMC test and no
  vibration qualification.
- It is not fabrication-approved. H4 requires an independent human hardware
  review; this document cannot grant it.
- It is not a Nexus/Link feature-equivalent board. Those features require
  future hardware expansion, listed separately in `docs/FEATURE-MATRIX.md` and
  the expansion roadmap.
- The upstream design's ERC/DRC exceptions are **not** cleared by this
  selection; they are enumerated in `AUDIT/PCB_CHECK_RESULTS.md`.

## 7. Next hardware actions after this selection

1. Resolve or formally justify each ERC/DRC category in
   `AUDIT/PCB_CHECK_RESULTS.md` (H2).
2. Replace blank BOM MPNs and obtain dated AUD supplier quotes (see
   `HARDWARE_BLOCKERS.md`, `HW-BOM-01`).
3. Turn the IDC40 pin map in `hardware/bench-interface/` into a breakout
   schematic, assembly drawing and continuity procedure (H3 review package).
4. Obtain independent review; do not fabricate before H4.
