#!/usr/bin/env python3
"""Measure clearances around the prototype-v1 mounting holes (HW-MECH-02).

Run inside the pinned KiCad container:  python3 check_mounting_holes.py /work

For every footprint whose reference starts with "H" and whose FPID is a
MountingHole, report the hole and the minimum edge-to-edge distance to:
copper of every other object on F.Cu/B.Cu (tracks, vias, pads, filled zones),
other footprints' courtyards, the board outline, and silkscreen. Exits 1 if
copper intrudes into the keep-out radius or the hole is plated or netted.
"""
from __future__ import annotations

import math
import sys
from pathlib import Path

import pcbnew

KEEPOUT_R_MM = 3.8
TOL_MM = 0.005  # polygon/arc approximation tolerance of the zone filler
M = pcbnew.ToMM


def dist_mm(poly: "pcbnew.SHAPE_POLY_SET", pt) -> float:
    if poly.OutlineCount() == 0:
        return math.inf
    return M(int(math.sqrt(poly.SquaredDistance(pt))))


def main() -> int:
    work = Path(sys.argv[1] if len(sys.argv) > 1 else "/work")
    board = pcbnew.LoadBoard(str(work / "v0.4.4d.kicad_pcb"))
    holes = [f for f in board.GetFootprints()
             if f.GetReference().startswith("H") and "MountingHole" in str(f.GetFPID().GetLibItemName())]
    edge = board.GetBoardEdgesBoundingBox()
    ok = True
    print("Mounting-hole clearance check (edge-to-centre distances in mm)")
    print(f"board outline bbox: ({M(edge.GetLeft())}, {M(edge.GetTop())}) - ({M(edge.GetRight())}, {M(edge.GetBottom())})")
    print(f"keep-out radius required: {KEEPOUT_R_MM} (numeric tolerance {TOL_MM})")
    for fp in sorted(holes, key=lambda f: f.GetReference()):
        ref = fp.GetReference()
        me = fp.m_Uuid.AsString()
        c = fp.GetPosition()
        pt = pcbnew.VECTOR2I(c.x, c.y)
        pads = list(fp.Pads())
        drill = M(pads[0].GetDrillSize().x)
        plated = pads[0].GetAttribute() != pcbnew.PAD_ATTRIB_NPTH
        netted = any(p.GetNetname() for p in pads)
        best = {"copper": (math.inf, ""), "zone fill": (math.inf, ""), "courtyard": (math.inf, ""), "silk": (math.inf, "")}
        for lay in (pcbnew.F_Cu, pcbnew.B_Cu):
            for t in board.GetTracks():
                if t.IsOnLayer(lay):
                    p = pcbnew.SHAPE_POLY_SET()
                    t.TransformShapeToPolygon(p, lay, 0, pcbnew.FromMM(0.005), pcbnew.ERROR_INSIDE)
                    d = dist_mm(p, pt)
                    if d < best["copper"][0]:
                        best["copper"] = (d, f"{t.GetClass()} {t.GetNetname()} {pcbnew.LayerName(lay)}")
            for z in board.Zones():
                if not z.GetIsRuleArea() and z.IsOnLayer(lay):
                    d = dist_mm(z.GetFilledPolysList(lay), pt)
                    if d < best["zone fill"][0]:
                        best["zone fill"] = (d, f"{z.GetNetname()} {pcbnew.LayerName(lay)}")
            for f in board.GetFootprints():
                if f.m_Uuid.AsString() == me:
                    continue
                for pad in f.Pads():
                    if pad.IsOnLayer(lay) and pad.GetAttribute() != pcbnew.PAD_ATTRIB_NPTH:
                        p = pcbnew.SHAPE_POLY_SET()
                        pad.TransformShapeToPolygon(p, lay, 0, pcbnew.FromMM(0.005), pcbnew.ERROR_INSIDE)
                        d = dist_mm(p, pt)
                        if d < best["copper"][0]:
                            best["copper"] = (d, f"pad {f.GetReference()}.{pad.GetNumber()} {pad.GetNetname()}")
        for f in board.GetFootprints():
            if f.m_Uuid.AsString() == me:
                continue
            for lay in (pcbnew.F_CrtYd, pcbnew.B_CrtYd):
                d = dist_mm(f.GetCourtyard(lay), pt)
                if d < best["courtyard"][0]:
                    best["courtyard"] = (d, f"{f.GetReference()} {pcbnew.LayerName(lay)}")
            for pad in f.Pads():
                if pad.GetAttribute() == pcbnew.PAD_ATTRIB_NPTH:
                    p = pcbnew.SHAPE_POLY_SET()
                    pad.TransformHoleToPolygon(p, 0, pcbnew.FromMM(0.005), pcbnew.ERROR_INSIDE)
                    d = dist_mm(p, pt)
                    if d < best["copper"][0]:
                        best["copper"] = (d, f"NPTH {f.GetReference()}")
        silk_items = list(board.GetDrawings()) + [g for f in board.GetFootprints()
                                                  if f.m_Uuid.AsString() != me
                                                  for g in f.GraphicalItems()]
        for g in silk_items:
            if g.GetLayer() in (pcbnew.F_SilkS, pcbnew.B_SilkS):
                p = pcbnew.SHAPE_POLY_SET()
                try:
                    g.TransformShapeToPolygon(p, g.GetLayer(), 0, pcbnew.FromMM(0.005), pcbnew.ERROR_INSIDE)
                except Exception:
                    continue
                d = dist_mm(p, pt)
                if d < best["silk"][0]:
                    owner = g.GetParentFootprint()
                    best["silk"] = (d, f"{g.GetClass()} {owner.GetReference() if owner else 'board graphic'}")
        to_edge = min(M(c.x - edge.GetLeft()), M(edge.GetRight() - c.x),
                      M(c.y - edge.GetTop()), M(edge.GetBottom() - c.y))
        kz = [z for z in board.Zones() if z.GetIsRuleArea() and z.GetZoneName() == f"{ref}_keepout"]
        print()
        print(f"{ref}: centre ({M(c.x):.3f}, {M(c.y):.3f}) drill {drill} plated={plated} net={'yes' if netted else 'none'}")
        print(f"  BOM-excluded={fp.IsExcludedFromBOM()} pos-excluded={fp.IsExcludedFromPosFiles()} board-only={fp.IsBoardOnly()}")
        print(f"  keep-out rule area present: {bool(kz)}")
        print(f"  centre to board edge: {to_edge:.3f} (hole wall to edge {to_edge - drill / 2:.3f})")
        for k, (d, what) in best.items():
            label = "discrete copper (track/via/pad/hole)" if k == "copper" else k
            print(f"  nearest {label}: {d:.4f} [{what}]")
        if plated or netted or not kz or min(best["copper"][0], best["zone fill"][0]) < KEEPOUT_R_MM - TOL_MM or best["courtyard"][0] < 3.45:
            ok = False
            print("  RESULT: FAIL")
        else:
            print("  RESULT: PASS")
    print()
    print("Top-right corner site search (HW-MECH-02): grid x 190.0-207.0, y 39.5-52.0 mm, 0.5 mm step")
    print("criterion: >= 3.8 mm to tracks/vias/pads/NPTH on F.Cu/B.Cu and >= 3.45 mm to courtyards")
    obstacles = []
    for t in board.GetTracks():
        for lay in (pcbnew.F_Cu, pcbnew.B_Cu):
            if t.IsOnLayer(lay):
                p = pcbnew.SHAPE_POLY_SET()
                t.TransformShapeToPolygon(p, lay, 0, pcbnew.FromMM(0.005), pcbnew.ERROR_INSIDE)
                obstacles.append(("cu", f"{t.GetClass()} {t.GetNetname()}", p))
    for f in board.GetFootprints():
        for pad in f.Pads():
            p = pcbnew.SHAPE_POLY_SET()
            if pad.GetAttribute() == pcbnew.PAD_ATTRIB_NPTH:
                pad.TransformHoleToPolygon(p, 0, pcbnew.FromMM(0.005), pcbnew.ERROR_INSIDE)
            else:
                pad.TransformShapeToPolygon(p, pcbnew.F_Cu if pad.IsOnLayer(pcbnew.F_Cu) else pcbnew.B_Cu,
                                            0, pcbnew.FromMM(0.005), pcbnew.ERROR_INSIDE)
            obstacles.append(("cu", f"pad {f.GetReference()}.{pad.GetNumber()}", p))
        for lay in (pcbnew.F_CrtYd, pcbnew.B_CrtYd):
            if f.GetCourtyard(lay).OutlineCount():
                obstacles.append(("crt", f"courtyard {f.GetReference()}", f.GetCourtyard(lay)))
    feasible, best_pt = 0, (-1.0, None, "")
    x = 190.0
    while x <= 207.0 + 1e-9:
        y = 39.5
        while y <= 52.0 + 1e-9:
            pt = pcbnew.VECTOR2I(pcbnew.FromMM(x), pcbnew.FromMM(y))
            cu = min((dist_mm(p, pt), n) for k, n, p in obstacles if k == "cu")
            cy = min((dist_mm(p, pt), n) for k, n, p in obstacles if k == "crt")
            if cu[0] >= KEEPOUT_R_MM and cy[0] >= 3.45:
                feasible += 1
            margin = min(cu[0] - KEEPOUT_R_MM, cy[0] - 3.45)
            if best_pt[1] is None or margin > best_pt[0]:
                best_pt = (margin, (x, y), f"copper {cu[0]:.3f} [{cu[1]}], courtyard {cy[0]:.3f} [{cy[1]}]")
            y += 0.5
        x += 0.5
    print(f"feasible sites: {feasible}")
    print(f"least-bad site: {best_pt[1]} -> {best_pt[2]}")
    print()
    print(f"holes found: {len(holes)}")
    print("RESULT:", "PASS" if ok and holes else "FAIL")
    return 0 if ok and holes else 1


if __name__ == "__main__":
    raise SystemExit(main())
