# Evo VIII JDM GSR — open issues

Status: **research only.** These are the blocking unknowns for any physical integration. They are
grouped by system; severity and gate are shown so the list can be re-prioritised. No issue is
closed by a web page alone.

Gate and severity definitions are in [`VEHICLE_COMPATIBILITY.md`](VEHICLE_COMPATIBILITY.md) §1.
Source IDs (S1–S22) are in [`../research/evo8/sources.md`](../research/evo8/sources.md).

## 1. Summary

| Count | Severity |
|---|---|
| 8 | S0 — blocking or safety-critical |
| 16 | S1 — high |
| 11 | S2 — medium |
| 4 | S3 — low |

The single most important action is **EVO-ID-001**: photograph the ECU label and count the
connectors. Almost every pin-level issue is downstream of that.

## 2. Issue register

### 2.1 Identity and architecture

| ID | Sev. | Gate | Issue | Evidence needed | What NOT to do |
|---|---|---|---|---|---|
| **EVO-ID-001** | S0 | 3 | ECU variant unknown. Evo VIII exists with a **4-plug** (Tyco Multilock) and a **3-plug** (JAE MX7) ECU. The pinouts, connector kits, and some functions differ. | Photo of ECU label (full part number, serial, barcode); photo of connector face with a ruler; connector count; VIN/build date; model code. | Do not order a plug-in adapter, ECU, or connector kit. Do not assume 4-plug because the car is a 2003. |
| **EVO-ID-002** | S1 | 3 | Market/model-year ambiguity. JDM GSR, JDM RS, JDM MR, UK 260, and USDM cars have different equipment and wiring. | VIN plate; model code (SNGFZ/SJGFZ etc.); build date; ECU label; ACD/AYC presence; 6MT confirmation. | Do not apply USDM or UK pin functions to the JDM car. Do not apply Evo VII data to an Evo VIII. |
| **EVO-ID-003** | S2 | 3 | Transmission not yet confirmed as W6MAA 6MT. | Gearbox stamp/plate; gear lever pattern; driveshaft/transfer visual. | Do not assume the 5MT W5M51 harness or clutch switch arrangement. |

### 2.2 Pinout and connectors

| ID | Sev. | Gate | Issue | Evidence needed | What NOT to do |
|---|---|---|---|---|---|
| **EVO-PIN-001** | S0 | 3 | No factory JDM Evo VIII pinout has been obtained. All pin numbers in the audit are **suspected**, from USDM/UK/Evo VII sources. | Factory JDM wiring diagram for the exact model, **or** pin-to-pin continuity from each ECU pin to the named sensor/actuator with the ECU disconnected. | Do not wire anything from a community pinout sheet. Do not treat two agreeing forums as verification. |
| **EVO-PIN-002** | S0 | 3 | Connector identity and condition unknown. Terminal sizes, keyways, latch type, and corrosion are uninspected. | Photographs of each plug (wire side and terminal side), moulded part numbers, terminal count, keyway position; inspect for heat/corrosion. | Do not probe with oversized pins. Do not force a connector. Do not re-pin a harness without a terminal extraction tool. |
| **EVO-PIN-003** | S1 | 3 | **Pin 21 conflict.** UK/Evo VII sheets call it "radiator fan speed"; the USDM sheet calls it "A/C condenser fan control". Both cannot be right for this car. | Trace pin 21 to the fan control module or condenser fan relay; measure PWM with the fan commanded; compare to the factory JDM diagram. | Do not connect a fan output to pin 21 until traced. A PWM module driven by a relay signal can be damaged. |
| **EVO-PIN-004** | S1 | 3 | **Pin 6 / pin 53 conflict.** Evo VII JDM uses pin 53 for the secondary-air solenoid; USDM/UK sheets use pin 6 for EGR. The JDM Evo VIII deleted EGR but kept secondary air (TIM 1-9/1-10). The actual pin is unknown. | Trace pins 6 and 53 to the secondary-air solenoid/valve; resistance and function test; factory JDM diagram. | Do not energise either pin as an emissions output until traced. |
| **EVO-PIN-005** | S2 | 3 | **Pin 55 conflict.** UK 260 and Evo VII sheets: IC spray relay. USDM sheet: EVAP vent solenoid. Market-specific; the JDM Evo VIII may use either or neither. | Trace pin 55 to the IC spray relay or EVAP valve; measure with key on. | Do not connect an IC-spray or EVAP output to pin 55 without tracing. Do not assume the USDM EVAP assignment. |
| **EVO-PIN-006** | S2 | 3 | Connector pin-numbering orientation is not confirmed (wire side vs component side; row order). | Photograph the moulded numbers; continuity-test pin 1 to a known function; document the view direction. | Do not count pins from a photo. Do not assume pin 1 is top-left. |

### 2.3 Trigger and sensors

**Cam-window software gap fixed in host code.** When cam is required, sequential sync requires an edge inside an explicitly configured crank-angle window; absent/outside windows reject phase and expose `cam_phase_rejected`. Tests: `cam_inside_window_gives_sequential`, `cam_outside_window_is_rejected`, `cam_without_window_is_rejected`, `cam_rejection_revokes_validated_phase`; `timeout_declares_lost_sync` still passes, and `cam_not_required_keeps_sequential_without_window` preserves optional-cam behaviour. These are synthetic host tests. Physical Evo cam-pattern, window and TDC-offset verification remain open under EVO-TRIG-001/002/003 below.

**Sensor-threshold single-sample latching gap fixed in host code (A3).** Coolant over-temperature, low oil pressure, lean under load, overboost and low battery voltage require N consecutive cut-condition evaluations per DTC before latching (`ProtectionLimits::latch_confirmation_samples`, default 3; zero treated as one). Clear or derate-only samples reset confirmation. Tests: `single_sample_overtemp_does_not_latch`, `n_consecutive_samples_latch`, `clearing_before_n_samples_resets_confirmation`; `overtemp_latches_and_requires_explicit_clear` and `low_oil_pressure_cuts_and_latches` now feed N samples. Calibration checksum/CRC and watchdog failures are deterministic integrity faults and retain immediate first-evaluation latching: the unchanged `calibration_crc_failure_latches_a_cut` and new `crc_mismatch_latches_on_first_evaluation` cover CRC. Cut/derate reactions remain immediate, non-latching faults including low fuel pressure are unchanged, and thresholds remain unchanged placeholders. Sensor/limit validation and physical fault-injection evidence remain open.

| ID | Sev. | Gate | Issue | Evidence needed | What NOT to do |
|---|---|---|---|---|---|
| **EVO-TRIG-001** | S0 | 3 | Trigger pattern unverified. Haltech establishes 2x crank + cam Hall sensors, but not the tooth angles; Speeduino's 4G63 decoder assumes 70°/110° edges, which is an implementation, not OEM documentation. Evo 8 MR–9 use a different configuration. | Two-channel scope, crank + cam, 720° at cranking and idle, with #1 TDC mark; edge count and angles; sensor supply and signal levels. | Do not configure a decoder from the Speeduino assumption. Do not attempt a start with an unverified offset. |
| **EVO-TRIG-002** | S1 | 3 | Crank/cam sensor supply and output type unconfirmed on the JDM harness (Haltech says 12 V Hall; the ECU may supply a different voltage). | Measure sensor supply with key on; measure signal high/low; identify 3-wire connector; sensor part number. | Do not feed 12 V into a 5 V sensor. Do not assume the sensor ground is chassis ground. |
| **EVO-TRIG-003** | S1 | 3 | Cam phase / TDC offset unknown. A 360° phase error will run but destroy timing. | Scope with #1 TDC mark; confirm cam pulse relative to crank; timing-light check. | Do not guess the offset. Do not rely on "it starts" as proof of correct phase. |
| **EVO-SENS-001** | S1 | 3 | MAF transfer function unknown. The JDM car is MAF-based; no MAP sensor is named in the TIM. | MAF part number; bench flow or vehicle log of frequency vs known air mass; IAT location. | Do not substitute a generic MAF curve. Do not convert to speed-density without a MAP sensor and a new calibration. |
| **EVO-SENS-002** | S2 | 3 | Sensor curves (IAT, CLT, TPS, baro) not measured. | Resistance at 0/20/80 °C for NTCs; TPS sweep; baro voltage vs local pressure. | Do not use default sensor curves as calibration data. |
| **EVO-SENS-003** | S1 | 3 | Lambda sensor count/type unknown for JDM (front only vs front + rear; narrowband). | Visual inspection; connector count; sensor part number; voltage swing test. | Do not connect a narrowband sensor to a wideband input. Do not assume a rear sensor exists. |
| **EVO-SENS-004** | S2 | 3 | Vehicle speed sensor type unknown (Hall vs reed vs inductive). | Scope while rolling; sensor part number; pull-up requirement. | Do not connect a VSS directly to a 5 V input without checking. |
| **EVO-SENS-005** | S3 | 3 | Clutch switch type (normally open/closed) and pin 43 assignment unverified. | Pedal continuity test; pin trace. | Do not assume the switch polarity for cranking or fuel cut. |
| **EVO-SENS-006** | S3 | 3 | No neutral or gear-position switch identified in Evo VIII 6MT documentation. | Inspect gearbox for switches; confirm only reverse and clutch are present. | Do not invent a neutral input for idle or launch strategy. |

### 2.4 Fuel, ignition, and idle

| ID | Sev. | Gate | Issue | Evidence needed | What NOT to do |
|---|---|---|---|---|---|
| **EVO-INJ-001** | S1 | 2→3 | Injector flow, dead time, and part number unknown. 560 cc/min is a community figure, not a factory value for this car. | Injector part number; resistance at 20 °C; flow bench at 3 bar; dead-time characterisation. | Do not use a generic 560 cc dead-time table. Do not exceed a safe duty cycle without a fuel-pressure check. |
| **EVO-INJ-002** | S1 | 3 | Injector resistance and resistor-pack presence unknown. If the injectors are low-impedance, operation without current limiting can over-current a driver. | Locate the resistor pack (USDM diagram shows one); measure resistance; confirm injector resistance. | Do not drive low-Z injectors from a saturated driver without the resistor pack or peak-and-hold hardware. |
| **EVO-IGN-001** | S0 | 2→3 | Igniter location unresolved (external vs integrated in coil). The wrong drive type can destroy coils or the ECU output. | Coil part number; connector pin count; measure the trigger signal with a scope; compare to a known igniter datasheet. | Do not connect an ignition output until the input type is confirmed. Do not assume "3-pin coil = logic input". |
| **EVO-IGN-002** | S1 | 3 | Wasted-spark pairing and coil wiring unverified (1&4, 2&3 assumed). | Continuity from coil towers to cylinders; visual; factory diagram. | Do not assume sequential ignition is possible. Do not rewire to COP without a new trigger and calibration. |
| **EVO-ACT-004** | S2 | 2→4 | Idle stepper phase order and coil resistance unknown. | Resistance per phase; sequence test on the bench; factory diagram. | Do not connect a stepper to injector outputs. Do not hold a phase continuously. |
| **EVO-ACT-005** | S2 | 2→4 | Boost solenoid type, resistance, and PWM frequency unknown; no overboost protection implemented. | Solenoid resistance; scope duty; boost pressure source; overboost limit definition. | Do not run boost control without an independent overboost cut. |
| **EVO-ACT-006** | S2 | 3→4 | Fuel-pressure solenoid function and pin 3 assignment unverified. | Resistance; function test; factory diagram. | Do not energise continuously. |
| **EVO-ACT-007** | S2 | 4 | Purge/EVAP strategy and legality. | Solenoid resistance; emissions requirement; factory diagram. | Do not delete the EVAP system without an emissions/legal review. |

### 2.5 Power, charging, cooling, and A/C

| ID | Sev. | Gate | Issue | Evidence needed | What NOT to do |
|---|---|---|---|---|---|
| **EVO-PWR-001** | S0 | 3 | Power/ground/engine-control-relay integrity unknown. A bad ground or backfeed can destroy the ECU or cause a fire. | Voltage drop on ground straps; relay coil and contact test; fuse ratings; key-off short checks. | Do not backfeed the harness. Do not install an ECU with a known ground fault. |
| **EVO-PWR-002** | S1 | 3 | Starter/charging circuit and ground strap condition unknown. | Cranking voltage; voltage drop; alternator test; starter current. | Do not crank a car with a loose battery or ground connection. |
| **EVO-ACT-001** | S1 | 3→4 | Two-speed fuel pump control (probable; relays 2/3 + resistor) not mapped. Single-speed operation may be lean at high load. | Relay/resistor trace; pump current; fuel pressure vs load; factory diagram. | Do not bypass the resistor or relay without a fuel-pressure check at full load. |
| **EVO-ACT-002** | S1 | 3→4 | Radiator fan is controlled through a fan control module (PWM probable), not a simple relay. Pin 21 is disputed. | Fan module pinout; PWM duty vs temperature; trace pin 21. | Do not drive the fan module with a relay on/off signal. |
| **EVO-ACT-003** | S1 | 3→4 | Alternator G/FR control. The ECU is part of the charging loop; pin 33 is described differently by different sources. | Measure G and FR signals; charging voltage with the stock ECU connected and disconnected; alternator part number. | Do not run the engine for long with the alternator control disconnected. Do not assume it self-regulates. |
| **EVO-ACT-008** | S1 | 3→4 | A/C compressor, pressure switch, condenser fans, and idle-up are ECU-controlled. Pin 24 is an additional "A/C control unit" line of unknown direction. | Switch/pressure-switch continuity; relay test; pin 24 trace and scope; idle-up behaviour; factory diagram. | Do not run A/C without idle-up validation. Do not bypass the pressure switch. Do not drive pin 24 as an output until its direction is confirmed. |
| **EVO-ACT-009** | S3 | 4 | Intercooler spray inputs/outputs and lamp/fluid-meter function are market-specific. | Switch continuity; relay test; lamp test; factory diagram. | Do not treat pin 35 as a simple lamp without tracing. |
| **EVO-ACT-010** | S3 | 3 | Power-steering pressure switch input unverified. | Continuity with steering load; pin trace. | Do not connect to a 12 V source. |

### 2.6 Security, ACD/AYC, ABS, diagnostics

| ID | Sev. | Gate | Issue | Evidence needed | What NOT to do |
|---|---|---|---|---|---|
| **EVO-SEC-001** | S0 | 3 | Immobiliser interaction. The immobiliser ECU authorises the engine ECU; replacing the ECU may prevent starting **or silently defeat the immobiliser**. The protocol is not public. | Key behaviour; MUT-III immobiliser status; wiring between immobiliser ECU and engine ECU; owner/insurer decision. | Do not disable, bypass, or emulate the immobiliser. Do not present defeat of the immobiliser as a feature. Do not publish a bypass procedure. |
| **EVO-ACD-001** | S0 | 3→4 | ACD/AYC dependence on the stock engine ECU. The 4WD-ECU receives "engine idling information" from the engine ECU and TPS from the sensor; removing the stock ECU may set faults or disable ACD/AYC. | 4WD-ECU pinout; scope the engine-ECU-to-4WD-ECU line; DTC behaviour with the stock ECU disconnected; road test on a closed course. | Do not remove the stock ECU without an ACD/AYC plan. Do not assume the 4WD-ECU is autonomous. Do not test ACD/AYC on public roads. |
| **EVO-ABS-001** | S1 | 3→4 | ABS is a separate safety system that feeds wheel speeds to the 4WD-ECU. | ABS-ECU wiring; confirm no engine-ECU dependency beyond the ABS monitor signal. | Do not cut, splice, or re-route ABS wiring. Do not disable ABS. |
| **EVO-DIAG-001** | S2 | 3 | Factory diagnostics use MUT-II/III over K-line (and a 12-pin connector); OpenTech's protocol is not MUT/OBD compatible. | DLC pinout; K-line scope; MUT-III capability; confirm 12-pin and 16-pin connectors. | Do not connect a non-isolated serial tool to the K-line. Do not assume OpenTech can clear factory DTCs. |
| **EVO-DIAG-002** | S2 | 3 | No CAN bus is documented on Evo VIII. Do not design around CAN for ABS/ACD/AYC. | Measure DLC pins 6 and 14; inspect 4WD-ECU/ABS-ECU wiring; if any CAN pins are populated, capture ACD/AYC and ABS traffic; factory diagram. | Do not assume CAN availability. Do not connect a CAN transceiver to an unknown bus. |

## 3. Gate closure checklist

A gate is not closed until the evidence is recorded in this repository and reviewed. Web pages do
not close a gate.

### Gate 0 → 1 (software → unpowered hardware)

- [ ] Schematic, PCB, BOM and assembly drawing exist and match the reviewed revision.
- [ ] ERC/DRC complete; exceptions reviewed.
- [ ] Visual inspection: correct parts, polarity, solder, no damage.
- [ ] Continuity and rail-to-rail resistance checks; no short to ground or battery.
- [ ] Connector/keying/pinout checked by a second person.
- [ ] Outputs default off with no power applied.

### Gate 1 → 2 (unpowered → powered bench)

- [ ] Current-limited supply, fuses, E-stop, and dummy injector/ignition loads.
- [ ] Firmware image and build hash recorded; recovery method available.
- [ ] Bench test plan reviewed; stop conditions briefed.
- [ ] Every input and output characterised; trigger decoder exercised with recorded vectors.
- [ ] Fault injection: short to ground, short to battery, open circuit on each bench leg.
- [ ] Protection and limp modes exercised; independent review of the bench evidence.

### Gate 2 → 3 (powered bench → vehicle electrical compatibility)

- [ ] EVO-ID-001: ECU label photo and connector count recorded.
- [ ] EVO-ID-002: VIN, model code, build date recorded.
- [ ] EVO-PIN-001: pin-to-pin continuity table complete for every used function.
- [ ] EVO-PIN-002: all ECU connector faces photographed with a scale.
- [ ] EVO-TRIG-001/002/003: crank/cam scope captured and reviewed.
- [ ] EVO-INJ-001/002: injector part number, resistance, resistor pack recorded.
- [ ] EVO-IGN-001/002: coil/igniter type and pairing recorded.
- [ ] EVO-SENS-001/002/003: MAF, temperature, TPS, and lambda data recorded.
- [ ] EVO-PWR-001/002: power, ground, and relay integrity recorded.
- [ ] EVO-ACT-001/002/003: fuel pump, fan, and alternator behaviour recorded.
- [ ] EVO-SEC-001: immobiliser behaviour documented (no bypass).
- [ ] EVO-ACD-001 / EVO-ABS-001: ACD/AYC and ABS interaction assessed.
- [ ] Owner authorization, insurance/legal review, and a non-damaging back-probe plan recorded.
- [ ] Independent vehicle electrical review scheduled.
- [ ] Sources re-checked and dated.

### Gate 3 → 4 (vehicle electrical → controlled engine testing)

- [ ] Key-on test with fuel and spark inhibited: no unintended outputs.
- [ ] Cranking test with injectors and coils disconnected: sync and sensor plausibility.
- [ ] ACD/AYC and immobiliser plan approved by the owner and, where required, the insurer.
- [ ] First start on a closed course or dyno with fire suppression and an observer.
- [ ] Charging, cooling, idle, and ACD/AYC behaviour observed and logged.

### Gate 4 → 5 (engine testing → vehicle operation)

- [ ] Independent review of the calibration, protection limits, and limp modes.
- [ ] Insurance and road-legality review.
- [ ] Documented rollback to the stock ECU.
- [ ] No open S0 or S1 issue.

## 4. What must never happen

These are not negotiable, regardless of schedule pressure:

1. **No pinout by similarity.** A USDM, UK, Evo VII, or Evo IX pin is not a JDM Evo VIII pin.
2. **No immobiliser defeat.** Not by software, not by wiring, not by a "secret" procedure.
3. **No ABS/ACD/AYC deletion** without an explicit safety and insurance review.
4. **No ignition or injector connection** before the drive type and current limit are measured.
5. **No first start** without a fire extinguisher, a second person, and a documented abort plan.
6. **No road test** with an open S0 issue or an unverified trigger offset.
7. **No claim of plug-and-play.** The default is unsupported and unverified.
