#!/usr/bin/env python3
"""Compare round-2 trigger-loss telemetry with the pinned round-1 Rust source.

Run verify-output-timing.py first. This builds the archived reference in a
temporary directory, with a separate reference build directory.
No upstream Speeduino binary is executed. No network access is required.
"""
import io
import hashlib
import json
from pathlib import Path
import shlex
import subprocess
import sys
import tarfile
import tempfile

workspace = Path(__file__).resolve().parents[1]
repository = workspace.parent
evidence = Path(sys.argv[1]).resolve()
reference_commit = "82f466a021e4b98af8931b5b3da9a3ba237a069b"
with (evidence / "reference-and-diff.txt").open("w") as record:
    def report(message):
        print(message, flush=True)
        record.write(message + "\n")
        record.flush()

    command = ["git", "archive", reference_commit, "ecu"]
    report("$ " + shlex.join(command))
    archive = subprocess.check_output(command, cwd=repository)
    report("EXIT=0; archive SHA256=" + hashlib.sha256(archive).hexdigest())
    with tempfile.TemporaryDirectory(prefix="ecu-trigger-reference-") as directory:
        with tarfile.open(fileobj=io.BytesIO(archive)) as source:
            source.extractall(directory, filter="data")
        output = workspace / "runs/trigger-reference"
        command = ["cargo", "run", "--offline", "--locked", "--manifest-path",
                   str(Path(directory) / "ecu/Cargo.toml"), "--target-dir", str(workspace / "target/trigger-reference"),
                   "-q", "-p", "ecu-sim-cli", "--", "run", "--scenario", "trigger_loss",
                   "--seed", "43", "--out", str(output)]
        report("$ " + shlex.join(command))
        result = subprocess.run(command, cwd=repository, stdout=subprocess.PIPE,
                                stderr=subprocess.STDOUT, text=True)
        report(result.stdout + f"EXIT={result.returncode}")
        if result.returncode:
            sys.exit(result.returncode)
        expected = json.loads((repository / "AUDIT/rust-round-1/manifest.json").read_text())
        key = "ecu/runs/output-timing/trigger_loss/log.json"
        reference_bytes = (output / "log.json").read_bytes()
        assert hashlib.sha256(reference_bytes).hexdigest() == expected["artifacts_sha256"][key]
        report("Reference log matches recorded round-1 SHA256=" + expected["artifacts_sha256"][key])
        reference = json.loads(reference_bytes)
        current_bytes = (workspace / "runs/output-timing/trigger_loss/log.json").read_bytes()
        report("Candidate log SHA256=" + hashlib.sha256(current_bytes).hexdigest())
        current = json.loads(current_bytes)
        assert reference["metadata"] == current["metadata"]
        assert reference["events"] == current["events"]
        assert len(reference["frames"]) == len(current["frames"])
        differences = []
        for old, new in zip(reference["frames"], current["frames"]):
            assert old.keys() == new.keys()
            for key in old:
                if key != "cycle_angle_deg":
                    assert old[key] == new[key], (old["timestamp_us"], key, old[key], new[key])
            if old["cycle_angle_deg"] != new["cycle_angle_deg"]:
                assert abs(abs(old["cycle_angle_deg"] - new["cycle_angle_deg"]) - 360.0) < 0.001
                differences.append({"time_us": old["timestamp_us"],
                                    "old_cycle_deg": old["cycle_angle_deg"],
                                    "new_cycle_deg": new["cycle_angle_deg"]})
        report("Metadata, log events and all telemetry fields except cycle_angle_deg match exactly.")
        assert len(differences) == 8
        report("Cycle angle differences: " + json.dumps(differences))
        assert (output / "outputs.json").read_bytes() == (workspace / "runs/output-timing/trigger_loss/outputs.json").read_bytes()
        report("Output traces match byte-for-byte; PASS")
