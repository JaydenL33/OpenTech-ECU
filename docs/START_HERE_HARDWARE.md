# Start here — which hardware project is which

There are two hardware projects in this repository. They are not the same board.

## 1. `hardware/prototype-v1/` — the selected bench prototype

Open this project:

```text
hardware/prototype-v1/v0.4.4d.kicad_pro
```

This is the **Speeduino v0.4.4d SMD** design, converted to native KiCad 9 and
given a documented minimal OpenTech revision. It is:

- fully routed: **0 unconnected items**
- 100.05 × 100.05 mm, 2-layer, Arduino Mega 2560 shield
- 4 injector channels, 4 ignition outputs, IDC40 bench connector
- H2 complete with documented exceptions; **not fabrication-approved**
- still electrically the upstream Speeduino design, deliberately

The title block may still say “Speeduino 0.4 / 4d” because the schematic is the
upstream design. The OpenTech changes are recorded in
`AUDIT/HARDWARE_DIFF.md`, not in the upstream title block.

What the hardware engineer actually added (R-01…R-10):

| Change | What it did |
|---|---|
| R-01 | Four starved GND thermal connections made solid |
| R-02 | Removed a duplicate VDDA track stub |
| R-03 | Converted the MAP-sensor mechanical tabs to NPTH |
| R-04 | Added three M3 NPTH mounting holes with copper keep-outs |
| R-05 | Repaired project-local footprint libraries |
| R-06 | Fixed the legacy `Device:CP1` symbol link |
| R-07 | Added five PWR_FLAGs |
| R-08 | Added 29 no-connect flags and removed dangling wires/labels |
| R-09 | Added 199 individually justified courtyard DRC exclusions |
| R-10 | Added fab-drawing dimensions/notes |

The **netlist is unchanged**. No injector, ignition, power or connector circuit
was redesigned. This is why the board still looks like Speeduino: it is
Speeduino, deliberately, as the proven bench baseline.

## 2. `hardware/kicad/evo8-bench-ecu/` — the old custom draft

This is the project whose title block says:

```text
OpenTech-ECU  REV A  /  UNREVIEWED BENCH PROTOTYPE
NOT A VEHICLE ECU / NOT AUTOMOTIVE QUALIFIED / DO NOT FABRICATE: UNROUTED
```

It is an **unreviewed STM32H743 custom board** with an **unrouted PCB** (499
unconnected items at the last check). It is not the selected baseline and must
not be fabricated. It is kept as an exploratory design and as a negative
example for audit. Do not open it expecting the prototype.

## 3. What “get a print off” means

To fabricate the selected prototype, you need:

1. `hardware/prototype-v1/v0.4.4d.kicad_pro` (not the old project).
2. Gerbers and drill files from `hardware/prototype-v1/manufacturing/`.
3. The assembly pack in `hardware/prototype-v1/manufacturing/jlcpcb/`.
4. The BOM/connector/MAP-sensor sourcing decisions completed.
5. A human engineering review (see `SELF_AUDIT_GUIDE.md`).
6. A quote from JLCPCB/PCBWay; then an explicit decision to order.

The current board is a **bench prototype**, not a vehicle ECU. It must not be
connected to the Evo VIII, an engine, injectors, coils or a fuel system.

## 4. The confusing part is real

The old project uses “OpenTech-ECU REV A” in its title block. The selected
project still uses the upstream Speeduino title block. Neither title block is a
reliable status indicator. Use these instead:

- `hardware/README.md` — which path is selected
- `AUDIT/HARDWARE_BASELINE.md` — why v0.4.4d was chosen
- `AUDIT/HARDWARE_DIFF.md` — exactly what changed
- `AUDIT/PCB_CHECK_RESULTS.md` — ERC/DRC and exceptions
- `HARDWARE_BLOCKERS.md` — what is still open

If a KiCad window shows “UNROUTED”, it is the old custom board. Close it and
open `hardware/prototype-v1/v0.4.4d.kicad_pro`.
