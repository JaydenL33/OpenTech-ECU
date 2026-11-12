#!/usr/bin/env python3
"""Generate the bench-only harness manifest from the reviewed-source CSVs.

The KiCad package is UNREVIEWED. This script does not make it reviewed; it only
converts the schematic's own connector and test-point exports into a stable JSON
shape that the tuning UI can display without pretending they are OEM cavities.
"""
from __future__ import annotations

import csv
import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
REPO = ROOT.parents[2]
OUT = REPO / "config" / "harness" / "opentech-bench-harness.json"

SUBSYSTEMS = {
    "power_protection": (
        "power",
        "Power protection & hardware inhibit",
        "Fused bench input, reverse-polarity protection, transient clamp, e-fuse, 5 V/3.3 V rails and the hardware arm/inhibit path.",
    ),
    "mcu": (
        "mcu",
        "STM32H743 MCU & service",
        "MCU, reset supervisor, boot strap, watchdog kick, SWD and clock. Pin map is the schematic's own pin assignment.",
    ),
    "injector_drivers": (
        "injectors",
        "Four bench injector sinks",
        "Low-side dummy-load outputs with gate command, drain, shunt and Kelvin test points. No real injector is approved.",
    ),
    "ignition_outputs": (
        "ignition",
        "Logic ignition outputs",
        "0/5 V logic commands to external, verified igniters. No coil primary drive and no high-voltage section.",
    ),
    "sensor_trigger": (
        "sensors",
        "Analogue & trigger conditioning",
        "Protected 0-5 V analogue channels plus comparator-conditioned crank/cam inputs for simulated signals.",
    ),
    "can_debug": (
        "comms",
        "CAN, USB & SWD",
        "CAN transceiver and termination header, USB device, Cortex SWD and service UART.",
    ),
    "aux_outputs": (
        "aux",
        "Auxiliary & relay sinks",
        "Four low-side bench loads plus one relay driver, all default-off.",
    ),
}

SHEET_ORDER = [
    "power_protection",
    "mcu",
    "injector_drivers",
    "ignition_outputs",
    "sensor_trigger",
    "can_debug",
    "aux_outputs",
]


def sha256(path: Path) -> str:
    h = hashlib.sha256()
    h.update(path.read_bytes())
    return h.hexdigest()


def load_csv(name: str) -> list[dict[str, str]]:
    with (ROOT / name).open(newline="", encoding="utf-8-sig") as f:
        return list(csv.DictReader(f))


def main() -> None:
    bom = load_csv("BOM.csv")
    pin_rows = load_csv("pin_map.csv")
    test_rows = load_csv("test_points.csv")

    sheet_of = {row["Reference"]: row["Sheet"] for row in bom}
    value_of = {row["Reference"]: row["Value"] for row in bom}
    mpn_of = {row["Reference"]: row.get("MPN", "") for row in bom}

    pins_by_ref: dict[str, list[dict[str, str]]] = {}
    for row in pin_rows:
        pins_by_ref.setdefault(row["Reference"], []).append(row)

    connectors = []
    for ref in sorted(pins_by_ref):
        if not (ref.startswith("J") or ref.startswith("JP")):
            continue
        pins = []
        for row in pins_by_ref[ref]:
            pins.append(
                {
                    "pin": row["Pin"],
                    "name": row["Pin name"],
                    "net": row["Net"],
                    "function": row["Function / constraint"],
                    "oem_assignment": row["OEM assignment"],
                }
            )
        connectors.append(
            {
                "reference": ref,
                "label": value_of.get(ref, ""),
                "sheet": sheet_of.get(ref, ""),
                "mpn": mpn_of.get(ref, ""),
                "pins": pins,
            }
        )

    connectors.sort(key=lambda c: (SHEET_ORDER.index(c["sheet"]) if c["sheet"] in SHEET_ORDER else 99, c["reference"]))

    subsystems = []
    for sheet in SHEET_ORDER:
        ident, name, description = SUBSYSTEMS[sheet]
        subsystems.append(
            {
                "id": ident,
                "name": name,
                "sheet": sheet,
                "description": description,
                "connectors": [c["reference"] for c in connectors if c["sheet"] == sheet],
            }
        )

    manifest = {
        "schema_version": 1,
        "id": "opentech-bench-ecu-v0.1",
        "title": "OpenTech bench ECU harness map",
        "status": "UNREVIEWED_BENCH_ONLY",
        "physical_use": "PROHIBITED",
        "notice": (
            "Board-local bench connectors only. Pin numbers are NOT Mitsubishi or any OEM cavity numbers. "
            "The schematic and PCB draft are UNREVIEWED; no connector, current rating or harness is approved for a vehicle."
        ),
        "source": {
            "design": "hardware/kicad/evo8-bench-ecu",
            "erc": "reports/erc.json (0 violations; unreviewed)",
            "drc": "reports/drc.json (14 violations, 499 unconnected; not manufacturable)",
            "files": [
                {"path": "BOM.csv", "sha256": sha256(ROOT / "BOM.csv")},
                {"path": "pin_map.csv", "sha256": sha256(ROOT / "pin_map.csv")},
                {"path": "test_points.csv", "sha256": sha256(ROOT / "test_points.csv")},
            ],
        },
        "generated_by": "hardware/kicad/evo8-bench-ecu/tools/export_harness_manifest.py",
        "subsystems": subsystems,
        "connectors": connectors,
        "test_points": [
            {
                "reference": row["Reference"],
                "net": row["Net"],
                "purpose": row["Purpose / measurement"],
            }
            for row in test_rows
        ],
        "reference_target": {
            "vehicle": "Mitsubishi Lancer Evolution VIII GSR (JDM, CT9A, 4G63T, 6MT)",
            "compatibility": "UNVERIFIED",
            "note": (
                "This bench harness is not an Evo VIII adapter. The factory ECU part number, connector architecture, "
                "pinout, trigger pattern, injectors and ignition topology remain unverified. See AUDIT/ECU_PINOUT_VERIFICATION.md."
            ),
        },
    }

    OUT.parent.mkdir(parents=True, exist_ok=True)
    OUT.write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")
    print(f"wrote {OUT.relative_to(REPO)}")
    print(f"connectors={len(connectors)} test_points={len(test_rows)} subsystems={len(subsystems)}")


if __name__ == "__main__":
    main()
