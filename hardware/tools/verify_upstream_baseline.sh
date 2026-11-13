#!/usr/bin/env bash
# Verify the preserved Speeduino v0.4.4d upstream baseline against SHA-256
# hashes recorded at acquisition time.
#
# Usage: hardware/tools/verify_upstream_baseline.sh
set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BASELINE="$(cd "${HERE}/../upstream-speeduino/v0.4.4d" && pwd)"
MANIFEST="${HERE}/../upstream-speeduino/SHA256SUMS"

cd "${BASELINE}"
sha256sum --check --strict "${MANIFEST}"
