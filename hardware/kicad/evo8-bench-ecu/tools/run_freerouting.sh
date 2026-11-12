#!/usr/bin/env bash
set -euo pipefail
TOOLS=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
source "$TOOLS/containers.env"
if [[ $# != 3 ]]; then
    echo "Usage: $0 DSN SES LOG" >&2
    exit 1
fi
DSN=$(realpath -- "$1")
SES=$(realpath -m -- "$2")
LOG=$(realpath -m -- "$3")
[[ -s "$DSN" && ! -e "$SES" ]]
[[ $(dirname -- "$DSN") == "$(dirname -- "$SES")" ]]
timeout 14400 docker run --rm --network none \
    --user 1000:1000 --tmpfs /tmp/fr:rw,uid=1000,gid=1000 -e HOME=/tmp/fr \
    -v "$(dirname -- "$DSN")":/work "$FREEROUTING_IMAGE" \
    java -jar /app/freerouting-executable.jar \
    -de "/work/$(basename -- "$DSN")" -do "/work/$(basename -- "$SES")" \
    -mp 100 -mt 1 --gui.enabled=false >"$LOG" 2>&1
[[ -s "$SES" ]]
