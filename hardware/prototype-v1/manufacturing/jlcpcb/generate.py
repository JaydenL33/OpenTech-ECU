#!/usr/bin/env python3
"""Generate a manufacturer-neutral assembly pack from the KiCad BOM/position files.

The pack deliberately splits the design into:
  * machine-placed parts (SMT + ordinary through-hole connectors),
  * two hand-solder items with no single physical placement (XA1 shield headers,
    A1 carrier sockets),
  * bare-board copper jumpers (no part).

LCSC numbers are left blank for rows that already have a verified MPN; the
assembler's BOM tool is expected to map MPN -> its own library and flag
alternatives. Only rows with no MPN are listed as unresolved decisions.
No part numbers, prices or stock are invented here.
"""
import csv
import pathlib
import re

ROOT = pathlib.Path("hardware/prototype-v1")
OUT = ROOT / "manufacturing" / "jlcpcb"
OUT.mkdir(parents=True, exist_ok=True)
OUTLINE = (110.475, 210.525, -137.025, -36.975)  # xmin, xmax, ymin, ymax (page mm, Y up)


def expand(refs: str):
    for token in refs.split(","):
        token = token.strip()
        m = re.fullmatch(r"([A-Z]+)(\d+)-(?:[A-Z]+)?(\d+)", token)
        if m:
            yield from (m.group(1) + str(n) for n in range(int(m.group(2)), int(m.group(3)) + 1))
        elif token:
            yield token


bom_rows = list(csv.DictReader(open(ROOT / "manufacturing/assembly/v0.4.4d-bom.csv")))
pos_rows = list(csv.DictReader(open(ROOT / "manufacturing/assembly/v0.4.4d-pos.csv")))
lookup = {ref: row for row in bom_rows for ref in expand(row["Refs"])}

# Composite / plug-in items that cannot be a single machine placement.
HAND_SOLDER = {"XA1", "A1"}
MACHINE = [p for p in pos_rows if p["Ref"] not in HAND_SOLDER]

machine_bom, cpl, unresolved = [], [], []
for p in MACHINE:
    ref = p["Ref"]
    row = lookup[ref]
    x, y = float(p["PosX"]), float(p["PosY"])
    if not (OUTLINE[0] <= x <= OUTLINE[1] and OUTLINE[2] <= y <= OUTLINE[3]):
        raise SystemExit(f"machine placement outside outline: {ref} at {x},{y}")
    if p["Side"] != "top":
        raise SystemExit(f"unexpected side for {ref}: {p['Side']}")
    machine_bom.append([row["Value"], ref, row["Footprint"], row["MPN"].strip(), ""])
    cpl.append([ref, p["PosX"], p["PosY"], "Top", p["Rot"]])
    if not row["MPN"].strip():
        unresolved.append([
            ref,
            row["Value"],
            row["Footprint"],
            "",
            "EE choose exact ordering code; verify body/fit against datasheet and PCB",
        ])

hand = [
    [
        "XA1",
        "Arduino Mega shield header set",
        "Shields:Arduino_Mega2560_Shield_IO_Only",
        "3 groups: 24-pad row, 28-pad row, 2x17 column",
        "Samtec TSW/HTSW male header family (candidate; exact segments/gender/tail length TBD)",
        "EE choose exact strips, gender and stack height against the real Mega; hand-solder after machine assembly",
    ],
    [
        "A1-SOCKET",
        "Pololu DRV8825 carrier sockets",
        "Module:Pololu_Breakout-16_15.2x20.3mm",
        "2 x 1x8, 2.54 mm",
        "Samtec SSQ-108-01-F-S (candidate; two 1x8 sockets) or equivalent",
        "EE confirm row spacing/orientation/stack height; hand-solder two 1x8 sockets; carrier remains plug-in",
    ],
]

# Bare copper jumpers are not parts. Keep them visible in the assembly note only.
bare_jumpers = [
    ["JP6-JP10", "SolderJumper_2_Bridged", "5", "bare copper, bridged by default, no part"],
    ["JP11", "Jumper_3_Bridged12", "1", "bare copper, 1-2 bridged by default; cut before external MAP"],
]

readme = f"""# Assembly pack (manufacturer-neutral)

Generated from the KiCad BOM and position file. This is an assembly-data
transform only: no circuit, net, footprint or component-value change.

| Item | Count | Notes |
|---|---:|---|
| Machine-placed references | {len(machine_bom)} | {sum(1 for r in machine_bom if lookup[r[1]]['Footprint'].startswith('Connector_') or lookup[r[1]]['Footprint'].startswith('Package_DIP') or lookup[r[1]]['Footprint'].startswith('TerminalBlock'))} ordinary through-hole connectors/socket; rest SMT |
| CPL rows | {len(cpl)} | all Top; every origin inside the outline |
| Rows with a verified MPN | {len(machine_bom) - len(unresolved)} | LCSC blank by design; assembler maps MPN to its library |
| Connector rows needing an EE choice | {len(unresolved)} | listed in bom-unresolved.csv |
| Hand-solder items | {len(hand)} | XA1 shield headers and A1 carrier sockets; no machine placement |
| Bare copper jumpers | 6 | JP6-JP11; no parts, no assembly |
| Customer-supplied plug-ins | 2 | Arduino Mega, Pololu DRV8825 carrier |

The previous composite XA1 origin (110.0, -107.8 mm) was outside the board
outline and is no longer in the CPL. XA1 is instead a hand-solder header kit:
one 24-pad row, one 28-pad row and one 2x17 column. The Pololu carrier itself
is not in the BOM; two 1x8 board-side sockets are the hand-solder item.

## Files

- `bom-jlcpcb.csv` — Comment, Designator, Footprint, MPN, LCSC Part #.
- `cpl-jlcpcb.csv` — Designator, Mid X, Mid Y, Layer, Rotation.
- `bom-hand-solder.csv` — XA1/A1-SOCKET hand-solder items only.
- `bare-copper-jumpers.csv` — JP6-JP11 board features; no parts.
- `bom-unresolved.csv` — connector rows with no verified MPN; EE decision.
- `generate.py` — reproducible generator; no network access. All CSV output is
  quoted with Python's csv writer so footprints containing commas stay aligned.

## Limits

This is not a quote, DFM approval, H3 or H4 sign-off. LCSC mappings, stock,
prices, rotations and pin-1 polarity must be confirmed by the assembler and
the EE. The board remains a bench prototype.
"""
def write_csv(name: str, header: list[str], rows: list[list[str]]) -> None:
    """Write a properly quoted CSV. Do not hand-join fields: footprints and
    notes contain commas (for example the Phoenix PT-1,5 terminal block)."""
    import csv as _csv
    with (OUT / name).open("w", newline="", encoding="utf-8") as f:
        writer = _csv.writer(f)
        writer.writerow(header)
        writer.writerows(rows)


write_csv(
    "bom-jlcpcb.csv",
    ["Comment", "Designator", "Footprint", "MPN", "LCSC Part #"],
    machine_bom,
)
write_csv(
    "cpl-jlcpcb.csv",
    ["Designator", "Mid X", "Mid Y", "Layer", "Rotation"],
    cpl,
)
write_csv(
    "bom-hand-solder.csv",
    ["Item", "Description", "Footprint", "Quantity/Core", "Candidate", "Decision"],
    hand,
)
write_csv(
    "bom-unresolved.csv",
    ["Designator", "Comment", "Footprint", "MPN", "Decision"],
    unresolved,
)
write_csv(
    "bare-copper-jumpers.csv",
    ["Item", "Footprint", "Qty", "Note"],
    bare_jumpers,
)
(OUT / "README.md").write_text(readme)
print(f"machine_bom={len(machine_bom)} cpl={len(cpl)} unresolved={len(unresolved)} hand={len(hand)}")
assert len(machine_bom) == len(cpl) == len(MACHINE)
assert {r[1] for r in machine_bom} == {r[0] for r in cpl}
assert len(unresolved) == sum(1 for r in machine_bom if not r[3])
print("PASS")
