#!/usr/bin/env python3
"""Reproduce host output/trigger timing checks, commands, exit codes and hashes.

Run from any directory with a working Rust + C linker toolchain on PATH.
Artifacts under ecu/runs are generated; the evidence directory is caller-selected.
"""

import hashlib
import json
import os
from pathlib import Path
import shlex
import subprocess
import sys


workspace = Path(__file__).resolve().parents[1]
repository = workspace.parent
evidence = Path(sys.argv[1]).resolve() if len(sys.argv) > 1 else workspace / "runs/timing-evidence"
evidence.mkdir(parents=True, exist_ok=True)
commands = [
    ["git", "rev-parse", "HEAD"],
    ["rustc", "--version"],
    ["cargo", "--version"],
    ["cc", "--version"],
    ["cargo", "fmt", "--all", "--", "--check"],
    ["cargo", "clippy", "--locked", "--workspace", "--all-targets", "--", "-D", "warnings"],
    ["cargo", "test", "--locked", "--workspace"],
    ["cargo", "test", "--locked", "-p", "ecu-simulator", "--test", "trigger_waveform",
     "full_cycles_match_independent_f64_wheel_vectors", "--", "--nocapture"],
    ["cargo", "check", "--locked", "-p", "ecu-types", "-p", "ecu-fuel", "-p", "ecu-ignition",
     "-p", "ecu-trigger", "-p", "ecu-scheduler", "-p", "ecu-protection", "--no-default-features"],
]
for scenario, seed in [("mixture_sweep", "42"), ("trigger_loss", "43")]:
    commands.append([
        "cargo", "run", "--locked", "-q", "-p", "ecu-sim-cli", "--", "run", "--scenario", scenario,
        "--seed", seed, "--out", f"runs/output-timing/{scenario}", "--check-determinism",
    ])

results = []
with (evidence / "verification.txt").open("w") as record:
    record.write(f"cwd: {workspace}\nPATH: {os.environ.get('PATH', '')}\n")
    for command in commands:
        label = shlex.join(command)
        print(label, flush=True)
        result = subprocess.run(command, cwd=workspace, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True)
        record.write(f"\n$ {label}\n{result.stdout}\nEXIT={result.returncode}\n")
        record.flush()
        results.append({"command": command, "exit": result.returncode})
        print(result.stdout[-1800:], end="", flush=True)
        if result.returncode:
            sys.exit(result.returncode)

artifacts = {}
for scenario in ["mixture_sweep", "trigger_loss"]:
    directory = workspace / "runs/output-timing" / scenario
    for name in ["calibration.json", "log.json", "outputs.json", "triggers.json", "summary.json"]:
        path = directory / name
        artifacts[str(path.relative_to(repository))] = hashlib.sha256(path.read_bytes()).hexdigest()
    trace = json.loads((directory / "outputs.json").read_text())
    excerpt = {key: value for key, value in trace.items() if key != "outputs"}
    excerpt["first_three_retained_outputs"] = trace["outputs"][:3]
    (evidence / f"{scenario}-trace-excerpt.json").write_text(json.dumps(excerpt, indent=2) + "\n")
    inputs = json.loads((directory / "triggers.json").read_text())
    assert inputs["simulation_only"] is True
    assert inputs["events_emitted"] == inputs["events_dropped"] + len(inputs["inputs"])
    assert inputs["inputs"][0]["sequence"] == inputs["events_dropped"]
    excerpt = {key: value for key, value in inputs.items() if key != "inputs"}
    excerpt["first_three_retained_inputs"] = inputs["inputs"][:3]
    (evidence / f"{scenario}-trigger-excerpt.json").write_text(json.dumps(excerpt, indent=2) + "\n")

source_paths = sorted(
    path for directory in [workspace / "crates", workspace / "apps/ecu-sim-cli", workspace / "apps/studio-service", workspace / "firmware/stm32"]
    for path in directory.rglob("*.rs")
)
source_paths += [workspace / "Cargo.lock", workspace / "tools/verify-output-timing.py"]
manifest = {str(path.relative_to(repository)): hashlib.sha256(path.read_bytes()).hexdigest() for path in source_paths}
(evidence / "manifest.json").write_text(json.dumps({
    "commands": results, "sources_sha256": manifest, "artifacts_sha256": artifacts,
}, indent=2) + "\n")
print(f"Evidence: {evidence}")
