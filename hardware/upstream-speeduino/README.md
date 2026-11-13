# Preserved Speeduino hardware baseline

This directory contains **byte-for-byte copies** of the selected official
Speeduino hardware design. It is a provenance record, not a working project.

| Field | Value |
|---|---|
| Repository | https://github.com/speeduino/Hardware |
| Commit | `ee6a9242834891877744ff2137663b6ca8d79ed6` |
| Commit date | 2026-04-04T17:33:26+11:00 |
| Upstream path | `v0.4/SMD/Latest/` |
| Revision | v0.4.4d (schematic Rev `4d`, dated 2021-03-27) |
| Licence | CERN-OHL-S-2.0 (upstream README statement) |
| Integrity | `../SHA256SUMS` (hashes of all files in `v0.4.4d/`) |

The upstream tree has **no `LICENSE` file** at this commit. The canonical
CERN-OHL-S-2.0 text at `LICENSE-CERN-OHL-S-2.0.txt` is an OpenTech-ECU
addition, not an upstream file.

Verify integrity:

```bash
hardware/tools/verify_upstream_baseline.sh
```

Do not edit files in `v0.4.4d/`. Derived, native-format files live in
`hardware/prototype-v1/`. See `AUDIT/UPSTREAM_PROVENANCE.md` for the full
acquisition and conversion record.
