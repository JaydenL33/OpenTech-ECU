# Hardware Blockers — OpenTech-ECU

| Field | Value |
|---|---|
| Document ID | HW-BLOCKERS |
| Status | Living register; H0 complete, H1 partial, H2/H3 open |
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
| **H1 — Complete PCB design package** | **PARTIAL** | Native KiCad 9 schematic + PCB, all 147 components have footprints, 0 unconnected items, closed outline, gerbers/drill/BOM generated. Position file is not assembly-ready (150 proto pads). Missing: mounting holes, mechanical/assembly drawing, complete BOM MPNs, library-link cleanup. |
| **H2 — Engineering checks completed** | **NOT PASSED** | ERC 101 violations, DRC 609 (238 errors), connectivity PASS. Exceptions are classified in `AUDIT/PCB_CHECK_RESULTS.md`; several remain open. |
| **H3 — Independent fabrication review candidate** | **NOT READY** | Review package exists but H2 exceptions, BOM and mechanical package are incomplete. |
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

### HW-ERC-01 — 101 ERC violations remain

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
| Status | **OPEN** |

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
| Status | **OPEN** |

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
| Status | **OPEN** |

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
| Status | **OPEN** |

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
| Status | **OPEN** |

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
| Status | **ACCEPTED EXCEPTION — pending review** |

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
| Status | **ACCEPTED EXCEPTION — cleanup deferred** |

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
| Status | **OPEN** |

### HW-MECH-03 — Pick-and-place contains 150 unannotated proto pads

| Field | Value |
|---|---|
| Severity | S2 (blocks automated assembly / H3) |
| Affected files | `hardware/prototype-v1/manufacturing/assembly/v0.4.4d-pos.csv` |
| Problem | The position file has 291 rows: 141 real components plus **150 `REF**` proto-area pads** (`Pin_D1.0mm_L10.0mm`). An assembly house would treat these as placements, producing a wrong or rejected job. Six solder jumpers (JP6–JP11) are not in the position file. |
| Evidence | `grep -c '^"REF\*\*"' v0.4.4d-pos.csv` → 150; total rows 291. |
| Required action | Mark the 150 proto pads "exclude from position files" (and exclude from BOM) in a derived revision, or generate an assembly position file filtered to the 141 real placements. Document JP6–JP11 placement/hand-solder intent. |
| Owner | Hardware workstream |
| Autonomous | Yes |
| Verification | Position file has exactly 141 rows (or a documented populated subset); no `REF**`; cross-check with BOM |
| Status | **OPEN** |

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
| Affected files | `hardware/bench-interface/` (missing), `AUDIT/EVO8_BENCH_TEST_PLAN.md` (plan only) |
| Problem | The v0.4.4d IDC40 pinout has not been turned into a labelled breakout with current-limited supply, fuses, dummy loads, E-stop and scope points. Without it, safe powered bring-up is not defined. |
| Evidence | No `hardware/bench-interface/` directory; existing bench plan is generic. |
| Required action | Produce an IDC40 breakout schematic/assembly drawing, test-point map and safe power-up procedure tied to the v0.4.4d pinout. |
| Owner | Hardware workstream |
| Autonomous | Yes (design/package); no power-on |
| Verification | Reviewer can map every IDC40 pin to a labelled terminal/test point; continuity check procedure documented |
| Status | **OPEN** |

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
| Status | **OPEN** |

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
2. **HW-DRC-02 / HW-DRC-03 / HW-ERC-01/02** — reduce H2 exceptions with
   reproducible re-checks.
3. **HW-BENCH-01 / HW-MECH-01 / HW-MECH-02 / HW-MECH-03** — complete the H3
   review package, including the IDC40 breakout, assembly drawing, mounting
   holes and a filtered pick-and-place file.
4. **HW-FW-02 (new, low priority)** — pin an official Speeduino firmware tag
   for the release package instead of the in-tree snapshot; re-run the build and
   record the tag/hash.
5. **HW-REVIEW-01** — independent review, then human H4 decision.

HW-VEH-01 and HW-EXP-01 must not be used as excuses to stop H1–H3 bench work.
