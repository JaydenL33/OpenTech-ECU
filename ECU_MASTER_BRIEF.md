Rust-First ECU Engineering Mandate
Haltech Nexus R5 + Link G5 Voodoo Pro V2 Feature-Parity Research and OpenCode Execution Brief
Research snapshot: 12 November 2026.
Target: Original, open-source, Speeduino-informed ECU platform; Rust-first software; virtual ECU and tuning studio first; physical vehicle operation only after independent engineering validation.
Execution budget: approximately US$26 in DeepSeek API credit.
Status: Research and implementation specification, not a validated ECU design or a safe engine calibration.
1. Mission
Build an original next-generation engine management platform with functionality benchmarked against Haltech Nexus R5/NSP and Link G5 Voodoo Pro V2/PCLink. Reproduce capabilities and engineering workflows, not proprietary code, brand assets, confidential firmware, locked protocols, or pixel-identical interfaces.

Produce actual working Rust code, a cross-platform tuning application, a virtual ECU, deterministic calibration algorithms, test evidence, and real KiCad design assets. The first end-to-end milestone is a host-executable virtual four-cylinder ECU controlled by a real tuning interface, not a vehicle-ready PCB.
Non-negotiable distinction
A wideband lambda sensor measures combustion oxygen/fuelling consequences, not optimal or knock-safe ignition timing. Implement lambda-based fuel correction separately from ignition recommendation. Never label estimated ignition maps safe or engine-ready without verified source calibration and expert validation; never perform automatic spark advance based on lambda error.
2. Verified competitive baselines
Haltech Nexus R5 — official published capabilities
Hardware and I/O: 18 peak/hold injector drivers; 12 low-current ignition outputs; 23 analogue 0–5 V inputs; 10 synchronous pulsed inputs; differential crank/home inputs; two knock inputs; two onboard wideband controllers (NTK/LSU 4.9); three CAN networks; USB-C; Wi-Fi; 512 MB onboard logging; up to 300 channels at up to 1 kHz/channel; four 25 A high-current outputs; twelve 8 A high-side outputs; four 8 A half-bridges; eight PWM-capable low-side digital outputs. The device integrates a PDM-like subsystem and supports dual electronic throttle.

Engine/motorsport software: long-term learning up to 4D; 32×32×8 4D mapping; dual closed-loop oxygen; dual knock; multistage fuel injection; four-cam control; flex fuel; multi-level engine protection; advanced torque/driveshaft RPM strategies; 4D closed-loop boost; CO₂ control; launch; traction; anti-lag and rolling anti-lag; rotational idle; flat-shift; transbrake; six-stage nitrous; ride height/shock travel; race timer; selectable automatic-transmission functions.

NSP workflow: vehicle/engine/harness configuration wizards, sensor and injector setup, configurable base maps for supported plug-in systems, live programming, target-lambda tables, short-term and long-term trim, log analysis, low-power USB configuration with physical outputs disabled on applicable models.

Important limits: Haltech explicitly warns that base maps are starting configurations, not validated tunes; ignition base timing must be physically synchronised using a timing light for applications that require it.

Sources:
- https://www.haltech.com/product/ht-195000-nexus-r5-vcu/
- https://support.haltech.com/portal/en/kb/articles/short-and-long-term-fuel-trim
- https://support.haltech.com/portal/en/kb/articles/target-lambda
- https://support.haltech.com/portal/en/kb/articles/nexus-plug-in-ecu-nissan-skyline-r32-r33-r34-gt-r
- https://support.haltech.com/portal/en/kb/articles/engine-protection
- https://support.haltech.com/portal/en/kb/articles/flex-fuel-tuning-guide
- https://support.haltech.com/portal/en/kb/articles/fuel-film-model-map-prediction
Link G5 Voodoo Pro V2 — official published capabilities
Hardware and I/O: 12 dedicated injector drives, eight dedicated ignition drives and four configurable injection/ignition/auxiliary drives (24 total: not 16 + 12 simultaneous independent drives); 14 analogue inputs; four temperature inputs; 10 digital inputs; two trigger; two knock; eight multifunction ADIO; dual integrated lambda; dual e-throttle H-bridges; two CAN; USB-C; Wi-Fi; Ethernet; GPS; accelerometer; 1 GB logging; up to 250 channels; total logging throughput advertised at 100 kHz; direct injection supported with an external driver box; automatic transmission support.

Engine/motorsport software: traditional and modelled VE fuel equations and modelled multifuel; flexible 3D/4D tables and configurable axes; staged injection; injector characterisation; fuel pressure compensation; closed-loop lambda and long-term fuel trim; offline mixture mapping; individual-cylinder trim; knock detection/retard; open-/closed-loop boost; flexible limiters; VVT; traction; launch; anti-lag; flat/paddle shift; flex fuel; nitrous; configurable engine protection, torque and CAN interactions; cruise control.

PCLink workflow: real-time programming, editable maps, interactive 3D plotting, multiple tables, configurable workspace pages, sensor/setup procedures, gauges, DTCs/status, log time plot/navigation, XY plot, histogram, statistics, log/lap overlays and linked cursors. G4X/G5 plug-in base files are sample starting configurations, not recommended final tuning values. Link's Quick Tune, Mixture Map and closed-loop/long-term learning should be studied as distinct workflows.

As-of-date nuance: Link published a 10 November 2026 announcement of upcoming PCLink capabilities, including expanded OEM CAN/transmission modes, gear-based torque management, injector characterisation import wizard, LinkCAN sniffing/transmit and knock-audio monitoring. Treat these as announced, not necessarily shipped, until an actual released version confirms availability. Current PCLink downloads page lists version 7.8.2 dated 29 April 2026.

Sources:
- https://linkecu.com/products/g5-voodoo-pro-2
- https://linkecu.com/pages/pc-link
- https://support.linkecu.com/learn/latest/product-features/motorsport-features
- https://support.linkecu.com/learn/latest/product-features/features-engine-protection
- https://support.linkecu.com/learn/latest/get-started/getting-running/fuel-equations
- https://support.linkecu.com/learn/latest/get-started/getting-running
- https://forums.linkecu.com/topic/14142-mixture-map-and-closed-loop-lambda/
- https://linkecu.com/blogs/news/new-pclink-features-nov-2026
Upstream technology and licensing
- Speeduino firmware: https://github.com/speeduino/speeduino — GPL-2.0.
- Speeduino hardware: https://github.com/speeduino/Hardware — CERN-OHL-S-2.0 (strong reciprocity for covered hardware derivatives).
- RusEFI reference: https://github.com/rusefi/rusefi — inspect actual repository licences before borrowing code.
- Rust STM32 ecosystem: https://docs.rs/embassy-stm32/latest/embassy_stm32/ — device support must be verified for exact selected MCU and critical timer/peripheral.
A Rust port of Speeduino code is not automatically licence-free merely because the programming language changed. Preserve applicable licences and notices for derived implementations. Keep original modules separate where feasible and record provenance.
3. Feature-parity backlog
The tags mean: P0 = essential working simulation/tooling; P1 = major usable functionality; P2 = advanced, bench-only development; P3 = major hardware/validation programme. A ✓ on a competitor claim is not a claim that our code implements it.
A. ECU engine fundamentals
- P0: 4-stroke engine model, configurable cylinders 1–12 in simulator, firing order, displacement, compression ratio metadata, aspiration, maximum RPM, engine trigger pattern, crank/cam phasing, absolute ignition offset.
- P0: crank-tooth timestamp model, missing-tooth trigger decoder, phase/synchronisation state machine, deterministic loss-of-sync response, event scheduling simulation for injection and spark.
- P0: sensor pipeline and strongly typed quantities (RPM, kPa, K/°C, lambda, microseconds, degrees crank, battery voltage) with invalid/stale status and calibration metadata.
- P0: speed-density/VE fuel calculation with documented assumptions, injector pulse-width/duty model, battery voltage and injector dead-time correction, unit and boundary tests.
- P1: alpha-N/TPS and blended load strategies; transient enrichment and fuel film model; cranking, after-start, warm-up, IAT and coolant compensations; sequential, batch, staged injection.
- P1: configurable ignition advance maps, dwell controls, correction hierarchy, per-cylinder trims, rev limiting and timing-light calibration workflow.
- P2: VVT control, advanced engine modes, multi-stage injection, closed-loop idle, combustion balance, direct-injection modelling only (physical DI requires a validated driver system).
B. Lambda and automated fuel calibration — highest priority
- P0: virtual wideband controller telemetry: measured lambda, target lambda, wideband readiness, heater/status, timestamp/age, signal validity, bank/cylinder association, exhaust delay and noise in simulator.
- P0: configurable target-lambda map, explicit distinction between lambda and fuel-specific AFR conversion.
- P0: a two-mode auto-tuning architecture:
  1. Offline Mixture Map: import/replay a log, map samples by RPM/load into cells, filter steady-state observations, show measured-vs-target lambda, confidence/sample counts, and propose bounded VE changes with a preview/diff. Apply only with user approval.
  2. Online closed-loop lambda: bounded short-term trim controller to follow target under validated operating conditions; disable or freeze correction when sensor/operating state is invalid.
- P1: long-term trim accumulator separated from base VE, cell/bank-aware learning with limits, learning gain, persistence, ageing and reset; a reviewer action to merge selected learned corrections into base maps.
- P1: dual-bank lambda adaptation, per-cylinder support only when sensor allocation and exhaust layout genuinely allow it; multiple widebands over CAN, time alignment and sensor diagnostics.
- P1: fuel model compensation awareness: avoid double-correcting when STFT/LTFT are active; use pre-trim equivalent or account for existing trim explicitly; log source corrections with each proposed update.
- P1: optional interpolation and smoothing of learned cells, never fabricate data for unvisited cells; clearly show uncertainty/coverage.
- P2: guided dyno data collection, steady-state dwell/coverage tools, calibration quality scoring, mixture-map comparisons, injector characterisation import and validation.
Required learning gates: engine warm and stable, valid warmed-up wideband, sufficient dwell and sample count, bounded sensor age, synchronised engine state, negligible transient throttle/load change, no overrun fuel cut, no misfire indication, no exhaust leak indication, no active critical DTC, no knock/overboost/overtemp/low-pressure alarm, plausible fuel pressure/injector duty, no sensor saturation, clear trust boundaries for WOT/high-load cells. Under nonconforming conditions, abstain and explain why. Start in observation/recommendation-only mode; any physical real-time adaptation requires independent validation.

Test cases: lean/rich steady-state convergence, wrong injector dead-time, sensor stuck rich/lean, CAN telemetry lag, exhaust leak, sensor warm-up, misfire false-lean, acceleration enrichment, overrun cut, under-fuel pressure, cell oscillation, corrupted trim table, sparse observations, trim merge/undo, multiple banks, thermal drift.
C. Configurable base maps and setup wizard — highest priority
- P0: guided engine profile editor; displacement, cylinder count, firing order, cam/crank trigger, turbo/supercharger/NA, injector flow at specified delta-pressure, fuel/stoichiometry, regulator strategy, sensor calibration, coil type, dwell source, wideband assignment, I/O/pin mapping.
- P0: user-configurable VE map and target-lambda map from clearly labelled estimated templates, synthetic example data and/or user-provided licensed reference maps; export/import a versioned open-format calibration with CRC/hash and validation.
- P0: visual 2D table editor with axes, cell selection, keyboard shortcuts, interpolation, axis resampling, undo/redo, copy/paste and 3D surface viewer.
- P1: structured starter-template library with provenance, supported engine family, injector/sensor assumptions, verified vs estimated status and explicit unsupported combinations.
- P1: simulated cranking/idle startup wizard, sensor calibration checklists, pin conflict detector, I/O wizard, multi-fuel blending options, multiple selectable maps.
- P1: preflight report that flags missing injector data, missing trigger calibration, unsynchronised ignition reference, missing lambda readiness, questionable fuel pressure and hazardous combinations.
- P1: map versioning, semantic differences, compare/two-way merge, rollback, signed metadata, immutable snapshots.
Critical label: All generated maps are UNVALIDATED STARTING ESTIMATES — NOT AN ENGINE-SAFE TUNE until an authorised tuner verifies sensor calibration, injector characteristics, fuel pressure, mechanical configuration and ignition base offset on actual hardware.
D. Estimated ignition timing maps — design separately
- P0: editable ignition table and corrections, offline-only conservative starter templates with explicit engine/combustion/fuel assumptions, provenance, unknown where no credible reference exists.
- P0: ignition recommendation/report structure containing: source, operating region, RPM/load, proposed change, why, confidence, uncertainty, upper/lower bounds, required validation and rejection reason.
- P1: import licensed, verified engine-specific existing maps with traceability; fit smooth surfaces within known constraints, highlight interpolation/extrapolation gaps. No implied optimality.
- P1: log overlay of measured knock level/knock retard, torque, MAP, IAT, ethanol %, coolant temp, gear, boost and ignition corrections. Build knock-event review and recommendation tools for qualified tuner use.
- P2: dyno test-session assistant for supervised controlled ignition optimisation using torque and reliable knock instrumentation. Propose bounded offline changes; no unattended physical advance, especially under high load/boost.
- P3: validated cylinder-specific knock/ignition correction on physical hardware, requiring signal integrity, windowing, noise characterisation, knock thresholds and engine-specific calibration.
Never infer MBT (minimum spark for best torque), knock limit or safe timing from lambda alone. Never auto-advance spark on a vehicle based on an LLM guess.
E. Protections and diagnostics
- P0: a fault/state-machine simulator with sensor plausibility, stale signal, crank-sync loss, calibration CRC failure, watchdog, lost CAN, low oil/fuel pressure, lean-condition alert, overtemp and overboost.
- P1: configurable severity-based reactions: warning, reduced torque/boost, bounded trims, limp mode, hard limit or cut where appropriate. Include recovery/latching policy and exhaustive decision logs.
- P1: fault codes, freeze-frame snapshots, alert dashboard, sensor traceability and logged protection actions. Show fault tree analysis and emergency response design.
- P2: physical output-interlock planning, brownout/load-dump/EMC review, independent monitoring requirements and fault injection.
- P3: independently validated e-throttle, PDM, knock actuation and on-engine safety strategy; no prototype actuation without formal engineering review.
F. Motorsport and advanced control
- P1 simulator: simple open-loop boost; launch limiter; flat-shift, configurable input switches, gear/speed calculations, flex-fuel maps and compensation tables.
- P2: closed-loop boost with multiple dimensions, boost by gear/speed, traction/slip, nitrous stages, anti-lag, rotational idle, transbrake, rolling launch, torque model, VVT, staged fuel, GP PWM/logic, user-defined maths and race timer.
- P2: transmission control as a separate bounded-scope integration project; OEM CAN templates with licensing and compatibility notes; avoid inventing CAN frames.
- P3: dual electronic throttle, high-current power distribution, paddle shift/gearbox actuators, direct injection, active wastegate/motor or other high-energy actuation — independently reviewed and validated physical systems.
G. Tuning application and telemetry — differentiate on UX
- P0: desktop app (Tauri v2 Rust backend + TypeScript/React UI), native local service and virtual ECU connection; no mock controls.
- P0: live dashboard for RPM, TPS, MAP, lambda, target lambda, ignition, VE cell, fuel trims, injector PW/duty, coolant/IAT, errors; configurable sample rates and backpressure behaviour.
- P0: editable 2D/3D fuel/ignition/target-lambda maps, current-cell indicator, offline compare, autosave, export/import, undo/redo.
- P0: record/replay and CSV export; deterministic demo scenarios.
- P1: PCLink-class analytics — time plot, XY plot, histograms, per-cell mixture map coverage, linked cursors, overlays, lap/session compare, trigger scope, CAN monitor, alerts, annotation and data bookmarking.
- P1: in-app base-map/setup wizard, fault-centred diagnosis, injector characterisation library with source/provenance, guided calibration checklist and keyboard-first tuning ergonomics.
- P2: CAN configuration editor, GPS trace, custom maths, plugin/sensor adapters, wireless telemetry, firmware update infrastructure with robust rollback/authenticity protections.
- P3: real ECU connection/firmware flashing only once hardware interface and power-fail behaviour have been tested.
H. Hardware and manufacturability
- P0: feasibility trade study for STM32 family with first-class Rust HAL; evaluate STM32H7 versus automotive-focused NXP S32K based on realistic driver, timer, watchdog, CAN and build/debug maturity. Do not pretend an unsupported HAL exists. Select one demonstrable development board.
- P1: actual KiCad project for 4-cyl sequential fuel/spark, two trigger inputs, configurable analogue sensors, CAN, power protection, debug, external CAN wideband first. Actual netlist, placed components, design rules, BOM and pin assignments.
- P1: KiCad CLI ERC/DRC and issue logs; no false green status. Use approved driver IC references and reviewable power stages; require protections and documented thermal budget.
- P2: validated onboard lambda controller/conditioning; more outputs; digital inputs, data logging, PDM controller as separate validated submodule.
- P3: automotive environmental/EMC, transient, thermal and functional safety validation, prototype/harness and instrumented test rig. A KiCad file and passing DRC do not establish a safe circuit.
4. Rust-first architecture
Use a Cargo workspace, with separation between host, firmware, app and test layers.
ecu/
  Cargo.toml
  crates/
    ecu-types/            # units, identifiers, serialisable calibration models
    ecu-calibration/      # N-D tables, interpolation, validation, versioning
    ecu-fuel/             # VE, modeled fueling, injector compensation
    ecu-ignition/         # spark scheduling requests, timing tables, provenance
    ecu-trigger/          # pattern decoder and sync state machine
    ecu-scheduler/        # deterministic event computation abstraction
    ecu-lambda/           # STFT, LTFT, offline mixture map, learning gates
    ecu-protection/       # independent protection logic and fault decisions
    ecu-protocol/         # versioned framed protocol, CRC and message schemas
    ecu-simulator/        # physics-lite engine, sensors, exhaust delay, faults
    ecu-diagnostics/      # DTCs, structured logs and playback schemas
  firmware/
    stm32/                # no_std, exact MCU and HAL after feasibility review
  apps/
    tuning-studio/        # Tauri Rust commands + React/TypeScript UI
    ecu-sim-cli/          # headless virtual ECU and scripted scenarios
  hardware/
    kicad/                # real schematics, PCB, BOM, pinout, ERC/DRC
  tests/
    fixtures/
    scenarios/
  docs/
    evidence.md
    feature-matrix.md
    safety-case.md
    engineering-log.md
    hardware.md
    calibration.md
Mandatory engineering constraints:
- Portable core crates compile for std host simulation; embedded-relevant crates compile under #![no_std] (feature gate as needed).
- Prefer unit-safe quantities, explicit fixed-precision/range checks, bounded memory, no heap allocation in hard real-time paths, bounded schedulability, safe concurrency and deterministic state transitions.
- ISR timing-critical capture/scheduling must use hardware capture-compare peripherals. Async Rust can service communications, UI-facing state and slow peripherals but must not be mistaken for a deterministic ignition timing guarantee.
- Define message schema and explicit version negotiation; no blind writes to ECU memory; immutable calibration snapshots and atomic commits with CRC and rollback.
- Use serde where appropriate on the host, and embedded-appropriate wire formats (postcard or equivalently justified) subject to measured costs.
- cargo fmt, clippy, unit/property tests, simulation tests, host-to-target compile checks, static checks, documentation of missing tools.
- Avoid unnecessary unsafe. Audit each unsafe block, hardware interface and atomic/shared state.
- Rust does not confer functional safety certification. Independent system-level validation remains mandatory.
5. Specific algorithms and acceptance tests
Lambda offline mixture-map acceptance
Use a synthetic log with known VE map error and simulated wideband delay. The system must:
1. Record all sample inputs, sensor status, timestamp, active trims and bank.
2. Filter invalid samples through documented, user-visible gates.
3. Associate valid measurements with RPM/load map cells using appropriate time alignment.
4. Compute aggregate observed/target lambda error with confidence/sample coverage.
5. Generate proposed, bounded VE adjustments; clearly distinguish measured cells and interpolated cells.
6. Compare previewed map vs original and support accept/reject/undo.
7. Never apply a proposal directly to real injector outputs in the virtual-first milestone.
8. Flag uncertain samples rather than learning from them.
9. Reproduce the same answer for the same seeded log.
Note: a conceptual proportional VE factor for steady state is VE_new ≈ VE_old × (lambda_measured / lambda_target) assuming the fuel model, trims, injector calibration and conditions are consistent. This is not a complete real-world control algorithm. Validate sign conventions, units, sensor latency and trims mathematically and with property tests before using it even in simulation.
Closed-loop and long-term correction acceptance
Test bounded STFT response, integrator windup prevention, delayed sensor dynamics, learning gates, bank separation, saturations, loss/recovery of sensor validity, short-term vs long-term coexistence and writeback merge/undo. No unconditional adaptation at WOT. Preserve independent base and learned tables.
Ignition recommendations acceptance
Given an input profile without verified source timing data, return INSUFFICIENT EVIDENCE rather than inventing an ignition map. Given an appropriately licensed verified reference map, allow offline resampling with strict origin metadata, warning flags for different compression ratio/fuel/forced induction and unvalidated status. No automatically generated spark-advance instructions to real hardware.
Calibration wizard acceptance
A valid simulator profile produces a versioned configuration file, complete semantic validation report, non-empty maps explicitly marked estimated, and a running virtual ECU. An invalid profile gives actionable errors. Hard block undefined crank patterns, mismatched injector/driver types, missing I/O or sensor calibration and incompatible units.
Deterministic simulation acceptance
Simulate cranking → sync → idle → load transient → sensor failure → protection → recovery and save a telemetry log that can be replayed in the tuning app. Check event ordering, state transitions and simulated output scheduling invariants. Test crank speed acceleration/deceleration, rollover and missing teeth.
6. Execution sequence within ~US$26
1. Inventory: inspect available compiler, Rust target/HAL, Node/Tauri dependencies, KiCad CLI and repository; create real Git repo; record licences and constraints. Keep research targeted.
2. First end-to-end vertical slice: Rust model + Rust simulator executable + stable protocol + Tauri UI showing live virtual RPM/MAP/lambda and editable VE map. Build and run tests.
3. Automated fuel tuning slice: simulated wideband with delay → offline Mixture Map → bounded cell suggestions → map diff → accept/reject → repeatable tests.
4. Configuration slice: engine profile and input/outputs wizard, base VE/lambda templates, timing-map estimate with provenance safeguards, calibration import/export and undo.
5. Protection/diagnostics: faults, priorities, structured logging, playback, state machine tests.
6. Embedded feasibility: no_std compile, choose specific supported MCU and build a demonstrable HAL skeleton with basic timer/event interfaces; do not manufacture unsupported peripherals.
7. PCB feasibility: create a valid KiCad project and iterate ERC/DRC, BOM and pinout; mark unreviewed areas.
8. Extras: only after passing integrated tests, add advanced feature models and graphical polish.
Budget: use DeepSeek tokens economically; target US$22 spend and reserve US$4 for review, but if there is no billing API, acknowledge spend is unknown. Require periodic concise progress checkpoints and final status with exact commands/results. Stop for genuine external blockers or at an externally enforced cap; do not assume a prompt can force a hard payment limit.
7. OpenCode master execution instruction (paste this section into OpenCode)
You are an autonomous, senior automotive controls, embedded Rust, PCB and tuning software engineering team. Your mission is to create an original open-source, Rust-first ECU development platform informed by Speeduino, benchmarking Haltech Nexus R5 and Link G5 Voodoo Pro V2/PCLink. The comprehensive requirements and citations are in this Markdown document. Read the document fully and turn the feature-parity backlog into tracked issues or a feature matrix.

Do not merely write a plan. Build a working, host-executable, integrated demonstration now. Make technically defensible decisions, but never claim unverified physical safety, high-performance capability, test success, or compatibility.

Make the first milestone a real Tauri-based tuning studio connected over a versioned protocol to a Rust virtual ECU, demonstrating crank/RPM simulation, VE and target-lambda tables, live telemetry, an offline lambda Mixture Map that produces bounded reviewable fuel-map corrections, starter engine configuration, editable ignition map and a deterministic engine-protection response. Include replayable synthetic logs and tests. I want real controls, no decorative fake UI.

Use a Rust Cargo workspace; no_std portable core and an STM32-class hardware target if feasible, Rust host simulator, Rust Tauri backend and React/TypeScript front end. Establish Rust code quality (cargo fmt, cargo clippy, tests) and automatic CI. Speeduino's existing C++ is reference material to investigate, not something to blindly translate. Honour its GPL-2.0 and the official hardware's CERN-OHL-S-2.0 when deriving work.

Keep online STFT, offline Mixture Map and LTFT as distinct modules. Do not contaminate learning with misfires, exhaust leaks, delayed or invalid wideband sensors, short-lived transients, overrun fuel cut, knock, active DTCs or untrusted high-load measurements. Implement observable gate explanations, confidence coverage, update bounds, rollback and bank handling. The first version must only suggest/offline-apply fuel corrections in a virtual ECU; physical auto-learning is deferred until reviewed.

Support user-configurable engine profiles and base-map templates, but tag all guesses as unvalidated. Base ignition timing estimates require credible licensed sources, provenance and explicit uncertainty. Lambda cannot determine safe ignition timing. Do not automatically advance physical ignition. Include a physical timing-light calibration checklist as a requirement for a future hardware-ready installation.

Benchmark all major feature families: sequential/staged injection, VE and multifuel equations, flex fuel, configurable 3D/4D maps, knock monitoring/retard, boost, VVT, idle, launch, traction, anti-lag, nitrous, shift strategies, CAN, logging, power distribution, protection, DBW, transmission and diagnostics. Record each as Verified/Unverified benchmark and Implemented/Simulated/Planned/Blocked in our project. Treat Link's 10 Nov 2026 PCLink additions as announced unless a released version confirms them.

Produce real KiCad schematic/PCB files, but first focus on a 4-cylinder protected port-injection ECU with external CAN wideband. Do not pretend electrical-rule checks are functional-safety validation. Defer physical electronic throttle, direct injection, onboard high-current PDM and other safety-critical actuation until independent engineering reviews.

At each work unit: implement, compile, test, fix, record factual progress, commit cleanly if Git is available, then proceed to the next most valuable unfinished work. Ask no routine questions. Do not require me to approve ordinary engineering decisions. Use autonomy for local development, not purchasing equipment, deploying services or controlling vehicles. Reserve part of my approximately US$26 DeepSeek budget for final verification; do not assert spend without billing data.

BEGIN WITH THE INTEGRATED RUST VIRTUAL ECU + LAMBDA AUTO-CALIBRATION + TUNING UI. THEN EXPAND.
8. Final report format expected from OpenCode
For each milestone show:
- Working path and exact launch command.
- What is real, simulated, stubbed, unimplemented or hardware-blocked.
- Tests run, raw command outcomes and failures.
- Licensing/provenance notes.
- Risk and independent engineering review blockers.
- Next highest-value task and estimated API budget only if metered.
