# Vehicle configuration packages

Vehicle packages are **research/configuration manifests**, not tunes and not
proof of compatibility.

- `mitsubishi-evo8-jdm-gsr-6mt.json` is the primary reference profile.
- All vehicle-specific values are `UNVERIFIED` unless stated otherwise.
- The simulator may load a synthetic profile; physical connection is blocked at
  Gate 0 until pinout, trigger, injector, ignition and safety-system evidence
  exist.

Do not build a harness from these files. See `AUDIT/OPEN_ISSUES.md` and
`AUDIT/ECU_PINOUT_VERIFICATION.md`.
