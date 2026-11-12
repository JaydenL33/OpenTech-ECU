# Hardware Status

| Field | Value |
|---|---|
| Document ID | AUD-10 |
| Status | Hardware status and requirements only; **no manufacturing-approved hardware exists**; prototype-v1 baseline selected (H0 pass, H1 partial, H2 open) |
| Date | 2026-11-13 |
| Evidence cut-off | 2026-11-13, working tree |
| Source manifest | `93dc046e3a28ff4ebc30559192d16a889e40b135820f86a96fbbe14e8b35434a` |
| Related | AUD-03, AUD-04, AUD-08, AUD-09 |

> **Plain statement (updated 2026-11-13).** No manufacturing-approved PCB,
> gerber release, assembly drawing, enclosure or harness exists. The current
> **prototype-v1 baseline** is an unmodified Speeduino **v0.4.4d SMD** design
> (upstream commit `ee6a9242834891877744ff2137663b6ca8d79ed6`) with a native
> KiCad 9 review copy at `hardware/prototype-v1/`. It is **fully routed**
> (KiCad 9 DRC: 0 unconnected items) and has fresh gerber/drill/BOM/position
> exports, but **H2 is not passed**: ERC reports 101 violations and DRC reports
> 609 (238 errors), with documented exceptions and open items. The older
> `hardware/kicad/evo8-bench-ecu/` custom STM32 package remains an **unrouted
> placement draft** (499 unconnected items, 14 violations) and is **not** the
> baseline. Neither design is fabrication-approved or vehicle-safe.
>
> New evidence: `AUDIT/UPSTREAM_PROVENANCE.md`, `AUDIT/HARDWARE_BASELINE.md`,
> `AUDIT/PCB_CHECK_RESULTS.md`, `HARDWARE_BLOCKERS.md`.

## 0. 2026-11-13 update — selected baseline vs legacy custom draft

| Item | Selected prototype-v1 baseline | Legacy custom `evo8-bench-ecu` |
|---|---|---|
| Electrical source | Speeduino v0.4.4d SMD, upstream commit pinned | OpenTech-ECU custom STM32H743 draft |
| KiCad format | Native KiCad 9 `.kicad_sch`/`.kicad_pcb` (converted from upstream KiCad 5) | Native KiCad 9 |
| Routing | **Complete — 0 unconnected items** | **Unrouted — 499 unconnected items** |
| DRC | 609 violations (238 errors, 371 warnings); 0 unconnected | 14 violations; 499 unconnected |
| ERC | 101 violations (44 errors, 57 warnings) | 0 reported, unreviewed |
| BOM | 47 grouped rows; 11 rows lack MPN | 305 lines, unreviewed |
| Manufacturing exports | 9 gerbers + drill + IPC-D-356 + BOM + schematic PDF; position file has 150 proto pads (not assembly-ready) | None |
| Firmware compatibility | Upstream Speeduino `megaatmega2560` target; build passes (`AUDIT/FIRMWARE_TEST_EVIDENCE.md`) | No validated firmware/HAL |
| Review status | H2 open; not fabrication-approved | Unreviewed; not usable |
| Gate | H0 pass; H1 partial; H2 not passed | Not a baseline |

The rest of this document retains the earlier audit of the custom draft. Where
it says "the PCB", read it as the legacy custom board unless the new baseline is
explicitly named.

## 1. Hardware artefacts: exists vs absent

| Artefact | Exists in audited repo? | Evidence / note |
|---|---|---|
| Hardware status README | Yes | `hardware/README.md`; explicitly states no manufacturing-ready board and bench-prototype boundaries |
| Rust host control code | Yes | `ecu/crates/*`; 130 tests pass |
| Virtual ECU / simulator | Yes | `ecu-simulator`; six scenarios |
| `no_std` firmware skeleton | Yes | `ecu/firmware/stm32`; target check passes; no HAL/timer/driver |
| Reference Speeduino firmware build | Yes — build only | `pio run -e megaatmega2560` SUCCESS; hex/ELF hashes in `AUDIT/FIRMWARE_TEST_EVIDENCE.md`; not flashed |
| MCU HAL / board support | No | No vendor HAL, pin map, clock tree, linker image, startup code |
| Timer capture/compare implementation | No | No crank/cam capture, injector/ignition output compare |
| Injector/ignition driver code | No | No GPIO/timer/driver calls; `PHYSICAL_OUTPUTS_AUTHORISED=false` |
| Watchdog integration | No | Skeleton accepts a watchdog flag; no hardware watchdog |
| Flash/calibration storage | No | No bootloader, dual-bank, or calibration flash layout |
| CAN driver | No | No transceiver/driver; simulator `can_ok=true` |
| KiCad schematic | Yes — selected baseline native; legacy custom **UNREVIEWED** | `hardware/prototype-v1/*.kicad_sch` (upstream v0.4.4d, native KiCad 9, 7 sheets); legacy `hardware/kicad/evo8-bench-ecu/` 7 sheets, unreviewed |
| KiCad PCB | Selected baseline routed; legacy custom **NOT USABLE** | `hardware/prototype-v1/v0.4.4d.kicad_pcb`: 297 footprints (147 components + 150 proto pads), 0 unconnected, 609 DRC (238 errors). Legacy custom: 305 footprints, 499 unconnected, 14 DRC |
| KiCad project | Yes — selected baseline converted | `hardware/prototype-v1/v0.4.4d.kicad_pro` (KiCad 9); legacy `.kicad_pro` unreviewed |
| BOM | Yes — baseline BOM has gaps | `prototype-v1/manufacturing/assembly/v0.4.4d-bom.csv`: 47 grouped rows, 11 without MPN. Legacy custom 305 lines, unreviewed |
| ERC report | Yes — baseline exceptions open | `prototype-v1/verification/erc.rpt`: 101 violations (44 errors). Legacy custom `reports/erc.json` 0 violations, unreviewed |
| Gerbers/drill | Yes — baseline review exports only | `prototype-v1/manufacturing/gerbers/` 9 layers + job; `manufacturing/drill/` Excellon + map. Not DFM-checked, not fabricated |
| Assembly drawing | No | Not present |
| Enclosure/mechanical | No | Not present |
| Bench harness manifest | Yes, unreviewed source | `config/harness/opentech-bench-harness.json`; 29 bench connectors, 54 test points; no OEM cavities |
| Harness/adapter for vehicle | No | Not present; Evo connector unverified |
| Bench fixture | No | AUD-09 is a plan only |
| EMC/thermal test data | No | Not present |
| Evo ECU part number/connector/pinout | No | `vehicle.rs`: `None`/`false`; all unverified |
| Evo trigger/injector/coil data | No | Unverified |

**Not a manufacturing package.** The selected prototype-v1 baseline is a real, routed, upstream-derived design with review exports, but its ERC/DRC exceptions, BOM gaps, bench breakout and independent review are incomplete. The legacy custom schematic package remains an unreviewed hypothesis. Manufacturing readiness requires a reviewed schematic, PCB layout, DRC with a documented rule set, BOM derating/lifecycle, stack-up, assembly drawings, test coverage, and a build-house DFM review. None exists. An ERC/DRC pass is a KiCad rule result only; it does not prove electrical safety or Evo compatibility.

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

The architecture is deliberately generic. No pin, timer channel, connector cavity, driver part number or current rating is assigned. `hardware/README.md` states the first target is a bench prototype with an STM32H7-class MCU; the unreviewed schematic captures an STM32H743VIT6, but that is not a validated part selection or HAL support claim.

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
| HW-01 | Unreviewed schematic; unrouted PCB with failing DRC | Cannot build or review manufacturing | Review schematic/nets; complete routing; fix DRC; DFM |
| HW-02 | Codex KiCad attempt unreviewed | Could be mistaken for a valid design | Treat as draft only; independent review; do not manufacture |
| HW-03 | MCU/HAL not selected/proven (schematic uses STM32H743VIT6, unreviewed) | Firmware plan may be infeasible | Dev-board proof for exact part/peripherals |
| HW-04 | No timer capture/compare implementation | No timing evidence | Implement and measure; AUD-04 |
| HW-05 | Unreviewed injector sink/flyback design | Driver/coil damage risk | Injector characterization; driver SOA; scope |
| HW-06 | Unreviewed ignition interface | Coil/igniter damage/no spark | Verify Evo topology; design interface |
| HW-07 | No knock interface | No knock protection | Sensor/filter/gain/window design; validation |
| HW-08 | Unreviewed CAN physical layer | Wideband/PC/dash unavailable | Transceiver/termination/protection review |
| HW-09 | Unreviewed watchdog/safe-state hardware | Hang can leave outputs active | External watchdog; default-off; fault injection |
| HW-10 | Unreviewed power/transient design | Brownout/load-dump damage | Input protection; UVLO; test plan |
| HW-11 | No thermal/EMC/mechanical design | Reliability/compliance unknown | Calculations; chamber; pre-compliance; enclosure |
| HW-12 | No bench fixture/E-stop | Unsafe bring-up | Build AUD-09 fixture |
| HW-13 | Evo pinout/connector/trigger/harness unverified | Wrong wiring/trigger | Complete Gate 3 evidence |
| HW-14 | No manufacturing/DFM review | Build yield/assembly risk | Build-house review; panel/stack-up |
| HW-15 | No independent review | Design unvalidated | Schedule electrical/functional review |

## 6. What would change the status

A hardware status change requires all of the following, in order:

1. Requirements in AUD-03 assigned values and traced to schematic nets/components.
2. For the selected v0.4.4d v1 path: confirm the Arduino Mega 2560 platform meets the bench requirements and build the matching firmware; the long-term STM32/S32K selection remains a separate trade study.
3. Reviewed schematic; PCB/BOM/gerbers produced and reviewed; ERC/DRC run with documented rules and exceptions.
4. BOM derating/lifecycle, thermal, EMC plan, connector/pinout evidence.
5. The selected v0.4.4d design reviewed line-by-line: every component value, net and protection choice checked; not assumed correct. The legacy Codex/KiCad custom schematic is a separate unreviewed draft.
6. Bench fixture, current-limited supply, fuses, dummy loads, E-stop (AUD-09).
7. Gate 1 unpowered inspection passed; Gate 2 powered bench evidence produced.
8. Independent electrical review signed.

Until then: **requirements only, no manufacturing claim, no vehicle connection.**

## 7. Explicit non-claims

- No manufacturing-approved PCB, gerber release or assembly drawing exists. The selected v0.4.4d baseline has native KiCad 9 files, review gerbers/drill and a generated BOM, but ERC 101 / DRC 609 exceptions, 11 BOM rows without MPN, no assembly drawing and no independent review remain.
- The legacy custom `evo8-bench-ecu` schematic/BOM is unreviewed; its PCB has 499 unconnected items and 14 DRC violations and is not manufacturable.
- No MCU/HAL/timer/driver/watchdog firmware implementation exists for the custom STM32 path. The selected v0.4.4d baseline targets upstream Speeduino on Arduino Mega 2560, and a reproducible firmware build now passes (`AUDIT/FIRMWARE_TEST_EVIDENCE.md`), but no firmware has been flashed and no hardware has been powered.
- No electrical safety, EMC, thermal, vibration, IP or automotive qualification is claimed.
- No connector pinout, ECU part number or plug-and-play compatibility is claimed.
- No engine or vehicle readiness is claimed. Gate 1+ remains blocked.
