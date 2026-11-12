# Evo VIII JDM GSR — reference profile (verified data and explicit unknowns)

Status: **research reference, not a calibration and not a tune.** Every value below is either
quoted from the January 2003 Mitsubishi Technical Information Manual (TIM) for the JDM Lancer
Evolution VIII, or explicitly marked as unverified. Nothing here has been checked against the
physical car. Source IDs (S1–S22) are defined in
[`../research/evo8/sources.md`](../research/evo8/sources.md).

## 1. Identity

| Field | Value | Basis |
|---|---|---|
| Make / model | Mitsubishi Lancer Evolution VIII GSR | TIM cover; model-code table |
| Chassis | CT9A (GH-CT9A) | TIM model-code table |
| Market | Japan (JDM) | TIM is the JDM new-model manual |
| Model codes | GSR: **SNGFZ**, **SJGFZ**; RS: SNDFZ (5MT), SJDFZ (6MT) | TIM 0-4 (index/model table) |
| Engine | 4G63-DOHC-T/C, 2.0 L turbocharged inline-4 | TIM 1-2 |
| Transmission | 6-speed manual **W6MAA** (GSR); RS may have W5M51 5MT or W6MAA 6MT | TIM Section 2 (manual transmission) |
| Drive | AWD; ACD standard; Super AYC standard on GSR, optional on RS | TIM 2-2, 2-12, 2-14 |
| Model year of the owner's car | **UNKNOWN** — the 2004 MR is documented with a different (3-plug) ECU architecture; the 2003 fitment is unresolved | See EVO-ID-001 |
| ECU part number | **UNKNOWN** for this car. Candidates only: MN132874 (4-plug, 2003 RS/GSR per USDM list; photo shows 4 bays), MN132750 (Evo 8 GSR, connector type unconfirmed), MN13750 (3-plug, manual, seller listing) | S16, S17 — none verified for this VIN |
| VIN / chassis number | **UNKNOWN** | Owner action |

> **UNVERIFIED until matched to the physical car.** The model codes above identify the *type*,
> not the individual vehicle. Do not order connectors, adapters, or calibrations from this table.

## 2. Engine specification (factory, JDM 2003)

| Item | Value | Source |
|---|---|---|
| Displacement | 1,997 cc | TIM 1-2 |
| Bore × stroke | 85.0 × 88.0 mm | TIM 1-2 |
| Compression ratio | 8.8 : 1 | TIM 1-2 |
| Combustion chamber | Pentroof, DOHC 4-valve | TIM 1-2 |
| Fuel | Unleaded premium | TIM 1-2 |
| Maximum output | 206 kW (280 PS) at 6,500 rpm | TIM 1-2 |
| Maximum torque | 392 N·m at 3,500 rpm | TIM 1-2 |
| Fuel system | Electronic controlled multipoint fuel injection | TIM 1-2 |
| Ignition system | Electronic controlled **two-coil**; wasted-spark pairing inferred | TIM 1-2 |
| Firing order | **UNKNOWN / not quoted** — do not assume | — |

## 3. Transmission (W6MAA)

| Gear | Ratio | Source |
|---|---|---|
| 1st | 2.909 | TIM Section 2, W6MAA specification |
| 2nd | 1.944 | TIM Section 2, W6MAA specification |
| 3rd | 1.434 | TIM Section 2, W6MAA specification |
| 4th | 1.100 | TIM Section 2, W6MAA specification |
| 5th | 0.868 | TIM Section 2, W6MAA specification |
| 6th | 0.693 | TIM Section 2, W6MAA specification |
| Reverse | 2.707 | TIM Section 2, W6MAA specification |
| Final drive | 4.583 | TIM Section 2, W6MAA specification |

## 4. Control-system inventory (factory, JDM 2003)

### Inputs named by the TIM control-system diagram (1-10)

Air flow sensor · intake air temperature sensor · barometric pressure sensor · engine coolant
temperature sensor · throttle position sensor · A/C switch · A/C load signal · camshaft position
sensor · crank angle sensor · alternator FR sensor · vehicle speed sensor · power-steering fluid
pressure switch · detonation (knock) sensor · intercooler water spray (auto) · intercooler water
spray (manual) · oxygen sensor · ignition switch IG · ignition switch ST · power supply.

### Outputs named by the TIM control-system diagram (1-10)

No. 1–4 injectors · idle speed control servo (stepper motor) · No. 1 & No. 4 ignition coil ·
No. 2 & No. 3 ignition coil · engine control relay · fuel pump relay 2 · fuel pump relay 3 ·
A/C relay · radiator fan controller · A/C condenser fan relays (HI, LOW) · alternator G terminal ·
air flow sensor filter reset · fuel pressure control solenoid · wastegate solenoid ·
secondary air control solenoid · intercooler water spray relay · intercooler water spray light ·
tachometer · check-engine light · oxygen sensor heater · purge control solenoid ·
diagnosis output · diagnosis output for MUT-II.

### Deliberately deleted vs Evo VII (TIM 1-9)

- MDP (manifold differential pressure) sensor — **discontinued**.
- EGR control — **discontinued**.
- Diagnostic control terminal — **discontinued** (the MUT-II/III path remains).

> Consequence: the JDM Evo VIII is **MAF-based** with **no factory manifold-pressure sensor**
> named in the TIM. Any speed-density conversion needs a new MAP sensor and a new IAT location.
> See EVO-SENS-001 and the compatibility matrix.

## 5. ACD / AYC (factory, JDM 2003)

| Item | Value | Source |
|---|---|---|
| System | Active Centre Differential + Super AYC | TIM 2-12, 2-14 |
| Controller | Separate **4WD-ECU** | TIM 2-12 |
| Change vs Evo VII | 4WD-ECU ABS monitor terminal **No. 49 → No. 48**; other terminals same as Evo VII | TIM 2-12 |
| Consequence of mismatched 4WD-ECU | Diagnosis code 65 after 60 s; ACD and AYC stop | TIM 2-12 |
| ACD/AYC link to engine ECU | Engine ECU sends "idling state" to 4WD-ECU; TPS is a separate input to 4WD-ECU | S20 (Evo VII TIM 2-15) |

## 6. Immobiliser (factory, JDM 2003)

| Item | Value | Source |
|---|---|---|
| Fitted | Yes, per TIM equipment section | TIM 7-1 |
| Components | Ignition key transponder · ring aerial · immobiliser ECU · engine ECU | TIM 7-1 |
| Behaviour | Immobiliser ECU compares the key ID and "will control the engine ECU only if it matches" | TIM 7-1 |
| Key registration | MUT-II/III; up to 8 keys | TIM 7-1 |
| Effect on engine | Inhibits fuel injection and disables starting | TIM 7-1 |

> This is a security function. It must not be disabled or bypassed. See EVO-SEC-001.

## 7. Unverified but commonly quoted values (do not use as calibration data)

| Item | Quoted value | Why it is not verified |
|---|---|---|
| Injector flow | 560 cc/min at 3 bar | Aftermarket/community sources, not the factory TIM. See EVO-INJ-001. |
| Injector resistance | 2–3 Ω at 20 °C, low-impedance (quoted); resistor-pack presence unverified | Community conversion sheet S11 and forum sources; not measured on this car. |
| Fuel pressure | Not quoted | Must be measured. |
| Firing order | Not quoted | Must be confirmed from the engine, not assumed. |
| Rev limit / boost limit | Not quoted | Calibration data, not identity data. |
| Trigger tooth angles | Not quoted | Must be scoped. See EVO-TRIG-001. |

## 8. Explicit unknowns for the calibration wizard

The following must stay blank/blocked in any wizard profile until the owner supplies evidence:

1. ECU part number and connector architecture (4-plug vs 3-plug).
2. Crank/cam trigger pattern and sensor supply voltage.
3. Injector part number, flow, dead time, and resistor-pack presence.
4. Ignition coil/igniter type and drive polarity.
5. MAF transfer function (frequency vs air mass) and IAT sensor curve.
6. CLT sensor curve, TPS calibration, and baro sensor scaling.
7. Lambda sensor count and type (narrowband front only vs front + rear).
8. Fuel pressure base and manifold reference.
9. ACD/AYC interaction with the stock engine ECU.
10. Immobiliser authorisation path.

No tune, no base map, and no ignition table is provided by this document.
