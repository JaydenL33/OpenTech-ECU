#!/usr/bin/env python3
"""HW-H2-01 revision of hardware/prototype-v1 (R-04 ... R-09).

Run inside the pinned KiCad container from the repository root:

    IMG=kicad/kicad@sha256:e638b79b0321f29395a5b783e94bb9f3c73303e8da15da27b8f5cb4b67a37729
    RUN="docker run --rm --user $(id -u):$(id -g) \
         -v $PWD/hardware/prototype-v1:/work -v $PWD/hardware/tools:/tools:ro $IMG"
    $RUN python3 /tools/apply_hw_h2_01_revision.py sch  /work   # R-06, R-07, R-08 (schematic)
    $RUN python3 /tools/apply_hw_h2_01_revision.py pcb  /work   # R-04, R-05, R-09 (PCB + libraries)
    $RUN kicad-cli pcb drc --format json --severity-all --severity-exclusions \
         --output /work/verification/drc-for-exclusions.json /work/v0.4.4d.kicad_pcb
    $RUN python3 /tools/apply_hw_h2_01_revision.py exclusions /work   # HW-DRC-01 exclusions

Every step is idempotent: re-running reports "already applied" and leaves the
files byte-identical. Each change is recorded in AUDIT/HARDWARE_DIFF.md.

  R-04  Three M3 (3.2 mm NPTH) corner mounting holes H1-H3 with a 3.8 mm radius
        copper/track/via/pad keep-out on F.Cu and B.Cu. No net, no plating,
        board-only, excluded from BOM and position files. The fourth (top-right)
        corner is blocked by J3 and the VDD edge track; see HARDWARE_DIFF.md.
  R-05  Library-link repair: project-local footprint libraries Shields, Misc and
        Speeduino_Legacy are written from the footprints embedded in the board
        (byte-for-byte geometry); C19 is relinked to
        Speeduino_Legacy:C_0805_2012Metric_Pad1.15x1.40mm_HandSolder in both the
        schematic and the PCB. Unused library-table entries whose files never
        existed upstream are removed.
  R-06  Device:CP1 (KiCad 5 name) -> Device:C_Polarized_US (KiCad 9 name). The
        KiCad 9 library symbol is geometrically identical (same pin numbers,
        positions, lengths and types), so no connection changes.
  R-07  Five PWR_FLAG symbols where power enters through a connector, diode,
        jumper or fuse: GND and 12V-SW at J3, VRegIn after D14, Vdrive after
        JP1, VDDA after F1.
  R-08  No-connect flags on 29 genuinely unused pins (unused Arduino header and
        ICSP pins, the unused second pole of S1, VNLD5090 STATUS outputs of U7/U9,
        J2 pin 6, DRV8825 FLT/M0/M1/M2). The three dangling ICSP global labels
        MISO/MOSI/SCLK (pins absent from the PCB footprint) and their 1.27 mm
        wires are removed. Two dangling 8.89 mm wire stubs and one dangling
        no-connect flag on the Power sheet are removed.
  R-09  Per-item DRC exclusions (with comments) for the 199 courtyard overlaps
        involving unpopulated proto pads or the J1/J5/J6/J8 headers that sit on
        one continuous 2.54 mm pitch. The JP6-JP11 jumper violations are NOT
        excluded; they stay visible in the report as documented exceptions.
"""
from __future__ import annotations

import hashlib
import json
import math
import re
import sys
import uuid
from pathlib import Path

NS = uuid.UUID("5f2d2f9e-33ef-4b75-a14f-a473c0000001")  # deterministic UUIDs
ROOT_UUID = "60cb48aa-a10d-4423-aa12-37d6b383555c"
SHEET_PATH = {
    "v0.4.4d.kicad_sch": f"/{ROOT_UUID}",
    "connector.kicad_sch": f"/{ROOT_UUID}/00000000-0000-0000-0000-00005cd19033",
    "power.kicad_sch": f"/{ROOT_UUID}/00000000-0000-0000-0000-00005cd18ec3",
    "Fuel_Ignition.kicad_sch": f"/{ROOT_UUID}/00000000-0000-0000-0000-00005cd18c17",
    "outputs.kicad_sch": f"/{ROOT_UUID}/00000000-0000-0000-0000-00005cd191f5",
}
KICAD_SYM = Path("/usr/share/kicad/symbols")
KICAD_FP = Path("/usr/share/kicad/footprints")
LEGACY_C19 = "C_0805_2012Metric_Pad1.15x1.40mm_HandSolder"


def uid(tag: str) -> str:
    return str(uuid.uuid5(NS, tag))


def sha(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def sexpr_block(text: str, start: int) -> int:
    """Return index one past the parenthesis that closes text[start]."""
    depth = 0
    in_str = False
    i = start
    while i < len(text):
        c = text[i]
        if in_str:
            if c == "\\":
                i += 1
            elif c == '"':
                in_str = False
        elif c == '"':
            in_str = True
        elif c == "(":
            depth += 1
        elif c == ")":
            depth -= 1
            if depth == 0:
                return i + 1
        i += 1
    raise ValueError("unbalanced s-expression")


def lib_symbol(lib_file: Path, name: str, new_name: str) -> str:
    text = lib_file.read_text()
    i = text.index(f'(symbol "{name}"')
    block = text[i:sexpr_block(text, i)]
    block = block.replace(f'(symbol "{name}"', f'(symbol "{new_name}"', 1)
    # library files are indented one tab less than schematic lib_symbols
    return "\n".join(("\t" + ln) if ln else ln for ln in block.split("\n"))


# ----------------------------------------------------------------------------
# Schematic edits (plain text, KiCad 9 s-expression format)
# ----------------------------------------------------------------------------
NO_CONNECTS = {
    # (sheet, x, y): reason
    "v0.4.4d.kicad_sch": [
        (120.65, 57.15, "XA1 SDA (shield pad unused; D20 used via J5)"),
        (120.65, 59.69, "XA1 SCL (shield pad unused; D21 used via J5)"),
        (120.65, 92.71, "XA1 A11 unused"),
        (120.65, 95.25, "XA1 A12 unused"),
        (120.65, 97.79, "XA1 A13 unused"),
        (120.65, 100.33, "XA1 A14 unused"),
        (120.65, 102.87, "XA1 A15 unused"),
        (120.65, 105.41, "XA1 AREF unused"),
        (120.65, 113.03, "XA1 IOREF unused"),
        (120.65, 130.81, "XA1 3V3 unused"),
        (120.65, 140.97, "XA1 VIN unused"),
        (147.32, 20.32, "XA1 ICSP MISO, no pad in IO_Only footprint"),
        (149.86, 20.32, "XA1 ICSP MOSI, no pad in IO_Only footprint"),
        (152.40, 20.32, "XA1 ICSP SCK, no pad in IO_Only footprint"),
        (154.94, 20.32, "XA1 ICSP 5V, no pad in IO_Only footprint"),
        (157.48, 20.32, "XA1 ICSP GND, no pad in IO_Only footprint"),
        (160.02, 20.32, "XA1 ICSP RESET, no pad in IO_Only footprint"),
    ],
    "power.kicad_sch": [
        (180.34, 128.27, "S1 NC_2 (second pole unused)"),
        (180.34, 130.81, "S1 C_2 (second pole unused)"),
        (180.34, 133.35, "S1 NO_2 (second pole unused)"),
    ],
    "outputs.kicad_sch": [
        (55.88, 46.99, "U7 STATUS1 unused"),
        (55.88, 54.61, "U7 STATUS2 unused"),
        (63.50, 149.86, "J2 pin 6 unused"),
        (201.93, 46.99, "U9 STATUS1 unused"),
        (201.93, 54.61, "U9 STATUS2 unused"),
        (205.74, 102.87, "A1 FLT unused"),
        (205.74, 123.19, "A1 M0 unused (module pull-down)"),
        (205.74, 125.73, "A1 M1 unused (module pull-down)"),
        (205.74, 128.27, "A1 M2 unused (module pull-down)"),
    ],
}

# (sheet, ref, x, y, rotation, net, optional extra wire from (x0,y0))
PWR_FLAGS = [
    ("connector.kicad_sch", "#FLG01", 222.25, 81.28, 0, "GND at J3.1", None),
    ("connector.kicad_sch", "#FLG02", 222.25, 86.36, 180, "12V-SW at J3.2", None),
    ("power.kicad_sch", "#FLG03", 139.70, 62.23, 0, "VRegIn after D14", None),
    ("power.kicad_sch", "#FLG04", 185.42, 54.61, 0, "VDDA after F1", (180.34, 54.61)),
    ("Fuel_Ignition.kicad_sch", "#FLG05", 156.21, 125.73, 180, "Vdrive after JP1", (151.13, 125.73)),
]

REMOVE_WIRES = {
    "power.kicad_sch": [((193.04, 68.58), (184.15, 68.58)), ((193.04, 78.74), (184.15, 78.74))],
    "v0.4.4d.kicad_sch": [((147.32, 19.05), (147.32, 20.32)), ((149.86, 19.05), (149.86, 20.32)),
                          ((152.40, 19.05), (152.40, 20.32))],
}
REMOVE_LABEL_UUIDS = {
    "v0.4.4d.kicad_sch": ["ad759c24-e05f-4864-9df2-bfbcef93ae29",  # MISO
                          "a3bf5fce-09e3-4ac8-8e1a-a2242e3d18a7",  # MOSI
                          "11bc0b06-f255-4ce7-891d-8ff0fbbd9fac"],  # SCLK
}
REMOVE_NC_UUIDS = {"power.kicad_sch": ["723a6674-9a27-4d4e-9b23-221427fb6369"]}


def fmt(v: float) -> str:
    s = f"{v:.4f}".rstrip("0").rstrip(".")
    return s if s != "-0" else "0"


def wire_sexpr(a, b, tag):
    return (f"\t(wire\n\t\t(pts\n\t\t\t(xy {fmt(a[0])} {fmt(a[1])}) (xy {fmt(b[0])} {fmt(b[1])})\n"
            f"\t\t)\n\t\t(stroke\n\t\t\t(width 0)\n\t\t\t(type default)\n\t\t)\n"
            f"\t\t(uuid \"{uid(tag)}\")\n\t)\n")


def nc_sexpr(x, y, tag):
    return f"\t(no_connect\n\t\t(at {fmt(x)} {fmt(y)})\n\t\t(uuid \"{uid(tag)}\")\n\t)\n"


def pwr_flag_sexpr(sheet, ref, x, y, rot, tag):
    ty = y - 3.81 if rot == 0 else y + 3.81
    ry = y - 1.905 if rot == 0 else y + 1.905
    return f"""\t(symbol
\t\t(lib_id "power:PWR_FLAG")
\t\t(at {fmt(x)} {fmt(y)} {rot})
\t\t(unit 1)
\t\t(exclude_from_sim no)
\t\t(in_bom yes)
\t\t(on_board yes)
\t\t(dnp no)
\t\t(uuid "{uid(tag)}")
\t\t(property "Reference" "{ref}"
\t\t\t(at {fmt(x)} {fmt(ry)} 0)
\t\t\t(effects
\t\t\t\t(font
\t\t\t\t\t(size 1.27 1.27)
\t\t\t\t)
\t\t\t\t(hide yes)
\t\t\t)
\t\t)
\t\t(property "Value" "PWR_FLAG"
\t\t\t(at {fmt(x)} {fmt(ty)} 0)
\t\t\t(effects
\t\t\t\t(font
\t\t\t\t\t(size 1.27 1.27)
\t\t\t\t)
\t\t\t)
\t\t)
\t\t(property "Footprint" ""
\t\t\t(at {fmt(x)} {fmt(y)} 0)
\t\t\t(effects
\t\t\t\t(font
\t\t\t\t\t(size 1.27 1.27)
\t\t\t\t)
\t\t\t\t(hide yes)
\t\t\t)
\t\t)
\t\t(property "Datasheet" "~"
\t\t\t(at {fmt(x)} {fmt(y)} 0)
\t\t\t(effects
\t\t\t\t(font
\t\t\t\t\t(size 1.27 1.27)
\t\t\t\t)
\t\t\t\t(hide yes)
\t\t\t)
\t\t)
\t\t(property "Description" "Special symbol for telling ERC where power comes from"
\t\t\t(at {fmt(x)} {fmt(y)} 0)
\t\t\t(effects
\t\t\t\t(font
\t\t\t\t\t(size 1.27 1.27)
\t\t\t\t)
\t\t\t\t(hide yes)
\t\t\t)
\t\t)
\t\t(pin "1"
\t\t\t(uuid "{uid(tag + '/pin1')}")
\t\t)
\t\t(instances
\t\t\t(project "v0.4.4d"
\t\t\t\t(path "{SHEET_PATH[sheet]}"
\t\t\t\t\t(reference "{ref}")
\t\t\t\t\t(unit 1)
\t\t\t\t)
\t\t\t)
\t\t)
\t)
"""


def remove_top_item(text: str, kind: str, predicate) -> tuple[str, int]:
    removed = 0
    pos = 0
    pat = re.compile(r"\n\t\(" + re.escape(kind) + r"[\s\n]")
    while True:
        m = pat.search(text, pos)
        if not m:
            return text, removed
        i = m.start()
        j = sexpr_block(text, i + 2)
        block = text[i + 2:j]
        if predicate(block):
            text = text[:i] + text[j:]
            removed += 1
            pos = i
        else:
            pos = j


def insert_items(text: str, items: str) -> str:
    # root sheet: before sheet_instances; sub-sheets: before the final ")"
    k = text.rindex("\n\t(sheet_instances") if "\n\t(sheet_instances" in text else text.rstrip().rindex("\n)")
    return text[:k + 1] + items + text[k + 1:]


def add_lib_symbol(text: str, name: str, block: str) -> str:
    if f'(symbol "{name}"' in text:
        return text
    i = text.index("\n\t(lib_symbols")
    j = sexpr_block(text, i + 2)
    close = text.rindex("\n\t)", i, j)
    return text[:close] + "\n\t\t" + block + text[close:]


def do_sch(work: Path) -> int:
    changes: list[str] = []
    pf_block = lib_symbol(KICAD_SYM / "power.kicad_sym", "PWR_FLAG", "power:PWR_FLAG").lstrip("\t")
    cp_block = lib_symbol(KICAD_SYM / "Device.kicad_sym", "C_Polarized_US",
                          "Device:C_Polarized_US").lstrip("\t")
    sheets = sorted({*NO_CONNECTS, *REMOVE_WIRES, *REMOVE_LABEL_UUIDS, *REMOVE_NC_UUIDS,
                     *(p[0] for p in PWR_FLAGS), "power.kicad_sch"})
    for sheet in sheets:
        path = work / sheet
        before = sha(path)
        text = path.read_text()
        orig = text

        # R-06: CP1 -> C_Polarized_US (power sheet holds both instances)
        if '(symbol "Device:CP1"' in text:
            i = text.index('(symbol "Device:CP1"')
            text = text[:i] + cp_block + text[sexpr_block(text, i):]
            n = text.count('(lib_id "Device:CP1")')
            text = text.replace('(lib_id "Device:CP1")', '(lib_id "Device:C_Polarized_US")')
            changes.append(f"R-06 {sheet}: Device:CP1 -> Device:C_Polarized_US ({n} instances)")

        # R-08 removals
        for a, b in REMOVE_WIRES.get(sheet, []):
            pat_a = f"(xy {fmt(a[0])} {fmt(a[1])}) (xy {fmt(b[0])} {fmt(b[1])})"
            pat_b = f"(xy {fmt(b[0])} {fmt(b[1])}) (xy {fmt(a[0])} {fmt(a[1])})"
            text, n = remove_top_item(text, "wire", lambda blk: pat_a in blk or pat_b in blk)
            if n:
                changes.append(f"R-08 {sheet}: removed wire {a}->{b}")
        for u in REMOVE_LABEL_UUIDS.get(sheet, []):
            text, n = remove_top_item(text, "global_label", lambda blk, u=u: f'(uuid "{u}")' in blk)
            if n:
                changes.append(f"R-08 {sheet}: removed dangling global label {u}")
        for u in REMOVE_NC_UUIDS.get(sheet, []):
            text, n = remove_top_item(text, "no_connect", lambda blk, u=u: f'(uuid "{u}")' in blk)
            if n:
                changes.append(f"R-08 {sheet}: removed dangling no-connect {u}")

        # R-08 no-connect flags
        new_items = ""
        for x, y, why in NO_CONNECTS.get(sheet, []):
            tag = f"nc/{sheet}/{x}/{y}"
            if f'(uuid "{uid(tag)}")' in text:
                continue
            new_items += nc_sexpr(x, y, tag)
            changes.append(f"R-08 {sheet}: no-connect at ({x}, {y}) {why}")

        # R-07 PWR_FLAGs
        flags = [p for p in PWR_FLAGS if p[0] == sheet]
        if flags:
            text = add_lib_symbol(text, "power:PWR_FLAG", pf_block)
        for _, ref, x, y, rot, why, wire_from in flags:
            tag = f"flag/{sheet}/{ref}"
            if f'(uuid "{uid(tag)}")' in text:
                continue
            if wire_from:
                new_items += wire_sexpr(wire_from, (x, y), tag + "/wire")
            new_items += pwr_flag_sexpr(sheet, ref, x, y, rot, tag)
            changes.append(f"R-07 {sheet}: {ref} PWR_FLAG at ({x}, {y}) for {why}")

        # R-05: C19 footprint relink (power sheet)
        old_fp = f'(property "Footprint" "Capacitor_SMD:{LEGACY_C19}"'
        if old_fp in text:
            text = text.replace(old_fp, f'(property "Footprint" "Speeduino_Legacy:{LEGACY_C19}"')
            changes.append(f"R-05 {sheet}: C19 footprint -> Speeduino_Legacy:{LEGACY_C19}")

        if new_items:
            text = insert_items(text, new_items)
        if text != orig:
            path.write_text(text)
        print(f"{sheet}: sha256 {before[:12]} -> {sha(path)[:12]}")

    # R-05: library tables
    sym = work / "sym-lib-table"
    sym_new = ("(sym_lib_table\n  (version 7)\n"
               "  (lib (name \"v0.4.4c-rescue\")(type \"Legacy\")(uri \"${KIPRJMOD}/v0.4.4c-rescue.lib\")"
               "(options \"\")(descr \"Upstream Speeduino v0.4.4 rescue library; holds every custom symbol\"))\n)\n")
    fp = work / "fp-lib-table"
    fp_new = ("(fp_lib_table\n  (version 7)\n"
              "  (lib (name \"Shields\")(type \"KiCad\")(uri \"${KIPRJMOD}/lib/Shields/Shields.pretty\")"
              "(options \"\")(descr \"Extracted from the routed v0.4.4d board (HW-H2-01 R-05)\"))\n"
              "  (lib (name \"Misc\")(type \"KiCad\")(uri \"${KIPRJMOD}/lib/Misc/Misc.pretty\")"
              "(options \"\")(descr \"Extracted from the routed v0.4.4d board (HW-H2-01 R-05)\"))\n"
              "  (lib (name \"Speeduino_Legacy\")(type \"KiCad\")(uri \"${KIPRJMOD}/lib/Speeduino_Legacy/Speeduino_Legacy.pretty\")"
              "(options \"\")(descr \"KiCad 5 footprint variants absent from KiCad 9 libraries (HW-H2-01 R-05)\"))\n)\n")
    for p, content in ((sym, sym_new), (fp, fp_new)):
        if p.read_text() != content:
            p.write_text(content)
            changes.append(f"R-05 rewrote {p.name}")

    print(f"changes applied: {len(changes)}")
    for c in changes:
        print("  -", c)
    return 0


# ----------------------------------------------------------------------------
# PCB edits (pcbnew API)
# ----------------------------------------------------------------------------
HOLES = [  # ref, x, y, reference-text x offset (outline 110.475..210.525 x 36.975..137.025 mm)
    ("H1", 113.975, 40.475, 6.0),    # top-left, 3.5 mm inset
    ("H2", 113.975, 133.525, 6.0),   # bottom-left, 3.5 mm inset
    ("H3", 207.025, 133.525, -6.0),  # bottom-right, 3.5 mm inset (clears upstream logo);
                                     # reference on F.Fab: no free silkscreen space
]
KEEPOUT_R_MM = 3.8   # 7.0 mm washer / M3 pan head + 0.3 mm margin
KEEPOUT_SEGMENTS = 32


def do_pcb(work: Path) -> int:
    import pcbnew

    board_path = work / "v0.4.4d.kicad_pcb"
    before = sha(board_path)
    board = pcbnew.LoadBoard(str(board_path))
    changes: list[str] = []
    nm = pcbnew.FromMM

    # R-05: export embedded footprints to project-local libraries (before any edit)
    exports = {"XA1": "Shields", "U5": "Misc", "S1": "Misc", "C19": "Speeduino_Legacy"}
    for ref, lib in exports.items():
        fp = board.FindFootprintByReference(ref)
        libdir = work / "lib" / lib / f"{lib}.pretty"
        name = fp.GetFPID().GetLibItemName().wx_str() if hasattr(fp.GetFPID().GetLibItemName(), "wx_str") \
            else str(fp.GetFPID().GetLibItemName())
        target = libdir / f"{name}.kicad_mod"
        if target.exists():
            print(f"R-05 {ref}: {target.relative_to(work)} already present")
            continue
        libdir.mkdir(parents=True, exist_ok=True)
        clone = pcbnew.FOOTPRINT(fp)
        clone.SetParent(None) if hasattr(clone, "SetParent") else None
        clone.SetPosition(pcbnew.VECTOR2I(0, 0))
        clone.SetOrientationDegrees(0)
        clone.SetReference("REF**")
        for pad in clone.Pads():
            pad.SetNetCode(0)
        pcbnew.PCB_IO_MGR.PluginFind(pcbnew.PCB_IO_MGR.KICAD_SEXP).FootprintSave(str(libdir), clone)
        changes.append(f"R-05 exported {ref} -> lib/{lib}/{lib}.pretty/{name}.kicad_mod")

    # R-05: relink C19 to the local legacy library
    c19 = board.FindFootprintByReference("C19")
    if str(c19.GetFPID().GetLibNickname()) != "Speeduino_Legacy":
        c19.SetFPID(pcbnew.LIB_ID("Speeduino_Legacy", LEGACY_C19))
        changes.append(f"R-05 C19 FPID -> Speeduino_Legacy:{LEGACY_C19}")
    else:
        print("R-05 C19: already relinked")

    # R-04: mounting holes
    existing = {f.GetReference() for f in board.GetFootprints()}
    zone_names = {z.GetZoneName() for z in board.Zones()}
    for ref, x, y, ref_dx in HOLES:
        if ref in existing:
            print(f"R-04 {ref}: already present")
        else:
            fp = pcbnew.FootprintLoad(str(KICAD_FP / "MountingHole.pretty"), "MountingHole_3.2mm_M3")
            fp.SetFPID(pcbnew.LIB_ID("MountingHole", "MountingHole_3.2mm_M3"))
            fp.SetReference(ref)
            fp.SetValue("MountingHole_3.2mm_M3")
            fp.SetPosition(pcbnew.VECTOR2I(nm(x), nm(y)))
            fp.SetExcludedFromBOM(True)
            fp.SetExcludedFromPosFiles(True)
            fp.SetBoardOnly(True)
            if hasattr(fp, "SetDNP"):
                fp.SetDNP(False)
            board.Add(fp)
            # keep the silkscreen reference on the board, beside the hole
            fp.Reference().SetPosition(pcbnew.VECTOR2I(nm(x + ref_dx), nm(y)))
            if ref == "H3":
                fp.Reference().SetLayer(pcbnew.F_Fab)
            for pad in fp.Pads():
                assert pad.GetAttribute() == pcbnew.PAD_ATTRIB_NPTH, "mounting hole must be NPTH"
                pad.SetNetCode(0)
            changes.append(f"R-04 {ref}: MountingHole_3.2mm_M3 NPTH at ({x}, {y}) mm")

        # Board-level rule area (a footprint-level one would differ from the
        # library footprint and would flag the hole's own NPTH pad).
        # The polygon circumscribes the keep-out circle so that its flats (the
        # inradius) are exactly KEEPOUT_R_MM from the hole centre.
        vertex_r = KEEPOUT_R_MM / math.cos(math.pi / KEEPOUT_SEGMENTS)
        if f"{ref}_keepout" in zone_names:
            z_old = [z for z in board.Zones() if z.GetZoneName() == f"{ref}_keepout"][0]
            v0 = z_old.Outline().CVertex(0)
            if abs(pcbnew.ToMM(v0.x) - (x + vertex_r)) < 1e-3 and abs(pcbnew.ToMM(v0.y) - y) < 1e-3:
                print(f"R-04 {ref}_keepout: already present")
                continue
            board.Remove(z_old)
            changes.append(f"R-04 {ref}_keepout: replaced inscribed polygon with circumscribed one")
        zone = pcbnew.ZONE(board)
        zone.SetIsRuleArea(True)
        zone.SetDoNotAllowTracks(True)
        zone.SetDoNotAllowVias(True)
        zone.SetDoNotAllowPads(False)  # pad clearance verified analytically (>= 9 mm)
        zone.SetDoNotAllowCopperPour(True)
        zone.SetDoNotAllowFootprints(False)
        lset = pcbnew.LSET()
        lset.AddLayer(pcbnew.F_Cu)
        lset.AddLayer(pcbnew.B_Cu)
        zone.SetLayerSet(lset)
        zone.SetZoneName(f"{ref}_keepout")
        outline = zone.Outline()
        outline.NewOutline()
        for k in range(KEEPOUT_SEGMENTS):
            a = 2 * math.pi * k / KEEPOUT_SEGMENTS
            outline.Append(nm(x + vertex_r * math.cos(a)), nm(y + vertex_r * math.sin(a)))
        board.Add(zone)
        changes.append(f"R-04 {ref}_keepout: rule area, min radius {KEEPOUT_R_MM} mm (32-gon), F.Cu+B.Cu "
                       "(no tracks, vias or copper pour)")

    # R-10: fabrication-drawing annotations on User.Drawings (never fabricated)
    marker = "FAB DRAWING NOTES (HW-H2-01)"
    if any(d.GetClass() == "PCB_TEXT" and d.GetText().startswith(marker) for d in board.GetDrawings()):
        print("R-10: fab drawing annotations already present")
    else:
        L, T, R, B = 110.475, 36.975, 210.525, 137.025
        h1, h2, h3 = HOLES[0], HOLES[1], HOLES[2]
        dims = [
            ((L, B), (R, B), 12.0, "board width"),
            ((R, T), (R, B), -9.0, "board height"),
            ((h2[1], h2[2]), (h3[1], h3[2]), 7.0, "H2-H3 pitch"),
            ((h1[1], h1[2]), (h2[1], h2[2]), 6.0, "H1-H2 pitch"),
        ]
        for a, b, height, what in dims:
            d = pcbnew.PCB_DIM_ALIGNED(board)
            d.SetLayer(pcbnew.Dwgs_User)
            d.SetStart(pcbnew.VECTOR2I(nm(a[0]), nm(a[1])))
            d.SetEnd(pcbnew.VECTOR2I(nm(b[0]), nm(b[1])))
            d.SetHeight(nm(height))
            d.SetUnits(pcbnew.EDA_UNITS_MM) if hasattr(d, "SetUnits") else None
            d.SetPrecision(pcbnew.DIM_PRECISION_X_XX) if hasattr(pcbnew, "DIM_PRECISION_X_XX") else None
            board.Add(d)
            d.Update()
            changes.append(f"R-10 dimension {what} on User.Drawings")
        note = (f"{marker}\n"
                "Review copy - NOT released for fabrication (H4 = human approval)\n"
                "1. Outline 100.05 x 100.05 mm, Edge.Cuts. 2 Cu layers, 1.6 mm (upstream).\n"
                "2. H1-H3: 3.2 mm NPTH for M3, 3.5 mm from edges, no net,\n"
                "   no copper/track/via within 3.8 mm radius (F.Cu + B.Cu).\n"
                "3. No top-right hole: J3 terminal + VDD edge track occupy it.\n"
                "4. U5 tabs: 2 x 4.1 mm NPTH. All other holes plated.\n"
                "5. Material, Cu weight, finish, mask/silk colour: ASSUMPTIONS,\n"
                "   see hardware/prototype-v1/FABRICATION_NOTES.md.")
        t = pcbnew.PCB_TEXT(board)
        t.SetLayer(pcbnew.Dwgs_User)
        t.SetText(note)
        t.SetPosition(pcbnew.VECTOR2I(nm(L), nm(B + 18.0)))
        t.SetTextSize(pcbnew.VECTOR2I(nm(1.5), nm(1.5)))
        t.SetTextThickness(nm(0.2))
        t.SetHorizJustify(pcbnew.GR_TEXT_H_ALIGN_LEFT)
        t.SetVertJustify(pcbnew.GR_TEXT_V_ALIGN_TOP)
        board.Add(t)
        changes.append("R-10 fabrication note text on User.Drawings")

    # Refill and save only when something changed: the KiCad 9 filler may emit
    # equivalent fill polygons in a different order, which would break the
    # byte-identical idempotency check.
    if changes:
        filler = pcbnew.ZONE_FILLER(board)
        filler.Fill(board.Zones())
        board.Save(str(board_path))
    print(f"board sha256 {before} -> {sha(board_path)}")
    print(f"changes applied: {len(changes)}")
    for c in changes:
        print("  -", c)
    return 0


# ----------------------------------------------------------------------------
# R-09: per-item courtyard DRC exclusions
# ----------------------------------------------------------------------------
def classify(a: str, b: str) -> tuple[str, str]:
    refs = {a, b}
    if refs == {"REF**"}:
        return ("P", "Unpopulated proto-area pad vs proto-area pad on the 2.54 mm grid. No net, no "
                     "component, no assembly placement. The Connector_Pin:Pin_D1.0mm_L10.0mm courtyard "
                     "(3.09 mm) is wider than the grid pitch. Copper clearance is checked separately "
                     "and passes.")
    if "REF**" in refs:
        hdr = (refs - {"REF**"}).pop()
        return ("H", f"Unpopulated proto-area pad vs {hdr} pin header placed on the same 2.54 mm proto "
                     "grid. Pad centres are 2.54 mm apart and copper clearance passes. The proto pad "
                     "receives no part, so the only physical body is the header housing (2.54 mm per "
                     "position).")
    return ("C", f"{a}/{b} 2.54 mm pin headers on one continuous pitch: end pins are exactly 2.54 mm "
                 "apart, so the housings butt with zero gap, as in a broken-off strip. Not a physical "
                 "clash. Each courtyard reaches about 0.57 mm past the 1.27 mm half-pitch housing, so "
                 "the courtyards overlap by about 1.14 mm. Reviewer: confirm the chosen header MPN "
                 "housing is no more than 2.54 mm per position (HW-BOM-01).")


def do_exclusions(work: Path) -> int:
    import pcbnew

    report = json.loads((work / "verification" / "drc-for-exclusions.json").read_text())
    board = pcbnew.LoadBoard(str(work / "v0.4.4d.kicad_pcb"))
    by_uuid = {f.m_Uuid.AsString(): f for f in board.GetFootprints()}
    entries = []
    rows = []
    for v in report["violations"]:
        if v["type"] != "courtyards_overlap":
            continue
        ua, ub = v["items"][0]["uuid"], v["items"][1]["uuid"]
        fa, fb = by_uuid[ua], by_uuid[ub]
        loc = None
        for layer in (pcbnew.F_CrtYd, pcbnew.B_CrtYd):
            ca, cb = fa.GetCourtyard(layer), fb.GetCourtyard(layer)
            if ca.OutlineCount() and cb.OutlineCount():
                p = pcbnew.VECTOR2I(0, 0)
                if ca.Collide(cb, 0, None, p):
                    loc = (p.x, p.y)
                    break
        if loc is None:
            raise SystemExit(f"no courtyard collision found for {ua}/{ub}")
        ra, rb = fa.GetReference(), fb.GetReference()
        pa = (pcbnew.ToMM(fa.GetPosition().x), pcbnew.ToMM(fa.GetPosition().y))
        pb = (pcbnew.ToMM(fb.GetPosition().x), pcbnew.ToMM(fb.GetPosition().y))
        cls, why = classify(ra, rb)
        key = f"courtyards_overlap|{loc[0]}|{loc[1]}|{ua}|{ub}"
        comment = (f"HW-DRC-01 class {cls}: {ra}@({pa[0]:.3f},{pa[1]:.3f}) vs {rb}@({pb[0]:.3f},{pb[1]:.3f}). "
                   + why)
        entries.append([key, comment])
        rows.append((cls, ra, f"{pa[0]:.3f}", f"{pa[1]:.3f}", rb, f"{pb[0]:.3f}", f"{pb[1]:.3f}",
                     f"{pcbnew.ToMM(loc[0]):.4f}", f"{pcbnew.ToMM(loc[1]):.4f}", ua, ub, why))

    pro_path = work / "v0.4.4d.kicad_pro"
    pro = json.loads(pro_path.read_text())
    ds = pro["board"]["design_settings"]
    keep = [e for e in ds.get("drc_exclusions", [])
            if not (isinstance(e, list) and e[0].startswith("courtyards_overlap|"))]
    ds["drc_exclusions"] = keep + sorted(entries)
    pro_path.write_text(json.dumps(pro, indent=2) + "\n")

    import csv
    out = work / "verification" / "drc_exclusions.csv"
    with out.open("w", newline="") as fh:
        w = csv.writer(fh, lineterminator="\n")
        w.writerow(["class", "ref_a", "x_a_mm", "y_a_mm", "ref_b", "x_b_mm", "y_b_mm",
                    "marker_x_mm", "marker_y_mm", "uuid_a", "uuid_b", "justification"])
        for r in sorted(rows):
            w.writerow(r)
    counts = {}
    for r in rows:
        counts[r[0]] = counts.get(r[0], 0) + 1
    print(f"courtyard exclusions written: {len(entries)} {counts}")
    return 0


def main() -> int:
    if len(sys.argv) < 2 or sys.argv[1] not in ("sch", "pcb", "exclusions"):
        print(__doc__)
        return 2
    work = Path(sys.argv[2] if len(sys.argv) > 2 else "/work")
    return {"sch": do_sch, "pcb": do_pcb, "exclusions": do_exclusions}[sys.argv[1]](work)


if __name__ == "__main__":
    raise SystemExit(main())
