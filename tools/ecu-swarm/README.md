# Linux ECU multi-agent swarm runner

Runs three independent CLI coding sessions in isolated Git worktrees, and keeps timestamped output logs you can inspect during and after each run. Works with installed, signed-in OpenCode, Codex and Claude Code CLIs.

**This is a development orchestration tool, not evidence that an ECU is safe to fabricate or operate.** The independent audit command is a code/design review aid, not an automotive engineering signoff.

## Install into your *existing* repository

On a Linux host with `git`, `tmux`, `python3`, `bash`, `opencode`, `codex`, `claude` installed:

```bash
# Extract zip to /tmp, for example
unzip ecu-linux-swarm.zip -d /tmp
cd /path/to/your/existing/ecu-repo
mkdir -p tools/ecu-swarm
cp -a /tmp/ecu-linux-swarm/. tools/ecu-swarm/
chmod +x tools/ecu-swarm/swarm.sh tools/ecu-swarm/runner.sh tools/ecu-swarm/scripts/pretty-log.py
# Place your FULL revised course-correction prompt in PROJECT_DIRECTIVE.md
# Keep your existing AGENTS.md, CLAUDE.md, and TASKS.md; merge rather than overwrite.
cat tools/ecu-swarm/AGENTS-SUPPLEMENT.md >> AGENTS.md
# Inspect edits, then commit baseline, so every worktree sees these files.
git add tools/ecu-swarm AGENTS.md PROJECT_DIRECTIVE.md
git commit -m 'Add auditable Linux ECU swarm orchestration'
```

If `AGENTS.md` does not exist yet, `cat ... >> AGENTS.md` creates it. The append can be omitted if your current agent instructions already contain equivalent guidance. Avoid appending twice.

## Start

```bash
# From project root
./tools/ecu-swarm/swarm.sh doctor
./tools/ecu-swarm/swarm.sh init
./tools/ecu-swarm/swarm.sh start 2
./tools/ecu-swarm/swarm.sh attach
```

This runs up to **two sequential rounds per worker**, with three workers in parallel:

| tmux window | CLI | Workstream | Worktree |
|---|---|---|---|
| `hardware` | OpenCode / DeepSeek | Speeduino baseline, KiCad, parts, bench package | sibling `*-swarm-worktrees/hardware` |
| `rust` | Codex | Rust ECU simulator, tests and verification | sibling `*-swarm-worktrees/rust` |
| `tuning` | Claude Code | UI, calibration engine, offline lambda tuning | sibling `*-swarm-worktrees/tuning` |
| `monitor` | shell | session/log locations and help | no edits |

Workers take instructions from the project-root `PROJECT_DIRECTIVE.md`, `AGENTS.md`, and their own prompts. Their changes stay in separate branches until reviewed. A worker that fails or produces no changes in a round stops its own loop instead of burning more tokens.

This does **not** automatically merge agent changes, buy hardware, flash firmware, or connect anything to a car.

## View all session activity

```bash
./tools/ecu-swarm/swarm.sh status            # running panes and branches
./tools/ecu-swarm/swarm.sh logs              # list every captured run
./tools/ecu-swarm/swarm.sh attach            # interactive tmux windows
./tools/ecu-swarm/swarm.sh dashboard         # live tiled view of all three logs
./tools/ecu-swarm/swarm.sh watch hardware    # follow current/most recent hardware session
./tools/ecu-swarm/swarm.sh watch rust
./tools/ecu-swarm/swarm.sh watch tuning
./tools/ecu-swarm/swarm.sh stop              # stop current tmux session (does not delete logs)
```

Raw JSONL stdout, separate stderr, and per-round metadata are stored outside your Git repository, in a sibling directory named `<repo>-swarm-state/logs/`. The runner prints the actual full location. `watch` uses the human-friendly JSONL formatter; inspect the raw files with `jq` if necessary. **Logs may contain source code or secrets**, depending on CLI output: do not publish them unreviewed.

Native CLI session state also remains with each tool. For example, `opencode session list`, `opencode session export <id> --sanitize`, `codex resume`, and `claude --resume` can recover provider-specific sessions. Our runner captures events printed by each top-level CLI, but nested agent transcripts may need native tool session export and are not guaranteed to appear fully in the top-level JSONL.

## Independent audits

When a branch is ready, run Codex in read-only mode against it:

```bash
./tools/ecu-swarm/swarm.sh audit hardware
./tools/ecu-swarm/swarm.sh audit rust
./tools/ecu-swarm/swarm.sh audit tuning
```

The independent auditor records findings in `.jsonl` and `.stderr.log` under the state directory. Read the findings, reconcile the defects, and request a human electrical/automotive safety review before H4 fabrication approval. Never merge just because the auditor says 'looks good'.

## Integrate branch changes manually

```bash
./tools/ecu-swarm/swarm.sh status
# Read changes in each worktree. Review / verify. Then, from the main repo:
git log --oneline --all --decorate -20
# Example only — cherry-pick approved commits by exact SHA:
git cherry-pick <approved-commit-sha>
```

If a worker left uncommitted changes, inspect and commit from that worker's worktree before integration. **Do not merge directly from multiple branches without resolving project-specific conflicts and running tests.**

## More rounds

After the previous tmux session finishes, run `swarm.sh stop` (to remove the old pane set), then `swarm.sh start 2`. It reuses the worktrees and reads state from disk. This is intentionally **bounded**, not a free-running spend loop.

### Cost and permissions

- Set `OPENCODE_MODEL` (e.g. `deepseek/<actual-model-id>`) if you want to override OpenCode's configured default. Confirm it with `opencode models` first.
- `SWARM_SESSION` customises the tmux session name.
- `CLAUDE_PERMISSION_MODE` defaults to `acceptEdits`. You can set it to another **supported** mode if your local CLI requires it. Avoid `bypassPermissions`/`--dangerously-skip-permissions` on your workstation.
- Codex uses `--sandbox workspace-write`; its default approval settings still apply. Audit runs use `--sandbox read-only`.
- OpenCode uses `--auto`, which auto-approves permissions that have not been expressly denied. Review its effective permissions before running arbitrary project code.
- Authenticated Claude and Codex usage may have subscription or rate limits; DeepSeek/OpenCode may incur API charges. No script can guarantee a spending cap without provider-side limits.
- Require genuine peer review at H4 before manufacturing, and physical testing at H5–H6; passing AI or KiCad checks never establishes on-car readiness.
