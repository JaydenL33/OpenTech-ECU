# OpenTech-ECU provenance and licensing notice

OpenTech-ECU is an independent, Rust-first aftermarket ECU project. It is **based
on Speeduino** and is intended as a behavioural Rust port and successor platform,
not as a claim of endorsement by the Speeduino project.

## Speeduino reference source

- The original Speeduino firmware source remains in `speeduino/`, `lib/`, `test/`,
  `reference/` and `boards/` in this repository.
- Speeduino firmware is licensed **GNU GPL v2** (see `LICENSE`).
- OpenTech-ECU's Rust workspace is distributed under **GPL-2.0-or-later**, matching the Speeduino reference licence to preserve
  compatibility with derived behaviour and any code that may be adapted.
- The C++ tree is retained as a **behavioural oracle** for parity testing. It is
  not built by the Rust workspace and is not silently modified.
- No proprietary Haltech, Link, Mitsubishi or other OEM source code, firmware,
  connector drawings or encrypted protocols are included.

## Hardware licensing

- Speeduino hardware is published under **CERN-OHL-S-2.0**. Any derived
  hardware design must carry the applicable CERN-OHL-S-2.0 obligations and
  notices.
- The selected upstream baseline is Speeduino **v0.4.4d SMD**, commit
  `ee6a9242834891877744ff2137663b6ca8d79ed6`; it is preserved byte-for-byte at
  `hardware/upstream-speeduino/v0.4.4d/` with a SHA-256 manifest.
- The native KiCad 9 review copy at `hardware/prototype-v1/` is a format
  conversion with a KiCad 9 zone re-fill, not a new circuit design. It is **not**
  fabrication-approved. See `AUDIT/UPSTREAM_PROVENANCE.md`,
  `AUDIT/HARDWARE_DIFF.md` and `HARDWARE_BLOCKERS.md`.

## Names and trademarks

Mitsubishi, Lancer Evolution, Haltech, Link, Speeduino and other marks belong to
their respective owners. Reference to them is for compatibility research and
benchmarking only. OpenTech-ECU is not affiliated with or endorsed by any of them.

## Safety status

Nothing in this repository is certified, automotive-qualified or safe for
connection to a vehicle. Physical outputs are interlocked off in the current
software. Passing tests, ERC/DRC or an AI review is not a safety approval.
