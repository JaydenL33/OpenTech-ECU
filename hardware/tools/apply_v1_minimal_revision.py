#!/usr/bin/env python3
"""Apply the documented prototype-v1 minimal PCB revision (Option B).

Run inside the pinned KiCad container (it provides the `pcbnew` module):

    docker run --rm --user "$(id -u):$(id -g)" \
      -v "$PWD/hardware/prototype-v1:/work" \
      -v "$PWD/hardware/tools:/tools:ro" \
      kicad/kicad@sha256:e638b79b... python3 /tools/apply_v1_minimal_revision.py /work

Changes are limited to PCB/manufacturing fixes; no net, component value or
routing topology change is made. Each change is recorded in
`AUDIT/HARDWARE_DIFF.md`:

  R-01  The four GND pads with a single thermal spoke (C5.2, C10.2, C11.2,
        U4.3) are set to a solid zone connection instead of thermal relief.
        Electrical effect: lower-impedance GND connection at those pads.
        Assembly effect: these four pads need reflow/hot-air, not an iron.
  R-02  Remove a 0.552 mm VDDA track segment that lies exactly on top of the
        through VDDA track (redundant overlapping copper, unconnected end).
  R-03  Convert the two 4.1 mm U5 MAP-sensor mounting tabs from PTH pads with
        zero annular ring to NPTH mechanical holes. No net is affected.

Zone fills are recomputed after the edits so the saved board matches KiCad 9
DRC expectations.

The script is idempotent: re-running it on an already revised board reports
"already applied" for each item.
"""
from __future__ import annotations

import hashlib
import sys
from pathlib import Path

import pcbnew

# --- Exact geometry/nets of the items to change (verified by inspection) -----
STARVED_PADS = {("C5", "2"), ("C10", "2"), ("C11", "2"), ("U4", "3")}

# Redundant segment: both endpoints collinear with the through track
# (149.047374,132.991) -> (162.709,132.991) on net VDDA.
STUB_START_MM = (162.157, 132.991)
STUB_END_MM = (162.709, 132.991)
STUB_NET = "VDDA"

U5_TAB_DRILL_MM = 4.1


def mm(v) -> float:
    return round(pcbnew.ToMM(v), 4)


def near(a: float, b: float, tol: float = 1e-3) -> bool:
    return abs(a - b) <= tol


def sha256(path: Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as fh:
        for chunk in iter(lambda: fh.read(1 << 20), b""):
            h.update(chunk)
    return h.hexdigest()


def main() -> int:
    work = Path(sys.argv[1] if len(sys.argv) > 1 else "/work")
    board_path = work / "v0.4.4d.kicad_pcb"
    before = sha256(board_path)
    board = pcbnew.LoadBoard(str(board_path))
    applied = []

    # R-01 and R-03 share one footprint pass: iterating GetFootprints() again
    # after a track removal returns invalid SWIG proxies in KiCad 9.
    for fp in board.GetFootprints():
        ref = fp.GetReference()
        for pad in fp.Pads():
            # R-01: solid GND connection for the four starved-thermal pads.
            if (ref, pad.GetNumber()) in STARVED_PADS:
                if pad.GetLocalZoneConnection() == pcbnew.ZONE_CONNECTION_FULL:
                    print(f"R-01 {ref}.{pad.GetNumber()}: already solid")
                else:
                    pad.SetLocalZoneConnection(pcbnew.ZONE_CONNECTION_FULL)
                    applied.append(f"R-01 {ref}.{pad.GetNumber()} -> solid GND")
                    print(f"R-01 {ref}.{pad.GetNumber()}: set solid zone connection")

            # R-03: U5 mechanical mounting tabs -> NPTH.
            if ref != "U5":
                continue
            if pad.GetNumber() not in ("", "~") or pad.GetNetname():
                continue
            if not near(mm(pad.GetDrillSize().x), U5_TAB_DRILL_MM, 0.01):
                continue
            if pad.GetAttribute() == pcbnew.PAD_ATTRIB_NPTH:
                print("R-03: U5 tab already NPTH")
                continue
            pad.SetAttribute(pcbnew.PAD_ATTRIB_NPTH)
            applied.append("R-03 U5 mounting tab PTH -> NPTH")
            print(f"R-03: U5 tab at ({mm(pad.GetPosition().x)}, "
                  f"{mm(pad.GetPosition().y)}) set NPTH")

    # R-02: delete the overlapping VDDA stub, only if the exact segment exists.
    removed = False
    for track in list(board.GetTracks()):
        if track.GetNetname() != STUB_NET:
            continue
        start, end = track.GetStart(), track.GetEnd()
        if (near(mm(start.x), STUB_START_MM[0]) and near(mm(start.y), STUB_START_MM[1])
                and near(mm(end.x), STUB_END_MM[0]) and near(mm(end.y), STUB_END_MM[1])):
            board.Remove(track)
            removed = True
            applied.append("R-02 removed overlapping VDDA stub (0.552 mm)")
            print("R-02: removed redundant VDDA stub segment")
    if not removed:
        print("R-02: stub not present (already applied)")

    # R-03 was handled in the footprint pass above.

    # Recompute copper fills so DRC and manufacturing exports use fresh polygons.
    filler = pcbnew.ZONE_FILLER(board)
    filler.Fill(board.Zones())
    board.Save(str(board_path))

    after = sha256(board_path)
    print()
    print(f"board: {board_path}")
    print(f"sha256 before: {before}")
    print(f"sha256 after:  {after}")
    print(f"changes applied: {len(applied)}")
    for item in applied:
        print(f"  - {item}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
