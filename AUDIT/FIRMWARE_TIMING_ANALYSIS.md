# Firmware and Timing Analysis (v0.1 host simulation + no_std skeleton)

> ## STATUS: NO MCU TIMING EVIDENCE (no_std SKELETON ONLY)
>
> A `no_std` firmware skeleton exists at `ecu/firmware/stm32` and passes a `thumbv7em-none-eabihf` check, but there is no STM32/S32K HAL, timer configuration, capture/compare implementation, logic-analyser capture, or oscilloscope evidence. The v0.1 "ECU" is still a host-side 1 ms simulation. Every timing table below is a **budget or requirement**, not a measurement. Acceptance thresholds marked `TBD-REVIEW` are placeholders that require an engine-specific review.

| Field | Value |
|---|---|
| Document ID | AUD-04 |
| Status | Timing concept and measurement plan; not a timing validation |
| Audit date | 2026-11-12 |
| Scope | Future firmware timing chain for a 4-cylinder, sequential port-injection, inductive-ignition ECU; current no_std skeleton |
| Target context | Evo VIII 4G63T; trigger pattern, ECU pinout and coil topology **UNVERIFIED** |
| Evidence | Host simulation code/tests; `opentech-firmware-stm32` target check; no hardware/timing measurement |
| Related | AUD-02, AUD-03, AUD-08, AUD-09 |

## 1. What v0.1 actually does

| Property | v0.1 implementation | Consequence |
|---|---|---|
| Simulation step | `DT_S = 0.001` in `ecu-simulator/src/lib.rs` (`step_once`) | 1 ms model step; not a hard real-time deadline |
| Time source | Logical `time_us` incremented by 1000 per step | Deterministic simulation time, not MCU timer time |
| Trigger | Synthetic tooth events generated from `true_angle_deg` and `true_rpm` | Does not exercise capture latency, noise, or timer resolution |
| Scheduler | Angle-domain events computed by `ecu-scheduler` | Correct conceptual model; not executed by hardware timers |
| Outputs | `apply_events` updates model state only | No GPIO/timer/driver; no physical jitter |
| Service loop | Tokio interval ~20 ms, `advance_ms(20)` | UI/telemetry cadence, not control timing |
| Watchdog | `watchdog_ok` hard-coded `true` in simulator protection inputs | No watchdog evidence |
| CAN | `can_ok` hard-coded `true`; no CAN driver | No bus timing evidence |
| Firmware | `no_std` skeleton only | No ISR, HAL, timer capture/compare, linker image, startup code, or driver |
| Firmware target check | `opentech-firmware-stm32` for `thumbv7em-none-eabihf` | Proves no_std compilation only; no timing evidence |

**Conclusion:** v0.1 can demonstrate event ordering and deterministic model behaviour, and the no_std skeleton can compile for the target. It cannot demonstrate that a real ECU will spark or inject at the right crank angle within a required tolerance.

## 2. Timing chain for a future ECU

```text
crank/cam sensor
  -> conditioning/filter (hardware)
  -> timer capture (timestamp, hardware)
  -> decoder ISR (gap/tooth/phase, bounded)
  -> angle prediction (last tooth + speed/time)
  -> scheduler (next event angle/time)
  -> output compare / one-shot timer (hardware)
  -> injector driver / igniter
  -> actuator current / spark
```

Each stage adds error. The safety argument must budget error at the **actuator**, not at the Rust function boundary.

## 3. Reference tooth timing (36-1 example only)

The Evo trigger pattern is **unverified**. The table uses 36-1 only because the simulator/vehicle simulation template uses it; it is not a claim about the real 4G63. Tooth spacing is 10° crank.

| Engine speed | One revolution | 10° tooth period | 1° crank | 0.1° crank | 1 µs in degrees |
|---:|---:|---:|---:|---:|---:|
| 600 rpm | 100.0 ms | 2.778 ms | 277.8 µs | 27.8 µs | 0.0036° |
| 1000 rpm | 60.0 ms | 1.667 ms | 166.7 µs | 16.7 µs | 0.0060° |
| 2000 rpm | 30.0 ms | 833.3 µs | 83.3 µs | 8.33 µs | 0.0120° |
| 3000 rpm | 20.0 ms | 555.6 µs | 55.6 µs | 5.56 µs | 0.0180° |
| 4000 rpm | 15.0 ms | 416.7 µs | 41.7 µs | 4.17 µs | 0.0240° |
| 5000 rpm | 12.0 ms | 333.3 µs | 33.3 µs | 3.33 µs | 0.0300° |
| 6000 rpm | 10.0 ms | 277.8 µs | 27.8 µs | 2.78 µs | 0.0360° |
| 7000 rpm | 8.571 ms | 238.1 µs | 23.8 µs | 2.38 µs | 0.0420° |
| 7500 rpm | 8.000 ms | 222.2 µs | 22.2 µs | 2.22 µs | 0.0450° |
| 9000 rpm | 6.667 ms | 185.2 µs | 18.5 µs | 1.85 µs | 0.0540° |

**Implication:** at 7000 rpm, a 1 µs timestamp error is 0.042° crank; a 24 µs ISR latency is 1°. A "few microseconds" claim is not negligible. At 9000 rpm, 10 µs is 0.54°.

## 4. Decoder worst-case analysis

| Error source | Mechanism | Bound in v0.1 | What must be measured |
|---|---|---|---|
| Tooth quantization | Position known only at tooth edges; between teeth it is predicted | No explicit bound; `crank_angle_at` extrapolates from last tooth using RPM | Max prediction error vs acceleration and RPM |
| Gap detection | Gap recognized only at the first tooth after the gap | One tooth interval (10° at 36-1) plus processing | Latency from gap edge to sync/angle update |
| Dropped tooth | Decoder waits for next interval and then fails | One tooth interval; state may be wrong until detection | Detection latency and whether an event fires in between |
| Extra tooth | Treated as normal; count mismatch only at next gap | Up to one revolution (360°) before `Lost` | False-edge rejection threshold; event prevention |
| Cam phase | Latched after first cam; no timeout | No bound; cam loss can persist | Cam timeout; phase cross-check |
| Timeout | `tick()` after 250 ms default | 250 ms = 25 rev at 6000 rpm, ~29 rev at 7000 rpm | Speed-dependent timeout; missed-edge count |
| Acceleration | RPM updated at gap; angle extrapolated with stale RPM | Error grows with acceleration and time since tooth | Error vs rpm/s ramp; cranking acceleration |
| Deceleration | Same, plus gap ratio may exceed threshold | Not tested | Deceleration test; false-gap threshold |
| Timestamp resolution | Hardware capture ticks | Unknown | Timer tick, capture latency, rollover |
| Noise/EMI | False/missed edges | Not modelled | Injector/coil noise injection; shielded harness |

### 4.1 Simple angle-error illustration

If the last tooth was at speed `N` rpm and the engine accelerates at `a` rpm/s, the angle error after time `t` from using stale speed is approximately:

```text
angle_error_deg ≈ 0.5 * (a * 6 [deg/s per rpm/s]) * t^2
```

Examples (illustrative, not measured):

| Acceleration | Time since tooth | Approx. angle error |
|---:|---:|---:|
| 1,000 rpm/s | 8.57 ms (one rev at 7000 rpm) | 0.22° |
| 5,000 rpm/s | 8.57 ms | 1.1° |
| 10,000 rpm/s | 8.57 ms | 2.2° |
| 10,000 rpm/s | 20 ms | 12° |

Cranking and rapid throttle transients can exceed these rates. The decoder must either update speed per tooth or bound the prediction interval; a fixed 250 ms timeout is not a bound on angle error.

### 4.2 Required decoder measurements

1. Capture timestamp resolution and rollover behaviour.
2. Tooth-to-tooth interval jitter at steady RPM, 600-9000 rpm, with dummy injector/coil loads.
3. Sync time from crank start to `Sequential` (worst case, hot/cold, low battery).
4. False-gap rate under electrical noise and at max acceleration/deceleration.
5. Detection latency for missing tooth, extra tooth, cam loss, and signal loss.
6. Whether any injection/spark event is issued between the fault and the cut.
7. Cam phase verification window and behaviour with a missing/spurious cam edge.

## 5. Scheduling and output jitter budget

| Contributor | Type | v0.1 status | Required measurement | Candidate acceptance (TBD-REVIEW) |
|---|---|---|---|---|
| Capture timestamp quantization | Hardware | Not implemented | Timer tick vs crank angle | ≤ 0.1° at 7000 rpm (≈2.4 µs) |
| Decoder processing | ISR/software | Not implemented | Cycle count / scope | ≤ 0.25° at 7000 rpm (≈6 µs) |
| Angle prediction | Algorithm | Host only | Error vs acceleration | ≤ 0.5° at 7000 rpm under defined transient |
| Scheduler computation | Software | Host only | WCET | ≤ 0.25° |
| Output compare latency | Hardware | Not implemented | Compare output to driver input | ≤ 0.25° |
| Driver propagation | Hardware | Not implemented | Driver input to current/spark | Datasheet + scope |
| Total spark jitter | End-to-end | **No evidence** | Scope/logic analyser | ≤ 1.0° at 7000 rpm? Placeholder; review |
| Injector timing | End-to-end | **No evidence** | Scope current probe | ≤ 2° or ≤ 0.1 ms? Review |
| Dwell accuracy | End-to-end | **No evidence** | Scope primary current | Coil-datasheet dependent |
| Rev limiter latency | Software/hardware | Sim only | Scope | Define max overshoot |

**Do not treat the candidate values as approved.** A spark jitter target must come from the engine/knock/emissions requirements and the coil/driver data. The table exists to force a number and a measurement, not to assert one.

### 5.1 Angle vs time scheduling

- **Angle-domain events** are correct for spark/injection relative to TDC. The scheduler computes angles; firmware must convert to timer compares using current speed and a prediction.
- **Time-domain one-shot** outputs are needed for dwell and pulse width. A compare channel starts dwell; a second compare/one-shot ends it. The timer must not rely on a software loop to end a pulse.
- **Priority:** crank/cam capture and spark/injection output compare are hard real-time. CAN, logging, UI and calibration transfer are not.
- **No dynamic allocation** in the capture/schedule path. Fixed-size arrays, bounded loops.
- **No blocking** on a mutex shared with the UI in the ISR. Use lock-free/single-writer queues or copy snapshots.

## 6. ISR vs async Rust

| Mechanism | Suitable for | Not suitable for | Required proof |
|---|---|---|---|
| Timer capture interrupt | Crank/cam timestamps | Long processing | WCET, priority, no allocation |
| Timer compare/one-shot | Dwell, spark, injection | Variable-length work | Output compare behaviour, jitter |
| High-priority ISR | Decoder state, schedule update | CAN/UI/logging | Bounded execution; no flash write |
| Async task (Embassy/Tokio) | Communications, UI state, slow sensors | Deterministic spark/injection timing | Latency measurement; no priority inversion |
| Main loop | Diagnostics, logging, calibration | Actuator deadlines | Loop budget/WCET |

**Rust does not make async deterministic.** `async`/`await` is a scheduling abstraction; it is not a hardware timer guarantee. The project's own scheduler doc says this, and the audit agrees. Any firmware claim must show timer capture/compare, not an async task, drives the outputs.

## 7. Watchdog and safe state

| Requirement | Current v0.1 | Required future evidence |
|---|---|---|
| Independent watchdog | None; `watchdog_ok=true` | External window watchdog; reset on hang |
| Watchdog service policy | None | Service only when all critical tasks check in |
| Reset cause logging | None | Persist reset cause/watchdog count |
| Safe output state on reset | No outputs | Hardware default-off; verify with scope |
| Brownout detection | None | UVLO/reset supervisor; safe state |
| Clock failure detection | None | Clock monitor/watchdog |
| Debug halt behaviour | None | Outputs off; watchdog behaviour defined |
| Watchdog test | None | Inject hang; measure reset latency and output state |

A watchdog that is only a software flag is not a watchdog. The current `Dtc::Watchdog` is a placeholder input; it has no evidential value.

## 8. Loop budget and task rates

| Task | Candidate rate | Deadline | v0.1 analogue | Evidence |
|---|---:|---|---|---|
| Crank/cam capture | per edge (up to kHz) | capture latency | Synthetic events | None |
| Decoder/sync | per edge / 1 ms | bounded | `on_tooth`, `tick` | Host tests only |
| Spark/injection scheduling | per event | event angle | `EventSchedule` | Host tests only |
| Protection | 1-10 ms | fault detection time | 1 ms sim step | Host tests only |
| Fuel/ignition calculation | 1-10 ms or event | next event | 1 ms sim step | Host tests only |
| CAN RX/TX | 1-10 ms | message deadline | none | None |
| Logging | 10-100 ms | no control deadline | 50 Hz sim | Host only |
| UI/telemetry | 20-100 ms | no control deadline | 20 ms service tick | Host only |
| Calibration apply | on request, paused | no control deadline | service | Host only |

**Loop budget method:** measure worst-case execution time (WCET) of each task at max CPU clock, max flash wait states, max interrupt load, and worst-case branching. Sum critical tasks; verify deadline margin. Do not use average loop time. The host simulator's 1 ms step is not a WCET measurement.

## 9. CAN and UI isolation

| Requirement | Why | Current evidence | Required test |
|---|---|---|---|
| CAN/UI cannot block the control ISR | A slow UI or bus storm must not delay spark | Single-threaded host sim; no priorities | Priority test; bus flood; UI stall |
| Bounded queues | Prevent memory growth/backpressure into control | None | Queue fill test; drop policy |
| Telemetry timestamps | Detect stale data | Frame timestamp exists | Compare source vs receive time |
| No control dependency on UI | UI is an observer/approver, not a real-time element | No hardware path | Architecture review |
| CAN loss safe state | Wideband/dash loss must not cause unsafe actuation | `can_ok=true` hard-coded | Fault injection |
| Logging backpressure | Logging must not stall control | None | Fill log; measure loop |

## 10. Measurement plan on target hardware

### 10.1 Equipment

- Oscilloscope: at least 2 channels, 100 MHz+ analog bandwidth, adequate probe rating; current probe for injector/coil; differential probe where required.
- Logic analyser: 8+ channels, 100 MHz+ sample rate, timestamping; protocol decoder for CAN if available.
- Signal/trigger generator or a verified crank/cam simulator.
- Current-limited DC supply, fuses, dummy injector/ignition loads, E-stop.
- Temperature chamber or controlled heat source for thermal corners (if available).
- CAN analyser/interface.
- Calibrated DMM; shunt for current.

### 10.2 Probe points

| Point | Signal | Purpose |
|---|---|---|
| TP-CRANK | Conditioned crank signal | Tooth timing, noise, missing tooth |
| TP-CAM | Conditioned cam signal | Phase, cam loss |
| TP-INJ-1..4 | Driver input or current | Pulse width, timing, flyback |
| TP-IGN-1..4 | Igniter input or coil primary | Dwell, spark timing |
| TP-5V / TP-3V3 | Regulator rails | Brownout, noise |
| TP-VBAT | Battery sense | Cranking/transient |
| TP-WDG | Watchdog reset/service | Reset behaviour |
| TP-CAN-H/L | CAN differential | Bus integrity |
| TP-ESTOP | E-stop state | Safety chain |

### 10.3 Test matrix

| Test | Condition | Measurement | Pass criterion (TBD-REVIEW) | Stop condition |
|---|---|---|---|---|
| TIM-01 | Steady 600-9000 rpm, dummy loads | Tooth interval, jitter | No false/missed tooth; jitter within budget | Any false event or driver overheating |
| TIM-02 | Acceleration/deceleration ramp | Predicted vs actual angle | Angle error within budget | Sync loss or wrong event |
| TIM-03 | Missing tooth | Detection latency, output state | Cut before any wrong-angle event | Any spark/injection after fault |
| TIM-04 | Extra tooth/noise | False-event rate | No wrong event | Repeated false events |
| TIM-05 | Cam loss | State, fuel/spark | Detect and degrade/cut | Stays sequential with wrong phase |
| TIM-06 | Spark timing sweep | Command vs actual | Jitter within budget across RPM/load/temp | > budget or misfire |
| TIM-07 | Dwell sweep | Dwell time/current | Within coil data; no overshoot | Coil/driver overheat |
| TIM-08 | Injector pulse sweep | PW/current/flyback | Within driver SOA | Overcurrent/thermal |
| TIM-09 | Rev limiter | Overshoot/cut | Within defined limit | Uncontrolled overspeed |
| TIM-10 | Watchdog hang | Reset latency/output state | Outputs off; reset logged | Output active after hang |
| TIM-11 | Brownout sweep | Reset/output state | Safe state; no glitch | Unintended actuation |
| TIM-12 | CAN flood/UI stall | Control jitter | No missed events | Deadline miss |
| TIM-13 | Temperature corners | Timing drift | Within budget | Any unsafe drift |
| TIM-14 | Supply corners | Timing/current | Within budget | Reset/actuation glitch |

### 10.4 Sample size and recording

- Minimum 10 captures per operating point; record RPM, battery voltage, temperature, firmware hash, calibration checksum, probe setup, timebase, and trigger source.
- Save raw scope/logic-analyser files, not only screenshots; screenshots are not sufficient for timing claims.
- State the measurement uncertainty (probe, timebase, sample rate). A 1 µs claim from a 100 MS/s scope is at the edge of resolution.
- Test at cold/hot, low/high battery, and max load; do not extrapolate from one point.

## 11. Timing budget template (to be filled only with measurements)

| Stage | Budget (µs or °) | Measured | Margin | Evidence ID |
|---|---:|---:|---:|---|
| Capture timestamp | TBD-REVIEW | | | |
| Decoder ISR | TBD-REVIEW | | | |
| Angle prediction | TBD-REVIEW | | | |
| Scheduler | TBD-REVIEW | | | |
| Output compare | TBD-REVIEW | | | |
| Driver | TBD-REVIEW | | | |
| **Total spark** | TBD-REVIEW | | | |
| **Total injection** | TBD-REVIEW | | | |

## 12. What would falsify a future timing claim

- Any spark/injection event after sync loss or during a detected fault.
- Measured jitter above the approved budget at any RPM/load/temperature corner.
- A missed or duplicated event under CAN/UI load.
- Watchdog failure to reset a hung MCU, or outputs active after reset.
- Angle error growing beyond budget during acceleration/deceleration.
- A cam-loss or extra-tooth event that changes phase without detection.
- Any claim based only on host simulation, average loop time, or a screenshot.

## 13. Current evidence and gaps

| Evidence | Status |
|---|---|
| Host 1 ms deterministic simulation | Present; CLI determinism PASS for a fixed calibration |
| Scheduler angle tests | Present; 6 unit tests |
| Trigger decoder tests | Present; 10 unit tests; acceleration only |
| Firmware skeleton target check | PASS for `thumbv7em-none-eabihf`; no HAL/timer/driver |
| ISR/timer implementation | Absent |
| MCU clock/timer configuration | Absent |
| Scope/logic-analyser captures | None |
| Watchdog test | None; input hard-coded true |
| CAN/UI isolation test | None |
| Timing budget with measured numbers | None |
| Temperature/voltage corners | None |
| Independent timing review | None |

## 14. Explicit non-claims

- No MCU timing evidence exists; the firmware is a no_std skeleton with no HAL/timer/driver.
- No spark/injection jitter, latency, dwell or pulse-width measurement exists.
- No watchdog, brownout, clock-failure or reset safe-state evidence exists.
- The host 1 ms simulation is not a hard real-time guarantee.
- Rust/async does not provide deterministic ignition timing; hardware capture/compare is required.
- No Evo trigger pattern or timing offset is verified; 36-1 is a simulation placeholder only.
