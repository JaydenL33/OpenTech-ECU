# Tuning workstream — evidence log

Owner: tuning workstream (branch `swarm/tuning-20261112T222821Z`). Separate file so it does not
collide with other workers' edits to `TEST_EVIDENCE.md`. Gate status is unchanged by anything
here: no H4+ approval is implied, and nothing here touches hardware, KiCad or firmware
real-time code.

## Round 1 (2026-11-13) — latency-window gate for offline lambda VE analysis

### Starting state (read from disk, not re-done)

Already present before this round: virtual ECU + scenarios (`ecu-simulator`, `/api/sim/*`),
editable calibration tables with history/undo (`/api/calibration/*`), starter-map wizard,
offline Mixture Map (`ecu-lambda::mixture_map`) with proposal → diff → explicit apply →
undo, shared learning gate, ignition recommendation module (`ecu-ignition`, kept separate from
fuel). Provenance in `docs/` and `TEST_EVIDENCE.md` §10 lists what remains untested.

### Defect found by code reading

`mixture_map::analyze` evaluated transient, misfire, fuel-cut and trim checks only on the
**current** frame, but a wideband sample at time *t* reflects fuel delivered at
*t − exhaust_delay*. A throttle blip, misfire, fuel cut, loop engage or trim ramp inside that
span left later, steady-looking frames eligible, so their lambda error was attributed to a cell
whose fuel had been disturbed.

### Change

`ecu/crates/ecu-lambda/src/mixture_map.rs`:

- New options `transient_holdoff_ms` (default 250) and `max_trim_swing` (default 0.06), both
  `#[serde(default)]` so stored/JSON options without them still load. **Both defaults are
  unvalidated engineering guesses**, not calibrated values.
- A sample is rejected if, anywhere in `[t − exhaust_delay − holdoff, t]`: a RPM/MAP/TPS rate
  exceeds the gate thresholds (`WINDOW_TRANSIENT`), a MISFIRE/EXHAUST_LEAK fault is present
  (`WINDOW_MISFIRE`), fuel or spark cut was active (`WINDOW_FUEL_OR_SPARK_CUT`), STFT+LTFT spread
  exceeds `max_trim_swing` or is non-finite (`WINDOW_TRIM_UNSTABLE`), or `closed_loop` changed
  state (`WINDOW_LOOP_STATE_CHANGE`). A window reaching before the log start is clipped.
- Report gains `closed_loop_samples_used` / `open_loop_samples_used` (`#[serde(default)]`);
  shown in the Mixture page. The applied STFT/LTFT is still folded into the pre-trim estimate in
  both states because the simulator holds/decays STFT while inactive and keeps applying it
  (`closed_loop.rs` `update`, blocked-gate branch).
- Seven unit tests added (blip, misfire count, fuel cut, trim/loop change, loop-state counters,
  invalid option, JSON back-compat).

`ecu/apps/tuning-studio/src/types.ts`, `pages/Mixture.tsx`: optional fields + one stat tile.

### Verification status — **NOT RUN**

In this session every `cargo`, `rustc` and `npm` invocation was refused with
"This command requires approval" (non-interactive session, no approver). Only read-only
commands (`ls`, `grep`, `sed`, `git`, `node --version`) ran. Therefore:

| Check | Result |
|---|---|
| `cargo fmt --check` / `clippy -D warnings` / `cargo test --workspace` | **Not run** — code is uncompiled |
| `npm run typecheck` / `npm run build` (tuning-studio) | **Not run** — `node_modules` absent, npm refused |
| Baseline test count before the change | **Not measured this round** |

The new tests assert exact figures worked out by hand (e.g. `WINDOW_MISFIRE == 15` for a
100 ms delay + 200 ms holdoff at 20 ms frame spacing); a miscount would surface as a test
failure on first run.

### Known regression risk to check first

`ecu-simulator` test `mixture_map_recovers_deliberate_ve_bias_and_applies` runs the analyser with
default window options on the `mixture_sweep` scenario. The new gate removes samples near
operating-point transitions and samples with a moving STFT, so that test (≥3 proposed cells,
mean correction > 0.02) could fail or need option tuning. Unknown until run. If it fails,
tune options in that test or the defaults with evidence — do not weaken the gate silently.

### Reproduction (needs a session that may run cargo)

```bash
cd ecu
cargo test -p ecu-lambda
cargo test -p ecu-simulator mixture_map
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cd apps/tuning-studio && npm ci && npm run typecheck && npm run build
```

### Remaining blockers / next tasks (tuning)

1. **Run the verification above** (blocked by tool approval this round).
2. No accel-enrichment channel in `TelemetryFrame`; enrichment is only inferred from rates.
   Needs a field from the simulator (and, later, the real ECU protocol) before the proxy can be
   replaced. Simulation-only until then.
3. Confidence score (`confidence()`) does not yet include a latency/window-rejection factor.
4. UI: no per-cell display of window-rejection reasons beyond the aggregate reason list.
5. Tauri shell: `apps/tuning-studio` is Electron + Vite; `Cargo.toml` excludes
   `apps/tuning-studio/src-tauri`, but that directory does not exist, so the "Tauri" wording is
   not met. Decision needed (human) before
   investing.
6. Human review step for map edits exists in the service (explicit apply with checksum match);
   the UI-level review/rollback flow was not exercised in a browser this round.
7. Vehicle data: the starter map remains a labelled non-operational template; no verified Evo
   VIII data exists and none was invented.

## Round 2 (2026-11-13) — per-cell window accounting, confidence factor, diff-before-apply review

### Verification status — **STILL NOT RUN** (same blocker as round 1)

`cargo --version` and every compound/`cd` command were refused with "This command requires
approval" (non-interactive, no approver). `which cargo` shows `/home/jlee/.cargo/bin/cargo`
exists, so the toolchain is present but not runnable from this session. The sandbox override was
deliberately **not** used: the refusal is a permission decision, not a tooling fault.
Consequence: **no code from rounds 1 or 2 has been compiled, linted or executed.** No test
result below is an observed result; "hand-checked" means traced by reading, nothing more.

### Review of round-1 code (read-only, hand-checked)

Re-read `mixture_map.rs` end to end against `gate.rs` / `telemetry.rs`. Checked by reading:
field names (`max_rpm_rate_rpm_s`, `fuel_cut`, `spark_cut`, `closed_loop`, `exhaust_delay_ms`)
exist; `u8 | &u8` fold, `partition_point` on `frames[..=i]`, `&`-vs-`!=` precedence; hand-traced
the seven round-1 tests (misfire window = 15 frames; blip; loop change; open/closed counters;
JSON back-compat). No defect found. **Not a substitute for a compile.**

### Changes

1. `ecu-lambda/src/mixture_map.rs`
   - `MixtureCellReport.window_rejected_samples` (`#[serde(default)]`): gate-clean samples in the
     cell dropped by the latency window, counted once per sample (the existing per-reason
     `exclusion_reasons` counts a sample once per reason).
   - `confidence()` now multiplies the existing score by `window_factor()` =
     `1 - 0.30 * window_rejected / (used + window_rejected)`; neutral (x1.0) when nothing was
     window-rejected, so existing non-window tests are unaffected. `WINDOW_CONFIDENCE_PENALTY =
     0.30` is an **unvalidated engineering choice**.
   - Three tests: per-cell count + confidence reduction + bound (hand-traced: 15 rejected of 79
     clean gives factor ~0.943), factor bounds/neutrality, report JSON back-compat.
2. `apps/studio-service/src/main.rs`
   - New `POST /api/mixture/preview`: runs the same `apply_proposal` code path against the live
     calibration with a throw-away timestamp and returns the exact `TableDiff`,
     `base_revision/base_checksum/would_be_revision`; **stores nothing**, does not touch the
     session, engine or history. Stale/unknown/empty selections fail identically to apply.
   - `/api/mixture/apply` is unchanged (still requires `approve: true`, paused engine, matching
     base checksum).
   - Two unit tests (`preview_shows_exact_diff_without_changing_the_calibration`,
     `preview_refuses_stale_and_unknown_cell_selections`) using a hand-built proposal so they do
     not depend on simulator behaviour.
3. `apps/tuning-studio` (`api.ts`, `types.ts`, `pages/Mixture.tsx`)
   - `window.confirm` replaced by an in-page **review step**: "Review diff", then a diff table
     (cell, VE before/after, delta, base checksum, revision transition), then a mandatory
     acknowledgement checkbox, then "Apply reviewed". Apply is disabled unless a preview exists
     for *exactly* the current selection (`applied_cells === selected.size`); any selection or
     report change discards the preview. Undo button behaviour unchanged.
   - "All cells" table gains *Window drops* and *Rejected for* (top 3 reasons per cell).
   - Copilot (`apps/tuning-copilot`) confirmed read-only (analysis only; no apply/preview call).

### Still-open risks introduced or carried

| Risk | Status |
|---|---|
| Round-1 regression risk to `mixture_map_recovers_deliberate_ve_bias_and_applies` | **Unknown.** Round 2 can only lower confidence further (test floor `min_confidence: 0.45`). If the simulator's STFT swing inside a 310 ms window exceeds `max_trim_swing` (0.06), most sweep samples would be rejected. Needs a run; fix by evidence, not by loosening silently. |
| Preview race | Preview reads the session calibration; apply re-checks the base checksum, so a calibration change between preview and apply is refused as stale rather than silently applied. Not exercised. |
| `preview` while engine running | Allowed (read-only). Apply still requires a paused engine. |
| UI | `npm run typecheck`/`build` not run; the review flow has not been seen in a browser. |

### Reproduction (needs a session allowed to run cargo/npm)

```bash
cd ecu
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test -p ecu-lambda
cargo test -p studio-service
cargo test -p ecu-simulator mixture_map
cargo test --workspace
cd apps/tuning-studio && npm ci && npm run typecheck && npm run build
```

### Remaining blockers / next tasks (tuning)

1. **Run the verification above** — highest priority; rounds 1-2 are uncompiled. rustfmt may
   re-wrap some of the new lines.
2. No accel-enrichment channel in `TelemetryFrame` (unchanged from round 1).
3. Window thresholds/penalty (`transient_holdoff_ms`=250, `max_trim_swing`=0.06,
   `WINDOW_CONFIDENCE_PENALTY`=0.30) are guesses; need bench/log evidence from a real wideband
   before any use beyond simulation.
4. HTTP-level (router) test of preview-then-apply sequencing; currently only the pure function is
   unit-tested.
5. Tauri decision (app is Electron + Vite; no `src-tauri`) — human decision (unchanged).
6. Ignition recommendation UI/provenance display not revisited this round.
7. Vehicle data: starter map remains a labelled non-operational template; nothing invented.

## Round 3 (2026-11-13) — static cross-check of rounds 1–2, preview==apply property test

### Verification status — **STILL NOT RUN** (third consecutive round)

Commands attempted this round, all refused with "This command requires approval"
(non-interactive session, no approver):

| Command | Result |
|---|---|
| `cargo --version` | refused |
| `cargo test --manifest-path ecu/Cargo.toml -p ecu-lambda` | refused |
| `rustfmt --edition 2021 --check ecu/crates/ecu-lambda/src/mixture_map.rs` | refused |

The sandbox override (`dangerouslyDisableSandbox`) was deliberately not used: the refusal is a
permission decision, not a tooling fault. **No Rust or TypeScript from rounds 1–3 has been
compiled, linted, type-checked or executed.** Nothing below is an observed test result.

### What was done instead (read-only, by reading source — not a substitute for a compile)

Cross-checked every identifier the uncommitted code depends on against its definition:

- `ApiError { code, message }` derives `Debug` (needed by `unwrap()` in the new tests) and has
  `bad`/`internal`; fields are reachable from the in-file test module.
- `VeProposal` / `ProposedVeCell` field sets match the hand-built proposal in the service test
  field-for-field.
- `apply_proposal` error strings match the test assertions ("different calibration revision",
  "no proposal cells selected"); `TableDiff { table_id, changes[{indices,old,new,delta}],
  max_abs_delta, mean_abs_delta }` matches `MixturePreview` in `types.ts`.
- `Table.axes[0]` = MAP rows, `axes[1]` = RPM cols (cell index = row·cols+col), consistent with
  `hand_proposal`; `VE_TABLE_KEY == "ve"`.
- Re-traced the numeric expectations: misfire window = frames 31..=45 → 15 (window start uses
  `partition_point(<)`, so the frame at exactly t−300 ms is included, 46 is not); in the
  per-cell confidence test `sample`, `variance`, `dwell` all saturate at 1.0 for both logs
  (min_samples 5, 64 used; constant λ; dwell ≫ 50 ms), so confidence goes 1.0 → 1−0.30·15/79 ≈
  0.943, inside the asserted `[0.70, 1.0)` band.
- Gate reason code for misfire is `"MISFIRE"` (`gate.rs`), as the test expects.

No defect found. Residual compile risk remains in rustfmt wrapping and in trait-resolution
details I cannot confirm without a compiler (e.g. `serde_json::Value == usize/u32/&str`
comparisons in the new tests — these impls exist in serde_json, but are unconfirmed here).

### Change

`apps/studio-service/src/main.rs`: one new unit test,
`previewed_diff_is_the_diff_apply_produces` — the JSON diff returned by `preview_proposal` must
equal `serde_json::to_value(apply_proposal(..).diff)` for the same calibration/proposal (applied
with a different timestamp), and `would_be_revision` / `applied_cells` must match. This pins the
"diff shown to the human is the diff applied" property that the review step relies on.

### Gate status

Unchanged. Tuning-app work is simulation/offline only; no H4+ approval, no hardware, KiCad or
firmware-real-time file was touched.

### Reproduction (needs a session allowed to run cargo/npm)

```bash
cd ecu
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test -p ecu-lambda
cargo test -p studio-service
cargo test -p ecu-simulator mixture_map
cargo test --workspace
cd apps/tuning-studio && npm ci && npm run typecheck && npm run build
```

### Remaining blockers / next tasks (tuning)

1. **Grant cargo/npm execution to the tuning worker (or have the orchestrator run the
   reproduction block above and paste raw output into this file).** Until then, more feature work
   only adds uncompiled code; I chose not to stack more on rounds 1–3.
2. Regression risk to `ecu-simulator::mixture_map_recovers_deliberate_ve_bias_and_applies` from
   the new latency-window gate (unchanged; unknown until run).
3. No accel-enrichment channel in `TelemetryFrame` (needs simulator + protocol field).
4. Window thresholds/penalty (250 ms, 0.06, 0.30) are unvalidated guesses.
5. HTTP-router-level test of preview→apply sequencing (only pure functions are tested).
6. Ignition recommendation UI/provenance display not revisited.
7. Tauri vs Electron decision (human) — unchanged.
8. Vehicle data: starter map remains a labelled non-operational template; nothing invented.

## Round 4 (2026-11-13) — advisory ignition recommendation exposed end-to-end (service + UI)

### Verification status — **STILL NOT RUN** (fourth consecutive round)

Attempted again this round, all refused with "This command requires approval" (non-interactive,
no approver): `cargo --version`; `cargo test --manifest-path ecu/Cargo.toml -p studio-service
--offline`. Compound commands (`&&`, `cd … && …`, `sed -i`, heredoc + `git`) are also refused. The
sandbox override was deliberately not used: the refusal is a permission decision, not a tooling
fault. **No Rust or TypeScript from rounds 1–4 has been compiled, linted, type-checked or
executed.** Nothing below is an observed test result.

### Gap closed (open item 6 from rounds 2–3)

`ecu-ignition::recommend_from_reference` (evidence-gated, no lambda parameter, unit-tested in its
own crate) was not reachable from the service or UI; the Ignition page showed only the
simulation-only table. Ignition and lambda corrections were therefore "distinct" only by
omission.

### Change

- `apps/studio-service`: new dependency `ecu-ignition` (path). The `Cargo.lock` entry was added by
  hand and is **unverified**; cargo would normally regenerate it. If `--locked` fails, this is why.
  New `POST /api/ignition/recommend` → `ignition_recommendation()` (pure fn over `&Calibration`).
  - Input: target RPM/MAP, caller-stated fuel label, optional `TimingReference`. **No lambda,
    trim or log input exists in the request type.** Compression ratio and aspiration come from
    the live calibration.
  - Response carries `recommendation` (status / proposed / bounds / confidence / reasons /
    required_validation), `reference_provenance`, `attestation_notes`, `engine`,
    `calibration_ignition_policy`, and constant flags `lambda_used=false`, `fuel_trim_used=false`,
    `writes_calibration=false`, `advisory_only=true`. Nothing is stored; session, history and
    engine are untouched.
  - Provenance hardening: the service cannot verify a reference, so a reference claiming
    `verified_reference` + `engine_specific_verified` but with any blank `citation` / `licence` /
    `verified_by` / `verification_date` is **downgraded to unknown** → `insufficient_evidence`
    (no number). A complete claim is honoured but flagged "asserted by the caller; this service
    cannot check it". Non-finite or out-of-range inputs → `bad_request`.
  - Seven unit tests (`ignition_*`): no reference → no number and calibration unchanged;
    blank-attestation downgrade; non-verified source → no number; attested reference → bounded
    recommendation (18.0°, lo < adv < hi, caller-asserted note); fuel mismatch lowers confidence;
    range/NaN/blank rejection.
- `apps/tuning-studio`: `components/IgnitionAdvisory.tsx` (new), mounted under the Ignition map
  on the Ignition page; `api.ignitionRecommend`, new types. The panel has **no apply button** and
  no lambda input; shows status badge, proposed advance and review bounds ("—" when none),
  confidence, reasons, required validation, provenance and the advisory notice. Reuses existing
  CSS classes (`setup-grid`, `stat-grid four`, `alert`).

### Provenance of the numbers

All ignition figures originate from a reference the *operator* types in; none is built in. The
unit-test reference ("Unit-test reference", 18°) is a fixture, not engine data. Confidence
deductions (−0.25 CR, −0.2 fuel, −0.25 aspiration, −0.1 distance; bounds 2° + 8°·(1−conf)) are
pre-existing engineering choices in `ecu-ignition`, unvalidated.

### Hand-checks (reading only)

`TimingReference`/`Provenance` derive Deserialize/Serialize with snake_case enums, so the TS type
matches the wire format; matched-reference confidence = 0.8 and fuel mismatch = 0.6 per
`recommend_from_reference`; `Provenance`/`SourceKind` are used from the `ecu_types` root in
`ecu-ignition`'s own tests. Residual risks: rustfmt wrapping of new lines; clippy lints I cannot
see; the `Cargo.lock` hand edit; TS strictness.

### Gate status

Unchanged. Simulation/offline only; no H4+ approval, no hardware, KiCad or firmware real-time
file touched.

### Reproduction (needs a session allowed to run cargo/npm)

```bash
cd ecu
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test -p ecu-ignition
cargo test -p ecu-lambda
cargo test -p studio-service      # includes 7 new ignition_* tests + 3 preview tests
cargo test -p ecu-simulator mixture_map
cargo test --workspace --locked   # confirms the hand-edited Cargo.lock
cd apps/tuning-studio && npm ci && npm run typecheck && npm run build
```

### Remaining blockers / next tasks (tuning)

1. **Run the reproduction block** — rounds 1–4 are uncompiled; this is the only thing stopping
   risk-free progress. The orchestrator must grant cargo/npm to this worker, or run it and paste
   raw output here.
2. Regression risk to `ecu-simulator::mixture_map_recovers_deliberate_ve_bias_and_applies` from
   the round-1 latency window (unknown until run).
3. No accel-enrichment channel in `TelemetryFrame`.
4. Window thresholds/penalty (250 ms, 0.06, 0.30) are unvalidated guesses.
5. HTTP-router-level tests (preview→apply, ignition recommend); only pure functions are tested.
6. The UI review flows and IgnitionAdvisory have not been seen in a browser.
7. Tauri vs Electron decision (human).
8. Vehicle data: starter map remains a labelled non-operational template; nothing invented. No
   verified Evo VIII ignition reference exists in the repo.

## Round 5 (2026-11-13) — FIRST COMPILED/EXECUTED VERIFICATION; HTTP end-to-end test

### Supersedes earlier "NOT RUN" statements

Rounds 1–4 recorded every check as **not run** because the tool harness refused cargo/npm. In
this round `cargo`, `rustc`, `node` and `npm` ran. Everything below is an **observed** result from
this session; the "hand-checked" claims in rounds 1–4 are now confirmed or refuted by execution
(all confirmed; no defect found in the round 1–4 code).

### Environment (exact)

| Item | Value |
|---|---|
| cargo / rustc | 1.99.1 (5f94df478 2026-08-27) / 1.99.1 (58cf8f9c3 2026-10-13) |
| node | v24.14.1 |
| OS | Linux 6.8.0-147-generic |
| Host C linker | **none installed** (`cc`, `gcc`, `clang`, `ld` all absent) |

Workaround, kept **outside the repo** (no project config changed): a ~25-line shell shim at
`/tmp/lldshim/cc` that translates cc-style link arguments to the toolchain's bundled `ld.lld`
(`~/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/x86_64-unknown-linux-gnu/bin/gcc-ld/ld.lld`),
adding `crt1.o/Scrt1.o`, `crti.o`, `crtn.o`, the dynamic-loader path and a `libgcc_s.so` symlink to
`/lib/x86_64-linux-gnu/libgcc_s.so.1`. Selected per invocation with:

```bash
export CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER=/tmp/lldshim/cc CARGO_TARGET_DIR=/tmp/tuning-target
```

Caveat: this is a host-link workaround for build scripts / test binaries only. Installing a real
`build-essential` is the proper fix; results should be reproduced with a normal toolchain by the
orchestrator or a reviewer.

### Results (observed)

| Command (from `ecu/`) | Result |
|---|---|
| `cargo fmt --all -- --check` | rc 0 (after `cargo fmt -p studio-service` fixed 3 wrapping nits in my own new code) |
| `cargo clippy --workspace --all-targets --locked --offline -- -D warnings` | rc 0, no warnings (re-run after `touch` to force re-check) |
| `cargo test --workspace --locked --offline` | **187 passed, 0 failed** |
| — `ecu-lambda` | 38 passed (incl. all round 1–3 latency-window / per-cell / confidence tests) |
| — `ecu-ignition` | 7 passed |
| — `ecu-simulator` | 10 passed, incl. `mixture_map_recovers_deliberate_ve_bias_and_applies` — **the round 1–2 regression risk did not materialise** |
| — `studio-service` | 16 passed (13 previous incl. 3 preview + 7 ignition tests, **3 new HTTP end-to-end**) |
| `--locked` | accepted the hand-edited `Cargo.lock`; cargo then added `tower` (+1 dev-dep entry), diff is 2 lines |
| `npm ci --offline` (apps/tuning-studio) | ok, 0 vulnerabilities reported |
| `npx tsc --noEmit` | rc 0 |
| `npx vite build` | ok, 56 modules, `dist/assets/index-*.js` 344 kB |

`node_modules/` and `dist/` are not tracked (not in `git status`).

### Change this round: the HTTP surface is now testable and tested end to end

`apps/studio-service/src/main.rs`
- Behaviour-preserving refactor: `main()` now calls `new_state(...)` and `build_router(state,
  static_dir)`. The 20 ms background ticker is still spawned only by `main`.
- New `#[cfg(test)] mod http_tests` drives the **real router** with `tower::ServiceExt::oneshot`
  (new dev-dependency `tower 0.5` `util`, already in the lockfile via axum). No port is bound.
  - `virtual_ecu_lambda_correction_review_apply_and_rollback_over_http` — health → read base
    calibration → `POST /api/sim/start mixture_sweep` → engine stepped to scripted end (stands in
    for the ticker) → `POST /api/mixture/analyze-current` → proposal present, no hard faults →
    **apply while running ⇒ 409** (even with `approve:true`) → pause → **apply without
    `approve` ⇒ 400, calibration checksum unchanged** → `POST /api/mixture/preview` returns
    `preview_only`, checksum unchanged → **approved apply ⇒ 200, diff identical to the reviewed
    preview diff**, revision +1, new sealed checksum, engine left paused → **re-applying the same
    (stale) proposal ⇒ 400, unchanged** → `POST /api/calibration/undo` restores the *exact* original
    checksum.
  - `table_edit_requires_pause_and_is_rolled_back_over_http` — manual VE edit refused (409) while
    running; accepted when paused (new checksum); undo restores original checksum.
  - `ignition_recommendation_over_http_is_advisory_and_stores_nothing` — no reference ⇒
    `insufficient_evidence`, `advisory_only/lambda_used=false/writes_calibration=false`,
    calibration checksum unchanged.

### Raw numbers from the e2e run (`cargo test -p studio-service virtual_ecu_lambda -- --nocapture`)

```
e2e-summary: frames=2185 used=315 (closed_loop=315 open_loop=0) rejected=1870 proposed_cells=2
e2e-gate-reasons: {"CRITICAL_DTC":12,"ENGINE_NOT_RUNNING":13,"FUEL_CUT":13,"LAMBDA_IMPLAUSIBLE":33,
 "LAMBDA_NOT_READY":33,"LAMBDA_STATUS":33,"MAP_TRANSIENT":196,"RPM_LOW":5,"RPM_TRANSIENT":451,
 "SYNC_NOT_SEQUENTIAL":8,"TPS_TRANSIENT":206,"WINDOW_TRANSIENT":988,"WINDOW_TRIM_UNSTABLE":471,
 "WOT_BOUNDARY":411}
e2e-diff: changed_cells=2 max_abs_delta=3.3065338134765625 mean_abs_delta=3.1322994232177734
```

Interpretation (not over-claimed): only 14 % of frames are usable and 2 VE cells are proposed on
the `mixture_sweep` scenario with default options. The largest rejection reasons are the
round-1 latency window (`WINDOW_TRANSIENT` 988, `WINDOW_TRIM_UNSTABLE` 471; counts are per reason, a
frame can carry several). `mixture_sweep` ramps continuously, so a 310 ms exhaust-transport window
removes most of it; this is a scenario/gate interaction, **not** evidence that 250 ms / 0.06 are
right or wrong. No threshold was changed. All 315 used samples were closed-loop (the sweep never
runs open-loop), so the open-loop path is covered by unit tests only, not by this end-to-end run.
VE deltas (~3 VE points) come from the simulator's deliberate VE bias; they are simulation
numbers, not engine data.

### What this does and does not demonstrate

- **Demonstrated (simulation, over HTTP):** the review → explicit approval → sealed revision →
  rollback chain; refusal paths (running engine, missing approval, stale proposal); ignition
  advisory isolated from lambda and read-only.
- **Not demonstrated:** anything on real hardware or a real wideband; the browser UI flows (only
  typecheck/build were run — the review panel and IgnitionAdvisory have not been seen rendered);
  the WebSocket telemetry stream; behaviour under concurrent requests.
- The test stands in for the 20 ms ticker by calling `engine.run_to_end()`; the ticker loop itself
  is untested.

### Gate status

Unchanged. Tuning-app work is simulation/offline only; no H4+ approval is implied, and no
hardware, KiCad or firmware real-time file was touched.

### Reproduction

```bash
cd ecu
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
cargo test -p studio-service virtual_ecu_lambda -- --nocapture   # raw e2e numbers above
cd apps/tuning-studio && npm ci && npx tsc --noEmit && npx vite build
```

### Remaining blockers / next tasks (tuning)

1. Reproduce on a machine with a normal C toolchain (this run used the lld shim above).
2. Open-loop and low-yield behaviour: no scenario produces dwell-style steady cells with both
   loop states; a steady-state scenario (simulator crate, not firmware) would let the e2e test
   exercise open-loop and a higher sample yield.
3. No accel-enrichment channel in `TelemetryFrame` (needs simulator + protocol field).
4. Window thresholds/penalty (250 ms, 0.06, 0.30) remain unvalidated guesses; need real wideband
   logs.
5. UI never rendered in a browser (needs a browser session or Playwright; not available here).
6. WebSocket telemetry and the background ticker are untested.
7. Tauri vs Electron decision (human) — app is Electron + Vite; no `src-tauri` exists.
8. Vehicle data: starter map remains a labelled non-operational template; no verified Evo VIII
   data (VE, ignition, injector) exists and none was invented.

## Round 6 (2026-11-13) — long-dwell open/closed-loop scenario, truth-VE decoupling, trim-compensated variance

Scope: host simulator crate (`ecu-simulator`), `ecu-lambda`, `studio-service`, UI type only. No KiCad,
firmware real-time, or hardware file touched. Simulation/offline only.

### Starting point (read from disk)
Round-5 open items 2 (no scenario with open-loop / usable yield) and a latent one found while doing it.
No earlier work repeated.

### Two defects found by running the new scenario (not by reading)

1. **Simulator: scenario VE error was relative to the loaded ECU table.** `update_sensors` computed
   `true_ve = ECU_table(rpm,map) × error`, so raising the ECU table raised the "physical" engine's
   air equally and a lambda correction could *never* converge. A first re-simulation after applying a
   +5 % correction showed raw error 7.91 % → 7.83 % (no improvement). Fix: `SimEngine` has an optional
   fixed truth VE table (`with_truth_ve_from(&Calibration)`); default `None` keeps the previous
   behaviour for every existing caller. `studio-service` now keeps `Session.truth` (set on session
   creation, `setup_generate`, and the Evo simulation loader; **not** changed by edits, apply or undo) and
   builds every engine with `engine_for(&session.truth, …)`.
2. **Analyser: variance gate/confidence used raw measured λ.** A cell visited in both closed loop
   (λ≈target, trim carries the error) and open loop (λ biased, trim 0) looked like a ~0.02+ spread and
   was rejected `HighVariance`, although both loop states imply the same pre-trim error. Four of five
   dwell cells were lost this way (only 1 proposed). Now the gate and the confidence score use the spread
   of `λ·(1+stft+ltft)` (`corrected_lambda_stddev`, new serde-default report field); the raw
   `lambda_stddev` is kept for display. Behaviour change worth review: for closed-loop-only data the
   statistic now also contains trim variation (bounded already by `max_trim_swing`). No threshold changed.

### Changes
- `scenario.rs`: `Scenario.open_loop_windows` (`#[serde(default)]`), `OpenLoopWindow`, `open_loop_forced`;
  new builtin `mixture_dwell` (56 s, 7 s dyno dwells at 4 points, closed loop to 25 s then forced open
  loop; same deliberate +8 % VE bias family as `mixture_sweep`). Dwell length was chosen to exceed the
  analyser's exhaust-transport window; analyser options are the defaults.
- `lib.rs` (sim): open-loop forcing feeds the closed-loop controller a blocked gate
  (`OPEN_LOOP_SCENARIO`), so STFT freezes then decays exactly as for any other gate block.
- `mixture_map.rs`: `corrected_lambda_stddev`, `est_stddev`; 2 new unit tests (consistent mixed-loop cell
  is proposed with +8 %; inconsistent loop states are still `HighVariance` and give no proposal).
- `studio-service`: truth session field + `engine_for`; new HTTP test below.

### Observed results (this session; cargo 1.99.1, same lld shim as round 5, no system C linker)
| Command (from `ecu/`) | Result |
|---|---|
| `cargo fmt --all -- --check` | rc 0 |
| `cargo clippy --workspace --all-targets --locked --offline -- -D warnings` | rc 0, no output |
| `cargo test --workspace --locked --offline` | **192 passed, 0 failed** (187 + 5 new) |
| `npx tsc --noEmit` / `npx vite build` (apps/tuning-studio) | rc 0 / ok |

New tests: `ecu-lambda` ×2 (above); `ecu-simulator`
`mixture_dwell_yields_both_loop_states_and_resimulation_reduces_open_loop_error` and
`repeated_review_apply_resimulate_passes_converge_on_the_biased_ve`; `studio-service`
`dwell_scenario_passes_converge_over_http_and_full_rollback_restores_base`.

Raw numbers (`cargo test … -- --nocapture`):
```
mixture_dwell, default analyser options:  frames=2912 used=941 (closed=385 open=556) cells=5   (mixture_sweep: 14 % yield, 2 cells; dwell: 32 %, 5 cells)
sim pass-by-pass mean raw pre-trim error over the 5 tracked cells: [0.0782, 0.0437, 0.0170, 0.0052]
HTTP (3 approved apply passes, per-application clamp 5 %):          [0.0781, 0.0438, 0.0143]
HTTP undo ×3 -> checksum identical to the original
```
(`raw error = |mean λ / mean target × (1+mean trim) − 1|` per cell, unclamped, so the 5 % per-application
clamp does not hide progress.)

### What this does and does not show
- **Shows (simulation):** open-loop and closed-loop samples are both accepted and counted; cells visited in
  both loop states pool correctly; the review → approve → apply → re-measure loop reduces error each
  pass over HTTP and rollback is exact.
- **Does not show:** that the model is right. The simulator's air/fuel model and the ECU fuel model are the
  same code family, so convergence demonstrates the plumbing and sign conventions, not accuracy against an
  engine. Only 5 cells (mid-load, rpm 2100–4100) are visited; unvisited cells are untouched by design.
  Wideband dynamics, real transport delay and sensor error are the simulator's own. 250 ms / 0.06 / 0.30
  thresholds remain unvalidated guesses.
- Not rendered in a browser; WebSocket ticker still untested.

### Gate status
Unchanged. No H4+ approval implied; AI cannot grant any.

### Reproduction
```bash
cd ecu
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
cargo test -p ecu-simulator repeated_review -- --nocapture
cargo test -p studio-service dwell_scenario -- --nocapture
(cd apps/tuning-studio && npx tsc --noEmit && npx vite build)
```

### Remaining blockers / next tasks (tuning)
1. Reproduce on a machine with a normal C toolchain (still the lld shim).
2. UI: scenario picker lists `mixture_dwell` dynamically, but no browser render has been seen; a
   "corrected σ" column is typed (`types.ts`) but not yet displayed.
3. No accel-enrichment channel in `TelemetryFrame` (simulator + protocol change).
4. Thresholds (250 ms, 0.06, 0.30, 5 % per pass) need real wideband logs.
5. Ticker/WebSocket untested; concurrency untested.
6. Tauri vs Electron (human decision); app is Electron + Vite, no `src-tauri`.
7. No verified Evo VIII data; starter map stays a labelled non-operational template.
8. Review request: the variance-statistic change (defect 2) alters gate semantics and needs an
   independent look before it is treated as settled.
