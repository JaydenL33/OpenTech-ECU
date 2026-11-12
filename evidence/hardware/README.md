# Hardware evidence

| File | What it is | Status |
|---|---|---|
| `firmware-build-megaatmega2560.log` | Full PlatformIO build log for the in-tree Speeduino firmware, `megaatmega2560` environment, clean build on 2026-11-13 | Build only; nothing flashed |
| `firmware-megaatmega2560-20261113.hex` | Flash image produced by that build; SHA-256 `0d0a4f7fc85abf26b62382b5aa82eae8250c3ddc8ae5efebd480023edec1ea04` | Reference artifact; not programmed |

Context and toolchain versions: `AUDIT/FIRMWARE_TEST_EVIDENCE.md`.

PCB check/export evidence is under
`hardware/prototype-v1/verification/`. Upstream provenance is under
`AUDIT/UPSTREAM_PROVENANCE.md`.

Nothing in this directory is evidence of a powered test, an assembled board, or
a vehicle-compatible ECU. Do not flash this image to hardware without the
independent review and bench procedure in `HARDWARE_BLOCKERS.md` and
`AUDIT/EVO8_BENCH_TEST_PLAN.md`.
