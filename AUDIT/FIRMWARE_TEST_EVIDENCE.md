# Firmware Test Evidence — Speeduino Reference Build

| Field | Value |
|---|---|
| Document ID | AUD-HW-FW-01 |
| Status | **Build passes; not flashed; no hardware executed** |
| Date | 2026-11-13 |
| Workstream | Firmware/hardware compatibility |
| Board baseline | Speeduino v0.4.4d SMD (Arduino Mega 2560 shield) |
| Hardware upstream | `ee6a9242834891877744ff2137663b6ca8d79ed6` |
| Firmware source tree | `5d217bf4471fa1391a5acd0aa341848ec43f5431` (`speeduino/` last change, 2026-11-06) |
| Environment | PlatformIO Core 6.2.0, `megaatmega2560`, platform atmelavr 5.3.0 |
| Evidence | `evidence/hardware/firmware-build-megaatmega2560.log`, `evidence/hardware/firmware-megaatmega2560-20261113.hex` |
| Related | `HARDWARE_BLOCKERS.md` `HW-FW-01`, `AUDIT/HARDWARE_BASELINE.md` |

> **What this proves.** The in-tree upstream-style Speeduino C++ firmware builds
> for the selected board's `megaatmega2560` environment with the pinned
> PlatformIO/toolchain, producing a deterministic `.hex`. It **does not** prove
> the firmware runs on a board, that the board is functional, or that any output
> is safe. Nothing was flashed and no hardware was powered.

## 1. Commands

```bash
python3 -m venv /tmp/fw-venv
/tmp/fw-venv/bin/pip install platformio==6.2.0
cd <repo-root>
/tmp/fw-venv/bin/pio run -e megaatmega2560 -t clean
/tmp/fw-venv/bin/pio run -e megaatmega2560
```

## 2. Toolchain

| Component | Version |
|---|---|
| PlatformIO Core | 6.2.0 |
| Platform `atmelavr` | 5.3.0 |
| `framework-arduino-avr` | 5.4.0 |
| `toolchain-atmelavr` | 1.70300.191015 (`avr-gcc` 7.3.0) |
| `FastCRC` | 1.41.0 |
| `SimplyAtomic` | 1.0.0 |
| `Time` | 1.6.1 |
| `avr-fast-div` | 1.4.0 |
| `avr-fast-shift` | 1.1.0 |
| `avr-libstdcpp` | git `5354296040a2289c911062daa82336762231e897` |
| `fp64lib` | 1.1.33 |
| `libdivide` | 5.4.0 |

## 3. Result

```text
Environment     Status    Duration
megaatmega2560  SUCCESS   00:00:09.847
========================= 1 succeeded in 00:00:09.847 =========================
```

Memory (from `avr-size -A firmware.elf`):

| Region | Used | Available | Utilisation |
|---|---:|---:|---:|
| Flash (`.text` + `.data`) | 193,960 bytes | 253,952 bytes | 76.4% |
| RAM (`.data` + `.bss`) | 6,357 bytes | 8,192 bytes | 77.6% |

Artifact hashes (clean rebuild produced the **same** hashes):

```text
0d0a4f7fc85abf26b62382b5aa82eae8250c3ddc8ae5efebd480023edec1ea04  firmware.hex
7fb19ae85f701cc68cd4b66f78e94b43ae3e35e1bfcf604d32731d687417d607  firmware.elf
```

The hex is retained at
`evidence/hardware/firmware-megaatmega2560-20261113.hex`.

## 4. Reproducibility notes

- The build was run twice from a clean `.pio` state; the hex and ELF hashes were
  identical both times.
- The build uses the in-tree firmware source, not a separate upstream release
  tag. The exact firmware tree commit is recorded above. A future release should
  pin an official Speeduino firmware tag/commit for the review package.
- `platformio.ini` selects `megaatmega2560` by default. The selected v0.4.4d
  hardware is an Arduino Mega 2560 shield, so this is the correct target family.
- The **Rust** `ecu/firmware/stm32` skeleton is a separate path and is not
  involved in this build. Nothing here implies the Rust firmware runs on the
  AVR.

## 5. What remains before firmware can be used on hardware

1. Independent review of the hardware and firmware pairing.
2. Bench power-up with current limit, no actuators, per
   `AUDIT/EVO8_BENCH_TEST_PLAN.md`.
3. Verify the board's actual Arduino header mapping against the schematic/pin map
   (the PCB uses an `Arduino_Mega2560_Shield_IO_Only` footprint).
4. Verify crank/cam input conditioning and injector/ignition output behaviour
   with dummy loads and a scope.
5. Explicit H4 human approval before fabrication, and H5/H6 evidence before any
   powered test claim.

## 6. Explicit non-claims

- No firmware was flashed to a board.
- No injector, ignition, fuel pump, fan, boost or idle output was actuated.
- No engine or vehicle was connected.
- A successful build is not a functional-safety or automotive qualification.
