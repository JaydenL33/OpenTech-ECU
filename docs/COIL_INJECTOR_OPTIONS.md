# Coil and injector options for prototype-v1

This note explains what the selected board can and cannot drive. It is a
decision aid, not a modification instruction. It has been corrected against the
independent netlist review in `AUDIT/HARDWARE_NEXT_STEPS_REVIEW.md`.

## 1. What is actually on the board

| Function | Device | What it is |
|---|---|---|
| 4 injector outputs | **U1 and U3** (two dual-channel VNLD5090) | Protected automotive low-side drivers for INJ-1..4 |
| 4 auxiliary low-side outputs | **U7 and U9** | BOOST, IDLE, HC-1, HC-2 — **not injectors** |
| 4 ignition outputs | U2/U4, IXDN602 | Dual gate drivers with only a 10 Ω series resistor to IDC40 |
| Ignition supply | JP1 | Selects 5 V (1–2) or 12 V-SW (2–3) as `Vdrive`; must be fitted |
| Coil primary drivers | none | **No IGBTs, no coil clamp, no coil current path on this board** |
| 12 V protection | none | F1/F2 are 5 V-side polyfuses; D14/D15 protect only the regulator input |

Important corrections:

- **U7/U9 are not injector drivers.** They drive boost, idle and two
  high-current auxiliary outputs.
- **D5/D6/D11/D12 are indicator-LED series diodes, not injector flyback
  diodes.** The injector turn-off energy is absorbed by the VNLD5090 internal
  clamp. The aux outputs have external freewheel diodes D16–D19.
- **There is no fuse or reverse-polarity protection on `12V-SW`.** Use an
  external inline fuse, a keyed/polarised supply lead and a current-limited
  supply on the bench. Reverse polarity on J3 is not survivable by design.
- **The ignition output current is limited only by the 10 Ω resistor.** A short
  or a coil primary on `IGN-x-OUT` can dissipate ~2.5 W at 5 V or ~14 W at
  12 V in a 1206 resistor. Never connect a coil primary there.
- **Back-drive risk:** a coil/igniter input with an internal pull-up above 5 V
  can push current into the 5 V rail through the IXDN602 when JP1 is at 5 V.
  Measure the open-circuit trigger voltage before connecting anything.

### J4 IDC40 traps

- **IGN-3 is pin 16, IGN-4 is pin 15** (not 15/16).
- Injectors are interleaved with aux outputs: INJ-3 = 5/7, INJ-4 = 9/11; pin 6
  is HC-2, pin 8 IDLE, pin 10 HC-1.
- The schematic symbol is counter-clockwise; the footprint uses normal odd/even
  IDC numbering. Pin numbers match, but **audit by number, not by drawing
  position**.
- **J4 has no +12 V pin.** Injector/coil supply is external.
- Only four GND pins (17/19/23/36), shared with sensor returns. Do not put
  high-current returns through the ribbon.
- **J4.21 is hard-wired to the on-board MAP sensor U5** through JP11 (default
  bridged). For an external MAP, either cut JP11 A–C and bridge B–C, or do not
  fit U5. Otherwise the two outputs fight.

## 2. Smart coils vs dumb coils

“Coil drive” means two very different things.

### A. Smart coil (integrated igniter)

A coil with a built-in igniter/ignition module. The ECU sends a low-current
logic-level dwell signal; the coil switches its own primary current.

- **Do not classify a coil from its family name.** R35, Yaris, Audi/VW and Evo
  VIII coils must be classified from the exact part number, a datasheet or an
  OEM workshop manual. “Commonly used by the aftermarket” is not evidence.
- The current board’s IXDN602 outputs can be set to 5 V with JP1 1–2. That is
  the right *kind* of signal, but it has not been verified for any specific
  coil. The output is a 2 A gate driver, not a dedicated 5 V logic buffer.
- JP1 2–3 gives ~12 V and can overstress a 5 V-only coil input.
- JP1 open leaves the gate drivers unpowered; outputs are undefined. Fit it.
- Test one channel with a 1 kΩ dummy and a scope first; then one coil, with a
  grounded spark gap or plug, never an open secondary.

Before connecting a coil, record:

1. Exact part number and source document.
2. Pin functions: +12 V, power GND, signal GND, trigger (IGT), feedback (IGF).
3. Active-high or active-low trigger; idle level.
4. Trigger input voltage rating (is 5 V enough, is 12 V allowed?).
5. Trigger input current at 5 V — a few mA only; near 100 mA means stop.
6. Internal pull-up voltage on the trigger pin (must not exceed 5 V for JP1
   5 V).
7. Recommended dwell vs battery voltage, maximum dwell, over-dwell/thermal
   shutdown.
8. Coil primary resistance/inductance if available.
9. Bench sequence: dummy → one coil → one channel at a time; current probe on
   the coil supply.

### B. Dumb coil (no igniter)

A coil that needs the ECU to switch primary current. That requires an
IGBT/coil-driver stage, gate resistor, gate-emitter pull-down, clamp/flyback,
bulk capacitance, current limiting, thermal layout and creepage. None of that
is on prototype-v1.

A v2/daughter board should take the **5 V logic** IGN-x-OUT and generate the
gate drive **next to the IGBT**, not send a 12 V gate signal down the IDC
ribbon. It needs its own fuse, power connector, high-current ground return,
test points and independent review.

## 3. Injectors

- **High-impedance injectors (measured ≥ ~10 Ω cold)** are the straightforward
  case; ~1 A per channel is far below the driver class. The VNLD5090 internal
  clamp handles turn-off.
- **Low-impedance injectors (≈2–4 Ω)** should **not** be connected directly.
  The current is several amps per channel, the IDC/ribbon and shared dual-channel
  SOIC thermal path are not sized for it, and there is no peak-and-hold.
  Options: external ballast resistor pack with EE sign-off (and dead-time
  recalibration), or a v2 peak-and-hold driver.
- “Subaru injector” is not a resistance class. Measure every injector and
  record part number and cold resistance. The Evo VIII OEM injector impedance
  is not established in this repository.
- Dwell and injector dead time are firmware/calibration settings. On this board
  the Mega runs Speeduino firmware; the Rust firmware is a separate skeleton.

## 4. Dwell

Dwell is firmware, not PCB. For a smart coil, start with the manufacturer’s
dwell recommendation and verify coil/driver temperature with a current probe.
For a dumb coil, dwell/current must be validated against the IGBT/coil SOA and
a hardware over-dwell limit is recommended.

## 5. Recommended path

1. Build prototype-v1 as the bench-learning platform.
2. Use **one verified smart coil** only after the §2A checklist; do not connect
   the Evo or a running engine.
3. For dumb coils, design a separate coil-driver board/v2; do not hack v1.
4. Start with high-Z injectors; low-Z needs ballast/peak-and-hold and EE review.
5. Record every part number, resistance, pinout, dwell, supply voltage, scope
   capture and temperature.

## 6. What v2 would need

- 4 ignition IGBT/coil drivers with clamps and gate networks.
- Coil supply bulk/ceramic decoupling and a fused, keyed power connector.
- Separate high-current coil ground to the supply star point.
- Hardware over-dwell/current limit; VNLD STATUS feedback to the MCU.
- Input conditioning and a defined off state when v1 is unpowered.
- Thermal/creepage review, ERC/DRC, BOM, test points and independent review.
- Injector peak-and-hold only if low-Z injectors are required.

## 7. Safety boundary

No vehicle, engine, injector, coil or fuel-system connection until the board has
been reviewed and bench-tested. Use a current-limited supply, external fuse,
keyed lead, dummy loads, a scope and an emergency disconnect. Never fire a coil
with an open secondary; use a grounded spark gap or plug. Do not probe a coil
primary with an oscilloscope input that is not rated for the flyback voltage.
