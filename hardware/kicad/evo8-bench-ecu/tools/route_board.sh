#!/usr/bin/env bash
set -euo pipefail
B=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)
REPO=$(git -C "$B" rev-parse --show-toplevel)
KC=/tmp/opencode/evo-hardware/.toolchain/root/usr/bin/kicad-cli
export LD_LIBRARY_PATH=/tmp/opencode/evo-hardware/.toolchain/root/usr/lib/x86_64-linux-gnu
source "$B/tools/containers.env"
if [[ $# != 1 ]]; then
    echo "Usage: $0 WORKDIR (must not exist)" >&2
    exit 1
fi
WORKDIR=$(realpath -m -- "$1")
[[ ! -e "$WORKDIR" ]]
mkdir -- "$WORKDIR"
mkdir -- "$WORKDIR/draft" "$WORKDIR/routed" "$WORKDIR/plane-vias" "$WORKDIR/final"
BOARD=evo8-bench-ecu.kicad_pcb
PROJECT=evo8-bench-ecu.kicad_pro
git -C "$REPO" show "1be37135:hardware/kicad/evo8-bench-ecu/$BOARD" >"$WORKDIR/draft/$BOARD"
git -C "$REPO" show "1be37135:hardware/kicad/evo8-bench-ecu/$PROJECT" >"$WORKDIR/draft/$PROJECT"
cp -- "$WORKDIR/draft/$PROJECT" "$WORKDIR/routed/$PROJECT"
cp -- "$WORKDIR/draft/$PROJECT" "$WORKDIR/plane-vias/$PROJECT"
cp -- "$WORKDIR/draft/$PROJECT" "$WORKDIR/final/$PROJECT"
kicad_python() {
    docker run --rm --network none --user "$(id -u):$(id -g)" -e HOME=/tmp \
        -v "$B/tools":/tools:ro -v "$WORKDIR":/work "$KICAD_IMAGE" python3 "$@"
}
kicad_python /tools/export_dsn.py --board "/work/draft/$BOARD" --out /work/board.dsn
"$B/tools/run_freerouting.sh" "$WORKDIR/board.dsn" "$WORKDIR/board.ses" "$WORKDIR/freerouting.log"
kicad_python /tools/import_ses.py --board "/work/draft/$BOARD" --ses /work/board.ses \
    --out "/work/routed/$BOARD"
kicad_python /tools/add_plane_vias.py --in "/work/routed/$BOARD" \
    --out "/work/plane-vias/$BOARD" --report /work/plane-vias.json
kicad_python /tools/fill_zones.py --in "/work/plane-vias/$BOARD" --out "/work/final/$BOARD"
/usr/bin/python3 -B "$B/tools/check_board.py" "$WORKDIR/draft/$BOARD" \
    "$WORKDIR/final/$BOARD" >"$WORKDIR/check.json"
"$KC" pcb drc --format json --severity-all --output "$WORKDIR/drc.json" "$WORKDIR/final/$BOARD"
