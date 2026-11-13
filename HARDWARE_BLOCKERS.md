# Hardware Blockers — OpenTech-ECU

| Field | Value |
|---|---|
| Document ID | HW-BLOCKERS |
| Status | Living register. H0 complete. H1 partial (BOM MPNs). H2 complete with documented exceptions; focused independent review returned ACCEPT WITH FINDINGS. H3 not ready. HW-H2-01 (R-04…R-10) closed the mechanical, library and ERC items. |
| Date | 2026-11-13 |
| Workstream | Hardware |
| Baseline | Speeduino v0.4.4d SMD, upstream commit `ee6a9242834891877744ff2137663b6ca8d79ed6` |
| Review copy | `hardware/prototype-v1/` |
| Related | `AUDIT/HARDWARE_BASELINE.md`, `AUDIT/PCB_CHECK_RESULTS.md`, `AUDIT/UPSTREAM_PROVENANCE.md` |

> This register replaces generic "not manufacturable" warnings with specific,
> testable work items. A blocker is only closed when the stated verification
> passes and the evidence is committed. No blocker below authorises fabrication,
> vehicle connection, or output actuation.

## Gate status

| Gate | Status | Evidence / reason |
|---|---|---|
| **H0 — Verified upstream reference** | **PASS (hardware + build)** | Exact Speeduino Hardware commit, path, licence and hash manifest selected (`AUDIT/UPSTREAM_PROVENANCE.md`). Firmware tree commit `5d217bf4…` builds for `megaatmega2560` with PlatformIO 6.2.0; artifact hashes recorded (`AUDIT/FIRMWARE_TEST_EVIDENCE.md`). Caveat: firmware is an in-tree snapshot, not a tagged upstream release; pin a tag in the review package. |
| **H1 — Complete PCB design package** | **PARTIAL** | Native KiCad 9 schematic + PCB. All 147 components have footprints, 0 unconnected items, closed outline. Gerbers, drill (+ report), BOM, position (141 placements), IPC-D-356 and netlist generated. **Added by HW-H2-01:** 3 M3 NPTH mounting holes, top/bottom assembly drawings, fab drawing, `FABRICATION_NOTES.md` (stack-up and assumptions), repaired library links. **Still missing:** complete BOM MPNs (HW-BOM-01). |
| **H2 — Engineering checks completed** | **COMPLETE WITH DOCUMENTED EXCEPTIONS; focused independent review ACCEPT WITH FINDINGS (not a pass)** | ERC **49 (4 errors, 45 warnings)**, every item dispositioned line by line. The 4 errors are U6 spare clamp channels. DRC **397 reported (31 errors, 366 warnings) + 199 per-item exclusions**, 0 unconnected. The 31 errors are the intentional JP6–JP11 jumpers, not excluded. Connectivity PASS 514/514. Mounting-hole check PASS. Independent continuation review matched 199/199 exclusions by UUID pair, verified all jumpers and rendered the drawings. **Findings:** crowded assembly annotations need human readability/orientation review; standoff detail is in the notes; header MPN, support strategy and H2 exception acceptance remain human decisions. See `AUDIT/PCB_CHECK_RESULTS.md`. Warnings still open: 223 cosmetic silk, 143 footprint-library vintage. |
| **H3 — Independent fabrication review candidate** | **NOT READY** | The mechanical/assembly package now exists. Still open: BOM MPNs (HW-BOM-01), footprint-library audit, reviewer acceptance of the H2 exceptions, and independent review (HW-REVIEW-01). |
| **H4 — Fabrication approved** | **BLOCKED — human only** | Requires independent engineering review and explicit human approval. AI cannot grant. |
| **H5 — Assembled hardware** | **NOT STARTED** | No board fabricated or populated. |
| **H6 — Powered bench testing** | **NOT STARTED** | No power applied. |
| **H7–H9** | **BLOCKED** | Require H4–H6 plus vehicle/engine evidence and independent review. |

## Blocker register

### HW-BASE-01 — Upstream hardware baseline was not preserved

| Field | Value |
|---|---|
| Severity | S1 (blocks H0) |
| Affected files | `hardware/upstream-speeduino/`, `AUDIT/UPSTREAM_PROVENANCE.md` |
| Problem | The repository had no pinned, hash-verified Speeduino hardware source. Only a custom STM32 bench board existed, so there was no firmware-compatible upstream reference. |
| Evidence | Before this round: no `hardware/upstream-speeduino/`, no `AUDIT/UPSTREAM_PROVENANCE.md`. |
| Required action | Clone `speeduino/Hardware` at a fixed commit, preserve the selected revision byte-for-byte, record licence and SHA-256 manifest. |
| Owner | Hardware workstream |
| Autonomous | Yes |
| Verification | `hardware/tools/verify_upstream_baseline.sh` → all files `OK`, exit 0 |
| Status | **CLOSED** |
| Resolved by | `219aa0855f94f0a02765e1b32e08f85d4591877a`; see `AUDIT/UPSTREAM_PROVENANCE.md` and `hardware/upstream-speeduino/SHA256SUMS` |

### HW-BASE-02 — Baseline revision not selected against alternatives

| Field | Value |
|---|---|
| Severity | S1 (blocks H0) |
| Affected files | `AUDIT/HARDWARE_BASELINE.md` |
| Problem | The directive required an evidence-based choice among v0.3, v0.4 SMD/THT and other boards, not an assumption that the newest board is best. |
| Evidence | v0.3 has no editable KiCad source (Fritzing `.fzz` only); DropBear V2 uses Teensy 4.1 + Delphi Sicma; v0.4.4d is a routed Mega 2560 shield with IDC40. |
| Required action | Record criteria, measured routing/DRC facts and a decision. |
| Owner | Hardware workstream |
| Autonomous | Yes |
| Verification | Review `AUDIT/HARDWARE_BASELINE.md` §2–3 against the upstream tree |
| Status | **CLOSED** (selection is a candidate, not fabrication approval) |
| Resolved by | `219aa0855f94f0a02765e1b32e08f85d4591877a`; `AUDIT/HARDWARE_BASELINE.md` |

### HW-FMT-01 — No native KiCad schematic/PCB for the selected board

| Field | Value |
|---|---|
| Severity | S1 (blocks H1) |
| Affected files | `hardware/prototype-v1/*.kicad_sch`, `*.kicad_pcb`, `*.kicad_pro` |
| Problem | Upstream v0.4.4d is KiCad 5 format. The directive requires native `.kicad_sch`/`.kicad_pcb` review files. |
| Evidence | Upstream header `(kicad_pcb (version 20171130) …)`; native copy header `(version 20241229) (generator "pcbnew")`. |
| Required action | Open/save the upstream hierarchy in KiCad 9, refill zones, preserve all electrical content, verify connectivity equivalence. |
| Owner | Hardware workstream |
| Autonomous | Yes |
| Verification | `hardware/tools/check_prototype_v1.sh`; connectivity `RESULT: PASS`; DRC `0 unconnected items` |
| Status | **CLOSED (format-only change; review required)** |
| Resolved by | `219aa0855f94f0a02765e1b32e08f85d4591877a`; see `AUDIT/UPSTREAM_PROVENANCE.md` §4 and `AUDIT/PCB_CHECK_RESULTS.md` §4 |

### HW-ERC-01 — Original 101 ERC violations (closed by HW-H2-01)

| Field | Value |
|---|---|
| Severity | S2 (blocks H2) |
| Affected files | `hardware/prototype-v1/*.kicad_sch`, `verification/erc.rpt` |
| Problem | 44 errors and 57 warnings: 26 unconnected unused pins, 9 undriven power pins, 9 undriven inputs, 37 dangling global labels, 11 intentional net aliases, 2 wire stubs, 1 dangling no-connect, plus library-link warnings. |
| Evidence | `hardware/prototype-v1/verification/erc.rpt`; classification in `AUDIT/PCB_CHECK_RESULTS.md` §3. |
| Required action | Add no-connect flags/PWR_FLAGs where appropriate, terminate the two wire stubs, remove the dangling no-connect, and record any remaining exceptions line-by-line. |
| Owner | Hardware workstream |
| Autonomous | Yes |
| Verification | Re-run `kicad-cli sch erc`; target 0 unexplained errors |
| Status | **CLOSED — 0 unexplained errors; 4 documented exceptions pending reviewer acceptance** |
| Resolved by | HW-H2-01 R-07 (5 PWR_FLAGs) and R-08 (29 no-connects on unused pins; ICSP labels, 2 stubs and the dangling NC removed). ERC 101 (44 E / 57 W) → 49 (4 E / 45 W). The remaining errors are `pin_not_driven` on spare SP720 channels U6.7/12/13/15 (exceptions E-ERC-I7/I11/I12/I14). All 49 items are dispositioned in `AUDIT/PCB_CHECK_RESULTS.md` §3. Netlist partition unchanged. |

### HW-ERC-02 — Missing upstream library links

| Field | Value |
|---|---|
| Severity | S3 (blocks clean H2) |
| Affected files | `hardware/prototype-v1/fp-lib-table`, `sym-lib-table`, `verification/erc.rpt` |
| Problem | Upstream references `Shields`, `Misc`, `Connector_Automotive`, `IC_Automotive` and a hand-solder 0805 footprint variant that are not in the upstream commit. 4 footprint-link warnings and 2 `Device:CP1` symbol warnings remain. |
| Evidence | ERC categories `footprint_link_issues` (4), `lib_symbol_issues` (2). |
| Required action | Recover the missing libraries from the Speeduino ecosystem or replace links with embedded/local equivalents; document any substitution. |
| Owner | Hardware workstream |
| Autonomous | Yes (network/source research) |
| Verification | Re-run ERC; no missing-library warnings except documented legacy exceptions |
| Status | **CLOSED** |
| Resolved by | HW-H2-01 R-05/R-06. Project-local `Shields`, `Misc` and `Speeduino_Legacy` footprint libraries were extracted from the board's embedded footprints. C19 is relinked to `Speeduino_Legacy:`. `Device:CP1` → `Device:C_Polarized_US` (identical pins). Dead `Connector_Automotive`/`IC_Automotive`/`Arduino`/`Connector`/`Misc` table entries were removed: no symbol used them, because every custom symbol resolves through `v0.4.4c-rescue`. ERC `footprint_link_issues` 4→0, `lib_symbol_issues` 2→0; DRC `lib_footprint_issues` 4→0. |

### HW-DRC-01 — 199 courtyard overlaps not mechanically reviewed

| Field | Value |
|---|---|
| Severity | S2 (blocks H2/H3) |
| Affected files | `hardware/prototype-v1/v0.4.4d.kicad_pcb`, `verification/drc.rpt` |
| Problem | KiCad reports 199 courtyard overlaps, concentrated at the Arduino shield, J1/J5 connector area and U5 module. Courtyard overlap may be a real physical clash or a footprint-courtyard artefact. |
| Evidence | DRC category `courtyards_overlap`, 199 errors. |
| Required action | Produce a mechanical assembly drawing/3D check; resolve real overlaps; document accepted footprint-courtyard artefacts. |
| Owner | Hardware workstream + independent reviewer |
| Autonomous | Partly (analysis); final mechanical sign-off needs human review |
| Verification | Re-run DRC; visual/3D clearance review against assembly drawing |
| Status | **ANALYSIS COMPLETE — 199 per-item exclusions with justification; reviewer acceptance pending** |
| Resolved by | HW-H2-01 R-09. 174 overlaps are proto pad ↔ proto pad and 22 are proto pad ↔ J1/J5/J6/J8. The other 3 are J1↔J5, J5↔J6 and J6↔J8: headers on one continuous 2.54 mm pitch that butt with zero gap. No overlap involves an SMD part, IC or module, and none is a physical clash. Each is excluded by its own marker key with a comment. No rule severity was changed. List: `hardware/prototype-v1/verification/drc_exclusions.csv`. Reviewer must confirm the header MPN housing pitch (HW-BOM-01). |

### HW-DRC-02 — 4 starved GND thermal spokes

| Field | Value |
|---|---|
| Severity | S2 (assembly/current-sharing) |
| Affected files | `hardware/prototype-v1/v0.4.4d.kicad_pcb` |
| Problem | C5.2, C10.2, C11.2 and U4.3 have one thermal spoke to the GND pour; rule minimum is 2. Connectivity exists but solderability/current sharing is compromised. |
| Evidence | DRC `starved_thermal` ×4 at `(146.5375,116.6)`, `(142.1375,129.15)`, `(142.1375,124.65)`, `(156.75,98.665)` mm. |
| Required action | Adjust zone thermal settings/spoke placement or add stitching; re-fill and re-check. |
| Owner | Hardware workstream |
| Autonomous | Yes |
| Verification | Re-run DRC; 0 `starved_thermal` |
| Status | **CLOSED** |
| Resolved by | `cc2ed0fd`; R-01 in `AUDIT/HARDWARE_DIFF.md` §2.1 sets C5.2/C10.2/C11.2/U4.3 to a solid GND zone connection; DRC `starved_thermal` 4 → 0, connectivity PASS |

### HW-DRC-03 — VDDA track stub

| Field | Value |
|---|---|
| Severity | S3 |
| Affected files | `hardware/prototype-v1/v0.4.4d.kicad_pcb` |
| Problem | A 0.552 mm VDDA track at `(162.157, 132.991)` mm has an unconnected end. DRC reports 0 unconnected items, so this is likely a redundant stub, but it must be confirmed and removed/terminated. |
| Evidence | DRC `track_dangling` ×1. |
| Required action | Inspect the VDDA route, remove the stub or connect it intentionally; re-run DRC and connectivity. |
| Owner | Hardware workstream |
| Autonomous | Yes |
| Verification | Re-run DRC; 0 `track_dangling`; connectivity PASS |
| Status | **CLOSED** |
| Resolved by | `cc2ed0fd`; R-02 in `AUDIT/HARDWARE_DIFF.md` §2.2 removed the 0.552 mm segment that duplicated the through VDDA track; `track_dangling` 1 → 0, 0 unconnected, connectivity unchanged |

### HW-DRC-04 — Intentional solder-jumper shorts and mask bridges

| Field | Value |
|---|---|
| Severity | S4 (accepted exception, review required) |
| Affected files | `hardware/prototype-v1/v0.4.4d.kicad_pcb` |
| Problem | 12 `shorting_items` and 19 `solder_mask_bridge` violations are all JP6–JP11, whose footprints deliberately bridge pads. If these were not intentional, they would be real shorts. |
| Evidence | All violations reference JP6–JP11; footprint values `SolderJumper_2_Bridged` / `Jumper_3_Bridged12`. |
| Required action | Confirm each jumper's default configuration against the schematic; keep as documented DRC exceptions or model them as net ties. Do not globally suppress the rule. |
| Owner | Hardware workstream + reviewer |
| Autonomous | Analysis yes; acceptance needs review |
| Verification | Schematic/BOM cross-check of JP6–JP11; DRC exception list with justification |
| Status | **ACCEPTED EXCEPTION — pending review (verified, not excluded)** |
| Verification record | HW-H2-01: all 12 `shorting_items` + 19 `solder_mask_bridge` lie on JP6 (140.00, 111.20), JP7 (140.00, 113.80), JP8 (140.00, 116.40), JP9 (140.00, 119.00), JP10 (140.00, 121.60) on B.Cu, and on JP11 (157.99, 116.33) on F.Cu. None is elsewhere. Defaults were cross-checked with the netlist. JP6–JP10 link Flex/FAN/FUELPUMP/TACHO/Clutch to PROTO-1…5 (IDC40 27/29/31/33/35, J7). JP11 links MAP-EXT (IDC40 21) to on-board sensor U5.1. Line-by-line table: `AUDIT/PCB_CHECK_RESULTS.md` §5.2. These items are left as visible DRC errors, deliberately not excluded or remodelled. |

### HW-DRC-05 — U5 mechanical mounting tabs flagged as pads

| Field | Value |
|---|---|
| Severity | S4 (accepted exception) |
| Affected files | `hardware/prototype-v1/v0.4.4d.kicad_pcb` |
| Problem | U5 (MPX4250) has two 4.1 mm mounting tabs modelled as 4.1 mm PTH pads: zero annular copper, `padstack` + `annular_width` errors. |
| Evidence | Pad size = drill = 4.1 mm at `(113.5,72.172)` and `(113.5,49.022)` mm. |
| Required action | Confirm they are mechanical only; ideally convert to NPTH in a future footprint revision. |
| Owner | Hardware workstream |
| Autonomous | Yes (footprint cleanup) |
| Verification | Visual review + DRC after NPTH conversion |
| Status | **CLOSED** |
| Resolved by | `cc2ed0fd`; R-03 in `AUDIT/HARDWARE_DIFF.md` §2.3 converts both tabs to NPTH; `padstack` 2 → 0 and `annular_width` 2 → 0; drill file declares `T9C4.100` NonPlated/NPTH. Mechanical screw/tab stack still needs review under HW-DRC-01. |

### HW-MECH-02 — No dedicated board mounting holes on v0.4.4d

| Field | Value |
|---|---|
| Severity | S2 (blocks H1/H3 mechanical package) |
| Affected files | `hardware/prototype-v1/v0.4.4d.kicad_pcb` |
| Problem | The upstream v0.4.4d board has **no NPTH mounting holes**. It is an Arduino Mega shield and relies on the header array for mechanical location. There are no M3/M2.5 standoff holes for a bench enclosure or test fixture. |
| Evidence | `pcbnew` reports 0 NPTH pads and 0 `MountingHole` footprints; Edge.Cuts is a single 100.05 × 100.05 mm outline. The 2 × 4.1 mm holes belong to the U5 MAP sensor tabs, not board mounting. |
| Required action | In a minimal derived revision (Option B), add 4 corner mounting holes with keepouts after a clearance check against tracks/pads/enclosure; or document an approved header-mounted standoff strategy. Do not drill blind. |
| Owner | Hardware workstream + mechanical reviewer |
| Autonomous | Analysis/placement yes; mechanical approval requires review |
| Verification | DRC after hole placement; clearance to copper ≥ manufacturer rule; assembly drawing shows standoffs |
| Status | **CLOSED (3 holes) — 4th corner documented alternative; mechanical review pending** |
| Resolved by | HW-H2-01 R-04. H1 (113.975, 40.475), H2 (113.975, 133.525) and H3 (207.025, 133.525) mm: 3.2 mm NPTH, no net, excluded from BOM/position, 3.8 mm F.Cu+B.Cu keep-out. Measured: discrete copper ≥ 5.11 mm, fill 3.80 mm, courtyards ≥ 9.76 mm. DRC adds 0 violations (`verification/mounting-holes.txt`). The top-right corner has no feasible site (0 of the grid points, J3 + VDD edge track). The alternative is Mega-standoff support + J3 cable strain relief (`hardware/prototype-v1/FABRICATION_NOTES.md` §3). Standoff length is to be measured on hardware. |

### HW-MECH-03 — Pick-and-place contains 150 unannotated proto pads

| Field | Value |
|---|---|
| Severity | S2 (blocks automated assembly / H3) |
| Affected files | `hardware/prototype-v1/manufacturing/assembly/v0.4.4d-pos.csv` |
| Problem | The position file has 291 rows: 141 real components plus **150 `REF**` proto-area pads** (`Pin_D1.0mm_L10.0mm`). An assembly house would treat these as placements, producing a wrong or rejected job. Six solder jumpers (JP6–JP11) are not in the position file. |
| Evidence | `grep -c '^"REF\*\*"' v0.4.4d-pos.csv` → 150; total rows 291. |
| Required action | Mark the proto pads "exclude from position files" (and exclude from BOM) in a derived revision, or generate an assembly position file filtered to the 141 real placements. Document JP6–JP11 placement/hand-solder intent. |
| Owner | Hardware workstream |
| Autonomous | Yes |
| Verification | Position file has exactly 141 rows (or a documented populated subset); no `REF**`; cross-check with BOM |
| Status | **CLOSED (metadata-only; no copper change)** |
| Resolved by | `3200e3d5`; all 150 `REF**` footprints marked exclude-from-BOM/position, regenerated `v0.4.4d-pos.csv` has 141 real placements and 0 `REF**`; DRC unchanged 609, connectivity PASS |

### HW-BOM-01 — 11 BOM rows lack manufacturer part numbers

| Field | Value |
|---|---|
| Severity | S2 (blocks H3) |
| Affected files | `hardware/prototype-v1/manufacturing/assembly/v0.4.4d-bom.csv` |
| Problem | A1 (Pololu DRV8825 module), IC3 (DIP-8 socket), J1/J6/J7, J2, J3, J4 (IDC40), J5, J8, JP6–JP10, JP11 and XA1 have no MPN: 11 grouped rows covering 17 populated placements. The released BOM must not contain placeholders. Separately, 150 unannotated `REF**` proto-area pads are not BOM items but must be explained in the assembly drawing. |
| Evidence | `v0.4.4d-bom.csv`: 47 grouped rows, 147 populated placements, 36 rows with MPN, 11 without. Detail in `AUDIT/BOM_REVIEW.md`. Upstream CSV gives MPNs for most passive/active parts but not these. |
| Required action | Assign real, purchasable MPNs (connector series, socket, jumper, Arduino shield/module); record supplier and alternatives. |
| Owner | Hardware/BOM workstream |
| Autonomous | Yes (research), purchases prohibited |
| Verification | BOM audit: no blank MPN in release BOM; supplier links resolve |
| Status | **OPEN** |

### HW-BOM-02 — No dated AUD supplier quotes or cost comparison

| Field | Value |
|---|---|
| Severity | S2 (blocks cost claim) |
| Affected files | `AUDIT/BOM_REVIEW.md` (missing), `docs/HARDWARE_COST.md` (missing) |
| Problem | The directive requires actual supplier prices in AUD where accessible, with date, shipping/tax/assembly caveats, and a purchase-vs-build comparison under A$250 if credible. No quotes have been collected in this round. |
| Evidence | No cost document exists; this round deliberately did not invent prices. |
| Required action | Collect dated distributor/reseller quotes for the BOM and an assembled v0.4.4d board; publish AUD comparison with explicit exclusions. |
| Owner | Hardware/BOM workstream |
| Autonomous | Yes (read-only web research) |
| Verification | Prices reproducible from dated URLs; arithmetic checked |
| Status | **OPEN** |

### HW-BENCH-01 — Bench breakout/test-point package not built

| Field | Value |
|---|---|
| Severity | S2 (blocks safe bench use) |
| Affected files | `hardware/bench-interface/` (pin map only), `AUDIT/EVO8_BENCH_TEST_PLAN.md` (plan only) |
| Problem | The v0.4.4d IDC40 pin map now exists, but it has not been turned into a labelled breakout with current-limited supply, fuses, dummy loads, E-stop and scope points. Without the physical breakout, safe powered bring-up is not defined. |
| Evidence | `hardware/bench-interface/README.md` + `idc40_pinout.csv` (netlist-derived, 40 pads); no breakout schematic/assembly/continuity record. Physical pin-1 orientation unverified. |
| Required action | Produce an IDC40 breakout schematic/assembly drawing, test-point map and safe power-up procedure tied to the v0.4.4d pinout; build and buzz-out only after H4 approval. |
| Owner | Hardware workstream |
| Autonomous | Yes (design/package); no power-on |
| Verification | Reviewer can map every IDC40 pin to a labelled terminal/test point; continuity check procedure documented |
| Status | **OPEN — pin map complete; breakout/continuity missing** |

### HW-MECH-01 — No fabrication/assembly drawing or enclosure definition

| Field | Value |
|---|---|
| Severity | S2 (blocks H3) |
| Affected files | `hardware/prototype-v1/manufacturing/assembly/` |
| Problem | Position/BOM/schematic PDF exist, but there is no assembly drawing, drill drawing, stack-up note or mounting/enclosure definition. |
| Evidence | Manufacturing package contains gerbers, drill, position, BOM, schematic PDF only. |
| Required action | Add assembly drawings (top/bottom), fabrication notes, board stack-up and mounting-hole coordinates. |
| Owner | Hardware workstream |
| Autonomous | Yes |
| Verification | Independent reviewer can assemble from the package without guessing orientation |
| Status | **PACKAGE PRODUCED — independent verification pending** |
| Resolved by | HW-H2-01. Outputs: `manufacturing/assembly/v0.4.4d-assembly-top.pdf` and `-bottom.pdf`; `manufacturing/drill/v0.4.4d-fab-drawing.pdf` (dimensions, actual-size drills, notes); `v0.4.4d-drill-report.txt`; `verification/board-bottom.png`; `hardware/prototype-v1/FABRICATION_NOTES.md`. The note covers 2-layer 1.6 mm from upstream evidence; FR-4, copper weight, finish and colours as ASSUMPTIONS; hole coordinates; drill summary; and reflow vs hand-solder intent. The criterion "reviewer can assemble without guessing" can only be met by an independent reviewer. |

### HW-FW-01 — Firmware build path (closed)

| Field | Value |
|---|---|
| Severity | S2 (H0 supporting evidence) |
| Affected files | `platformio.ini`, `speeduino/`, `evidence/hardware/` |
| Problem | The selected hardware is an Arduino Mega 2560 shield, matching the `megaatmega2560` PlatformIO environment. The firmware had not been built in this worktree; no toolchain or binary evidence was recorded. |
| Evidence | `platformio.ini` default env `megaatmega2560`; `pio` was not installed on the host. |
| Required action | Build the corresponding Speeduino firmware in a pinned container/venv, record compiler/toolchain versions, binary hash and upload procedure. |
| Owner | Firmware workstream (coordinate with hardware) |
| Autonomous | Yes (build only; no hardware flashing) |
| Verification | `pio run -e megaatmega2560` → SUCCESS; clean rebuild produced identical hex/ELF hashes |
| Status | **CLOSED (build only; not flashed)** |
| Resolved by | `219aa0855f94f0a02765e1b32e08f85d4591877a`; `AUDIT/FIRMWARE_TEST_EVIDENCE.md`, `evidence/hardware/firmware-build-megaatmega2560.log` |

A residual review action remains: pin an official Speeduino firmware tag/commit
for the release package instead of the in-tree snapshot. This is not a build
blocker.

### HW-FW-02 — Pin an official firmware tag for the release package

| Field | Value |
|---|---|
| Severity | S3 |
| Affected files | `platformio.ini`, `speeduino/`, `AUDIT/FIRMWARE_TEST_EVIDENCE.md` |
| Problem | The successful build uses the in-tree firmware snapshot (last `speeduino/` commit `5d217bf4471fa1391a5acd0aa341848ec43f5431`), not a tagged upstream Speeduino release. The artefact hash is recorded, but a reviewer cannot map it to a published firmware version. |
| Evidence | `AUDIT/FIRMWARE_TEST_EVIDENCE.md` §4. |
| Required action | Choose the matching official Speeduino firmware tag for v0.4.4d hardware, build it in a pinned environment, record tag/commit and hashes. |
| Owner | Firmware workstream |
| Autonomous | Yes (build only) |
| Verification | `git describe --tags` + `pio run -e megaatmega2560` + hex hash |
| Status | **OPEN** |

### HW-REVIEW-01 — No independent hardware review

| Field | Value |
|---|---|
| Severity | S1 (blocks H3/H4) |
| Affected files | `AUDIT/INDEPENDENT_REVIEW_GUIDE.md` (exists but needs update for this baseline) |
| Problem | No qualified human or independent model has reviewed the selected design, exceptions, BOM or bench plan. |
| Evidence | This register and `AUDIT/PCB_CHECK_RESULTS.md` are self-produced. |
| Required action | Package the review set, obtain independent review, resolve findings, then seek explicit human H4 approval. |
| Owner | Human project owner + independent reviewer |
| Autonomous | No |
| Verification | Signed review record with reviewer identity, date, findings and dispositions |
| Status | **BLOCKED — human** |

### HW-VEH-01 — Evo VIII connector/pinout/trigger unverified

| Field | Value |
|---|---|
| Severity | S1 for vehicle use; **does not block H1/H2** |
| Affected files | `AUDIT/ECU_PINOUT_VERIFICATION.md`, `AUDIT/VEHICLE_COMPATIBILITY.md`, `research/evo8/` |
| Problem | No verified JDM Evo VIII GSR ECU part number, connector cavities, harness pinout, trigger pattern, injector impedance or coil topology. |
| Evidence | Existing audit records all fields unverified; no OEM documentation in repo. |
| Required action | Physical measurement and OEM documentation; do not guess. |
| Owner | Vehicle integration workstream |
| Autonomous | No (requires vehicle/document access) |
| Verification | Scope captures + continuity + connector markings reconciled to OEM data |
| Status | **BLOCKED — vehicle evidence required** |

### HW-EXP-01 — Nexus/Link hardware expansion not designed

| Field | Value |
|---|---|
| Severity | S3 (roadmap, not v1 blocker) |
| Affected files | `docs/FEATURE-MATRIX.md`, future expansion design |
| Problem | The eventual Nexus-like feature set needs more I/O, knock, DBW, flex fuel, CAN, logging, etc. The v0.4.4d baseline does not provide all of it. |
| Evidence | Feature matrix; v0.4.4d has 4 injector/4 ignition and limited I/O. |
| Required action | Maintain a separate expansion roadmap; do not distort the proven v1 baseline to chase features. |
| Owner | Hardware roadmap workstream |
| Autonomous | Yes (planning/design later) |
| Verification | Expansion requirements traced to a future board revision, not claimed as v1-supported |
| Status | **OPEN — roadmap only** |

## Order of work

1. **HW-BOM-01 / HW-BOM-02** — make the purchasing case real and costed.
2. **HW-ERC-01/02, HW-DRC-01, HW-MECH-01/02** — done in HW-H2-01. Focused
   independent review returned ACCEPT WITH FINDINGS; human acceptance of the
   exceptions and mechanical assumptions remains.
3. **Footprint-library audit** (143 `lib_footprint_mismatch`) and cosmetic silk
   cleanup — needed before H3.
4. **HW-BENCH-01** — IDC40 breakout and safe power-up package.
5. **HW-FW-02 (new, low priority)** — pin an official Speeduino firmware tag
   for the release package instead of the in-tree snapshot; re-run the build and
   record the tag/hash.
6. **HW-REVIEW-01** — independent review, then human H4 decision.

HW-VEH-01 and HW-EXP-01 must not be used as excuses to stop H1–H3 bench work.

## HW-BOM-01/02 sourcing follow-up — 2026-11-13

Still OPEN. See docs/JLCPCB_ASSEMBLY.md and docs/HARDWARE_COST.md. No verified LCSC mappings or completed AUD quote; no parts substituted. Harwin M20-9990545 (J1/J6/J7), M20-9990645 (J2), M20-9990445 (J5), M20-9990345 (J8) are candidates only. Manufacturer product information reproduced at https://www.farnell.com/datasheets/4376834.pdf gives 3-position nominal length 7.62 mm but explicitly says End Stackable: No. Drawing/tolerance confirmation remains required before accepting header courtyard exclusions. No approved alternatives. A1 Pololu item 2133 is a customer plug-in; board-side sockets remain unresolved. JP6–JP11 are copper features, not purchased placements. IC3, J3, keyed J4 and mating cable socket, XA1 header set remain unsourced. H3/H4 unchanged.
