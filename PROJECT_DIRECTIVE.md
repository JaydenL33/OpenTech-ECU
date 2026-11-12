# CRITICAL PROJECT COURSE CORRECTION
## STOP PRODUCING ONLY PCB DRAFTS. BUILD A REAL, LOW-COST SPEEDUINO-DERIVED ECU PROTOTYPE.

You are continuing development of our Rust-first, open-source engine management platform, with the eventual goal of a fully functional aftermarket ECU for my Japanese-specification Mitsubishi Lancer Evolution VIII GSR, equipped with a six-speed manual transmission.

We have already established ambitious requirements around:

- Haltech Nexus and Link ECU feature parity.
- Rust embedded firmware and supporting libraries.
- Modern desktop tuning software.
- Wideband lambda auto-tuning.
- Base-map generation and management.
- Ignition map estimation and validation.
- Advanced engine management.
- Independent engineering auditability.
- Vehicle-specific compatibility.
- Hardware-in-the-loop simulation.
- Automotive safety and reliability.

**ALL OF THESE REQUIREMENTS REMAIN VALID.**

However, our current development strategy has a serious problem.

The project repeatedly reports:

> STATUS: UNREVIEWED BENCH SCHEMATIC + UNROUTED PCB DRAFT. NOT MANUFACTURABLE. NOT AUTOMOTIVE-QUALIFIED. NOT AN EVO VIII PLUG-IN. DO NOT FABRICATE OR CONNECT TO A VEHICLE.

I understand these limitations.

I am NOT asking you to falsely certify the PCB or ignore automotive safety.

I am asking you to actively engineer the design towards becoming a genuinely buildable, affordable, testable hardware prototype rather than repeatedly reporting the same limitations.

**The correct response to an unrouted PCB is to establish a valid baseline, complete its design, run checks and resolve blockers. Not merely write another warning.**

# 1. OUR NEW IMMEDIATE OBJECTIVE

The immediate deliverable is:

**A real, low-cost, manufacturable-for-review, Speeduino-derived four-cylinder ECU prototype that can be independently reviewed, manufactured after approval, assembled, powered safely on a bench and tested with simulated engine signals and dummy loads.**

"Manufacturable-for-review" means that the design files and fabrication package are technically complete enough for an independent engineer and PCB manufacturer to examine. It does not mean that fabrication, energising, or vehicle installation has been approved.

This is not yet required to be automotive-qualified or plug-and-play with the Mitsubishi Evo VIII.

Those remain future milestones.

The first board must:

1. Be based as closely as practical on proven Speeduino hardware.
2. Have real and complete KiCad schematics.
3. Have an actually routed PCB.
4. Have a credible bill of materials with purchasable parts.
5. Have consistent electrical symbols and footprints.
6. Include manufacturability checks.
7. Include mechanical and assembly documentation.
8. Have a hardware-software interface compatible with the selected Speeduino baseline.
9. Support safe, current-limited bench verification after review.
10. Have a clear test plan and fault-isolation procedures.

Do not design a new flagship Haltech-class ECU from scratch for this milestone.

Reuse existing engineering wherever possible.

# 2. REUSE THE ACTUAL SPEEDUINO HARDWARE

This is the most important architectural change.

Official references:

https://github.com/speeduino/Hardware

https://github.com/speeduino/speeduino

https://wiki.speeduino.com/

The official Speeduino hardware repository already contains KiCad designs for boards including:

- Speeduino v0.3.
- Speeduino v0.4.
- DropBear.
- Supporting VR conditioner hardware.

Study the actual available schematic and PCB source files.

Do not merely read their README files.

Do not generate a fresh PCB from memory when a suitable routed open-source design already exists.

## Evaluate the v0.4 family first

Speeduino v0.4 is particularly interesting because it was designed as a lower-cost board with four injector channels, four ignition channels and an IDC40 interface intended for external wiring integration.

Investigate the precise capabilities and limitations of the actual available revisions.

Identify the specific revision you recommend using.

Do not assume the newest-looking board is automatically the best choice.

Compare v0.3, v0.4 and other appropriate designs against:

- Four-cylinder engine support.
- Actual source-file completeness.
- Existing PCB routing.
- Firmware compatibility.
- Documentation quality.
- Component availability.
- Assembly difficulty.
- Expected fabrication cost.
- Existing interface connector.
- Suitability for bench testing.
- Potential for an Evo VIII adapter harness.

Choose the simplest technically defensible baseline.

## MINIMISE REDESIGN

Prefer the following order:

**Option A — Reuse an existing Speeduino board unchanged.**

Use a published board revision with established Speeduino firmware support and a complete, inspectable hardware design.

**Option B — Make minimal modifications to an existing Speeduino PCB.**

Fork the selected design and change only what is necessary for the prototype.

Preserve proven circuitry and routing when appropriate.

**Option C — Develop a new board based on the existing circuits.**

Only choose this when there is a documented technical reason why Options A and B cannot meet the current requirements.

Do not choose Option C simply because an original design seems more exciting.

Every changed circuit must be identified and separately verified. It must be requring modification to give us Nexus functionality.


# 3.  Rust ECU implementation

Continue developing:

- Rust `no_std` control libraries.
- Fuel calculation.
- Ignition calculation.
- Trigger decoding.
- Map interpolation.
- Lambda-based fuel correction.
- Diagnostics.
- Communications.
- Safety state machines.
- Hardware abstraction.
- Virtual ECU simulator.

Run Rust components against reference vectors and, where appropriate, observed behaviour from existing Speeduino implementations.

Where meaningful, use differential testing to reveal disagreements.

Document all material differences.

The two firmware paths should converge through explicit compatibility tests and evidence, not assumptions.

Do not imply that an Arduino Mega can run the Rust firmware without verifying the target architecture, available memory, toolchain and peripheral support.

A Rust firmware port can eventually target more capable hardware in a later revision.

# 4. BUILD A LOW-COST HARDWARE DEVELOPMENT KIT

I want physical hardware that I can afford to buy and test at home.

Optimise the first prototype for:

- Low cost.
- Component availability.
- Simplicity.
- Reusability.
- Easy assembly.
- Easy debugging.
- Replaceable modules.
- Clear test points.
- Commonly available development tools.

Investigate two practical acquisition paths:

**Path 1: Purchase an existing compatible Speeduino board or kit.**

**Path 2: Manufacture and assemble the selected official or minimally modified PCB design.**

Provide a cost comparison.

Use AUD as the primary currency.

Aim for an ECU hardware cost under A$250 if reasonably achievable, excluding general-purpose lab tools.

If that is unrealistic, give the lowest technically credible cost and explain why.

Use actual supplier prices when accessible; clearly date quotations and identify missing shipping, taxes or assembly costs.

Do not fabricate supplier pricing.

## Required bill of materials

Include:

- Manufacturer part number.
- Component description.
- Quantity.
- Footprint.
- Preferred supplier.
- Alternative supplier.
- Estimated or quoted price.
- Availability.
- Substitution restrictions.
- Assembly method.
- Risk classification.

Do not use placeholder components in the released manufacturing BOM.

Do not substitute automotive power, analogue, trigger-conditioning or output-driver components based only on pin compatibility.

## Bench test accessories

Design or specify:

- Current-limited power source.
- Fuses and protective circuitry.
- Power switching and output isolation.
- Simulated sensor inputs.
- Crank/cam waveform generation.
- Safe dummy loads.
- USB or serial communications.
- Optional CAN interface.
- Measurement points for oscilloscope testing.
- Emergency power disconnect.

The initial test setup must not require a running car.

Do not require a dyno, engine, vehicle wiring harness or specialist automotive simulator to begin basic electronic testing.

A basic USB logic analyser, multimeter and appropriately rated oscilloscope can be useful, but recognise which tests require more capable equipment.

# 5. FINISH THE KICAD PCB DESIGN

Inspect our existing PCB work first.

Determine whether it is better to repair our current design or replace it with an imported official Speeduino baseline.

Do not preserve a poor design merely because time has already been spent on it.

I want ACTUAL KiCad source files.

The deliverables must include:

- A valid `.kicad_pro` project.
- Native `.kicad_sch` schematic files.
- Native `.kicad_pcb` layout files.
- Fully assigned footprints.
- Resolved electrical connectivity.
- Intended signal routing completed.
- Appropriate copper planes and return paths.
- Grounding and power distribution review.
- Board outline.
- Mounting holes.
- Reference designators.
- Fabrication drawings.
- Assembly drawings.
- Drill files.
- Gerbers.
- BOM.
- Pick-and-place files where assembly automation is intended.
- A complete export and validation script.

Use native KiCad tools and libraries where practical.

Do not fabricate a false representation of KiCad output.

## Required validation

Run actual available tools, including:

- KiCad ERC.
- KiCad DRC.
- Unrouted-net checking.
- Netlist/connectivity checks.
- Footprint validation.
- Board-outline checks.
- Mechanical clearance review.
- Gerber generation.
- Drill-file generation.

Capture command outputs and exit codes.

Track every unresolved violation.

Do not silence errors by globally disabling rules, removing test cases or hiding warnings.

Where inherited upstream designs contain known exceptions, explain and justify each one.

An ERC/DRC pass is necessary evidence of design consistency, but it is not evidence of automotive electrical reliability.

## Routing requirement

If the selected Speeduino hardware design already contains valid routing, preserve and validate it.

If routing is incomplete, complete it.

Do not finish with an unrouted PCB and claim the hardware milestone is done.

If available tools cannot route or edit the PCB adequately, clearly identify the missing capability and stop making false claims of completion.

Still deliver the best complete, verifiable upstream design as a separate baseline rather than leaving us with only an unfinished custom layout.

# 6. ESTABLISH A REALISTIC HARDWARE READINESS LADDER

Use these exact milestone categories.

### H0 — Verified upstream reference

An exact Speeduino PCB revision, firmware revision, source commit, licence and documented build path have been selected.

### H1 — Complete PCB design package

Native KiCad files exist. Schematics, footprints, board outline and intended routing are complete.

### H2 — Engineering checks completed

ERC, DRC, connectivity and manufacturing export checks have been executed, with evidence and outstanding exceptions documented.

### H3 — Independent fabrication review candidate

A complete release package is available for another AI model and a qualified hardware reviewer to inspect.

This status is not permission to fabricate.

### H4 — Fabrication approved

Independent review of the final revision has been completed, critical design issues resolved and a human has explicitly approved the board for prototype manufacture.

### H5 — Assembled hardware

The real PCB has been manufactured, populated and inspected.

### H6 — Powered bench testing

Physical power-up, communications, I/O tests, timing tests and fault injection have been performed and recorded.

### H7 — Engine-system validation candidate

An independently reviewed hardware/firmware combination is ready for professionally supervised engine-system testing.

### H8 — Vehicle-specific integration

The actual Evo VIII wiring, signals and mechanical requirements are confirmed; all vehicle interface hazards are reviewed.

### H9 — Qualified vehicle-operation candidate

Relevant engineering, calibration, functional safety and regulatory reviews have been independently completed.

**You may progress H0 through H3 autonomously through software engineering and design work.**

H4 and above require external physical evidence and/or explicit human approval.

Never claim physical tests have passed without physical test data.

Never automatically approve a higher release gate.

# 7. STOP REPEATING WARNINGS WITHOUT RESOLVING THEM

The repeated warning about an unrouted, non-manufacturable board is not useful on its own.

Replace it with an actionable engineering blocker system.

Create:

`HARDWARE_BLOCKERS.md`

For every blocker, record:

- Unique blocker ID.
- Severity.
- Affected files.
- Exact problem.
- Evidence.
- Required action.
- Current owner or workstream.
- Whether it is solvable autonomously.
- Verification command or review requirement.
- Status.
- Commit that resolved it, if applicable.

Examples:

- Unassigned footprint → Assign a verified footprint.
- Unrouted PCB net → Complete or restore routing.
- Missing BOM → Export and verify component data.
- Unknown component availability → Research supply alternatives.
- ERC error → Correct the circuit or document a valid exception.
- Unknown Evo VIII factory pinout → Keep vehicle integration blocked, but continue bench ECU development.
- Automotive qualification incomplete → Keep vehicle operation blocked, but do not block software simulation or a separately reviewed bench PCB design.

**Do not let an H8 vehicle-compatibility requirement prevent completion of H1 or H2.**

Never delete or misclassify a legitimate safety blocker merely to make the progress dashboard look better.

# 8. INTEGRATE WITH THE EXISTING SIMULATOR

The simulator must be an engineering instrument, not a visual demonstration.

Continue implementing:

- Virtual four-cylinder engine.
- Crank/cam trigger generation.
- RPM and load scenarios.
- MAP, TPS, IAT and coolant sensors.
- Lambda measurements.
- Sensor latency and noise.
- Injector scheduling verification.
- Ignition scheduling verification.
- Fault injection.
- Virtual ECU telemetry.
- Tuning software communications.
- Calibration changes.
- Log playback.

Develop a pathway for exporting simulated crank/cam waveforms to supported signal-generation hardware.

Provide test vectors with expected timing relationships and measurable tolerances.

Do not assume simulation proves that external ignition or injector hardware behaves correctly.

The simulator should support repeatable automated regression tests.

# 9. KEEP OUR HALTECH AND LINK GOALS

Do not abandon the ambitious long-term feature set.

Maintain:

- Advanced fuel modelling.
- Automatic VE map tuning using wideband lambda measurements.
- Short-term and long-term fuel trims.
- Configurable base maps.
- Engine-specific setup wizards.
- Ignition map recommendations.
- Knock analysis.
- Boost control.
- VVT.
- Flex fuel.
- Data logging.
- CAN networking.
- Advanced safety monitoring.
- Motorsport functions.
- Professional tuning interface.

But decouple future software functionality from the first PCB milestone.

The first physical board does not need every Nexus R5 feature.

Use a capabilities matrix distinguishing:

- Supported by the baseline hardware.
- Supported by existing firmware.
- Implemented in Rust.
- Simulation-only.
- Requires hardware expansion.
- Requires independent validation.
- Future feature.

Never claim a feature is hardware-supported merely because a corresponding tuning-screen control exists.

## Automatic tuning safeguards

Continue developing wideband-based VE correction, but initially operate in offline analysis, simulation or explicitly supervised modes.

Fuel corrections must consider sensor validity, transient conditions, measurement delay, misfire, exhaust leaks and correction limits.

Do not automatically write unverified calibration changes to a running engine.

Ignition estimates must remain separate from lambda-based fuel correction and must not be described as knock-safe or ready to run.

# 10. MITSUBISHI EVO VIII REMAINS THE TARGET VEHICLE

My actual intended vehicle is:

- Japanese-specification Mitsubishi Lancer Evolution VIII GSR.
- 4G63 turbo engine.
- Six-speed manual transmission.

Research authentic factory wiring and sensor documentation.

But do not allow the missing exact ECU part number, connector or harness pinout to block the generic prototype.

Develop a modular interface architecture:

**Speeduino-derived core ECU → Bench connector/breakout → Future Evo VIII-specific harness adapter.**

Do not guess the factory ECU connector or wiring.

Document unresolved vehicle-specific data.

Do not connect the prototype to the Evo VIII until the real vehicle interface has been independently verified.

# 11. EXTREME AUDITABILITY IS STILL MANDATORY

I intend to have this project reviewed by stronger AI models and appropriately qualified automotive electronics engineers.

Every important hardware and software decision must be inspectable.

Maintain:

`AUDIT/`

With:

- `UPSTREAM_PROVENANCE.md`
- `HARDWARE_BASELINE.md`
- `HARDWARE_DIFF.md`
- `BOM_REVIEW.md`
- `PCB_CHECK_RESULTS.md`
- `REQUIREMENTS_TRACEABILITY.md`
- `FIRMWARE_TEST_EVIDENCE.md`
- `BENCH_TEST_PLAN.md`
- `SAFETY_CASE.md`
- `OPEN_HAZARDS.md`
- `INDEPENDENT_REVIEW_GUIDE.md`

Include exact upstream commit hashes.

Record modifications to Speeduino files.

For every circuit modified from upstream, explain:

1. What changed.
2. Why it changed.
3. The electrical effect.
4. Relevant component ratings.
5. Expected operating limits.
6. Failure behaviour.
7. The tests performed.
8. Remaining review requirements.

A reviewer should be able to reproduce the hardware design outputs and firmware builds without trusting the original AI developer.

Do not confuse successful automated checks with independent safety approval.

Preserve source and licence obligations, including applicable Speeduino GPL and CERN-OHL-S requirements.

# 12. RESOURCE MANAGEMENT

We are using DeepSeek through OpenCode with a limited API budget.

Prioritise engineering value per token.

Avoid:

- Repeated full-repository analysis.
- Rewriting existing documentation without a functional change.
- Creating speculative features with no integration.
- Inventing alternative PCB designs unnecessarily.
- Endless architecture discussions.
- Repeated safety disclaimers with no corrective work.
- Replacing mature upstream implementations without justification.

Prefer:

- Reusing working source files.
- Running meaningful verification tools.
- Making small changes.
- Fixing identified failures.
- Producing reproducible outputs.
- Maintaining concise, evidence-backed progress records.

Do not assume the originally available API credit remains unspent.

If exact remaining credit cannot be determined, do not invent a balance.

# 13. IMMEDIATE ACTION PLAN

Do the following NOW, in order.

### Step 1 — Audit our existing repository

Identify what we already have.

Inspect the current KiCad design, firmware, simulator, Rust modules, tuning application and test scripts.

Keep working components.

Identify incomplete and speculative components.

### Step 2 — Acquire official Speeduino design sources

Retrieve the official hardware repository.

Evaluate v0.4 and other appropriate boards.

Select a specific source commit and board revision.

Explain why that board is the best starting point.

### Step 3 — Create a preserved upstream hardware baseline

Store the selected original design in a clearly identified location.

Do not corrupt or overwrite upstream source files.

Create a reproducible source acquisition procedure.

Record licences and provenance.

### Step 4 — Compare our design against Speeduino

Identify which of our custom circuits are already implemented in the Speeduino design.

Eliminate unnecessary redesign.

Choose which parts, if any, genuinely require modification.

### Step 5 — Deliver a complete PCB design

Prioritise an intact, already routed Speeduino design over a speculative new PCB.

Validate the selected project with available KiCad tools.

Repair actual defects or documented compatibility issues.

Export a complete reviewable manufacturing package.

### Step 6 — Deliver a credible purchasing plan

Produce a real BOM with sourceable parts.

Compare purchasing an assembled Speeduino board against building one from the KiCad design.

Optimise for low cost and minimal risk.

Do not buy anything.

### Step 7 — Establish firmware compatibility

Build the corresponding Speeduino firmware.

Document supported development hardware.

Preserve it as a known reference for the Rust development effort.

### Step 8 — Integrate the Rust simulator and tuning software

Make the virtual ECU and calibration application useful for testing the selected hardware architecture.

Add regression tests and reference comparisons.

### Step 9 — Produce bench-test documentation

Document how to inspect, power and test the physical prototype after independent design review and approved assembly.

Include fault injection and measurement requirements.

### Step 10 — Generate independent review materials

Package the hardware design, validation evidence, BOM, unresolved hazards and relevant firmware references for external audit.

# 14. REQUIRED FINAL DELIVERABLES

Create the following structure, adapting it to the repository where necessary:

```text
hardware/
  upstream-speeduino/
  prototype-v1/
    schematics/
    pcb/
    manufacturing/
      gerbers/
      drill/
      bom/
      assembly/
    verification/
  bench-interface/

firmware/
  upstream-speeduino/
  rust-ecu-core/

simulator/
tuning-app/
tests/

docs/
  HARDWARE_BASELINE.md
  HARDWARE_BLOCKERS.md
  HARDWARE_COST.md
  HARDWARE_BUILD_GUIDE.md
  HARDWARE_TEST_PLAN.md
  EVO8_COMPATIBILITY.md
  DEVELOPMENT_STATUS.md

AUDIT/
  UPSTREAM_PROVENANCE.md
  HARDWARE_DIFF.md
  PCB_CHECK_RESULTS.md
  BOM_REVIEW.md
  REQUIREMENTS_TRACEABILITY.md
  FIRMWARE_TEST_EVIDENCE.md
  SAFETY_CASE.md
  OPEN_HAZARDS.md
  INDEPENDENT_REVIEW_GUIDE.md
```

Adapt existing directories rather than duplicating working code.

## DEFINITION OF SUCCESS FOR THIS DEVELOPMENT PASS

At the end, I should have:

1. An identified, genuine Speeduino hardware reference.
2. A preserved or minimally modified routed PCB design.
3. Actual KiCad validation results.
4. A manufacturing package suitable for independent review.
5. A component sourcing and estimated-cost report.
6. A known firmware build path.
7. A documented and executable virtual ECU test path.
8. A practical home bench-testing plan.
9. A concise list of remaining blockers.
10. A clear distinction between reviewable design, approved fabrication, physical bench testing and eventual vehicle suitability.

Do not tell me that the entire project is finished.

Do not tell me the prototype is automotive-qualified.

Do not claim manufacturing readiness without the relevant design evidence.

But also do not leave us with another completely unrouted theoretical PCB.

**Make practical, verifiable engineering progress towards real hardware.**

If a particular custom design cannot be completed with available tools, adopt an actual suitable Speeduino board as the prototype baseline, demonstrate that its source and fabrication outputs are available, and continue from there.

I care more about having a real, affordable, inspectable and testable ECU development platform than a completely original but unfinished PCB.

**EXECUTE THE COURSE CORRECTION NOW.**