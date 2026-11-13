#!/usr/bin/env bash
set -Eeuo pipefail
ROLE="${1:?role}"; ROUNDS="${2:?rounds}"; ROOT="${3:?root}"; STATE="${4:?state}"; WT="${5:?worktree}"
TARGET="${6:-}"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
mkdir -p "$STATE/logs" "$STATE/index"
case "$ROLE" in hardware|rust|tuning|audit) ;; *) echo "unknown role: $ROLE" >&2; exit 2 ;; esac
[[ "$ROUNDS" =~ ^[1-9][0-9]*$ ]] || { echo 'rounds must be positive integer' >&2; exit 2; }
[[ -d "$WT/.git" || -f "$WT/.git" ]] || { echo "Not a Git worktree: $WT" >&2; exit 2; }
cd "$WT"
if [[ "$ROLE" == audit ]]; then
  prompt_base="$(cat "$SCRIPT_DIR/prompts/audit.md")"
  BASE="$(cat "$STATE/baseline.sha")"
  prompt_base+=$'\n\nBaseline commit: '"$BASE"$'. Target: '"$TARGET"$'. You may use `git diff '"$BASE"'...HEAD` and `git diff` to inspect changes.'
else
  prompt_base="$(cat "$SCRIPT_DIR/prompts/${ROLE}.md")"
fi
for (( round=1; round<=ROUNDS; round++ )); do
  stamp="$(date -u +%Y%m%dT%H%M%SZ)-${BASHPID}-${round}"
  prefix="$STATE/logs/${ROLE}-${stamp}"
  raw="${prefix}.jsonl"; err="${prefix}.stderr.log"; meta="${prefix}.meta"
  printf 'role=%s\nround=%s\nstart_utc=%s\nworktree=%s\n' "$ROLE" "$round" "$(date -u --iso-8601=seconds)" "$WT" > "$meta"
  echo "$raw" > "$STATE/index/${ROLE}.latest"
  before_head="$(git rev-parse HEAD)"
  before_dirty="$(git status --porcelain=v1 | sha256sum | cut -d' ' -f1)"
  prompt="$prompt_base"
  if [[ "$ROLE" != audit ]]; then
    prompt+=$'\n\nExecution round '"$round"$'. Read current on-disk state. Advance the next independent unblocked task; do not repeat prior work. Update audit/verification docs as appropriate. Continue implementation during this run; end with factual tests and remaining blockers.'
  fi
  case "$ROLE" in
    hardware)
      cmd=(opencode run --format json --auto --agent build)
      [[ -z "${OPENCODE_MODEL:-}" ]] || cmd+=(--model "$OPENCODE_MODEL")
      ;;
    rust) cmd=(codex exec --json --sandbox workspace-write) ;;
    tuning) cmd=(claude -p --verbose --output-format stream-json --permission-mode "${CLAUDE_PERMISSION_MODE:-acceptEdits}") ;;
    audit) cmd=(codex exec --json --sandbox read-only) ;;
  esac
  printf 'command=' >> "$meta"; printf '%q ' "${cmd[@]}" >> "$meta"; printf '\n' >> "$meta"
  printf '[%s] %s starting round %s; log: %s\n' "$(date -u --iso-8601=seconds)" "$ROLE" "$round" "$raw"
  set +e
  ("${cmd[@]}" "$prompt") 2> >(tee -a "$err" >&2) | tee "$raw" | python3 -u "$SCRIPT_DIR/scripts/pretty-log.py" "$ROLE"
  status=${PIPESTATUS[0]}
  set -e
  printf 'end_utc=%s\nexit_code=%s\n' "$(date -u --iso-8601=seconds)" "$status" >> "$meta"
  printf '[%s] %s round %s exit: %s\n' "$(date -u --iso-8601=seconds)" "$ROLE" "$round" "$status"
  if (( status != 0 )); then
    echo "Stopped $ROLE after failed CLI exit (do not loop on errors)."
    exit "$status"
  fi
  [[ "$ROLE" != audit ]] || break
  after_head="$(git rev-parse HEAD)"
  after_dirty="$(git status --porcelain=v1 | sha256sum | cut -d' ' -f1)"
  if [[ "$before_head" == "$after_head" && "$before_dirty" == "$after_dirty" && -z "$(git status --porcelain=v1)" ]]; then
    echo "No repository changes this round. Halting $ROLE to conserve tokens."
    break
  fi
  if (( round < ROUNDS )); then sleep 3; fi
done
printf 'Agent %s finished its bounded sequence; outputs remain in %s\n' "$ROLE" "$STATE/logs"
