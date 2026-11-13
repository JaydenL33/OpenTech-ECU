# Resume here — project paused 2026-11-13

This repository is the complete record. The local working trees were removed
after this commit; clone the remote to resume:

```bash
git clone https://github.com/JaydenL33/OpenTech-ECU.git
cd OpenTech-ECU
git checkout master
```

## Current state at pause

- Host software: 252 workspace tests pass; deterministic simulator, calibration
  store, audit log and tuning UI.
- Hardware: Speeduino v0.4.4d SMD review copy (`hardware/prototype-v1/`), fully
  routed, 0 unconnected; H2 complete with documented exceptions; H3 not ready;
  **not fabrication-approved**.
- Legacy `hardware/kicad/evo8-bench-ecu/` is an unrouted custom draft. Do not
  fabricate it.
- No board has been fabricated, assembled or powered. No vehicle connection.

## Read first when resuming

- `docs/START_HERE_HARDWARE.md` — which KiCad project is which.
- `docs/SELF_AUDIT_GUIDE.md` — how to audit the board and bench-test it.
- `docs/COIL_INJECTOR_OPTIONS.md` — smart vs dumb coils, injector limits.
- `AUDIT/HARDWARE_NEXT_STEPS_REVIEW.md` — independent netlist review and open
  risks.
- `HARDWARE_BLOCKERS.md` — blockers, including the missing 12 V fuse/reverse
  protection and the 143-footprint audit.
- `docs/HARDWARE_COST.md` — 5-board AUD estimate (not a quote).

## Immediate next steps

1. Resolve the MPX4250AP MAP sensor (EOL/zero stock): owner-supply, DNP U5 +
   external MAP on J4.21, or an approved substitute.
2. Choose the nine connector/socket MPNs and resolve/consign zero-stock parts.
3. Get an independent human EE review with a written pass/fail (H4).
4. Regenerate the assembly pack, get real JLCPCB/PCBWay quotes and DFM.
5. Bench-test with an external fuse, keyed lead, current-limited supply and
   dummy loads; never connect an engine or vehicle first.
6. For dumb coils, design a daughter board/v2; prototype-v1 is not a coil
   driver.

## Latest commits

Use `git log --oneline -20` after cloning. The remote branches also preserve
the review and workstream history.
