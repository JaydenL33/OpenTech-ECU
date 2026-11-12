# Hardware Status

| Field | Value |
|---|---|
| Document ID | AUD-10 |
| Status | Hardware status and requirements only; **no manufacturing-ready hardware exists** |
| Date | 2026-11-12 |
| Evidence cut-off | 2026-11-12T05:37Z, working tree |
| Source manifest | `984aa392bcb457bfd60a14cff7d505959e3534fe3afe4cf9428ea542ec649aad` |
| Related | AUD-03, AUD-04, AUD-08, AUD-09 |

> **Plain statement.** No manufacturing-ready KiCad schematic, PCB, BOM, gerber, drill file, assembly drawing, enclosure or harness exists in this repository. A separate Codex attempt at a bench-prototype KiCad package is reportedly underway and is **UNREVIEWED**. It must not be treated as valid, manufacturable, electrically safe, or Gate 1 evidence until ERC/DRC, BOM/derating, thermal, EMC and independent engineering review are complete.

## 1. Hardware artefacts: exists vs absent

| Artefact | Exists in audited repo? | Evidence / note |
|---|---|---|
| Rust host control code | Yes | `ecu/crates/*`; 128 tests pass |
| Virtual ECU / simulator | Yes | `ecu-simulator`; six scenarios |
| `no_std` firmware skeleton | Yes | `ecu/firmware/stm32`; target check passes; no HAL/timer/driver |
| MCU HAL / board support | No | No vendor HAL, pin map, clock tree, linker image, startup code |
| Timer capture/compare implementation | No | No crank/cam capture, injector/ignition output compare |
| Injector/ignition driver code | No | No GPIO/timer/driver calls; `PHYSICAL_OUTPUTS_AUTHORISED=false` |
| Watchdog integration | No | Skeleton accepts a watchdog flag; no hardware watchdog |
| Flash/calibration storage | No | No bootloader, dual-bank, or calibration flash layout |
| CAN driver | No | No transceiver/driver; simulator `can_ok=true` |
| KiCad schematic | No | No `.kicad_sch` found |
| KiCad PCB | No | No `.kicad_pcb` found |
| KiCad project | No | No `.kicad_pro` found |
| BOM | No | No ECU BOM found |
| Gerbers/drill | No | No `.gbr`/`.drl` found |
| Assembly drawing | No | Not present |
| Enclosure/mechanical | No | Not present |
| Harness/adapter | No | Not present; Evo connector unverified |
| Bench fixture | No | AUD-09 is a plan only |
| EMC/thermal test data | No | Not present |
| Evo ECU part number/connector/pinout | No | `vehicle.rs`: `None`/`false`; all unverified |
| Evo trigger/injector/coil data | No | Unverified |

**Not a manufacturing package.** A folder of KiCad files, even if it appears later, is not a manufacturable product. Manufacturing readiness requires a reviewed schematic/PCB, ERC/DRC with a documented rule set, BOM with derating/lifecycle, stack-up, assembly drawings, test coverage, and a build house DFM review. None exists.

## 2. Candidate architecture (not a selection)

```text
12 V input -> fuse -> reverse-polarity protection -> transient clamp -> main relay
   |-> 5 V sensor rail -> MAP/TPS/CLT/IAT/oil/fuel/battery sense
   |-> 3.3 V MCU rail -> MCU (candidate) -> debug/watchdog/recovery
   |-> actuator supply -> 4x injector low-side drivers (flyback, diagnostics)
   |-> ignition interface -> 4x logic-level outputs or coil drivers (topology TBD)
   |-> crank/cam conditioning -> timer capture
   |-> knock conditioning -> ADC/timer window
   |-> CAN transceiver -> external wideband / PC / dash
```

The architecture is deliberately generic. No pin, timer channel, connector cavity, driver part number or current rating is assigned.

## 3. MCU trade study inputs (no selection)

The master brief asks for a trade study between STM32H7 and automotive-focused NXP S32K. This is an input list, not a decision. No unsupported HAL claim is made.

| Criterion | Why it matters | STM32H7 candidate | S32K candidate | Evidence required before selection |
|---|---|---|---|---|
| Rust HAL maturity for the exact part | A family port may not cover the needed timer/ADC/CAN | Verify exact-part support in the chosen HAL; do not assume | Verify exact-part Rust support; do not assume | Buildable dev-board test: clock, timer capture, compare, ADC, CAN, watchdog |
| Timer capture/compare channels | Crank/cam capture + 4 injector + 4 ignition outputs | Count/resolution must be verified per exact part | eMIOS/timer resources must be verified | Timer allocation table; scope |
| ADC channels/resolution/speed | MAP/TPS/CLT/IAT/oil/fuel/battery/knock | Verify | Verify | Noise/resolution measurement |
| CAN | External wideband, PC, dash | Verify classic CAN/CAN-FD as required | Verify | Bus test |
| Watchdog | Independent window watchdog/reset | Verify | Verify | Fault injection |
| Safety/qualification | Automotive environment | Verify exact part qualification | Verify automotive qualification | Datasheet/qualification report |
| Package/thermal | Board size/heat | Verify | Verify | Thermal calculation |
| Debug/trace | Bring-up/timing measurement | Verify SWD/trace | Verify | Debug probe test |
| Long-term supply | Production risk | Verify lifecycle | Verify | Vendor statement |
| Toolchain/licence | Reproducible build | Verify | Verify | Build log/licence |
| Cost/availability | Budget | Quote | Quote | Purchasing record |

**Decision rule:** no MCU is selected until a development board proves the exact timer capture/compare, ADC, CAN, watchdog and Rust build. A host `cargo check` or a family-level HAL claim is not proof.

## 4. Connector strategy options

| Option | Description | Pros | Risks | Prerequisite |
|---|---|---|---|---|
| OEM mating connector | Board mates the factory ECU connector | Reversible; no harness cut | Exact housing/terminal/keying unknown; pinout unknown; current/space limits | ECU part number, connector markings, pinout, terminal series |
| Adapter harness | Factory connector to new ECU | Reversible; bench/vehicle transition | Contact resistance, strain, sealing; still needs pinout | Verified connector/pinout; harness drawing |
| Breakout board | Bench terminal block | Easy probing; safe | Not for vehicle; no sealing/vibration | Bench plan AUD-09 |
| New sealed harness | Standalone ECU harness | Full control; new sensors | Labor; vehicle modification; ACD/AYC/dash integration | Full electrical design and vehicle review |
| Bare wire/pigtail | Not acceptable for vehicle | Cheap | Unsealed, unreliable, dangerous | Prohibited for vehicle |

No connector is selected. Do not order an adapter or publish a pinout from a forum image or a similar model.

## 5. Unresolved hardware items

| ID | Item | Impact | Required action |
|---|---|---|---|
| HW-01 | No reviewed schematic/PCB/BOM/gerbers | Cannot build or review | Produce/review a KiCad package; ERC/DRC; DFM |
| HW-02 | Separate Codex KiCad attempt unreviewed | Could be mistaken for a valid design | Treat as draft only; independent review; do not manufacture |
| HW-03 | MCU/HAL not selected/proven | Firmware plan may be infeasible | Dev-board proof for exact part/peripherals |
| HW-04 | No timer capture/compare implementation | No timing evidence | Implement and measure; AUD-04 |
| HW-05 | No injector driver/flyback design | Driver/coil damage risk | Injector characterization; driver SOA; scope |
| HW-06 | No ignition interface | Coil/igniter damage/no spark | Verify Evo topology; design interface |
| HW-07 | No knock interface | No knock protection | Sensor/filter/gain/window design; validation |
| HW-08 | No CAN physical layer | Wideband/PC/dash unavailable | Transceiver/termination/protection |
| HW-09 | No watchdog/safe-state hardware | Hang can leave outputs active | External watchdog; default-off; fault injection |
| HW-10 | No power/transient design | Brownout/load-dump damage | Input protection; UVLO; test plan |
| HW-11 | No thermal/EMC/mechanical design | Reliability/compliance unknown | Calculations; chamber; pre-compliance; enclosure |
| HW-12 | No bench fixture/E-stop | Unsafe bring-up | Build AUD-09 fixture |
| HW-13 | Evo pinout/connector/trigger/harness unverified | Wrong wiring/trigger | Complete Gate 3 evidence |
| HW-14 | No manufacturing/DFM review | Build yield/assembly risk | Build-house review; panel/stack-up |
| HW-15 | No independent review | Design unvalidated | Schedule electrical/functional review |

## 6. What would change the status

A hardware status change requires all of the following, in order:

1. Requirements in AUD-03 assigned values and traced to schematic nets/components.
2. MCU selected after a dev-board proof of timers/ADC/CAN/watchdog/Rust build.
3. Schematic/PCB/BOM/gerbers produced and reviewed; ERC/DRC run with documented rules and exceptions.
4. BOM derating/lifecycle, thermal, EMC plan, connector/pinout evidence.
5. A separate Codex/KiCad attempt, if used, reviewed line-by-line and not assumed correct.
6. Bench fixture, current-limited supply, fuses, dummy loads, E-stop (AUD-09).
7. Gate 1 unpowered inspection passed; Gate 2 powered bench evidence produced.
8. Independent electrical review signed.

Until then: **requirements only, no manufacturing claim, no vehicle connection.**

## 7. Explicit non-claims

- No manufacturing-ready PCB, BOM, gerbers or assembly exists.
- No KiCad ERC/DRC has been run on a reviewed design.
- No separate Codex KiCad attempt has been validated; it is unreviewed.
- No MCU/HAL/timer/driver/watchdog implementation exists.
- No electrical safety, EMC, thermal, vibration, IP or automotive qualification is claimed.
- No connector pinout, ECU part number or plug-and-play compatibility is claimed.
- No engine or vehicle readiness is claimed. Gate 1+ remains blocked.
