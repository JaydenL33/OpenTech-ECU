# ECU pinout verification — Evo VIII JDM GSR (CT9A, 4G63, 6MT)

Status: **research only. No pin in this document is approved for wiring.** The default is
UNVERIFIED. Nothing here is a plug-and-play claim.

## 1. Why this document exists

The owner's car is a Japanese-market Evo VIII GSR with the 6-speed W6MAA. No ECU part number,
connector photograph, or factory JDM wiring diagram is available yet. Public sources describe at
least **two different Evo VIII ECU architectures**, and the community pinout sheets disagree with
each other on several pins. This document records what each source actually says, assigns a
confidence, and lists the measurement that would settle it.

## 2. Confidence and status definitions

| Label | Meaning |
|---|---|
| **Verified** | Confirmed by a factory document for this exact model/market. **No pin below reaches this level.** |
| **Probable** | Corroborated by two or more independent sources (factory-adjacent, vendor, or detailed community), with residual variant risk. Still requires continuity check. |
| **Unverified** | One community source, a cross-generation inference, or a hypothesis. Must be physically checked before use. |
| **Unknown** | No usable evidence. Do not guess. |

Source IDs (S1–S22) are defined in [`research/evo8/sources.md`](../research/evo8/sources.md).

## 3. Bottom line

| Question | Answer | Confidence |
|---|---|---|
| Is the ECU 4-plug or 3-plug? | **Unknown for this car.** Both exist under the "Evo VIII" name. | Unknown |
| 4-plug pin count | 26 + 16 + 12 + 22 = 76 populated positions | Probable (S9, S10, S11) |
| 3-plug pin count | 35 + 28 + 30 = 93 positions | Probable (S7 factory connector table) |
| Connector families | 4-plug: Tyco/AMP Multilock 040/070 hybrid. 3-plug: JAE MX7. | Probable (S10, S18) |
| JDM GSR pin functions | Only hypothesised from USDM/UK/Evo VII/Evo VIII 3-plug sheets | Unverified |
| Trigger pattern | 2x crank + cam Hall sensors; exact tooth angles unknown | Probable sensor type; Unverified pattern |
| Injector type | 4 × low-impedance, 2–3 Ω, with resistor pack; flow quoted 560 cc/min | Unverified |
| Ignition | Two coils; wasted-spark pairing inferred; no factory COP | Probable |
| Igniter location | **Unresolved** — external vs integrated in coil | Unknown |
| CAN bus | None documented on Evo VIII | Probable |
| Immobiliser | Separate immobiliser ECU authorises the engine ECU | Verified function; protocol unknown |

## 4. The two Evo VIII ECU architectures

| Feature | Config A — 4-plug | Config B — 3-plug |
|---|---|---|
| Known fitment | 2003 USDM Evo VIII; UK Evo VIII 260; Evo VII JDM (same numbering family) | Evo VIII MR (2004, C-49/50/51); a 3-plug non-MR Evo VIII sheet also exists (S2); shares connector family with Evo IX |
| Connector codes | C-121 (26), C-119 (16), C-117 (12), C-115 (22) — USDM codes | C-49 (35), C-50 (28), C-51 (30) — MR codes |
| Connector family | Tyco/AMP Multilock 040/070 hybrid | JAE MX7 |
| Pin numbering | 1–26, 31–46, 51–62, 71–92 | 1–35, 41–68, 71–100 (per community sheet; not reconciled to C-49/50/51) |
| Evidence | S9, S10, S11, S4, S5, S8 | S7 (factory), S2/S3 (community), S10/S18 (connector family) |
| Distinguishing feature | 4 connectors, Tyco Multilock | 3 connectors, JAE MX7; wastegate solenoid 2 on MR |

**Competing hypotheses (UNRESOLVED):**

- **H-A — 2003 JDM GSR is 4-plug.** Basis: the Evo VII JDM sheet uses the 4-plug numbering
  family; the 2003 USDM car and the UK 260 are 4-plug; the 4-plug family is the older architecture.
- **H-B — 2003 JDM GSR is 3-plug.** Basis: a 3-plug sheet titled "Evo VIII" exists (S2) with no
  MR-specific wastegate 2; JDM models sometimes lead export-market electronics; JustJDM lists both
  3-pin and 4-pin Evo VIII ECUs (S16).

Neither hypothesis is proven. The ECU label and connector count decide it. See **EVO-ID-001**.

> Do **not** order a 3-plug adapter because a web page says "Evo VIII". Do **not** order a 4-plug
> adapter because the car is a 2003. Photograph the ECU first.

## 5. Connector research

### 5.1 Config A — 4-plug (Tyco/AMP Multilock 040/070 hybrid)

| Position | Harness code (USDM) | Pins | Tyco plug P/N (harness side) | Notes |
|---|---|---|---|---|
| A | C-121 | 26 | 917992 | 10 large pins, 16 small (S10) |
| B | C-119 | 16 | 917981 | small pins only per S10 |
| C | C-117 | 12 | 917975 | small pins only |
| D | C-115 | 22 | 917989 | 6 large pins, 16 small |
| ECU-side | — | 76 | 178780 | 76-pin female cap assembly |
| Small terminal | — | — | 175265-1 | 22–18 AWG (S10) |
| Large terminal | — | — | 175269-1 | 20–16 AWG (S10) |

Source: S10 (community wiki, 2011). The part numbers are plausible and internally consistent with
the 26/16/12/22 counts, but they have **not** been checked against a Tyco drawing or the car.
JDM applicability is unverified.

### 5.2 Config B — 3-plug (JAE MX7)

| Position | Harness code (MR) | Pins | JAE plug P/N | Notes |
|---|---|---|---|---|
| A | C-49 | 35 | MX7-A-35SC | grey; 6 large pins per plug (S10) |
| B | C-50 | 28 | MX7-A-28SC | grey |
| C | C-51 | 30 | MX7-A-30SC | grey |
| Small terminal | — | — | MX7-SGC3-7000 | 18–20 AWG |
| Large terminal | — | — | MX7-PWC2-1E-4500 | 16–20 AWG |

Source: S7 for the connector codes and pin counts (factory, MR supplement p.1-5); S10 for the
JAE part numbers (community). S18 independently states that 3-connector Evo 8 ECUs use Evo 9
pins. **The pin numbering within each plug is not established.**

### 5.3 Other relevant connectors

| Connector | Family | Evidence | Use |
|---|---|---|---|
| ACD/AYC 4WD-ECU | JAE MX5-E, 22-pin and 26-pin | S10 | Do not disturb; separate controller |
| Diagnostic 12-pin | Sumitomo MT-series M(6244-1121) / F(6244-5121) | S10 | MUT-II/III and flash |
| OBD-II 16-pin | Tyco 179631 / 348822 (vehicle side) | S10 | K-line on pin 7 |

## 6. Verification matrix — Config A (4-plug), suspected JDM GSR

> **Read this first.** The pins below are the **4-plug USDM/UK numbering family**. They are
> suspected, not confirmed, for the JDM GSR. "Electrical characteristic" is quoted only where a
> source states it; everything else is marked UNKNOWN. No wire colours are given because the
> community sheets' colours are market-specific and unverified.

### 6.1 Power, ground, and relay control

| Signal | Suspected pin | Connector | Source | Electrical characteristic | Verification method | Confidence | Blocking issue |
|---|---|---|---|---|---|---|---|
| 12 V through MFI relay | 12 | C-121 | S9, S4 | Switched 12 V, current UNKNOWN | Key-on voltage; relay test | Probable | EVO-PIN-001 |
| 12 V through MFI relay | 25 | C-121 | S9, S4 | Switched 12 V, current UNKNOWN | Key-on voltage | Probable | EVO-PIN-001 |
| Backup 12 V (battery) | 80 | C-115 | S9, S4, S5 | Constant 12 V, current UNKNOWN | Key-off voltage; fuse trace | Probable | EVO-PWR-001 |
| Ignition switch IG | 82 | C-115 | S9, S4, S5 | Switched 12 V | Key-on voltage | Probable | EVO-PIN-001 |
| Chassis ground | 13 | C-121 | S9, S4 | Ground | Resistance to chassis | Probable | EVO-PWR-001 |
| Chassis ground | 26 | C-121 | S9, S4 | Ground | Resistance to chassis | Probable | EVO-PWR-001 |
| Sensor ground | 40 | C-119 | S9, S4 | Reference ground | Continuity to sensor return | Probable | EVO-PIN-001 |
| Sensor ground | 92 | C-115 | S9, S4, S5 | Reference ground | Continuity to sensor return | Probable | EVO-PIN-001 |
| MFI / engine control relay control | 38 | C-119 | S9 ("ground"), S4 ("ECU relay") | Low-side driver; current UNKNOWN | Relay coil trace; key-on relay click | Probable | EVO-PWR-001 |
| Fuel pump relay control | 39 | C-119 | S9 ("control, ground"); S4 ("fuel pump relay 3") | Low-side driver or relay 3 feed — **ambiguous** | Trace to relay; current clamp | Unverified | EVO-ACT-001 |

### 6.2 Injectors, ignition, trigger, knock

| Signal | Suspected pin | Connector | Source | Electrical characteristic | Verification method | Confidence | Blocking issue |
|---|---|---|---|---|---|---|---|
| Injector 1 | 1 | C-121 | S9, S4, S5, S11 | Pulse output; driver topology UNKNOWN; injector 2–3 Ω (community) | Injector resistance; scope pulse | Probable | EVO-INJ-001 |
| Injector 3 | 2 | C-121 | S9, S4, S5, S11 | Pulse output; driver topology UNKNOWN; 2–3 Ω | as above | Probable | EVO-INJ-001 |
| Injector 2 | 14 | C-121 | S9, S4, S5, S11 | Pulse output; driver topology UNKNOWN; 2–3 Ω | as above | Probable | EVO-INJ-001 |
| Injector 4 | 15 | C-121 | S9, S4, S5, S11 | Pulse output; driver topology UNKNOWN; 2–3 Ω | as above | Probable | EVO-INJ-001 |
| Ignition coil 1 (cyl 1 & 4) | 10 | C-121 | S9, S4, S5, S11 | Ignition output; coil/igniter type UNKNOWN | Coil P/N; connector pin count; scope | Probable pin / Unknown drive | EVO-IGN-001 |
| Ignition coil 2 (cyl 2 & 3) | 23 | C-121 | S9, S4, S5, S11 | Ignition output; coil/igniter type UNKNOWN | as above | Probable pin / Unknown drive | EVO-IGN-001 |
| Crank angle sensor signal | 89 | C-115 | S9, S4, S5 | Hall-effect, 12 V supply (S12); signal swing UNMEASURED | 3-wire sensor check; 2-channel scope | Probable | EVO-TRIG-001 |
| Cam position sensor signal | 88 | C-115 | S9, S4, S5 | Hall-effect, 12 V supply (S12); signal swing UNMEASURED | 3-wire sensor check; scope | Probable | EVO-TRIG-001 |
| Knock sensor | 78 | C-115 | S9, S4, S5 | Piezo; shielding and amplitude UNKNOWN | Sensor resistance; knock tap | Probable | EVO-SENS-003 |

### 6.3 Load, temperature, position, lambda

| Signal | Suspected pin | Connector | Source | Electrical characteristic | Verification method | Confidence | Blocking issue |
|---|---|---|---|---|---|---|---|
| MAF (volume-airflow/Karman type, probable) signal | 90 | C-115 | S9, S4, S5 | Frequency output; transfer UNKNOWN | MAF P/N; frequency at idle/cruise | Probable | EVO-SENS-001 |
| MAF reset | 19 | C-121 | S9, S4, S5 | Digital output; type UNKNOWN | Scope; MAF P/N | Probable | EVO-SENS-001 |
| Intake air temperature | 72 | C-115 | S9, S4, S5 | Thermistor analogue (NTC probable); curve UNKNOWN | Resistance at 0/20/80 °C | Probable | EVO-SENS-002 |
| Barometric pressure | 85 | C-115 | S9, S4, S5 | Analogue; range UNKNOWN | Voltage at key-on vs local baro | Probable | EVO-SENS-002 |
| Engine coolant temperature | 83 | C-115 | S9, S4, S5 | Thermistor analogue (NTC probable); curve UNKNOWN | Resistance at 0/20/80 °C | Probable | EVO-SENS-002 |
| TPS signal | 84 | C-115 | S9, S4, S5 | Analogue, 0–5 V nominal; measured range UNKNOWN | Sweep with key on | Probable | EVO-SENS-002 |
| TPS / sensor 5 V reference | 81 | C-115 | S9, S4, S5 | 5 V reference (source states +5 V) | Key-on voltage under load | Probable | EVO-SENS-002 |
| Front O2 signal | 76 | C-115 | S9, S4, S5 | Narrowband zirconia (probable); range UNKNOWN | Sensor type; voltage swing | Probable | EVO-SENS-003 |
| Rear O2 signal | 75 | C-115 | S9 (USDM), S4 (UK) | Narrowband; JDM presence **UNKNOWN** | Visual/connector count | Unverified | EVO-SENS-003 |
| Front O2 heater | 60 | C-117 | S9, S4 | Heater output; topology and current UNKNOWN | Resistance; current | Probable | EVO-SENS-003 |
| Rear O2 heater | 54 | C-117 | S9, S4 | Heater output; topology UNKNOWN; JDM presence UNKNOWN | Resistance; current | Unverified | EVO-SENS-003 |
| Fuel tank pressure (USDM) | 61 | C-117 | S9 only | Analogue; not expected on JDM | Visual; pin trace | Unverified | EVO-PIN-001 |
| Fuel level (USDM) | 46 | C-119 | S9 only | Analogue; not expected on JDM | Visual; pin trace | Unverified | EVO-PIN-001 |
| Fuel level low (USDM) | 74 | C-115 | S9 only | Digital; not expected on JDM | Visual | Unverified | EVO-PIN-001 |
| Fuel temperature (USDM) | 77 | C-115 | S9 only | Analogue; not expected on JDM | Visual | Unverified | EVO-PIN-001 |
| MAP sensor (USDM) | 73 | C-115 | S9 only | Analogue; **no MAP sensor is named in the JDM TIM** | Visual; confirm no sensor | Unverified | EVO-SENS-001 |

### 6.4 Actuators

| Signal | Suspected pin | Connector | Source | Electrical characteristic | Verification method | Confidence | Blocking issue |
|---|---|---|---|---|---|---|---|
| ISC stepper A1 | 4 | C-121 | S9, S4, S5 | Stepper phase; resistance UNKNOWN | Coil resistance; sequence | Probable | EVO-ACT-004 |
| ISC stepper B1 | 5 | C-121 | S9, S4, S5 | Stepper phase | as above | Probable | EVO-ACT-004 |
| ISC stepper A2 | 17 | C-121 | S9, S4, S5 | Stepper phase | as above | Probable | EVO-ACT-004 |
| ISC stepper B2 | 18 | C-121 | S9, S4, S5 | Stepper phase | as above | Probable | EVO-ACT-004 |
| Wastegate solenoid | 11 | C-121 | S9, S4, S5 | Solenoid driver; topology and current UNKNOWN | Resistance; PWM scope | Probable | EVO-ACT-005 |
| Fuel pressure solenoid | 3 | C-121 | S9, S4, S5 | Solenoid driver; topology UNKNOWN | Resistance; function test | Probable | EVO-ACT-006 |
| Purge solenoid | 9 | C-121 | S9, S4, S5 | Solenoid driver; topology UNKNOWN | Resistance | Probable | EVO-ACT-007 |
| Secondary air solenoid (JDM) | 53 | C-117 | S5 (Evo VII JDM) | Solenoid driver; topology UNKNOWN | Trace to air valve; resistance | Unverified | EVO-PIN-004 |
| EGR solenoid (USDM/UK) | 6 | C-121 | S9, S4, S5 | Solenoid driver; topology UNKNOWN; **EGR deleted on JDM Evo VIII** | Confirm no EGR valve | Unverified | EVO-PIN-004 |
| A/C compressor clutch | 8 | C-121 | S9, S4, S14 | Low-side driver ("control, ground" per S9) | Relay trace; current | Probable | EVO-ACT-008 |
| A/C control unit signal | 24 | C-121 | S9 ("A/C control (?)"), S4 ("A/C control unit") | Output or request line — type UNKNOWN | Trace to A/C control unit; scope | Unverified | EVO-ACT-008 |
| A/C condenser fan high | 32 | C-119 | S9, S4, S14 | Relay control; type UNKNOWN | Relay trace | Probable | EVO-ACT-002 |
| A/C condenser fan low | 34 | C-119 | S9, S4 | Relay control; type UNKNOWN | Relay trace | Probable | EVO-ACT-002 |
| Radiator fan PWM | 21 | C-121 | S4 ("rad fan speed"), S5; **S9 says A/C condenser fan** | PWM into fan control module; **disputed** | Trace to fan module; scope duty | Unverified | EVO-PIN-003 |
| Fuel pump relay 2 | 22 | C-121 | S4, S5; S9 "fuel pump relay" | Relay control; type UNKNOWN | Relay trace; current | Probable | EVO-ACT-001 |
| IC spray relay / EVAP vent | 55 | C-117 | S4 ("IC spray relay"), S5 ("IC spray relay"), S9 ("EVAP vent") | **Market-conflicted output** | Trace to IC spray relay or EVAP valve | Unverified | EVO-PIN-005 |
| IC spray relay (USDM position) | 57 | C-117 | S9; S4/S5 say N/C | Relay control | Trace | Unverified | EVO-ACT-009 |
| IC spray auto input | 44 | C-119 | S9, S4, S5 | Switch input | Switch continuity | Probable | EVO-ACT-009 |
| IC spray manual input | 91 | C-115 | S9, S4, S5 | Switch input | Switch continuity | Probable | EVO-ACT-009 |
| IC spray lamp / fluid meter | 35 | C-119 | S9 ("fluid meter"), S4 ("warning lamp") | Output/input — **disputed** | Trace | Unverified | EVO-ACT-009 |

### 6.5 Vehicle, security, diagnostics, charging

| Signal | Suspected pin | Connector | Source | Electrical characteristic | Verification method | Confidence | Blocking issue |
|---|---|---|---|---|---|---|---|
| Vehicle speed sensor | 86 | C-115 | S9, S4, S5 | Pulse input; type UNKNOWN | Scope while rolling | Probable | EVO-SENS-004 |
| Clutch pedal position switch | 43 | C-119 | S9, S4, S5 | Switch input | Pedal continuity | Probable | EVO-SENS-005 |
| Starter signal (IG-ST) | 71 | C-115 | S9, S4, S5 | Crank input; nominal battery voltage, UNMEASURED | Key-to-ST voltage | Probable | EVO-PWR-002 |
| Immobiliser link | 51 | C-117 | S9, S4, S5 | **Protocol UNKNOWN** | Do not probe as a simple switch; consult MUT-III data | Unverified | EVO-SEC-001 |
| ACD/AYC signal | 87 | C-115 | S4, S5; S9 blank | **Type UNKNOWN** | Scope with ACD active; trace to 4WD-ECU | Unverified | EVO-ACD-001 |
| Alternator G terminal | 33 | C-119 | S9 ("regulator ground"), S4/S5 ("alternator G") | **Description conflict**; driver type UNKNOWN | Scope; charging test | Unverified | EVO-ACT-003 |
| Alternator FR terminal | 41 | C-119 | S9, S4, S5 | Frequency/duty input; type UNKNOWN | Scope; charging test | Probable pin / Unknown signal | EVO-ACT-003 |
| Tachometer output | 58 | C-117 | S9, S4, S5 | Pulse output; type UNKNOWN | Scope at cluster | Probable | EVO-PIN-006 |
| Check-engine light | 36 | C-119 | S9, S4, S5 | Lamp driver; topology UNKNOWN | Lamp test | Probable | EVO-PIN-006 |
| Diagnostic K-line | 62 | C-117 | S9, S4, S5 | K-line (S9); ISO 9141-2 family probable, levels UNMEASURED | Scope; scan-tool test | Probable | EVO-DIAG-001 |
| Diagnostic flash / MUT line | 79 | C-115 | S9, S4, S5 | Digital; type UNKNOWN | Scope | Unverified | EVO-DIAG-001 |
| Power-steering pressure switch | 37 | C-119 | S9, S4, S5 | Switch input | Continuity with steering load | Probable | EVO-ACT-010 |
| A/C pressure switch | 45 | C-119 | S4, S5; S9 "A/C control (?)" | Switch input | Continuity vs refrigerant pressure | Unverified | EVO-ACT-008 |

## 7. Config B (3-plug) — function inventory only

The factory MR supplement gives the connector codes and pin counts (S7, p.1-5). The community
sheet S3 (MR) gives a 1–35 / 41–68 / 71–100 function list, but **that numbering has not been
reconciled with C-49/C-50/C-51**. The non-MR 3-plug sheet S2 uses the same numbering with pin 6
unused. Treat the following as a function inventory, not a pinout.

| Function | Sheet pin (S3) | Confidence |
|---|---|---|
| Injector 1 / 4 / 2 / 3 | 1 / 2 / 9 / 24 | Unverified |
| Ignition coil 1 / 2 | 11 / 12 | Unverified |
| Crank angle sensor | 43 | Unverified |
| Cam position sensor | 50 | Unverified |
| TPS / water temp / air temp / baro | 78 / 44 / 62 / 51 | Unverified |
| MAF signal | 61 | Unverified |
| Knock sensor | 91 | Unverified |
| Front O2 | 71 | Unverified |
| ISC phases | 14 / 15 / 28 / 29 | Unverified |
| Fuel pressure solenoid | 48 | Unverified |
| Purge solenoid | 16 | Unverified |
| Wastegate solenoid 1 / 2 | 41 / 6 | Unverified |
| Secondary air solenoid | 5 | Unverified |
| Fuel pump relay 2 / 3 | 21 / 39 | Unverified |
| IC spray relay / auto / manual / lamp | 55 / 66 / 67 / 90 | Unverified |
| Radiator fan speed / A/C fan hi / lo | 18 / 30 / 31 | Unverified |
| A/C relay / pressure switch | 20 / 83 | Unverified |
| Alternator G / FR | 8 / 52 | Unverified |
| Tachometer / CEL | 45 / 22 | Unverified |
| Vehicle speed / ACD-AYC | 80 / 79 | Unverified |
| Immobiliser / diagnostics / OBD | 98 / 100 / 85 | Unverified |
| Starter / +12 V / grounds | 68 / 47,59,60,99 / 46,58,34,49 | Unverified |

## 8. Trigger system research

### 8.1 What is established

| Claim | Source | Confidence |
|---|---|---|
| From Evo 4 onward the trigger is a **2x crank trigger** (4 pulses per 720°), replacing the earlier cam-mounted 4x CAS | S12 | High (manufacturer) |
| Crank sensor is **Hall effect**, 3 wires, 12 V supply | S12 | High (manufacturer) |
| Cam sensor is **Hall effect**, mounted on the exhaust camshaft | S12, S13, S8 Group 16 p.16-2 | High (manufacturer/factory) |
| Evo 8 **MR–9** use a different trigger configuration from Evo 1–7; Evo 9 cam pattern is inverted | S12 | High (manufacturer) |
| The 4G63 decoder in the retained Speeduino tree assumes 8 crank edges per 720° at alternating **70°/110°** spacing plus cam sync | S21 | Low for OEM truth |

### 8.2 What is not established

- The actual tooth angles on this engine.
- Whether the crank wheel is 2 unequal teeth (70°/110°) or another arrangement.
- The cam pulse width, position, and number of pulses per cycle.
- The sensor supply source and whether it is 12 V or 5 V on the JDM harness.
- The signal polarity and idle level.
- The TDC offset relative to the cam pulse.

### 8.3 Competing hypotheses (all UNVERIFIED)

| Hypothesis | Basis | Consequence if wrong |
|---|---|---|
| H1: 2-tooth crank, unequal 70°/110°, one cam pulse per cycle | Speeduino 4G63 decoder (S21); Haltech "2x crank" (S12) | Wrong tooth angle → timing error, possible engine damage |
| H2: 2-tooth crank, equal spacing, cam phase only | Generic 2-tooth interpretation | Same |
| H3: MR/9-style trigger configuration (Evo 9 cam pattern inverted) | Haltech says Evo 8 MR–9 differ from Evo 1–7 (S12) | Sync failure or 360° phase error |
| H4: aftermarket trigger wheel fitted | Unknown vehicle history | Completely different decoder |

### 8.4 Required capture

1. Two-channel oscilloscope, crank signal and cam signal, common ground at the sensor.
2. Capture at **cranking** and at **idle**, 720° window, with a #1 TDC reference mark.
3. Record: edge count per 720°, angle between edges, pulse widths, high/low levels, noise.
4. Repeat with the sensor connector back-probed, not pierced.
5. Photograph the crank wheel and cam wheel through the sensor hole if accessible.

**Do not configure a decoder from H1 until the capture is reviewed.** See **EVO-TRIG-001**.

## 9. Component findings

### 9.1 Injectors

| Item | Finding | Source | Confidence |
|---|---|---|---|
| Count | 4 | S6, S9 | Verified (function) |
| Pins | 1, 14, 2, 15 (4-plug) | S9, S4, S5, S11 | Probable |
| Resistance | 2–3 Ω at 20 °C, low-impedance | S11, community | Unverified |
| Resistor pack | Shown in USDM wiring diagram ("INJECTOR RESISTOR") | S8 p.90-38 | Probable for USDM; JDM unverified |
| Flow | 560 cc/min at 3 bar (quoted) | Aftermarket/community | Unverified |
| Dead time | Not published | — | Unknown |

A low-impedance injector without the resistor pack will over-current a saturated driver. Verify
the resistor pack before connecting any ECU. See **EVO-INJ-001/002**.

### 9.2 Ignition

| Item | Finding | Source | Confidence |
|---|---|---|---|
| Topology | Two coils for four cylinders (TIM); wasted-spark pairing inferred, not stated | S6 (two-coil), S9 | Probable |
| Pins | Coil 1 (1&4) = 10; coil 2 (2&3) = 23 | S9, S4, S5 | Probable |
| Coil part | MD362907-type, 3-pin, listed for 1997–2007 Mitsubishi | S22 | Low–medium (catalogue fitment) |
| Igniter | **Unresolved**: Haltech says early 4G63 external igniter, later internal; no separate igniter component appears in the extracted 2003 USDM wiring text | S13, S8 | Unknown |
| Dwell | Not published | — | Unknown |

**Do not drive the coils until the trigger input type is measured.** A logic-level input and a
power-transistor input are not interchangeable. See **EVO-IGN-001**.

### 9.3 Knock

Piezo knock sensor on pin 78 (Probable). Shielding is not confirmed in the accessible sources. OpenTech has no Rust knock
conditioning or retard strategy implemented; the protection feature matrix lists knock alarms but
no sensor interface. Requires hardware and firmware. See **EVO-SENS-003**.

### 9.4 Idle speed control

Stepper ISC motor, four driven phases on pins 4, 5, 17, 18 (Probable); 6-terminal type (probable). The TIM calls it "idle
speed control servo (stepper motor)". OpenTech idle control is *planned*, not implemented.
See **EVO-ACT-004**.

### 9.5 Boost control

Single wastegate solenoid on pin 11 for the 2003 car (Probable). The MR adds a second solenoid
(S3, S7). OpenTech boost control is *planned*, not implemented. See **EVO-ACT-005**.

### 9.6 Cooling fans

The radiator fan is driven through a **fan control module** containing a field-effect transistor
and current detection (S8 p.90-42/90-43), not a simple relay. PWM control is probable, not stated. The A/C condenser fan has HI and LO
relay outputs on pins 32/34 (Probable). Pin 21 is disputed: UK/Evo VII sheets call it "rad fan
speed"; the USDM sheet calls it "A/C condenser fan control". See **EVO-PIN-003**, **EVO-ACT-002**.

### 9.7 Fuel pump

Two-speed fuel pump (probable): relays 2 and 3 plus a fuel pump resistor (S6, S8). Pin 22 is relay 2; pin 39
is described as either relay control or relay 3. Pin 55 is the **IC-spray relay** on the UK 260
and Evo VII sheets but the **EVAP vent** on the USDM sheet. See **EVO-ACT-001**, **EVO-PIN-005**.

## 10. What the owner must photograph and measure

### 10.1 Photographs

1. ECU label: full part number, serial, barcode, "Mitsubishi Electric" markings.
2. All sides of the ECU, especially the connector face, with a ruler.
3. Each harness plug, wire side and terminal side, showing keyways and latch.
4. Any moulded pin numbers or connector codes on the housings.
5. The crank sensor, cam sensor, coils, injectors, MAF, and throttle body connectors.
6. The 4WD-ECU and its connectors (do not disconnect).
7. The diagnostic connector(s) under the dash.

### 10.2 Key-off measurements

- Resistance from each suspected injector pin to the injector connector (with ECU disconnected).
- Injector resistance at 20 °C.
- Coil primary resistance and connector pin count.
- Continuity from each suspected sensor pin to the sensor connector.
- Resistance from each power/ground pin to chassis (detect shorts).
- Resistance of the crank and cam sensor windings if inductive; Hall sensors will not show a simple winding.

### 10.3 Key-on measurements (engine not running, fuel/spark disabled)

- Voltage on pins 12, 25, 80, 82.
- Voltage on pin 81 (5 V reference) and pin 38 (relay control).
- TPS sweep voltage.
- Sensor supply voltage at the MAF, cam, and crank sensors.
- Fan control module PWM with the fan commanded on (use a safe test method).

### 10.4 Cranking captures

- Crank and cam scope, 720°, as described in §8.4.
- Injector pulse and coil trigger with a current clamp / logic probe.
- Battery voltage during cranking.

## 11. Prohibited inferences

The following are **not** acceptable evidence and must not appear in a wiring decision:

- "It is an Evo VIII, so it is the same as the USDM Evo VIII."
- "It is a 6-speed, so it is an MR."
- "Evo VII and Evo VIII are the same platform."
- "The wire colour matches the UK sheet."
- "Pin 1 is at the top left because the photo looks like it."
- "The immobiliser can just be disabled."
- "The ACD/AYC will be fine without the stock ECU."

Each of these is a potential write-off or safety fault. See `AUDIT/OPEN_ISSUES.md`.
