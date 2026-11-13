# Upstream Provenance — Speeduino Hardware v0.4.4d (SMD)

| Field | Value |
|---|---|
| Document ID | AUD-HW-PROV-01 |
| Status | Preserved upstream baseline acquired and hash-verified |
| Date | 2026-11-13 |
| Workstream | Hardware |
| Selected design | Speeduino v0.4.4d, SMD variant |
| Upstream repository | https://github.com/speeduino/Hardware |
| Upstream commit | `ee6a9242834891877744ff2137663b6ca8d79ed6` |
| Commit date | 2026-04-04T17:33:26+11:00 |
| Commit subject | `Dropbear v2.0.2 BOM` |
| Upstream path | `v0.4/SMD/Latest/` |
| Preserved copy | `hardware/upstream-speeduino/v0.4.4d/` |
| Hash manifest | `hardware/upstream-speeduino/SHA256SUMS` |
| Native conversion | `hardware/prototype-v1/` (KiCad 9.0.9 format; Option B minimal PCB revision R-01…R-03, commit `cc2ed0fd`; no net/topology edit) |
| Related | `AUDIT/HARDWARE_BASELINE.md`, `AUDIT/PCB_CHECK_RESULTS.md`, `HARDWARE_BLOCKERS.md` |

> **Scope statement.** **H0 is satisfied** for the hardware and firmware build:
> exact Speeduino Hardware commit, board revision, licence, hash manifest, and a
> reproducible `megaatmega2560` firmware build with artifact hashes
> (`AUDIT/FIRMWARE_TEST_EVIDENCE.md`). The board is **not** fabrication-approved,
> not automotive qualified, and has not been powered. H1 is partially satisfied:
> native KiCad 9 files exist, routing is complete (0 unconnected items), and a
> review package is generated, but the H2 exceptions in
> `AUDIT/PCB_CHECK_RESULTS.md` remain open. H4+ require independent human review
> and physical evidence.

## 1. Exact source identity

The upstream repository was cloned on 2026-11-12/13 and pinned to the commit
above. The commit is the current `HEAD` of `speeduino/Hardware` at acquisition
time and contains 501 tracked files.

Reproduction:

```bash
git clone https://github.com/speeduino/Hardware.git
cd Hardware
git checkout ee6a9242834891877744ff2137663b6ca8d79ed6
git rev-parse HEAD
# expected: ee6a9242834891877744ff2137663b6ca8d79ed6
```

Selected source directory: `v0.4/SMD/Latest/`.

Title block of the upstream root schematic (`v0.4.4d.sch`):

```text
Title "0.4"
Date "2021-03-27"
Rev "4d"
Comp "Speeduino"
```

The upstream PCB header records the originating tool:

```text
(kicad_pcb (version 20171130) (host pcbnew "(5.1.9-0-10_14)")
```

This is a genuine KiCad 5 design, not a reconstruction. It was opened and saved
by KiCad 9.0.9 to produce the native `.kicad_sch`/`.kicad_pcb` files in
`hardware/prototype-v1/`. The preserved upstream files in
`hardware/upstream-speeduino/v0.4.4d/` were **not modified**.

## 2. Licence and obligations

The upstream repository has **no `LICENSE`/`COPYING` file** at this commit
(verified with `git ls-tree -r HEAD --name-only | grep -i 'licen\|copying'`).
The upstream `README.md` states:

> "These designs are release under the CERN Open Hardware License v2 Strong
> (CERN-OHL-S) ... As the 'strongly reciprocal' version of the CERN OHL is used,
> you may use, copy, modify and create variations of these designs freely,
> provided that any changes or variations are also released under this
> CERN-OHL-S."

The canonical CERN-OHL-S-2.0 text was therefore added at
`hardware/upstream-speeduino/LICENSE-CERN-OHL-S-2.0.txt`. It is an OpenTech-ECU
addition and is **not** part of the upstream commit. Any distribution of the
copied design or a modified prototype must retain the CERN-OHL-S-2.0 notice and
publish modifications under the same licence.

The Speeduino **firmware** in this repository is GPL-2.0; the hardware design is
CERN-OHL-S-2.0. The two licences apply to different artefacts and are not
interchangeable.

## 3. Integrity manifest

`hardware/upstream-speeduino/SHA256SUMS` covers all 32 files in the preserved
`v0.4.4d` directory. Verify with:

```bash
hardware/tools/verify_upstream_baseline.sh
```

Observed result at the time of writing: every file reported `OK`; exit code 0.
Key hashes:

```text
f853aded30c2e0ccfb7e573b946e9f9c27dc18db84dd268f11a215f4eff2ee4e  v0.4.4d.kicad_pcb
a65934fcf2cfdbd9610858b6269864df743f1e743fd77d0e75272982ba7a22a1  v0.4.4d.sch
7a237ba0e1d9eb2c2478b263b9ef6546cd178c0339b387d25dcfb44953f6306b  v0.4.4d.kicad_pro
cba037ead1c0d234eea98681fcab063fdba53e2ae18ba69586d184daa971f274  v0.4.4d BOM.csv
```

## 4. What was converted, and what was not

The conversion was a **file-format save only**:

1. The upstream `v0.4.4d.sch` hierarchy was opened read/write in Eeschema
   (KiCad 9.0.9) in an Xvfb session.
2. `Ctrl+S` upgraded the root and six hierarchical sheets to
   `*.kicad_sch` and upgraded `v0.4.4d.kicad_pro`.
3. The upstream `v0.4.4d.kicad_pcb` was loaded with the KiCad 9 `pcbnew`
   Python API and saved as `v0.4.4d.kicad_pcb` (format 20241229).
4. No component value, net, footprint, track, via, zone or board outline was
   edited by OpenTech-ECU.

Evidence that the conversion did not change connectivity:

- The converted schematic netlist has 520 `(ref, pin)` nodes and 190 nets.
- The PCB has 514 pad nodes. All 514 are common with the schematic.
- **0** schematic nets are split on the PCB; **0** PCB nets are split on the
  schematic.
- The 6 schematic-only nodes are `XA1` pins `5V2`, `GND4`, `MISO`, `MOSI`,
  `RST2`, `SCK` — the Arduino ICSP header pins, which are not present on the
  `Arduino_Mega2560_Shield_IO_Only` PCB footprint.
- Raw evidence: `hardware/prototype-v1/verification/connectivity.txt`.

The conversion procedure is recorded because it requires a GUI action; the
check/export pipeline itself is fully scripted in
`hardware/tools/check_prototype_v1.sh`.

### 4.1 Later minimal revision (post-conversion)

After the format conversion, prototype-v1 received three documented
PCB/manufacturing fixes (commit `cc2ed0fd`, reproducible via
`hardware/tools/apply_v1_minimal_revision.py`):

- R-01 — four starved-thermal GND pads set to a solid zone connection;
- R-02 — one duplicate 0.552 mm VDDA track segment removed;
- R-03 — two U5 mechanical tabs converted from PTH to NPTH.

No net, component value, footprint assignment or routing topology was changed.
The preserved upstream tree under `hardware/upstream-speeduino/` is untouched.
Full change record with electrical effect, ratings, failure behaviour and tests:
`AUDIT/HARDWARE_DIFF.md` §2. DRC 609 → 600 (238 → 230 errors); ERC unchanged at
101; connectivity PASS 514/514.

## 5. Reproducible checks

```bash
# ERC + DRC + all manufacturing exports + connectivity comparison
hardware/tools/check_prototype_v1.sh

# Upstream file integrity
hardware/tools/verify_upstream_baseline.sh

# Firmware reference build (separate venv; does not flash)
python3 -m venv /tmp/fw-venv
/tmp/fw-venv/bin/pip install platformio==6.2.0
/tmp/fw-venv/bin/pio run -e megaatmega2560
```

Firmware build evidence: `AUDIT/FIRMWARE_TEST_EVIDENCE.md`; hex and log under
`evidence/hardware/`.

Toolchain: `kicad/kicad@sha256:e638b79b0321f29395a5b783e94bb9f3c73303e8da15da27b8f5cb4b67a37729`
(KiCad 9.0.9, release build). Docker Engine 29.9.2 on Linux 6.8.0.

## 6. Known provenance limits

- The upstream repository does not ship the project-local `lib/` directories
  referenced by `fp-lib-table`/`sym-lib-table` (`Shields`, `Misc`,
  `Connector_Automotive`, `IC_Automotive`). The converted files embed the symbols
  and footprints, so the design opens and exports; library-link ERC warnings are
  recorded in `AUDIT/PCB_CHECK_RESULTS.md`.
- Upstream gerbers in `v0.4/SMD/Latest/gerbers/` were produced with an older
  KiCad and are retained for comparison only. The review package regenerates
  them from the native PCB.
- The upstream commit has no formal release tag at this revision; the commit
  hash is the authoritative identifier.
- No claim is made that the upstream design is electrically correct, safe, or
  suitable for the Evo VIII. This document only establishes *what* was acquired
  and that it is byte-identical to that upstream commit.
