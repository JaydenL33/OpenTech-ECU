# Public release checklist

| Field | Value |
|---|---|
| Status | Prepared for a public branch; not pushed |
| Date | 2026-11-13 |
| Base | `master` at `447fae20` |
| Working branch | `feature/support-ui-harness` (local only) |
| Gate | Gate 0 host software only |

This checklist records what was checked before making the branch available
publicly. It is not a safety approval and does not promote any release gate.

## 1. Provenance and licensing

- [x] Original Rust code is declared GPL-2.0-or-later in the workspace and
  package metadata.
- [x] The retained Speeduino C++ tree keeps its original GPL-2.0-or-later
  notices and is not relicensed.
- [x] `NOTICE.md` explains the Speeduino relationship: based on Speeduino,
  not a fork, C++ retained as a behavioural oracle.
- [x] The bench KiCad package carries its own CERN-OHL-S-2.0 licence file and
  is marked unreviewed.
- [x] No proprietary OEM firmware, connector drawings, calibration data,
  encryption key or vendor software is tracked.

## 2. Vendor and map data

- [x] No vendor-specific map container or extracted map table is tracked.
- [x] Local proprietary map archives are excluded through
  `.git/info/exclude`; they are not part of the public tree.
- [x] The harness page is generated from our own unreviewed KiCad exports.
- [x] No vendor pinout, cavity number or harness assignment is presented as
  verified or usable.
- [x] No vendor product documentation, logo or screenshot is included.

## 3. Secrets and local environment

- [x] No API keys, tokens, private keys or `.env` files are tracked.
- [x] Documentation uses placeholders such as `DEEPSEEK_API_KEY=...`.
- [x] No `/home/...` or `/tmp/...` developer paths remain in tracked source
  or documentation.
- [x] Build outputs (`target/`, `dist/`, `runs/`), dependencies
  (`node_modules/`) and local map archives are ignored.

## 4. Build, test and CI

- [x] `cargo fmt --all -- --check` passes.
- [x] `cargo clippy --workspace --all-targets -- -D warnings` passes.
- [x] `cargo test --workspace` passes: 130 tests, 0 failures.
- [x] Core `no_std` checks pass.
- [x] `thumbv7em-none-eabihf` firmware skeleton check passes.
- [x] Tuning Studio `npm run build` passes.
- [x] Copilot `npx tsc --noEmit` passes.
- [x] `.github/workflows/opentech-ci.yml` runs Rust, UI and copilot checks.
- [x] Headless Chromium smoke test covers the dashboard, maps, Harness page,
  Mixture Map, Setup, Diagnostics, Logs and Copilot.

CI has not been observed running on GitHub yet. A green local run is not a CI
attestation.

## 5. Claims and gaps

- [x] README states Gate 0 only and distinguishes implemented, simulated,
  unreviewed and blocked work.
- [x] The physical output interlock is still engaged; no injector or ignition
  hardware path exists.
- [x] The bench PCB is documented as unrouted, with failing DRC and no gerbers.
- [x] Evo VIII pinout, trigger, injector, ignition and harness data are marked
  UNVERIFIED.
- [x] No engine-safe, road-legal, EMC, automotive-qualified or plug-and-play
  claim is made.
- [x] No LLM write path, ignition advice or safety-gate bypass exists.

## 6. Before pushing

1. Run the exact commands in `evidence/VERIFICATION-SUPPORT-UI.txt`.
2. Confirm `git status` is clean and no local map archive is tracked.
3. Review the final diff for vendor names or local paths.
4. Push the branch only after that review.
5. Treat any open draft UI-parity branch as a separate review item; rebase or
   merge it intentionally rather than mixing unreviewed changes.

## 7. What this checklist does not cover

- Independent code review.
- Bench hardware, EMC, thermal or safety validation.
- A reviewed PCB, BOM or manufacturing package.
- Evo VIII vehicle compatibility.
- Any calibration suitability or engine-safety claim.
