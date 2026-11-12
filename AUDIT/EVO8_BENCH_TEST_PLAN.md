# Evo VIII Bench Test Plan (hardware-agnostic, NOT YET EXECUTED)

> ## STATUS: NOT YET EXECUTED
>
> This is a plan for a future bench test. No test in this document has been run. Only an unreviewed, unrouted PCB draft exists; no built ECU, harness, bench fixture, firmware image, or Evo VIII vehicle is available in the audited snapshot. Every pass/fail threshold marked `TBD-REVIEW` is a placeholder that requires an electrical/engine review before use.

| Field | Value |
|---|---|
| Document ID | AUD-09 |
| Status | **Plan only; NOT YET EXECUTED** |
| Date | 2026-11-12 |
| Scope | Future powered-bench test of a 4-cylinder Evo/4G63T ECU, using simulated crank/cam, dummy injector/ignition loads, current-limited supply, no fuel, no engine, no vehicle |
| Gate | Gate 2 (powered bench verification) after Gate 1 unpowered verification |
| Hardware assumption | Hardware-agnostic until a reviewed PCB exists. Do not assign pins/connectors from this document |
| Related | AUD-02, AUD-03, AUD-04, AUD-08, AUD-10 |

## 1. Scope and assumptions

- Target engine family: 4G63T inline-4, sequential port injection, inductive ignition. Exact Evo VIII trigger pattern, injector data, coil/igniter topology and pinout are **UNVERIFIED**.
- The DUT (device under test) may be a future OpenTech-ECU board or a vendor development board with a reviewed adapter. This plan does not define its pinout. The unreviewed `hardware/kicad` schematic is not a buildable DUT until Gate 1 review; do not power it from this plan.
- Crank/cam signals are simulated by a verified signal generator or a recorded waveform player. No engine.
- Injectors are replaced by dummy resistive/inductive loads. Ignition is observed at the driver/igniter input or a shielded dummy coil/spark gap; no fuel.
- The DUT is powered from a current-limited supply with a fuse. No vehicle battery, no vehicle harness, no alternator.
- All tests are performed with an independent E-stop that removes actuator power.
- A second person is recommended for first power and required for any high-energy/ignition test per local policy.

## 2. Safety rules (mandatory)

| Rule | Requirement |
|---|---|
| No vehicle | Do not connect the DUT to the Evo VIII or any vehicle harness. No road/track test. |
| No fuel | Do not connect fuel lines, fuel pump, or real injectors with fuel. Use dummy loads only. |
| Current limit | Set the supply current limit to the lowest value that supports the test; start below the expected idle current. |
| Fusing | Use a fuse coordinated with wiring/connector rating; do not rely on the supply limit alone. |
| E-stop | A physical E-stop must remove actuator power independently of software. Test it before powering actuators. |
| Isolation | Use an isolation transformer/isolated supply where required by the scope/probe setup. Do not create a ground loop through the scope. |
| PPE | Safety glasses; no jewelry; insulated tools; fire extinguisher appropriate for electrical/flammable fires. |
| High voltage | Ignition coil primary flyback and spark can be hazardous; use shielded/dummy loads and keep clear. |
| Wideband heater | If a wideband controller is connected, treat the heater as a high-current load; fuse/current-limit and allow cool-down. |
| Thermal | Monitor driver/regulator temperatures; stop if anything exceeds the reviewed limit or smells hot. |
| Two-person | First power, ignition tests, and fault injection should have a second person who can cut power. |
| Lockout | Tag/lock the supply when changing wiring. |
| No improvisation | Do not substitute a pinout, connector, driver, or load without a reviewed change. |

## 3. Equipment

| Item | Purpose | Notes |
|---|---|---|
| Current-limited DC supply | DUT power | Adjustable 0-20 V class; current limit and readback |
| Fuse holder + fuses | Input protection | Coordinated with wiring; start low |
| Emergency stop | Remove actuator power | Physical, latching, tested |
| DMM | Continuity/voltage/current | Calibrated |
| Oscilloscope | Timing/waveforms | 2-4 channels; adequate bandwidth; differential/isolated probes as needed |
| Logic analyser | Crank/cam/injector/ignition/CAN timing | 8+ channels, timestamping |
| Signal generator / trigger player | Simulated crank/cam | Verified waveform; RPM sweep |
| Dummy injector loads | Injector driver test | Resistive/inductive, rated for expected current; no fuel |
| Dummy ignition load | Ignition output test | Shielded coil/spark gap or equivalent; observe primary/igniter input |
| CAN analyser/interface | Bus test | If CAN is implemented |
| Thermocouples/thermal camera | Thermal | Driver/regulator/connector |
| Fire extinguisher | Emergency | Electrical/flammable rated |
| Insulated tools/covers | Safety | Prevent shorts |
| Test record | Evidence | See §10 |

## 4. Bench setup (text diagram)

```text
Mains / isolated supply
  -> current-limited DC supply -> fuse -> E-stop -> DUT input
  -> DUT logic/sensor rails (measure at test points)
  -> DUT injector outputs -> dummy injector loads -> return
  -> DUT ignition outputs -> shielded dummy ignition load -> return
  -> trigger generator -> DUT crank/cam inputs
  -> CAN analyser <-> DUT CAN (if present)
  -> scope/logic analyser probes on labelled test points
  -> no fuel, no engine, no vehicle
```

All connections are made with the supply off and the E-stop engaged. Verify polarity and continuity before power.

## 5. Pre-test checklist (all must pass before BT-01)

| # | Check | Pass condition | Evidence |
|---:|---|---|---|
| P-01 | DUT identity/revision | Board serial and design revision recorded | Photo/label |
| P-02 | Visual inspection | No damage, contamination, loose parts, wrong polarity | Photo/checklist |
| P-03 | Continuity: rails to ground | No short below reviewed threshold | DMM record |
| P-04 | Fuse rating | Coordinated with wiring/connector; installed | Photo/value |
| P-05 | Supply polarity | Correct; reverse-protection verified | DMM/photo |
| P-06 | E-stop | Removes actuator power; latched; tested | Test record |
| P-07 | Dummy loads | Correct type/rating; isolated; no fuel | Inspection |
| P-08 | Scope/probe rating | Adequate for voltage/transient; isolated as needed | Probe labels |
| P-09 | Firmware/calibration | Known hash/checksum; skeleton or reviewed image | Hash/checksum record |
| P-10 | Safe-state expectation | All outputs off before power; reviewed | Schematic/test point |
| P-11 | Fire safety | Extinguisher/first aid/clear area | Inspection |
| P-12 | Two-person briefing | Roles, stop conditions, E-stop location | Briefing record |

If any item fails, **stop**. Do not power the DUT.

## 6. Test steps

### Phase A — Unpowered / first power

| ID | Purpose | Setup / procedure | Pass criteria | Fail / stop condition | Evidence |
|---|---|---|---|---|---|
| BT-01 | Verify no output is energised with supply off | Measure each output/test point to ground; inspect dummy loads | All outputs de-energised; no continuity to supply | Any output energised or short | DMM/photo |
| BT-02 | First power, no actuators enabled | Set current limit low; apply supply; measure rails | Current below reviewed limit; rails within tolerance; no smell/heat | Overcurrent, rail out of tolerance, heat | Scope/DMM/log |
| BT-03 | Reset/safe state | Assert reset/brownout; measure outputs | All outputs off during and after reset | Any output glitch/active | Scope |
| BT-04 | E-stop function | Press E-stop while outputs enabled (dummy load) | Actuator power removed; DUT may remain logic-powered | Output remains energised | Scope/photo |
| BT-05 | Supply sweep | Sweep defined battery range `TBD-REVIEW` | Rails stable; no reset/glitch; safe state | Reset/actuation glitch | Scope/log |

### Phase B — Trigger and sync

| ID | Purpose | Setup / procedure | Pass criteria | Fail / stop condition | Evidence |
|---|---|---|---|---|---|
| BT-06 | Crank sync at low RPM | Feed verified crank pattern at defined low RPM | Decoder reaches expected sync; no false tooth | No sync or false sync | Logic analyser |
| BT-07 | RPM sweep | Sweep 600-9000 rpm (or reviewed range) | Sync maintained; tooth intervals plausible | Sync loss, false gap, missed tooth | Logic analyser |
| BT-08 | Cam phase | Add verified cam pulse | Sequential state only after valid cam; phase correct | Sequential without cam or wrong phase | Logic analyser |
| BT-09 | Gap mismatch | Inject missing/extra tooth | Sync loss/fail-closed; no wrong-angle output | Output event after fault | Logic analyser |
| BT-10 | Signal loss | Stop crank signal | Sync loss and fuel/spark cut within reviewed timeout | Output continues beyond limit | Scope/logic analyser |
| BT-11 | Noise/EMI | Inject defined noise/edge jitter | No false sync/output; filter/hysteresis works | Repeated false events | Scope |
| BT-12 | Cam loss | Remove cam after sync | Detect/degrade/cut; no wrong phase | Stays sequential with wrong phase | Logic analyser |

### Phase C — Injector outputs

| ID | Purpose | Setup / procedure | Pass criteria | Fail / stop condition | Evidence |
|---|---|---|---|---|---|
| BT-13 | Injector pulse width | Command defined pulses at several RPM/load points | Pulse width within reviewed tolerance; no double pulse | Wrong/missing/extra pulse | Current probe/scope |
| BT-14 | Injector timing | Compare command angle to output | Within timing budget `TBD-REVIEW` | Beyond budget | Logic analyser |
| BT-15 | Duty sweep | Sweep duty to reviewed limit | Driver temperature/current within SOA; cut at limit | Overcurrent/overheat | Current probe/thermal |
| BT-16 | Flyback | Observe turn-off with dummy inductive load | Clamp within driver rating; no ringing/avalanche | Overvoltage/heat | Scope |
| BT-17 | Open/short load | Open and short one dummy load (protected) | Driver detects/protects; no damage; safe state | Damage/unsafe output | Fault record |
| BT-18 | Output-off on fault | Force sync/protection fault | All injector outputs off | Any output active | Scope |

### Phase D — Ignition outputs

| ID | Purpose | Setup / procedure | Pass criteria | Fail / stop condition | Evidence |
|---|---|---|---|---|---|
| BT-19 | Dwell | Command defined dwell across RPM/voltage | Dwell within reviewed tolerance; no overlap | Over-dwell/coil heat | Scope |
| BT-20 | Spark timing | Compare command to output/igniter signal | Within timing budget `TBD-REVIEW` | Beyond budget | Logic analyser |
| BT-21 | Rev limiter | Increase RPM to limit | Cut/limit within reviewed overshoot | Uncontrolled overspeed | Scope/log |
| BT-22 | Spark cut | Activate protection/limiter | No spark output while cut | Spark continues | Scope |
| BT-23 | Coil/igniter interface | With verified dummy load, check polarity/level | Correct polarity/level; no false trigger | Wrong polarity/false spark | Scope |

### Phase E — Protection and sensors

| ID | Purpose | Setup / procedure | Pass criteria | Fail / stop condition | Evidence |
|---|---|---|---|---|---|
| BT-24 | Overtemp | Inject coolant signal above threshold | Derate/cut per reviewed strategy; latch; log | No response or unsafe continue | Log/scope |
| BT-25 | Low oil/fuel pressure | Inject low pressure signals | Cut/derate per strategy; latch where required | No response | Log |
| BT-26 | Lean/overboost/knock | Inject reviewed fault signals | Correct reaction; learning inhibited; no ignition advance from lambda | Wrong reaction/advance | Log/scope |
| BT-27 | Sensor open/short | Open/short each sensor input (protected) | Detect/default safe; no damage | Wrong control/damage | Fault record |
| BT-28 | Watchdog/hang | Simulate hang or stop servicing watchdog | Reset within reviewed time; outputs off; reset cause logged | No reset/output active | Scope/log |
| BT-29 | Brownout | Sweep supply down/up | Safe reset; outputs off; no glitch | Actuation glitch/corruption | Scope/log |
| BT-30 | Calibration corruption | Corrupt calibration/checksum in a test image | Reject/refuse; safe state | Accepts corrupted data | Log |

### Phase F — CAN/UI isolation and thermal

| ID | Purpose | Setup / procedure | Pass criteria | Fail / stop condition | Evidence |
|---|---|---|---|---|---|
| BT-31 | CAN bus | Connect analyser; monitor/telemetry | Bus stable; no invented frames; timeout safe | Bus errors/unsafe state | CAN log |
| BT-32 | UI load | Run tuning UI/telemetry during control | No missed control events; bounded latency | Timing disturbance | Logic analyser |
| BT-33 | Thermal soak | Run defined duty/ambient `TBD-REVIEW` | Temperatures within reviewed limits; no derating surprise | Over-temperature | Thermocouple |
| BT-34 | EMC pre-scan (optional) | If claiming EMC, run pre-compliance | Reviewed limits | Failure | Test report |

## 7. Pass/fail and stop conditions

**Global stop conditions** — cut actuator power and stop if any occurs:

- Any output energises when it should be off.
- Any spark/injection event after a detected sync/protection fault.
- Smoke, smell, discoloration, or temperature above the reviewed limit.
- Supply overcurrent or fuse open.
- E-stop fails to remove actuator power.
- Scope/probe shows a transient beyond the reviewed rating.
- Any unexpected fuel, fire, or personnel hazard.

**Pass** requires all applicable steps to meet their reviewed criteria, with raw evidence and no unresolved anomaly. A "pass with caveat" is not a pass for Gate 3; record it as conditional and define the evidence.

## 8. Emergency stop procedure

1. Press the physical E-stop (removes actuator power).
2. Turn off the current-limited supply.
3. Disconnect the DUT from the supply if safe.
4. Do not touch hot components, coils, or damaged wiring.
5. If fire: use the appropriate extinguisher; evacuate if needed; call emergency services.
6. If injury: first aid/emergency services; do not move a person near high voltage.
7. Record the event, state, and evidence; do not re-power until the cause is reviewed.

## 9. No-vehicle rules

- Never connect the DUT to the Evo VIII or any vehicle harness in this plan.
- Never energise a fuel pump or real injector with fuel.
- Never start an engine or perform a road/track test.
- Never bypass the immobiliser, ABS, ACD/AYC, or factory safety systems.
- Never use an OEM connector pinout that has not been verified.
- Never leave the bench powered unattended.

## 10. Acceptance thresholds (placeholders)

| Parameter | Placeholder | Required review |
|---|---|---|
| Supply current limit | `TBD-REVIEW` | Based on DUT design/BOM |
| Rail tolerances | `TBD-REVIEW` | MCU/sensor datasheets |
| Spark timing jitter | `TBD-REVIEW` | Engine/knock/coil requirements; AUD-04 |
| Injector pulse tolerance | `TBD-REVIEW` | Injector/driver data |
| Dwell tolerance | `TBD-REVIEW` | Coil datasheet |
| Watchdog reset latency | `TBD-REVIEW` | Safety review |
| Thermal limits | `TBD-REVIEW` | Component/ambient |
| CAN bus load/latency | `TBD-REVIEW` | Protocol/wideband requirements |

Do not fill these with guessed values. A number without a source is not an acceptance criterion.

## 11. Test record template

| Field | Entry |
|---|---|
| Test ID / date | |
| DUT serial / hardware revision | |
| Firmware hash / calibration checksum | |
| Supply voltage/current limit / fuse | |
| Ambient temperature | |
| Probe/scope setup | |
| Trigger pattern/RPM | |
| Result | PASS / FAIL / CONDITIONAL |
| Evidence files | |
| Anomalies/findings | |
| Operator/reviewer | |
| E-stop tested? | Yes/No |

## 12. Exit criteria to seek Gate 3

- All applicable BT steps pass with raw evidence.
- No unresolved anomaly, over-temperature, overcurrent, or unsafe output.
- Watchdog/brownout/safe-state tests pass.
- Timing evidence meets reviewed thresholds.
- Independent bench review signed.
- Evo pinout/trigger/injector/coil evidence is still required before Gate 3; this bench plan cannot supply it.

## 13. Explicit non-claims

- **NOT YET EXECUTED.** No result in this document is measured.
- Only an unreviewed, unrouted PCB draft exists; no built ECU, firmware image, harness, bench fixture, or Evo vehicle exists in the audited snapshot.
- No pinout, connector, current rating, timing threshold, or thermal limit is defined here.
- No electrical safety, EMC, automotive qualification, engine readiness, or vehicle readiness is claimed.
- A passed bench test would be Gate 2 evidence only, not permission for Gate 3-5.
