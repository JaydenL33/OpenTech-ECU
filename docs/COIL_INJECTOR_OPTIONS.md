# Coil and injector options for prototype-v1

This note explains what the selected board can and cannot drive. It is a
decision aid, not a modification instruction.

## 1. What is actually on the board

From the schematic/netlist and `hardware/README.md`:

| Function | Device | What it is |
|---|---|---|
| 4 injector outputs | U1/U3/U7/U9, VNLD5090-E | Protected automotive **low-side** drivers, 13 A class, for inductive loads |
| 4 ignition outputs | U2/U4, IXDN602 | Dual **gate drivers** with 10 Ω series resistors to IDC40 |
| Ignition supply | JP1 | Selects **5 V** (pin 1–2) or **12 V-SW** (pin 2–3) as `Vdrive` |
| Coil primary drivers | none | **No IGBTs, no coil clamp, no coil current path on this board** |

The board README is explicit: the four ignition channels are **logic-level /
gate-driver outputs for an external, verified igniter — never direct coil
drive**. The IDC40 exposes `IGN-1-OUT`…`IGN-4-OUT`; those are not coil primary
terminals.

## 2. Smart coils vs dumb coils

“Coil drive” means two very different things:

### A. Smart coil (integrated igniter)

Examples people commonly mean by R35, Yaris, Audi/VW: a coil with a built-in
igniter/ignition module. The ECU sends a **low-current logic-level dwell
signal**; the coil module switches the primary current itself.

- Dwell is still set by the ECU firmware.
- The coil has its own power, ground and signal pins; the signal is typically
  5 V logic, sometimes with a pull-up.
- The current board’s IXDN602 outputs can be set to 5 V via JP1. In principle
  that is the right kind of signal, but **this specific coil interface has not
  been verified** by the project. The output is a 2 A gate driver, not a
  dedicated 5 V logic buffer; check the coil input current/threshold and the
  output behaviour before connecting a real coil.
- Start with **one channel**, a current-limited supply, a scope and a known
  smart coil or a dummy logic load. Confirm polarity, idle level, dwell and
  spark command. Do not connect four coils and hope.

### B. Dumb coil (no igniter)

A coil that needs the ECU to switch the primary current. That requires:

- a high-voltage **IGBT or coil driver** per channel;
- gate drive, gate resistor, clamp/flyback and current limiting;
- thermal and SOA review;
- a coil primary supply and a safe test load.

The current board has **none of the power stage**. The IXDN602 can drive the
gate of an external IGBT at 12 V (JP1 2–3), but you still need the IGBTs,
clamps, layout and protection. This is a daughter-board or a **prototype-v2**
hardware revision, not a jumper change.

## 3. Injectors

The VNLD5090 is a protected low-side driver intended for inductive loads; the
board has four channels with flyback diodes to the IDC40.

- **High-impedance injectors** (roughly 12–16 Ω) are the straightforward case.
  Verify the driver’s current/thermal margins and the flyback path.
- **Low-impedance injectors** (roughly 2–4 Ω, which some Subaru/Bosch-style
  injectors are) draw much more current. They may need a **resistor pack** or a
  **peak-and-hold** driver strategy. The VNLD5090 is a protected low-side
  driver, not a dedicated peak-and-hold injector controller; verify the
  datasheet, current limit and thermal behaviour before connecting them.
- Measure the injector resistance yourself; do not trust a forum list.
- The firmware must have the correct injector dead time, battery correction and
  fuel pressure basis. Those are calibration inputs, not PCB features.

## 4. Dwell

Dwell is a **firmware** setting, not a PCB setting. On this board the Mega 2560
runs Speeduino firmware; Speeduino has ignition dwell settings and battery
correction. The Rust firmware in `ecu/firmware` is a separate `no_std` skeleton
and is **not** what runs on this board.

For smart coils, start with the coil manufacturer’s dwell recommendation,
reduce it if the coil or driver gets hot, and verify with a current probe/scope.
For dumb coils, dwell/current must be validated against the IGBT/coil SOA.

## 5. Recommended path for your bench build

1. **Build prototype-v1 as the learning platform.** It is routed, documented
   and has a real BOM/assembly path. Do not try to make it a coil-driver board
   by hacking traces.
2. **Use smart coils first** if you can identify the exact coil, pinout and
   logic level. Set JP1 to 5 V, test one channel with a dummy load/scope, then
   one coil. Do not connect the Evo or a running engine.
3. **For dumb coils, design a separate coil-driver board or a v2.** That board
   needs four IGBTs/coil drivers, gate resistors, clamps, bulk capacitance,
   current-limited test points and a thermal layout. Treat it as a new hardware
   revision with its own review.
4. **For injectors, start high-Z.** If you need low-Z Subaru injectors, measure
   them and decide resistor pack vs peak-and-hold before connecting them.
5. **Record everything:** coil/injector part numbers, resistance, pinout,
   dwell, supply voltage, scope captures, temperatures.

## 6. What the next hardware revision would need

If you want on-board dumb-coil drive, the v2 scope is:

- 4 × ignition IGBT/coil driver with appropriate voltage/current ratings;
- gate resistors and gate-emitter pull-downs;
- primary clamp/flyback and bulk decoupling;
- injector drivers matched to high-Z/low-Z choice;
- current-limited test points and a safe dummy-load path;
- thermal/creepage review and a new DRC/ERC pass;
- a new schematic/PCB revision, not a trace cut on v1.

That is a real design task for the hardware engineer and an independent
reviewer. The current v1 is still the right first board for learning,
connectors, power and bench process.

## 7. Safety boundary

No vehicle, engine, injector, coil or fuel-system connection until the board has
been reviewed and bench-tested. Use a current-limited supply, fuses, dummy
loads, a scope and an emergency disconnect. A coil primary can produce
high-voltage spikes; do not probe it with a laptop/oscilloscope input that is
not rated for it.
