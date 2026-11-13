
## ECU swarm continuity and provenance supplement

This repository is a Rust-first, Speeduino-derived, four-cylinder ECU development project. The primary eventual vehicle is a JDM Mitsubishi Lancer Evolution VIII GSR (six-speed). The current physical target is a low-cost, buildable **bench prototype candidate**, not a validated replacement ECU.

- Read `PROJECT_DIRECTIVE.md` before working, as well as current task and audit files. Preserve its requirements: Speeduino v0.4/other baseline assessment; already routed native KiCad PCB; manufacturing-for-review exports; AUD BOM under A$250 if credible; Rust simulation and ECU control; tuning UI, offline lambda VE auto-tuning; Nexus/Link feature parity roadmap; independent auditability.
- The user's addition that hardware must eventually be modified to provide Nexus-like functionality is a roadmap requirement, **not permission to disrupt a proven baseline or block the first bench PCB milestone**. Separate prototype v1 from future expanded revisions.
- Pick a specific task with verifiable output; make code/PCB changes, build/test, log results, update task/evidence records and commit coherent changes.
- Do not stop at planning or reiterating existing safety warnings. Convert each uncompleted design aspect into actionable blockers and progress on autonomous H0–H3 items.
- Do not infer that Rust embedded firmware runs on the selected Arduino/MCU. Respect upstream Speeduino baseline and validate target/toolchain compatibility. Maintain Rust host-sim modules independently.
- Never invent PCB validation, test outcomes, prices, connector pinouts, calibration maps, or model usage figures. Keep confidence and provenance explicit.
- Separate each workstream's branch/files from others. Do not change files in another worker's ownership unless instructed after integration.
- Maintain `AUDIT/` with sources, traceability, exact tool commands, tool versions, raw verification outputs, electrical/firmware risks, and current H0–H9 gate status.
- H4 fabrication requires external engineer review and explicit human approval; H5+ require physical evidence. Road testing requires further independent validation. AI cannot grant any of these approvals.
- No vehicle connection, physical output actuation, purchases, credentials access, secret exfiltration, force-push, or deletion of unrelated files.
- If a CLI session finishes, the orchestrator may start another bounded round using repository state. Never fake continuous execution or hide spending.
