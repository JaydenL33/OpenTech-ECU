# Evo VIII JDM GSR — vehicle compatibility

Status: **research only, not engineering sign-off.** This document describes what the vehicle
has, what OpenTech-ECU can do today, and what would be required before any physical integration.
It does **not** claim plug-and-play compatibility.

Related documents:

- [`ECU_PINOUT_VERIFICATION.md`](ECU_PINOUT_VERIFICATION.md) — pin-level evidence and confidence
- [`OPEN_ISSUES.md`](OPEN_ISSUES.md) — blocking unknowns and owner actions
- [`EVO8_REFERENCE_PROFILE.md`](EVO8_REFERENCE_PROFILE.md) — verified data and explicit unknowns
- [`../research/evo8/sources.md`](../research/evo8/sources.md) — annotated sources

## 1. Gate and severity definitions

The release-gate numbering is fixed by the master brief and must be used consistently.

| Gate | Name | Exit criterion |
|---|---|---|
| **0** | Software simulation | Rust core, simulation and tests; no hardware, no vehicle |
| **1** | Unpowered hardware verification | PCB inspection, netlist and continuity checks, connector verification, electrical design review; no power |
| **2** | Powered bench verification | Current-limited power, protected I/O, simulated sensors/triggers, fault injection, timing measurements; no engine |
| **3** | Vehicle electrical compatibility | Exact ECU variant identified; connector orientation and pin continuity recorded; key-off and key-on checks; engine not running |
| **4** | Controlled engine testing | First start, idle, charging, cooling and protection checks with qualified supervision and shutdown provision |
| **5** | Vehicle operation | Independent engineering validation, validated calibration, earlier gates passed, insurance/road-legality review |

| Severity | Meaning |
|---|---|
| **S0** | Blocking or safety-critical; could cause injury, fire, engine destruction, or an unrecoverable security regression |
| **S1** | High; prevents a correct build or can cause damage if guessed |
| **S2** | Medium; performance/functionality limitation with a safe workaround |
| **S3** | Low; documentation or convenience |

## 2. Vehicle identity

> **UNVERIFIED until matched to the physical car.** The owner has not yet supplied the ECU part
> number, connector photographs, or VIN. Every row below is a *type* identity, not a claim about
> the individual vehicle.

| Field | Value | Status |
|---|---|---|
| Make / model | Mitsubishi Lancer Evolution VIII GSR | Owner statement; type data TIM |
| Chassis | CT9A (GH-CT9A) | Type data (TIM 0-4) |
| Market | Japan (JDM) | Owner statement |
| Engine | 4G63-DOHC-T/C, 1,997 cc, turbo | Type data (TIM 1-2) |
| Transmission | 6-speed manual, expected **W6MAA** | Expected; confirm from gearbox stamp |
| Drive | AWD, ACD + Super AYC (GSR standard) | Type data (TIM 2-12/2-14) |
| Model year | **UNKNOWN** | Needed; 2003 and 2004 MR differ |
| ECU part number | **UNKNOWN** | Blocking (EVO-ID-001) |
| ECU connector count | **UNKNOWN** | Blocking (EVO-ID-001) |
| VIN / chassis number | **UNKNOWN** | Needed |
| Odometer | **UNKNOWN** | Needed |
| Modifications | **UNKNOWN** | Assume none until inspected |
| Immobiliser | Fitted (JDM standard) | Type data (TIM 7-1) |
| ABS | Fitted (GSR) | Expected; confirm visually |
| A/C | Expected fitted | Confirm visually |
| Fuel | Unleaded premium (JDM) | Type data (TIM 1-2) |

## 3. Variant separation — do not assume shared wiring

| Variant | Market | ECU architecture (evidence) | Key differences relevant to this project | Must not be assumed |
|---|---|---|---|---|
| **JDM GSR 2003 (target)** | Japan | 4-plug or 3-plug; **unconfirmed** | MAF; no MAP named in TIM; no EGR; secondary air; ACD + Super AYC; immobiliser; 6MT | Anything, until ECU photos |
| JDM RS | Japan | 4-plug or 3-plug; **unconfirmed** | 5MT W5M51 or 6MT W6MAA; AYC optional; lighter equipment | Same harness as GSR |
| JDM MR 2004 | Japan | **3-plug** (C-49/50/51) | Wastegate solenoid 2 added; immobiliser ECU changed/moved; 6MT | Same as 2003 GSR |
| UKDM Evo VIII 260 | UK | **4-plug** (1-26/31-46/51-62/71-92) | EGR, front + rear O2, different equipment | Same as JDM |
| USDM Evo VIII 2003 | USA | **4-plug** (C-121/119/117/115) | EGR, MAP sensor, fuel tank pressure, fuel level/temp inputs, rear O2 | Same as JDM |
| Evo VII JDM | Japan | **4-plug** (same numbering family) | EGR, SAS solenoid, MDP sensor, ACD/AYC | Same as Evo VIII |
| Evo IX | Global | **3-plug** (JAE MX7, same family as MR) | MIVEC, MAP sensor, different trigger/cam pattern | Same as Evo VIII |

Sources: S2–S9, S12, S16–S18. The 4-plug/3-plug split is documented; the exact JDM 2003 fitment
is not.

## 4. Compatibility matrix

**Status vocabulary** (as required by the audit brief):

- **Verified** — the function is confirmed and needs no OpenTech interface, or the interface is fully known and compatible.
- **Can support via adapter** — electrically conventional; needs a harness/connector adapter and
  firmware configuration, but no new sensor or power module.
- **Requires additional hardware** — needs a driver, sensor, module, or decoder that does not
  exist today.
- **Requires further research** — evidence is missing or sources conflict; no build decision yet.
- **Unsupported** — OpenTech cannot support it, or the vehicle does not have it.
- **Must not alter without review** — safety, security, emissions, or a separate controller is
  involved; changes require explicit review.

> Every "Can support via adapter" row is still subject to **EVO-PIN-001** (pinout unverified).
> "Verified" in this table means the *function* is understood, not that a pin has been confirmed.

### 4.1 Engine management

| Function | Evo VIII GSR as-built | OpenTech capability today | Status | Gate | Evidence / issue |
|---|---|---|---|---|---|
| Fuel injection | 4 injectors; low-impedance type and resistor pack reported for USDM, JDM unverified; sequential vs batch unconfirmed | Sequential scheduler simulated; no physical output; injector characterisation simulated only | Requires additional hardware | 2→3 | EVO-INJ-001/002 |
| Ignition | Two coils for four cylinders (wasted-spark pairing inferred); igniter location unresolved | Ignition scheduling simulated; physical export blocked; no coil driver characterisation | Requires additional hardware | 2→3 | EVO-IGN-001/002 |
| Crank/cam sync | 2x crank + cam Hall sensors; exact pattern unknown | Generic missing-tooth only; 4G63 dual-wheel decoder not implemented in Rust | Requires additional hardware | 3 | EVO-TRIG-001/002 |
| Load — MAF | MAF (volume-airflow/Karman type, probable) with integrated IAT; frequency output | No MAF input, no frequency capture, no transfer function | Unsupported | 3 | EVO-SENS-001 |
| Load — MAP | **No MAP sensor is named** in the JDM TIM control-system list; MDP was deleted (TIM 1-9). Presence must be visually confirmed. | Speed-density implemented in simulation | Unsupported (not evidenced) | 3 | EVO-SENS-001 |
| Throttle position | Analogue TPS, 5 V reference | Generic analogue input (host) | Can support via adapter | 3 | EVO-SENS-002 |
| Intake air temperature | Thermistor-type IAT in MAF (NTC probable), pre-turbo | Generic NTC model | Can support via adapter | 3 | EVO-SENS-002 |
| Coolant temperature | Thermistor-type CLT (NTC probable), pin 83 | Generic NTC model | Can support via adapter | 3 | EVO-SENS-002 |
| Barometric pressure | Analogue sensor | Generic analogue input | Can support via adapter | 3 | EVO-SENS-002 |
| Knock | Piezo sensor, pin 78; shielding UNKNOWN | No knock conditioning or retard strategy | Requires additional hardware | 2→4 | EVO-SENS-003 |
| Lambda | Switching-type (narrowband) front O2 (probable); rear O2 presence unknown on JDM | Wideband model only; no narrowband closed loop | Requires additional hardware | 3 | EVO-SENS-003 |
| Idle control | Stepper ISC, four driven phases (6-terminal type probable) | Idle control planned, not implemented | Requires additional hardware | 2→4 | EVO-ACT-004 |
| Boost control | Single wastegate solenoid (2003) | Boost control planned, not implemented | Requires additional hardware | 2→4 | EVO-ACT-005 |
| Fuel pressure solenoid | Fitted, hot-start pressure increase | No control strategy | Requires further research | 3→4 | EVO-ACT-006 |
| Purge / EVAP | Fitted | No output strategy | Must not alter without review | 4 | EVO-ACT-007, EVO-PIN-005 |
| Secondary air | Fitted on JDM (TIM 1-10) | No output strategy | Must not alter without review | 4 | EVO-PIN-004 |
| Engine control relay | ECU-controlled MFI relay | No low-side driver defined | Requires additional hardware | 3 | EVO-PWR-001 |
| Fuel pump control | Two-speed (probable), relays 2/3 + resistor | No pump controller | Must not alter without review | 3→4 | EVO-ACT-001 |
| Radiator fan | PWM (probable) via fan control module | No PWM fan output | Requires additional hardware | 3→4 | EVO-ACT-002 |
| A/C condenser fan | HI/LO relays | No outputs | Requires additional hardware | 3→4 | EVO-ACT-002 |
| A/C compressor / idle-up | ECU controls clutch and idle-up | No A/C strategy | Requires additional hardware | 3→4 | EVO-ACT-008 |
| Alternator control | ECU drives G terminal, monitors FR | No alternator control | Must not alter without review | 3→4 | EVO-ACT-003 |
| Starter signal | IG-ST input, pin 71 | Generic digital input | Can support via adapter | 3 | EVO-PWR-002 |
| Vehicle speed | VSS input, pin 86 | No VSS input configured | Can support via adapter | 3 | EVO-SENS-004 |
| Clutch switch | Pin 43 | Generic digital input | Can support via adapter | 3 | EVO-SENS-005 |
| Neutral switch | **Not identified** in Evo VIII 6MT documentation | No input | Unsupported | 3 | EVO-SENS-006 |
| Gear position | **No gear-position sensor identified** | No input | Unsupported | 3 | EVO-SENS-006 |
| Tachometer | ECU output, pin 58 | No tach output driver | Can support via adapter | 3 | EVO-PIN-006 |
| Check-engine light | ECU output, pin 36 | No lamp driver | Can support via adapter | 3 | EVO-PIN-006 |
| Immobiliser | Separate immobiliser ECU authorises engine ECU; MR changes the immobiliser ECU location/arrangement (S7) | No protocol, no emulation | Must not alter without review | 3 | EVO-SEC-001 |
| Factory diagnostics | MUT-II/III K-line + 12-pin connector | OpenTech protocol only; not MUT/OBD compatible | Requires further research | 3 | EVO-DIAG-001 |
| OBD-II | 16-pin DLC, K-line pin 7 | No OBD-II emulation | Unsupported | 3 | EVO-DIAG-001 |
| CAN | **No CAN documented** on Evo VIII | CAN stack planned | Unsupported | 3 | EVO-DIAG-002 |
| ACD | Separate 4WD-ECU; engine ECU sends idle information | No integration | Must not alter without review | 3→4 | EVO-ACD-001 |
| AYC (Super AYC) | Separate 4WD-ECU; hydraulic unit | No integration | Must not alter without review | 3→4 | EVO-ACD-001 |
| ABS | Separate ABS-ECU; feeds 4WD-ECU | No integration | Must not alter without review | 3→4 | EVO-ABS-001 |
| Intercooler spray | Auto/manual inputs, relay, lamp | No strategy | Requires additional hardware | 4 | EVO-ACT-009, EVO-PIN-005 |
| Power-steering pressure | Switch input, idle-up | Generic digital input | Can support via adapter | 3 | EVO-ACT-010 |
| Reverse light | Separate switch, not engine ECU | Not applicable | Verified (no ECU interface) | — | S7 wiring |

### 4.2 Summary counts

| Status | Count | Notes |
|---|---|---|
| Verified | 1 | Reverse light only — no ECU interface, not an engine-management function |
| Can support via adapter | 10 | TPS, IAT, CLT, baro, starter, VSS, clutch, tach, CEL, PAS |
| Requires additional hardware | 12 | Injectors, ignition, trigger, knock, lambda, idle, boost, relay, fans, A/C, IC spray |
| Requires further research | 2 | Fuel pressure solenoid, factory diagnostics |
| Unsupported | 6 | MAF, MAP (not evidenced), neutral, gear, OBD, CAN |
| Must not alter without review | 8 | Immobiliser, ACD, AYC, ABS, alternator, fuel pump, purge, secondary air |

Counts are approximate because several functions have a primary status plus a secondary caveat;
the table is the authority.

## 5. What "plug-and-play" would actually require

A true plug-in solution for this car would need all of the following. None of them exists today.

1. **Confirmed ECU variant.** 4-plug vs 3-plug changes the entire harness and connector kit.
2. **Mating connector kit.** Tyco Multilock 040/070 (4-plug) or JAE MX7 (3-plug), with the correct
   large/small terminals and seals. JDM verification pending.
3. **Pin-for-pin harness adapter.** Every function below mapped and continuity-tested, not inferred.
4. **MAF strategy or a documented MAP conversion.** The JDM car is MAF-based and no MAP sensor is
   named in the TIM; speed-density needs a new sensor, a new IAT location, and a new calibration.
5. **Injector drive matched to the measured resistance.** Resistor pack retained or peak-and-hold
   drivers specified, depending on what is measured.
6. **Igniter-compatible ignition drive.** Logic-level vs power-transistor input must be measured.
7. **Idle stepper drive.** Four phases, correct sequence, correct hold current.
8. **PWM radiator fan output** and two condenser-fan relay outputs.
9. **Two-speed fuel pump control** with the stock resistor and relays.
10. **Alternator G-terminal control.** Without it, charging behaviour is unknown.
11. **A/C compressor and idle-up strategy.** Without it, A/C may not work or idle may stall.
12. **Immobiliser strategy that does not defeat the security system.** This is a security and
    insurance decision, not a wiring trick.
13. **ACD/AYC strategy.** The 4WD-ECU expects engine idle information; removing the stock ECU may
    set faults or disable ACD/AYC.
14. **Wideband lambda** if closed-loop fuel control is required; the factory front sensor is a
    narrowband switching sensor (probable), not a wideband.
15. **Independent review and insurance/road-legality assessment** before road use.

## 6. Explicit non-claims

- This is **not** a plug-and-play assessment.
- This is **not** a wiring diagram.
- This is **not** an engineering sign-off.
- No pin, connector, voltage, current, or wire colour in this document is approved for build.
- No tune, base map, or ignition table is provided.
- The default status for every unlisted function is **Unsupported / Unverified**.
