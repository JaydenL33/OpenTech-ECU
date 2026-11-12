# Electrical Design Review — Future 4-Cylinder Evo Bench ECU

> ## STATUS: DESIGN REQUIREMENTS, NOT A VALIDATED SCHEMATIC
>
> No manufacturing-ready KiCad PCB, gerber, drill file or assembly drawing exists. An **UNREVIEWED bench-prototype schematic package** exists at `hardware/kicad/evo8-bench-ecu/` (7 hierarchical sheets, BOM, netlist, pin map, test points, ERC JSON reporting 0 violations from the generator). It has **no PCB layout, no DRC, no gerbers, no thermal/EMC validation and no independent review**. `kicad-cli` is not installed in the audit environment, so the ERC report was not independently reproduced. Nothing in this document is a manufacturable design, an electrical safety approval, an EMC claim, or a vehicle-installation instruction.

| Field | Value |
|---|---|
| Document ID | AUD-03 |
| Status | Requirements and review checklist only; not a schematic review |
| Audit date | 2026-11-12 |
| Scope | Future 4-cylinder, port-injection, inductive-ignition bench ECU for a 4G63T Evo VIII development programme |
| Target vehicle context | JDM Evo VIII GSR, CT9A, 4G63T, 6-speed manual. Exact ECU part number, connector, pinout, trigger, injector and ignition topology are **UNVERIFIED** |
| Evidence | Source inspection; no hardware artefacts; see AUD-10 |
| Related | AUD-02, AUD-04, AUD-08, AUD-09, AUD-10 |

> **Do not build from this document.** It defines what a future design must satisfy and how it must be reviewed. It does not define pin assignments, connector part numbers, component values, current ratings, or a safe schematic. Those require the physical vehicle evidence and a reviewed design. The unreviewed `hardware/kicad` schematic may appear to address some rows, but none is reviewed or validated; the requirement statuses remain "Not designed" until reviewed.

## 1. Scope and design intent

The intended future board is a **bench-first** ECU for a 4-cylinder engine:

- four sequential injector low-side outputs (staged/batch future);
- four ignition outputs compatible with the verified coil/igniter topology;
- crank and cam trigger inputs;
- MAP, TPS, CLT, IAT, oil pressure, fuel pressure, battery voltage;
- external CAN wideband input;
- CAN to a tuning PC/dash;
- protected 12 V input, regulated rails, watchdog, debug/recovery;
- current-limited bench bring-up with dummy loads before any engine.

It is **not** a plug-in replacement for the Evo VIII ECU. The OEM connector/pinout is unknown, the factory harness condition is unknown, and the factory ACD/AYC/immobiliser/dashboard interactions are not documented. A future design may use an adapter harness or a new standalone harness; that decision is unresolved.

## 2. What exists and what does not

| Artefact | Exists? | Evidence |
|---|---|---|
| Rust host control code | Yes | `ecu/crates/*`; tests; see AUD-05 |
| Virtual ECU / simulator | Yes | `ecu-simulator`; six scenarios |
| Tuning service/UI | Yes, host-only | `studio-service`; React/Vite + Electron main; Electron launch not validated |
| MCU firmware | Skeleton only | `ecu/firmware/stm32` no_std skeleton; no HAL, timer capture/compare, driver, watchdog integration or flash image |
| KiCad schematic | Yes, **UNREVIEWED** | `hardware/kicad/evo8-bench-ecu/`; 7 sheets; bench prototype only |
| KiCad PCB | No | No `.kicad_pcb`; no placement/routing/DRC |
| BOM | Yes, **UNREVIEWED** | 305 lines; MPNs/sourcing/derating not reviewed |
| ERC report | Yes, **UNREVIEWED** | `reports/erc.json`; 0 violations reported by generator's KiCad 9.0.9; not independently reproduced |
| Gerbers/drill | No | No `.gbr`/`.drl` found |
| Connector/pinout definition | No | `vehicle.rs` marks pinout/connector unverified; `ecu_part_number: None` |
| Harness drawing | No | Not present |
| Enclosure/mechanical | No | Not present |
| EMC/thermal test data | No | Not present |
| Bench fixture | No | Not present; AUD-09 is a plan only |

The retained Speeduino tree contains upstream board definitions (`boards/*.json`, `boards/*.ld`) and reference material. Those are **not** an OpenTech-ECU PCB design and must not be presented as one. They may be studied as behavioral/oracle material subject to licensing.

## 3. Candidate system architecture (not a selection)

```text
                12 V vehicle / bench supply
                         |
                 [fuse] [reverse-polarity] [TVS/transient] [main relay]
                         |
          +--------------+-------------------------------+
          |              |                               |
     [5 V sensor]   [3.3 V MCU]                  [actuator supply]
          |              |                               |
   MAP/TPS/CLT/IAT   MCU (candidate)          injector low-side drivers
   oil/fuel pressure   |  |  |  |              ignition outputs/igniters
   battery sense       |  |  |  +-- CAN transceiver --- CAN wideband / PC
                       |  |  +----- crank/cam conditioning
                       |  +-------- knock conditioning
                       +----------- debug/recovery, watchdog, safe enable

  All actuator outputs must have a hardware default-off state and an
  independent enable path. The MCU must not be the only thing holding
  an injector or coil off.
```

This block diagram is intentionally generic. It does not assign pins, timer channels, connector cavities, or driver part numbers.

## 4. Design requirements

Status is **Not designed** for every row unless stated. "Verification" is the evidence that would be required before the requirement can be marked satisfied. Values marked `TBD-REVIEW` must be set by a documented electrical review; they are not invented here.

### 4.1 Power input and transient protection

| ID | Requirement | Rationale | Verification required | Status |
|---|---|---|---|---|
| EDR-PWR-001 | Input shall accept the defined automotive operating range, including cranking sag, without reset or actuator glitch. Range `TBD-REVIEW`. | Cranking and low battery are normal | Bench supply sweep; scope; reset log | Not designed |
| EDR-PWR-002 | Input shall survive reverse polarity for a defined duration without damage or unsafe output. | Jump-start/wiring error | Reverse-polarity test on protected input | Not designed |
| EDR-PWR-003 | Input shall survive defined load-dump/transient pulses; test standard and levels `TBD-REVIEW`. | Alternator load rejection | Transient generator test; TVS coordination | Not designed |
| EDR-PWR-004 | Input shall be fused or current-limited at a rating coordinated with wiring and connector. | Fire/harness protection | Fuse coordination calculation; fault test | Not designed |
| EDR-PWR-005 | A main relay/ignition-switch input shall define power state and default-off behaviour. | Prevent run-on and unintended actuation | State table; key-off/key-on test | Not designed |
| EDR-PWR-006 | Logic and actuator rails shall be separated or sequenced so MCU reset cannot leave a driver enabled. | Safe state on brownout | Fault injection; rail sequencing scope | Not designed |
| EDR-PWR-007 | Input protection shall not create a hazardous failure mode (e.g., a shorted series device feeding the load). | Single-point failure | FMEA; component failure test | Not designed |
| EDR-PWR-008 | Battery voltage sense shall be calibrated and protected; accuracy target `TBD-REVIEW`. | Dead time, low-voltage protection, charging diagnosis | Calibration vs DMM; divider tolerance | Not designed |
| EDR-PWR-009 | Wideband heater supply, if onboard, shall be separately fused/current-limited and diagnosed. | High cold-start current | Current limit; heater open/short test | Not designed |
| EDR-PWR-010 | Bench bring-up shall use a current-limited supply and a fuse even if the board has protection. | Limit energy during first power | Procedure; photo/log | Not designed |

### 4.2 Regulators and power distribution

| ID | Requirement | Rationale | Verification required | Status |
|---|---|---|---|---|
| EDR-REG-001 | 5 V sensor rail shall be accurate, current-limited and protected against shorts to battery/ground. | Sensor readings and ratiometric sensors | Load sweep; short test; accuracy | Not designed |
| EDR-REG-002 | 3.3 V MCU rail shall meet MCU datasheet tolerance, ripple, startup and transient requirements. | MCU reliability | Datasheet comparison; scope | Not designed |
| EDR-REG-003 | Regulator dropout during cranking shall be characterised; hold-up shall cover defined reset/retention needs. | No reset during crank | Brownout sweep; reset log | Not designed |
| EDR-REG-004 | Regulators shall have thermal and current margins at worst-case ambient and load. | Reliability | Thermal calculation; chamber test | Not designed |
| EDR-REG-005 | Rail sequencing and reset shall put outputs in a defined safe state. | Prevent glitch actuation | Sequencing scope; output state test | Not designed |
| EDR-REG-006 | Analog reference shall be stable and Kelvin-referenced to the ADC/sensor return. | Sensor accuracy | Reference measurement; layout review | Not designed |
| EDR-REG-007 | Regulator fault (short, open, overcurrent) shall not damage the MCU or actuators. | Single fault tolerance | Fault injection | Not designed |

### 4.3 Ground topology

| ID | Requirement | Rationale | Verification required | Status |
|---|---|---|---|---|
| EDR-GND-001 | Power ground, analog ground and digital ground shall be defined with a single-point/star strategy and a documented current return path. | Ground offsets cause sensor errors | Schematic review; ground-current analysis | Not designed |
| EDR-GND-002 | Injector/coil return currents shall not share sensor return paths. | High di/dt corrupts analog readings | Layout review; scope during dummy load | Not designed |
| EDR-GND-003 | Crank/cam shield drain shall be terminated at the ECU according to sensor/vehicle practice; do not create a ground loop. | Trigger noise | Shield test; scope | Not designed |
| EDR-GND-004 | Sensor return shall be referenced to the ECU analog ground, not chassis randomly. | Ratiometric accuracy | Continuity/offset measurement | Not designed |
| EDR-GND-005 | Chassis/engine ground and ECU ground shall be bonded with a defined conductor; ground offset at crank shall be measured. | Starting current causes offsets | Ground-offset test with dummy starter current | Not designed |
| EDR-GND-006 | Ground loss shall not cause actuators to energise or sensors to read unsafe values. | Single-point failure | Ground-lift fault test (protected) | Not designed |

### 4.4 Sensor interfaces and filtering

| ID | Requirement | Rationale | Verification required | Status |
|---|---|---|---|---|
| EDR-SEN-001 | Each analog input shall define input range, impedance, filter, protection and open/short behaviour. | Wrong reading or damage | Schematic review; fault test | Not designed |
| EDR-SEN-002 | MAP, TPS, CLT, IAT, oil and fuel pressure inputs shall be compatible with the verified sensor transfer functions. | Correct engineering units | Sensor datasheet/measurement; calibration | Not designed |
| EDR-SEN-003 | Inputs shall survive short to 5 V, short to ground and battery voltage for a defined duration. | Harness faults | Fault test | Not designed |
| EDR-SEN-004 | ADC reference and input filtering shall meet the required resolution/noise target `TBD-REVIEW`. | Stable control | Noise measurement; FFT | Not designed |
| EDR-SEN-005 | Ratiometric sensors shall be measured against the same reference they use. | Supply variation error | Ratiometric test | Not designed |
| EDR-SEN-006 | Sensor status (open/short/out-of-range/stale) shall be detectable by hardware/firmware. | Fail-safe control | Diagnostic injection | Not designed |
| EDR-SEN-007 | Wideband interface shall use a verified controller protocol (analog or CAN) with timestamping and timeout. | Lambda trust | Controller datasheet; latency test | Not designed |
| EDR-SEN-008 | Battery voltage sense shall be filtered and protected against load-dump residuals. | Wrong dead time/protection | Transient test | Not designed |

### 4.5 Crank/cam trigger conditioning

| ID | Requirement | Rationale | Verification required | Status |
|---|---|---|---|---|
| EDR-TRG-001 | Crank input shall support the **verified** sensor type (VR or Hall), amplitude, polarity and threshold. Type is currently UNVERIFIED. | No sync without correct conditioning | Scope at cranking; sensor datasheet | Not designed |
| EDR-TRG-002 | Cam input shall support the verified cam sensor type and phase pattern. | Sequential phase | Scope crank+cam; pattern capture | Not designed |
| EDR-TRG-003 | Inputs shall have hysteresis/filtering and protection against ringing, ESD and shorts. | Noise causes false/missed teeth | Injector/coil noise test; ESD test | Not designed |
| EDR-TRG-004 | Differential inputs, if used, shall have defined common-mode range and termination. | Noise immunity | Schematic review; common-mode test | Not designed |
| EDR-TRG-005 | Shielded twisted pair and a dedicated sensor return shall be used where required by the sensor/vehicle. | EMI | Harness drawing; scope | Not designed |
| EDR-TRG-006 | The capture path shall preserve tooth timestamp resolution sufficient for the timing budget. | Angle accuracy | Timer analysis; see AUD-04 | Not designed |
| EDR-TRG-007 | A missing/extra tooth, cam loss and sensor short/open shall be detectable. | Fail-safe sync | Fault injection | Not designed |
| EDR-TRG-008 | Do not design to 36-1 or any specific pattern until the Evo pattern is physically verified. | Wrong decoder | Scope evidence; configuration review | **Blocked: pattern unverified** |

### 4.6 Injector low-side drivers

| ID | Requirement | Rationale | Verification required | Status |
|---|---|---|---|---|
| EDR-INJ-001 | Each injector shall have an independent low-side driver with a hardware default-off state. | Prevent unintended fuel | Reset/float test; schematic review | Not designed |
| EDR-INJ-002 | Driver shall be rated for the verified injector resistance/current and flyback energy; values `TBD-REVIEW`. | Driver survival | Injector datasheet; scope; thermal | Not designed |
| EDR-INJ-003 | Flyback/clamp/recirculation shall be defined and validated for the injector inductance. | Avalanche/damage | Scope turn-off; thermal | Not designed |
| EDR-INJ-004 | Peak/hold or PWM shall be used if required by the verified low-impedance injector; not assumed. | Opening time/thermal | Injector characterization | Not designed |
| EDR-INJ-005 | Per-channel open/short/overcurrent diagnostics shall be considered; at minimum, a fuse and test points. | Fault detection | Fault injection | Not designed |
| EDR-INJ-006 | Injector supply shall be fused/limited and separate from logic supply. | Fire/harness protection | Fuse coordination; fault test | Not designed |
| EDR-INJ-007 | Outputs shall remain off during reset, bootloader, debug halt and watchdog reset. | Safe state | Fault injection; reset tests | Not designed |
| EDR-INJ-008 | Connector/terminal current rating and wire gauge shall be defined from measured injector current. | Overheating | Current measurement; thermal | Not designed |
| EDR-INJ-009 | Do not select a driver until injector resistance/inductance and battery voltage range are verified. | Wrong driver | Bench characterization | **Blocked: injector data unverified** |

### 4.7 Ignition outputs and igniter compatibility

| ID | Requirement | Rationale | Verification required | Status |
|---|---|---|---|---|
| EDR-IGN-001 | Output type (logic-level to external igniter vs direct coil primary) shall be selected only after the Evo coil/igniter topology is verified. | Wrong interface can destroy coils/drivers | Vehicle measurement; service data | **Blocked: topology unverified** |
| EDR-IGN-002 | Logic-level outputs shall have defined polarity, pull-down, series resistance and protection consistent with the igniter. | No spark / false spark | Schematic review; bench test | Not designed |
| EDR-IGN-003 | Direct coil drive, if used, shall have a validated driver, clamp and thermal design. | Driver/coil damage | Coil datasheet; scope; thermal | Not designed |
| EDR-IGN-004 | Dwell shall be bounded by coil data and thermal limits, not only by the generic 45% model. | Coil overheating | Coil characterization; thermal | Not designed |
| EDR-IGN-005 | Spark output shall be default-off at reset and during MCU fault. | Uncontrolled spark | Reset/fault test | Not designed |
| EDR-IGN-006 | A spark monitor/test point shall be provided for bench verification. | Timing evidence | Scope point; test procedure | Not designed |
| EDR-IGN-007 | Ignition supply/return shall be separated from sensor returns and protected. | Noise/ground offset | Layout review; scope | Not designed |
| EDR-IGN-008 | The board shall not claim coil compatibility without the verified coil part number and primary resistance/inductance. | Wrong coil | Measurement; datasheet | **Blocked: coil data unverified** |

### 4.8 Knock interface

| ID | Requirement | Rationale | Verification required | Status |
|---|---|---|---|---|
| EDR-KNK-001 | Knock input shall match the verified sensor type (piezoelectric/other), mounting and expected signal range. | No detection or false detection | Sensor measurement; datasheet | Not designed |
| EDR-KNK-002 | Input shall provide bias/protection, band-pass filtering and gain appropriate to the sensor; values `TBD-REVIEW`. | Signal integrity | Bench signal injection; scope | Not designed |
| EDR-KNK-003 | Knock windowing shall be synchronized to crank angle with enough resolution. | Cylinder/event attribution | Firmware/timer design; scope | Not designed |
| EDR-KNK-004 | Shielded routing and a dedicated return shall be used; avoid injector/coil noise. | False knock | Noise test with dummy loads | Not designed |
| EDR-KNK-005 | Knock threshold calibration shall be engine-specific and validated with controlled excitation/audible review. | Avoid damage or over-retard | Engine/dyno test | Not designed |
| EDR-KNK-006 | Knock input failure (open/short) shall be detectable and fail-safe. | No protection | Fault injection | Not designed |

### 4.9 CAN and communication

| ID | Requirement | Rationale | Verification required | Status |
|---|---|---|---|---|
| EDR-CAN-001 | CAN transceiver shall meet the selected bus physical layer; bit rate and topology `TBD-REVIEW`. | Communication reliability | Schematic review; bus test | Not designed |
| EDR-CAN-002 | Bus termination (split/standard) and stub length shall be defined. | Reflections | Scope/eye diagram | Not designed |
| EDR-CAN-003 | CAN lines shall have ESD/transient protection and common-mode considerations. | Automotive transient | Transient test | Not designed |
| EDR-CAN-004 | Wideband CAN messages shall be timestamped and timeout-checked; no invented OEM frames. | Lambda trust | Protocol spec; fault test | Not designed |
| EDR-CAN-005 | CAN loss shall not leave actuators in an unsafe state. | Safe state | Fault injection | Not designed |
| EDR-CAN-006 | If isolation is required, it shall be specified and tested; do not assume. | Ground loops | Design review; isolation test | Not designed |
| EDR-CAN-007 | No CAN frame shall be transmitted that is not documented and reviewed; spoofing factory ACD/AYC/dashboard is prohibited without evidence. | Vehicle network safety | CAN capture; review | Not designed |

### 4.10 Connectors and harness interface

| ID | Requirement | Rationale | Verification required | Status |
|---|---|---|---|---|
| EDR-CON-001 | Connector strategy shall be selected from verified options: OEM mating connector, adapter harness, or new sealed harness. | No plug-and-play claim | Physical inspection; service data | **Blocked: connector unknown** |
| EDR-CON-002 | Terminal/housing current rating, sealing, keying and mating cycles shall be defined. | Reliability/safety | Datasheet; mating test | Not designed |
| EDR-CON-003 | Pin assignment shall be verified against the physical vehicle with key-off continuity and a second-person check. | Wrong wiring destroys hardware | Pinout procedure; photos | **Blocked: pinout unverified** |
| EDR-CON-004 | No pinout shall be published or fabricated from a forum post or a similar model. | Safety/legal | Authoritative service data | Not designed |
| EDR-CON-005 | Strain relief, service loop and labeling shall be defined. | Vibration/service | Mechanical review | Not designed |
| EDR-CON-006 | The OEM harness shall not be cut or back-probed without an approved procedure and owner authorization. | Vehicle damage | Procedure; review | Not designed |

### 4.11 Safe defaults, watchdog, debug and recovery

| ID | Requirement | Rationale | Verification required | Status |
|---|---|---|---|---|
| EDR-SAF-001 | All actuator outputs shall be hardware default-off at power-on, reset, brownout, watchdog reset and debug halt. | Primary safe state | Fault injection; reset tests | Not designed |
| EDR-SAF-002 | An external window watchdog shall monitor the MCU; timeout and reset behaviour shall be defined. | Detect hang | Datasheet; fault injection | Not designed |
| EDR-SAF-003 | A hardware enable/interlock shall be independent of the MCU for actuator power. | MCU cannot be sole guard | Schematic review; fault test | Not designed |
| EDR-SAF-004 | Reset cause, watchdog count and fault state shall be logged and readable after reset. | Diagnosis | Firmware/log test | Not designed |
| EDR-SAF-005 | Debug access shall be protected/locked in production; recovery shall require authentication. | Prevent tampering | Security review | Not designed |
| EDR-SAF-006 | Bootloader/recovery shall not permit unauthenticated firmware or calibration writes. | Cybersecurity | Threat model; test | Not designed |
| EDR-SAF-007 | Safe state on loss of crank/cam, sensor validity, CAN, or power shall be defined per output. | Fail-safe behaviour | Fault injection | Not designed |
| EDR-SAF-008 | The main relay/fuel pump output shall default off and have a hardware timeout. | Fire/fuel containment | Fault test | Not designed |

### 4.12 Thermal, EMC and mechanical

| ID | Requirement | Rationale | Verification required | Status |
|---|---|---|---|---|
| EDR-THM-001 | Driver/regulator thermal dissipation shall be calculated at worst-case ambient, load and duty; junction temperatures shall have margin. | Reliability | Calculation; chamber test | Not designed |
| EDR-THM-002 | Thermal shutdown, if used, shall fail safe and be diagnosed. | Driver protection | Datasheet; fault test | Not designed |
| EDR-THM-003 | Copper area, thermal vias and enclosure airflow/conductivity shall be defined. | Heat removal | Thermal simulation/test | Not designed |
| EDR-EMC-001 | Input/output filtering, TVS, ferrites and layout shall target defined emissions/immunity levels; levels `TBD-REVIEW`. | Automotive EMC | Pre-compliance scan | Not designed |
| EDR-EMC-002 | High di/dt loops (injector/coil) shall be minimized; sensitive analog/trigger traces shall be separated. | Noise | Layout review; scope | Not designed |
| EDR-EMC-003 | EMC compliance shall not be claimed without test data. | Legal/safety | Accredited test report | Not designed |
| EDR-MEC-001 | Enclosure, mounting, connector access and vibration resistance shall be defined for the bench and, later, vehicle environment. | Reliability/service | Mechanical drawing; vibration test | Not designed |
| EDR-MEC-002 | The board shall be serviceable with test points and labelled connectors. | Safe bring-up | Inspection | Not designed |
| EDR-MEC-003 | IP rating and chemical resistance shall be defined if used outside a dry bench. | Environment | Test report | Not designed |

### 4.13 Bench breakout requirements

| ID | Requirement | Rationale | Verification required | Status |
|---|---|---|---|---|
| EDR-BRK-001 | A breakout/test board shall expose every output/input with labelled test points and current-limited dummy loads. | Safe measurement | Schematic; inspection | Not designed |
| EDR-BRK-002 | Dummy injector loads shall emulate resistance/inductance without fuel; dummy ignition loads shall be shielded. | Safe bench test | Load design; thermal | Not designed |
| EDR-BRK-003 | An emergency stop shall remove actuator power, not just assert a software signal. | Human safety | E-stop test | Not designed |
| EDR-BRK-004 | The bench supply shall be current-limited and fused; no vehicle battery without review. | Limit energy | Procedure; inspection | Not designed |
| EDR-BRK-005 | Scope points shall be rated for the measured voltage/transient and isolated as required. | Measurement safety | Probe review | Not designed |

## 5. Candidate architecture trade study inputs

This is **not a selection**. It lists what must be compared and verified before a schematic. No unsupported HAL or vendor claim is made.

### 5.1 MCU candidates

| Criterion | Why it matters | STM32H7 candidate | S32K candidate | Evidence required |
|---|---|---|---|---|
| Rust HAL maturity for exact part | A port that compiles for one family may not support the exact timer/ADC/CAN | Verify `embassy-stm32` or other HAL support for the chosen part, timers, ADC, CAN, watchdog | Verify Rust support for exact S32K part and peripherals | Buildable blinky + timer + CAN on a dev board |
| Timer capture/compare channels | Crank/cam capture and 4 injector + 4 ignition outputs | Count and resolution must be verified; not all timers are equal | eMIOS/timer resources must be verified | Timer allocation table; scope |
| ADC channels/resolution | MAP/TPS/CLT/IAT/oil/fuel/battery/knock | Verify channel count, speed, reference | Verify channel count, speed, reference | Noise/resolution measurement |
| CAN controllers | Wideband, PC, dash | Verify classic CAN/CAN-FD support as required | Verify | Bus test |
| Watchdog | External/internal window watchdog | Verify independent watchdog and reset behaviour | Verify | Fault injection |
| Safety/qualification | Automotive environment | Industrial/automotive variants differ; verify | Automotive family; exact part qualification | Datasheet/qualification report |
| Package/thermal | Board size and heat | Verify | Verify | Thermal calculation |
| Debug/trace | Bring-up and timing measurement | Verify SWD/trace | Verify | Debug probe test |
| Long-term supply | Production risk | Verify lifecycle | Verify | Vendor lifecycle statement |
| Toolchain/licence | Reproducible build | Verify | Verify | Build log/licence |
| Cost/availability | Budget | Quote | Quote | Purchasing record |

**Decision rule:** no MCU is selected until a dev board proves the exact timer capture/compare, ADC, CAN, watchdog and Rust build. A successful host `cargo check` says nothing about MCU support.

### 5.2 Output stage options

| Option | Use | Pros | Risks | Evidence required |
|---|---|---|---|---|
| Smart low-side switch with diagnostics | Injectors | Integrated protection/current sense | Cost; accuracy; flyback rating | Datasheet; fault test |
| Discrete MOSFET + gate driver | Injectors/coils | Flexible | Flyback/thermal design; EMI | Schematic review; scope |
| Peak/hold driver | Low-Z injectors | Fast opening | Complexity/thermal | Injector characterization |
| Logic-level output to OEM igniter | Ignition | Minimal high-current on ECU | Requires verified igniter polarity/levels | Vehicle measurement |
| Smart coil direct drive | Ignition | Integrated coil | Coil-specific dwell/current | Coil datasheet |
| Relay/high-side for pump/fan | Aux | Simple | Default state/feedback | Fault test |

No option is selected. The Evo injector resistance, coil/igniter topology and fuel pump control are unverified.

### 5.3 Connector strategy options

| Option | Description | Pros | Cons/risks | Prerequisite |
|---|---|---|---|---|
| OEM mating connector | Build a board that mates the factory ECU connector | No harness cut; reversible | Exact housing/terminal/keying unknown; pinout unknown; limited current/space | ECU part number, connector markings, pinout, terminal series |
| Adapter harness | Factory connector to new ECU via adapter | Reversible; bench/vehicle transition | Contact resistance, strain, sealing; still needs pinout | Verified connector and pinout; harness drawing |
| Breakout board | Bench-only terminal block | Easy probing; safe | Not for vehicle; no sealing/vibration | Bench plan |
| New sealed harness | Standalone ECU harness | Full control; new sensors | Labor; vehicle modification; ACD/AYC/dash integration | Full electrical design and vehicle review |
| Bare wire/pigtail | Not acceptable for vehicle | Cheap | Unsealed, unreliable, dangerous | Prohibited |

## 6. Electrical review checklist

A reviewer should require the evidence in the right column before signing. "Pass" means the evidence exists and is reviewed, not that the reviewer likes the design.

| # | Check | Evidence required | Pass condition |
|---|---|---|---|
| 1 | Requirements traceable to a schematic | Requirement-to-net/component table | Every EDR requirement mapped or explicitly deferred |
| 2 | Power input protection | Schematic, TVS/fuse calculation, transient test plan | Reverse polarity/load dump/cranking defined |
| 3 | Regulator worst case | Datasheet calculations, thermal, dropout | Margins at min/max battery and max load |
| 4 | Ground topology | Ground diagram, current-return analysis | Injector/coil currents isolated from sensor returns |
| 5 | Sensor input protection | Schematic, fault test | Short to 5 V/ground/battery survives |
| 6 | Trigger conditioning | Scope at cranking, sensor datasheet | Clean edges across RPM/temperature; no false teeth |
| 7 | Injector driver stress | Injector characterization, flyback scope, thermal | Driver within SOA at max current/duty/temperature |
| 8 | Ignition interface | Verified coil/igniter topology, scope | Correct polarity/level/dwell; no false spark |
| 9 | Knock interface | Sensor data, filter/gain design, noise test | Signal usable; no false trigger from injector/coil |
| 10 | CAN physical layer | Schematic, termination, scope | Bus stable; faults contained |
| 11 | Connector/pinout | Physical inspection, continuity, second-person check | No unverified pin; no shared hazardous function |
| 12 | Safe defaults | Reset/brownout/watchdog/debug fault test | All outputs off; no glitch |
| 13 | Watchdog | External watchdog datasheet, fault injection | Reset on hang; safe outputs; reset cause logged |
| 14 | Debug/recovery | Security review, bootloader test | No unauthenticated write; recovery works |
| 15 | Thermal | Calculation/chamber test | Junction/ambient margins defined |
| 16 | EMC | Pre-compliance plan/report | Levels defined; no claim without data |
| 17 | Mechanical | Drawing, vibration/IP plan | Connector access, mounting, serviceability |
| 18 | BOM/derating | BOM with tolerance/derating | No unresolved NRND/obsolete parts; margins |
| 19 | Testability | Test point/breakout drawing | Every rail/output/input measurable safely |
| 20 | Fault injection | Test plan/results | Open/short/reverse/brownout/watchdog/sensor faults safe |
| 21 | Independent review | Signed record | Reviewer independent of designer; findings closed |
| 22 | Configuration control | Schematic/PCB/BOM revision hashes | Design matches reviewed revision |
| 23 | Evo evidence | Pinout/connector/trigger/injector/coil records | All Gate 3 items verified |
| 24 | No vehicle claim | Document review | No plug-and-play/EMC/qualified claim |

## 7. What KiCad ERC and DRC prove — and do not prove

| Check | Proves | Does not prove |
|---|---|---|
| ERC (electrical rules) | Some connectivity errors, unconnected pins, power-input conflicts, some pin-type conflicts | Component stress, thermal, EMC, transient survival, safety, firmware, harness, worst-case tolerance |
| DRC (design rules) | Copper spacing/width, annular rings, solder mask, silkscreen, courtyard, some differential/clearance rules | Dielectric withstand, creepage/clearance for mains/automotive transients, current capacity, thermal, vibration, assembly |
| Green ERC/DRC | The tool's rule set is satisfied for the drawn geometry | The circuit works, is safe, is manufacturable, or is automotive-qualified |

A design can pass ERC/DRC and still:
- overheat a driver at rated current;
- have no flyback path;
- brown out during cranking;
- oscillate because of a bad ground;
- fail EMC;
- have a connector that cannot carry the current;
- leave an output enabled on reset;
- be impossible to assemble or service.

The unreviewed package has an ERC JSON reporting 0 violations from the generator's KiCad 9.0.9, but it was not independently reproduced in this audit. No DRC exists because there is no PCB. Any future report must state the KiCad version, rule set, exclusions, and unresolved violations; a green status alone is not evidence.

## 8. Open electrical issues / blockers

| ID | Issue | Impact | Required action |
|---|---|---|---|
| ELEC-01 | Unreviewed schematic; no PCB/DRC/gerbers | Cannot build or review hardware | Review schematic/nets; create PCB; DRC; DFM. The unreviewed Codex package does not satisfy Gate 1 |
| ELEC-02 | Exact Evo ECU part number unknown | Connector and pinout cannot be selected | Photograph label/connectors; obtain service data |
| ELEC-03 | Pinout/connector unverified | Wrong wiring can destroy ECU/vehicle | Key-off continuity and second-person check; no forum guesses |
| ELEC-04 | Trigger pattern unverified | Wrong decoder/conditioning | Scope crank/cam at cranking; record pattern |
| ELEC-05 | Injector data unverified | Driver/fuel model wrong | Measure resistance/inductance; flow/dead-time bench |
| ELEC-06 | Coil/igniter topology unverified | Output stage may be incompatible | Trace/measure; service data |
| ELEC-07 | No watchdog/safe-state design | Hang can leave outputs active | External watchdog; hardware default-off; fault injection |
| ELEC-08 | No power/transient design | Brownout/load dump damage | Input protection requirements; test plan |
| ELEC-09 | No thermal/EMC design | Reliability/compliance unknown | Calculations, chamber, pre-compliance |
| ELEC-10 | No bench fixture/E-stop | Unsafe bring-up | Build AUD-09 fixture before powered test |
| ELEC-11 | No independent review | Design unvalidated | Schedule independent electrical review |
| ELEC-12 | No MCU/HAL proof | Firmware plan may be infeasible | Dev-board proof for exact part/peripherals |

## 9. Explicit non-claims

- This is not a reviewed schematic, PCB, BOM, gerber set, assembly drawing, or manufacturing package.
- The unreviewed schematic's ERC report (0 violations) has not been independently reproduced; no DRC exists; no electrical rule compliance is claimed.
- No component values, current ratings, fusing, thermal margins, creepage/clearance or EMC performance are validated.
- No connector pinout, ECU part number, harness drawing or plug-and-play compatibility is claimed.
- No automotive qualification (AEC-Q, ISO 16750, ISO 7637, ISO 11452, CISPR 25, etc.) is claimed.
- No functional safety (ISO 26262/IEC 61508) or freedom from interference is claimed.
- No engine or vehicle readiness is claimed. The design is bench-first and blocked until Gate 3 evidence exists.

## 10. Exit criteria to a real schematic review

1. Evo VIII ECU part number, connector, pinout, trigger, injector, coil/igniter and harness evidence recorded and independently checked.
2. MCU selected only after a dev board proves the required timers/ADC/CAN/watchdog and Rust build.
3. Requirements in this document assigned values (`TBD-REVIEW` closed) and traced to schematic nets/components.
4. Power/transient, ground, driver, ignition, knock, CAN, connector and safe-state designs reviewed by an independent electrical engineer.
5. KiCad ERC/DRC run with a documented rule set and all violations resolved or explicitly accepted with rationale.
6. BOM with derating, lifecycle and thermal analysis; assembly/manufacturing review.
7. Bench breakout, current-limited supply, fuses, dummy loads and E-stop ready per AUD-09.
8. No claim of EMC, automotive qualification, or vehicle readiness until test evidence exists.
