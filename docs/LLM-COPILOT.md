# LLM Tuning Copilot

Status: **implemented, local-only, with known audit and test gaps.** The Mastra/TypeScript service lives in `ecu/apps/tuning-copilot`; the Tuning Studio Copilot page calls it on `127.0.0.1:8787`. It is not a physical-control component and cannot write a calibration.

| Piece | State |
|---|---|
| Studio Copilot page | Present: `apps/tuning-studio/src/pages/Copilot.tsx`. |
| Mastra/TypeScript service | Present: `apps/tuning-copilot` (`@mastra/core`, TypeScript, Node HTTP). |
| Read-only tools | Present: telemetry, faults, calibration summary, log summary, Mixture Map analysis. |
| Structured output validation | Present: Zod `CopilotAnswerSchema`. |
| Offline/no-API-key mode | Present: `COPILOT_OFFLINE=1` or default when LLM is not enabled. |
| Optional LLM mode | Present: opt-in with `COPILOT_ENABLE_LLM=1` and a provider key. |
| Durable audit / actor identity / approval records | Absent. Tool calls are returned per request and not persisted. |
| Automated copilot tests / prompt-injection evals | `npm test` runs `tsc --noEmit` and passes. It is a typecheck, not a behavioral or adversarial eval suite. |
| Apply/write/ignition/hardware tool | Absent by design. |

## 1. Purpose

The copilot is an advisory assistant for a human tuner. It can:

- read the current virtual-ECU telemetry snapshot;
- read active/latched DTCs and freeze frames;
- read calibration identity, table provenance, ignition policy, and preflight results;
- read a compact log summary and recent events;
- run the audited offline Mixture Map against the current calibration;
- explain gates, faults, proposals, and evidence limits.

It cannot apply a calibration, change ignition, disable a safety gate, clear a latched fault, or touch hardware. The Rust service remains authoritative; the human remains the approver.

## 2. Architecture

```text
+-----------------------------+
| Tuning Studio (React/Vite)  |<-----------------------------+
+--------------+--------------+                              |
               | chat / proposal review                      | human approval
               v                                             |
+-----------------------------+     HTTP, 127.0.0.1:8787      |
| Copilot service (Mastra/TS) |----------------------------->|
| read-only tools + Zod       |                              |
+--------------+--------------+                              |
               |                                             v
               | read/analyze requests              +----------------------+
               +----------------------------------->| Rust studio-service  |
                                                     | 127.0.0.1:8765       |
                                                     | checks, revisions,   |
                                                     | in-memory undo       |
                                                     +----------+-----------+
                                                                |
                                                                v
                                                     virtual ECU / future ECU
```

- **Copilot server:** Node `http`, binds `127.0.0.1:8787`; CORS allowlist for the Vite and service origins; `POST /api/copilot`, `GET /health`.
- **Rust client:** `OPENTECH_API` defaults to `http://127.0.0.1:8765`; requests time out after 30 seconds.
- **Mastra agent:** default model `deepseek/deepseek-v4-pro`; override with `COPILOT_MODEL`; `maxRetries: 1`; `maxSteps: 8`.
- **LLM gating:** LLM mode requires `COPILOT_ENABLE_LLM=1` **and** one of `DEEPSEEK_API_KEY`, `OPENAI_API_KEY`, or `OPENROUTER_API_KEY`. Otherwise the service runs offline tools, not a silent fallback to an unvalidated provider.
- **Input limits:** request body 1 MB; message 8000 characters; history 20 messages accepted by the server, last 8 passed to the agent, each truncated to 4000 characters.
- **UI:** `VITE_COPILOT_URL` defaults to `http://127.0.0.1:8787`.

## 3. Tools

### 3.1 Implemented tools

| Tool ID | Access | Purpose | Side effects |
|---|---|---|---|
| `read_telemetry` | read | RPM, MAP, lambda/target, trims, ignition advance, injection, sync, protection, faults. | None |
| `read_faults` | read | Active/latched DTCs, occurrence counts, freeze frames, protection state. | None |
| `read_calibration_summary` | read | Calibration ID/revision/checksum, engine metadata, table provenance, ignition policy, preflight checks. Does not expose raw table arrays. | None |
| `run_mixture_map_analysis` | read/analyze | Run the Rust Mixture Map on a deterministic warm dyno sweep; return gate counts and bounded fuel-cell proposals. | None; no apply |
| `read_log_summary` | read | Frame count, duration, recent events, latest frames. | None |

The toolset is constructed per request. It has no apply, write, edit, clear-fault, ignition, firmware, or hardware tool. The only write-capable paths are human-operated UI endpoints (table edit and Mixture Map apply), which call the Rust service independently.

### 3.2 Planned tools

The design can add read-only tools for arbitrary telemetry windows, table slices, calibration diffs, protection-event history, and audit queries. They are not implemented. Any addition must remain read-only and schema-validated.

## 4. Structured output

The agent returns a Zod-validated object:

```json
{
  "reply": "string",
  "findings": ["string"],
  "proposedFuelCells": [
    { "rpm": 3000.0, "mapKpa": 60.0, "correctionPercent": 5.0 }
  ],
  "requiresHumanApproval": true,
  "riskLevel": "info | caution | blocked",
  "nextActions": ["string"],
  "ignitionAdvice": "none"
}
```

Schema limits: up to 12 findings, 40 proposed cells (the prompt asks for at most 20), 8 next actions. `ignitionAdvice` is a literal `"none"`; a model that returns anything else fails validation.

The model may only report fuel cells that came from the Mixture Map tool. It is instructed not to invent VE values, injector data, lambda targets, or safe limits. The copilot's numeric output is a summary of Rust analysis, not an independent calibration source.

## 5. Approval flow

1. The human asks the copilot to investigate a rich/lean condition or propose a fuel correction.
2. The copilot calls its read-only tools. The Rust service returns telemetry, logs, gates, and a deterministic proposal.
3. The copilot summarizes the proposal and sets `requiresHumanApproval: true`. It cannot apply it.
4. The human opens the Mixture Map page, reviews the cell diff, sample counts, confidence, gate exclusions, unvisited cells, and source log checksum.
5. The human selects cells and clicks Approve. The UI calls the Rust service `/api/mixture/apply` with `approve=true`.
6. The Rust service refuses while the simulator is running, calls `apply_proposal`, rechecks the base checksum/revision and cell bounds, creates a new sealed revision, and pushes it onto an in-memory history.
7. The UI can undo by popping the in-memory history.
8. The copilot can read the result and explain it. It cannot claim the change is safe or engine-validated.

Current gaps in step 6:

- the service does not verify that the proposal came from this service;
- there is no proposal expiry or actor identity;
- there is no durable audit record;
- the API accepts a crafted `VeProposal` JSON if it matches the current checksum/revision and bounds;
- undo history is in memory and disappears on restart.

These are requirements for the next phase, not completed features.

## 6. Trust boundary and prompt injection

The model is untrusted. Telemetry strings, log notes, event messages, calibration notes, and tool results can contain adversarial text. They are **data**, never instructions.

Implemented defenses:

- **Read-only tool allowlist.** No dynamic tool creation, no apply/write/hardware tool.
- **Structured output validation.** Zod schema rejects wrong shapes and any `ignitionAdvice` other than `"none"`.
- **System-prompt invariants.** The agent is told it is read-only, must not propose ignition/MBT/knock limits, must not claim safety, must not instruct hardware connection, and must treat tool/log text as untrusted data.
- **Input limits.** Message length, history length, body size, and per-request tool budget are bounded.
- **Local-only binding and CORS allowlist.** The copilot listens on loopback and permits configured UI origins.
- **Deterministic offline mode.** The same tools can run without an LLM, which is useful for testing and for avoiding provider dependence.

Not yet implemented:

- no independent prompt-injection classifier or content sanitizer beyond JSON/schema handling;
- no redaction pipeline for secrets in tool results;
- no durable audit, actor/model/prompt record, or hash chain;
- no automated prompt-injection/refusal eval suite;
- no output cross-check that reported cell values exactly match the Rust proposal (the model is instructed to copy them, not programmatically constrained to them).

The last item matters: a human should review the Mixture Map table, not the chat text. The chat is an explanation layer, not the approval artifact.

## 7. Forbidden operations

The copilot must never be able to:

- apply, write, or delete a calibration;
- change ignition advance, dwell, ignition tables, or ignition policy;
- enable a physical ignition export or bypass the timing-light requirement;
- disable, weaken, or raise learning gates, protection limits, rev limits, boost limits, or DTC latching;
- clear a latched DTC;
- actuate injectors, coils, throttle, boost, VVT, nitrous, or any hardware output;
- flash firmware, enter a bootloader, or write CAN frames;
- execute code, shell commands, SQL, or arbitrary network requests;
- read or exfiltrate secrets;
- modify, delete, or forge audit records;
- approve its own proposal or mark a tune as validated;
- present a synthetic ignition table as engine-safe;
- instruct a user to connect hardware or build a harness for an unverified profile.

Any future implementation that exposes one of these as a tool is a design failure, regardless of prompt instructions.

## 8. Audit logging

Current implementation: each tool use appends `{ name, summary, timestamp }` to a per-request array. The HTTP response returns this as `toolCalls`. It is not persisted, has no actor identity, and is not linked to an approval.

Required design:

| Field | Content |
|---|---|
| `event_id` | Unique ID. |
| `timestamp_unix_ms` | Event time. |
| `actor` | Human user or service identity. |
| `session_id` | Copilot/UI session. |
| `model_provider`, `model_id`, `model_version` | Model identity, if an LLM was used. |
| `prompt_hash` | Hash of the prompt, not necessarily full sensitive text. |
| `tool_name` | Allowlisted tool. |
| `arguments_hash` | Hash of validated arguments; redacted summary optional. |
| `result_hash` | Hash of the tool result. |
| `proposal_id` | Mixture Map proposal, if any. |
| `base_checksum`, `base_revision` | Calibration state before apply. |
| `selected_cells` | Cells approved by the human. |
| `decision` | Approved, rejected, expired, or failed validation. |
| `new_revision`, `new_checksum` | Resulting calibration revision. |
| `undo_revision` | Snapshot used for undo. |
| `reason` | Human-supplied reason, if any. |

Audit requirements:

- Append-only storage; no copilot write path.
- Stable ordering and IDs; export to JSONL/CSV for review.
- Tamper evidence (hash chain or signed batches) is desirable; do not claim it until implemented.
- Retention policy must be explicit. Do not log secrets or full proprietary logs by default.
- The audit trail must let a reviewer reconstruct who asked, what tools ran, what was proposed, who approved, and what changed.

## 9. Offline mode (no API key)

The copilot runs without an LLM key. It executes the same read-only tools deterministically and returns the same structured answer shape. No provider key or network access to an LLM is required.

```bash
# Terminal 1: Rust service on 8765 (required by the copilot).
cd ecu
cargo run -p studio-service -- --port 8765

# Terminal 2: copilot in deterministic offline mode.
cd ecu/apps/tuning-copilot
npm install
COPILOT_OFFLINE=1 npm run start
```

Check it:

```bash
curl -sS http://127.0.0.1:8787/health
curl -sS -X POST http://127.0.0.1:8787/api/copilot \
  -H 'content-type: application/json' \
  -d '{"message":"Run a mixture map analysis and explain the fuel corrections.","history":[]}'
```

Expected behavior: `mode: "offline-tools"`, no LLM call, a bounded proposal summary, `ignitionAdvice: "none"`, and per-request `toolCalls`.

Optional LLM mode:

```bash
cd ecu/apps/tuning-copilot
COPILOT_ENABLE_LLM=1 DEEPSEEK_API_KEY=... npm run start
# Optional: COPILOT_MODEL=provider/model
```

LLM mode is opt-in. Without both the enable flag and a key, the service stays in offline tools. It does not silently use a provider.

## 10. Evaluation and current gaps

`npm run build` passes. `npm test` runs `tsc --noEmit` and passes. That covers type validity only; there is no automated behavioral evidence for:

- tool selection;
- refusal of ignition/protection/hardware requests;
- prompt injection in logs, notes, or event messages;
- schema rejection of malformed model output;
- matching reported fuel cells to the Rust proposal;
- audit completeness or persistence.

Before the copilot is considered production-usable:

- add a deterministic offline behavioral test suite;
- add refusal and prompt-injection evals;
- add a structured-output contract test;
- add durable, append-only audit with actor/model/prompt/proposal records;
- add proposal provenance/expiry and server-side cross-checks;
- review the system prompt and tool descriptions for injection hazards;
- keep the no-write/no-hardware tool boundary.

A passing eval suite is evidence about the assistant, not about engine safety.

## 11. Implementation checklist

- [x] Create `apps/tuning-copilot` with Mastra.
- [x] Define the read-only tool client against the Rust service.
- [x] Add Zod structured-output validation and a no-ignition-advice literal.
- [x] Add deterministic offline tool mode with no API key.
- [x] Add a Tuning Studio Copilot page.
- [ ] Add durable audit, proposal provenance/expiry, and actor identity in the Rust service.
- [x] Make `npm test` run `tsc --noEmit` and pass.
- [ ] Add behavioral, refusal, and prompt-injection eval suites.
- [ ] Programmatically cross-check reported cells against the Rust proposal.
- [ ] Do not add an apply/write/hardware tool. Ever.
