# STM32 firmware skeleton (not vehicle-ready)

This crate is a `no_std` safety skeleton, not a functioning ECU firmware image.

What it contains:
- a crank/cam capture-to-decoder path using the tested `ecu-trigger` crate;
- independent protection evaluation using `ecu-protection`;
- a fail-closed output interface that forces all injector/ignition/fuel-pump outputs off;
- a compile-time physical-output interlock that remains false.

What it does **not** contain:
- an STM32 HAL or board pin mapping;
- hardware timer capture/compare;
- injector or ignition drivers;
- watchdog integration;
- flash/calibration storage;
- CAN, sensors, or bootloader;
- any physical output authorisation.

A target `cargo check` only proves the code is `no_std`-compatible. It does not
prove timing, electrical safety, or engine safety. See
`AUDIT/RELEASE_GATES.md` and `AUDIT/FIRMWARE_TIMING_ANALYSIS.md`.
