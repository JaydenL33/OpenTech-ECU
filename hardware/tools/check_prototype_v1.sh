#!/usr/bin/env bash
# Re-run the H2 checks and manufacturing exports for hardware/prototype-v1.
#
# The design is Speeduino v0.4.4d (SMD) saved to native KiCad 9 format with the
# documented minimal revisions R-01..R-10 (AUDIT/HARDWARE_DIFF.md). This script
# does not edit the design; it only runs checks and writes evidence.
#
# Requires Docker. The KiCad image is pinned by digest so the result is
# reproducible. No network access is required after the image is present.
set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "${HERE}/../prototype-v1" && pwd)"
IMAGE="kicad/kicad@sha256:e638b79b0321f29395a5b783e94bb9f3c73303e8da15da27b8f5cb4b67a37729"
RUN=(docker run --rm --user "$(id -u):$(id -g)" -v "${ROOT}:/work" -v "${HERE}:/tools:ro" "${IMAGE}")

mkdir -p "${ROOT}/verification" \
         "${ROOT}/manufacturing/gerbers" \
         "${ROOT}/manufacturing/drill" \
         "${ROOT}/manufacturing/assembly"

{
  echo "# Prototype v1 check run"
  echo "date_utc: $(date -u +%Y-%m-%dT%H:%M:%SZ)"
  echo "host_git_commit: $(git -C "${ROOT}" rev-parse HEAD 2>/dev/null || echo unavailable)"
  echo "kicad_image: ${IMAGE}"
  echo "kicad_cli_version: $("${RUN[@]}" kicad-cli --version)"
  echo
} > "${ROOT}/verification/run-context.txt"

# --- Electrical rules check on the native schematic -------------------------
set +e
"${RUN[@]}" kicad-cli sch erc \
  --output /work/verification/erc.rpt \
  /work/v0.4.4d.kicad_sch > "${ROOT}/verification/erc.stdout" 2>&1
erc_rc=$?
set -e
echo "erc_exit_code: ${erc_rc}" >> "${ROOT}/verification/run-context.txt"

# --- Design rules check on the native PCB -----------------------------------
set +e
"${RUN[@]}" kicad-cli pcb drc \
  --output /work/verification/drc.rpt \
  /work/v0.4.4d.kicad_pcb > "${ROOT}/verification/drc.stdout" 2>&1
drc_rc=$?
set -e
echo "drc_exit_code: ${drc_rc}" >> "${ROOT}/verification/run-context.txt"

# Same DRC including the per-item exclusions (each carries its justification
# comment), so excluded items stay visible and auditable.
set +e
"${RUN[@]}" kicad-cli pcb drc --severity-all --severity-exclusions \
  --output /work/verification/drc-with-exclusions.rpt \
  /work/v0.4.4d.kicad_pcb > "${ROOT}/verification/drc-with-exclusions.stdout" 2>&1
drcx_rc=$?
set -e
echo "drc_with_exclusions_exit_code: ${drcx_rc}" >> "${ROOT}/verification/run-context.txt"

# --- Manufacturing exports ---------------------------------------------------
"${RUN[@]}" kicad-cli pcb render \
  --output /work/verification/board-top.png --side top --zoom 1.0 \
  /work/v0.4.4d.kicad_pcb > "${ROOT}/verification/render.stdout" 2>&1

"${RUN[@]}" kicad-cli pcb export gerbers \
  --output /work/manufacturing/gerbers \
  --layers F.Cu,B.Cu,F.Paste,B.Paste,F.Silkscreen,B.Silkscreen,F.Mask,B.Mask,Edge.Cuts \
  /work/v0.4.4d.kicad_pcb > "${ROOT}/verification/gerber.stdout" 2>&1

"${RUN[@]}" kicad-cli pcb export drill \
  --output /work/manufacturing/drill \
  --format excellon --excellon-units mm \
  --generate-map --map-format pdf \
  --generate-report --report-path /work/manufacturing/drill/v0.4.4d-drill-report.txt \
  /work/v0.4.4d.kicad_pcb > "${ROOT}/verification/drill.stdout" 2>&1

"${RUN[@]}" kicad-cli pcb render \
  --output /work/verification/board-bottom.png --side bottom --zoom 1.0 \
  /work/v0.4.4d.kicad_pcb > "${ROOT}/verification/render-bottom.stdout" 2>&1

# Assembly drawings (fab + silkscreen + outline + fab notes) and the
# fabrication drawing (outline, actual-size drill shapes, dimensions, notes).
"${RUN[@]}" kicad-cli pcb export pdf --mode-single --include-border-title \
  --layers F.Fab,F.Silkscreen,Edge.Cuts,User.Drawings \
  --output /work/manufacturing/assembly/v0.4.4d-assembly-top.pdf \
  /work/v0.4.4d.kicad_pcb > "${ROOT}/verification/assembly-top.stdout" 2>&1

"${RUN[@]}" kicad-cli pcb export pdf --mode-single --include-border-title --mirror \
  --layers B.Fab,B.Silkscreen,Edge.Cuts \
  --output /work/manufacturing/assembly/v0.4.4d-assembly-bottom.pdf \
  /work/v0.4.4d.kicad_pcb > "${ROOT}/verification/assembly-bottom.stdout" 2>&1

"${RUN[@]}" kicad-cli pcb export pdf --mode-single --include-border-title \
  --layers Edge.Cuts,User.Drawings --drill-shape-opt 2 \
  --output /work/manufacturing/drill/v0.4.4d-fab-drawing.pdf \
  /work/v0.4.4d.kicad_pcb > "${ROOT}/verification/fab-drawing.stdout" 2>&1

"${RUN[@]}" kicad-cli pcb export pos \
  --output /work/manufacturing/assembly/v0.4.4d-pos.csv \
  --format csv --units mm --use-drill-file-origin \
  /work/v0.4.4d.kicad_pcb > "${ROOT}/verification/pos.stdout" 2>&1

"${RUN[@]}" kicad-cli pcb export ipcd356 \
  --output /work/manufacturing/assembly/v0.4.4d-ipc-d-356.net \
  /work/v0.4.4d.kicad_pcb > "${ROOT}/verification/ipcd356.stdout" 2>&1

"${RUN[@]}" kicad-cli sch export bom \
  --output /work/manufacturing/assembly/v0.4.4d-bom.csv \
  --fields 'Reference,Value,Footprint,${QUANTITY},${DNP},Manufacturer_Name,Manufacturer_Part_Number' \
  --labels 'Refs,Value,Footprint,Qty,DNP,Manufacturer,MPN' \
  --group-by 'Value,Footprint,Manufacturer,MPN' \
  /work/v0.4.4d.kicad_sch > "${ROOT}/verification/bom.stdout" 2>&1

"${RUN[@]}" kicad-cli sch export pdf \
  --output /work/manufacturing/assembly/v0.4.4d-schematic.pdf \
  /work/v0.4.4d.kicad_sch > "${ROOT}/verification/schpdf.stdout" 2>&1

"${RUN[@]}" kicad-cli sch export netlist --format kicadsexpr \
  --output /work/manufacturing/assembly/v0.4.4d.net \
  /work/v0.4.4d.kicad_sch > "${ROOT}/verification/netlist.stdout" 2>&1

# --- Connectivity comparison and summary ------------------------------------
set +e
"${RUN[@]}" python3 /tools/connectivity_check.py /work \
  > "${ROOT}/verification/connectivity.stdout" 2>&1
conn_rc=$?
set -e
echo "connectivity_exit_code: ${conn_rc}" >> "${ROOT}/verification/run-context.txt"

set +e
"${RUN[@]}" python3 /tools/check_mounting_holes.py /work \
  > "${ROOT}/verification/mounting-holes.txt" 2>&1
holes_rc=$?
set -e
echo "mounting_holes_exit_code: ${holes_rc}" >> "${ROOT}/verification/run-context.txt"

{
  echo "Connectivity comparison exit code: ${conn_rc}"
  echo "Mounting-hole clearance check exit code: ${holes_rc} ($(tail -n 1 "${ROOT}/verification/mounting-holes.txt"))"
  echo
  echo "ERC category counts:"
  grep -o '\[[a-z_]*\]' "${ROOT}/verification/erc.rpt" | sort | uniq -c | sort -rn
  echo
  echo "DRC category counts:"
  grep -o '\[[a-z_]*\]' "${ROOT}/verification/drc.rpt" | sort | uniq -c | sort -rn
  echo
  grep -E 'unconnected items|Found [0-9]+ DRC' "${ROOT}/verification/drc.stdout" || true
  echo
  echo "DRC excluded items (per-item, with comments; see drc-with-exclusions.rpt):"
  grep -c '(excluded)' "${ROOT}/verification/drc-with-exclusions.rpt" || true
  grep -B1 '(excluded)' "${ROOT}/verification/drc-with-exclusions.rpt" \
    | grep -o '^\[[a-z_]*\]' | sort | uniq -c | sort -rn || true
  echo
  echo "Manufacturing files:"
  find "${ROOT}/manufacturing" -type f -printf '%P\n' | sort
} > "${ROOT}/verification/summary.txt"

echo "Wrote checks to ${ROOT}/verification"
echo "ERC exit=${erc_rc} DRC exit=${drc_rc} connectivity exit=${conn_rc} mounting-holes exit=${holes_rc}"
cat "${ROOT}/verification/summary.txt"

