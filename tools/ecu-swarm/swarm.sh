#!/usr/bin/env bash
set -Eeuo pipefail
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(git -C "$HERE" rev-parse --show-toplevel 2>/dev/null || true)"
[[ -n "$ROOT" ]] || { echo 'Place kit inside a Git repository first.' >&2; exit 2; }
NAME="$(basename "$ROOT")"
PARENT="$(dirname "$ROOT")"
WT_PARENT="${SWARM_WORKTREES:-$PARENT/$NAME-swarm-worktrees}"
STATE="${SWARM_STATE:-$PARENT/$NAME-swarm-state}"
TMUX_SESSION="${SWARM_SESSION:-ecu-${NAME//./-}}"
sub="${1:-help}"
check_cmds() {
  local command
  for command in bash git tmux python3 opencode codex claude; do
    if command -v "$command" >/dev/null 2>&1; then printf 'OK %-9s %s\n' "$command" "$(command -v "$command")"; else printf 'MISSING %s\n' "$command"; fi
  done
  echo "Repo: $ROOT"
  echo "Worktrees: $WT_PARENT"
  echo "Logs: $STATE/logs"
  echo "tmux: $TMUX_SESSION"
}
create_wts() {
  [[ -f "$ROOT/PROJECT_DIRECTIVE.md" ]] || { echo 'Create PROJECT_DIRECTIVE.md with your revised full prompt first.' >&2; exit 2; }
  if [[ -n "$(git -C "$ROOT" status --porcelain=v1)" ]]; then
    echo 'Main repo has uncommitted changes. Commit/review before init so worktrees share a clean baseline.' >&2
    exit 2
  fi
  mkdir -p "$WT_PARENT" "$STATE/index" "$STATE/logs"
  if [[ ! -f "$STATE/baseline.sha" ]]; then
    git -C "$ROOT" rev-parse HEAD > "$STATE/baseline.sha"
  fi
  local stamp role
  stamp="$(date -u +%Y%m%dT%H%M%SZ)"
  for role in hardware rust tuning; do
    if [[ ! -e "$WT_PARENT/$role" ]]; then
      git -C "$ROOT" worktree add -b "swarm/${role}-${stamp}" "$WT_PARENT/$role" HEAD
    else
      printf 'Reusing %s\n' "$WT_PARENT/$role"
    fi
  done
  echo "Initialised; baseline $(cat "$STATE/baseline.sha")"
}
launch_window() {
  local window="$1" role="$2" rounds="$3"
  local command_line
  printf -v command_line 'bash %q %q %q %q %q %q' "$HERE/runner.sh" "$role" "$rounds" "$ROOT" "$STATE" "$WT_PARENT/$role"
  if [[ "$window" == hardware ]]; then
    tmux new-session -d -s "$TMUX_SESSION" -n "$window" "$command_line"
    tmux set-option -t "$TMUX_SESSION" remain-on-exit on
  else
    tmux new-window -d -t "$TMUX_SESSION" -n "$window" "$command_line"
  fi
}
case "$sub" in
  doctor) check_cmds ;;
  init) create_wts ;;
  start)
    rounds="${2:-2}"
    [[ "$rounds" =~ ^[1-9][0-9]*$ ]] || { echo 'Usage: swarm.sh start <positive-number-of-rounds>' >&2; exit 2; }
    [[ -d "$WT_PARENT/hardware" && -d "$WT_PARENT/rust" && -d "$WT_PARENT/tuning" ]] || { echo 'Run swarm.sh init first.' >&2; exit 2; }
    if tmux has-session -t "$TMUX_SESSION" 2>/dev/null; then echo "tmux session already exists: $TMUX_SESSION. Use stop or attach."; exit 2; fi
    launch_window hardware hardware "$rounds"
    launch_window rust rust "$rounds"
    launch_window tuning tuning "$rounds"
    tmux new-window -d -t "$TMUX_SESSION" -n monitor "bash -c 'echo Raw logs in $STATE/logs; echo Use swarm.sh logs, watch ROLE, or status; exec bash'"
    tmux select-window -t "$TMUX_SESSION:hardware"
    printf 'Started %s with %s rounds per worker. Attach: %s attach\n' "$TMUX_SESSION" "$rounds" "$0"
    ;;
  attach) tmux attach-session -t "$TMUX_SESSION" ;;
  stop) tmux kill-session -t "$TMUX_SESSION"; echo 'Stopped tmux session (logs/worktrees preserved).' ;;
  status)
    echo '== tmux =='; tmux list-windows -t "$TMUX_SESSION" -F '#{window_name} | dead=#{pane_dead} | pid=#{pane_pid}' 2>/dev/null || echo 'No running tmux session'
    echo '== branches =='
    for r in hardware rust tuning; do
      if [[ -d "$WT_PARENT/$r" ]]; then
        printf '%s | branch=%s | dirty_files=%s\n' "$r" "$(git -C "$WT_PARENT/$r" branch --show-current)" "$(git -C "$WT_PARENT/$r" status --porcelain | wc -l)"
      fi
    done
    echo "== log folder == $STATE/logs"
    ;;
  dashboard)
    if ! tmux has-session -t "$TMUX_SESSION" 2>/dev/null; then
      echo "No swarm tmux session. Run start first." >&2; exit 2
    fi
    if tmux list-windows -t "$TMUX_SESSION" -F '#{window_name}' | grep -qx dashboard; then
      tmux select-window -t "$TMUX_SESSION:dashboard"
      echo "Existing dashboard selected. Use attach to view it."
      exit 0
    fi
    local_cmd=""
    printf -v local_cmd 'bash %q watch hardware' "$HERE/swarm.sh"
    tmux new-window -d -t "$TMUX_SESSION" -n dashboard "$local_cmd"
    printf -v local_cmd 'bash %q watch rust' "$HERE/swarm.sh"
    tmux split-window -d -h -t "$TMUX_SESSION:dashboard" "$local_cmd"
    printf -v local_cmd 'bash %q watch tuning' "$HERE/swarm.sh"
    tmux split-window -d -v -t "$TMUX_SESSION:dashboard" "$local_cmd"
    tmux select-layout -t "$TMUX_SESSION:dashboard" tiled
    tmux select-window -t "$TMUX_SESSION:dashboard"
    echo "Dashboard ready. Use swarm.sh attach; Ctrl+B then n/p changes windows."
    ;;
  logs)
    if [[ -d "$STATE/logs" ]]; then
      find "$STATE/logs" -maxdepth 1 -type f -name '*.meta' -printf '%TY-%Tm-%Td %TH:%TM:%TS %f\n' | sort -r
    else echo 'No log folder yet'; fi
    ;;
  watch)
    role="${2:?use hardware, rust, tuning, or audit}"
    [[ "$role" =~ ^(hardware|rust|tuning|audit)$ ]] || { echo 'Unknown role' >&2; exit 2; }
    if [[ -f "$STATE/index/$role.latest" ]]; then
      file="$(cat "$STATE/index/$role.latest")"
    else
      file="$(find "$STATE/logs" -maxdepth 1 -name "$role-*.jsonl" -type f 2>/dev/null | sort | tail -1)"
    fi
    [[ -n "$file" && -f "$file" ]] || { echo 'No session log found.' >&2; exit 2; }
    echo "Following $file (Ctrl+C to stop watching; session continues)"
    tail -n +1 -F "$file" | python3 -u "$HERE/scripts/pretty-log.py" "$role"
    ;;
  audit)
    role="${2:?use hardware, rust, or tuning}"
    [[ "$role" =~ ^(hardware|rust|tuning)$ ]] || { echo 'Unknown review target' >&2; exit 2; }
    [[ -d "$WT_PARENT/$role" ]] || { echo "Missing worktree: $role" >&2; exit 2; }
    bash "$HERE/runner.sh" audit 1 "$ROOT" "$STATE" "$WT_PARENT/$role" "$role"
    ;;
  help|*)
    cat <<'HELPTEXT'
Usage: ./tools/ecu-swarm/swarm.sh <command>
  doctor         Check CLIs and directories
  init           Create 3 Git worktrees and baseline SHA (requires clean Git repo)
  start [2]      Run 3 independent agents in tmux, each for at most N rounds
  attach         Attach to live tmux windows (Ctrl+B then n/p to switch)
  status         Show tmux panes and branch dirty status
  logs           List all recorded sessions
  dashboard      Create tiled tmux window with live logs from 3 agents
  watch ROLE     Read and follow ROLE's JSONL output (hardware/rust/tuning/audit)
  audit ROLE     Run a separate read-only Codex review of a worker branch
  stop           Stop tmux (keeps worktrees/logs)
HELPTEXT
    ;;
esac
