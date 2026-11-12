#!/usr/bin/env python3
"""Add deterministic GND/PGND through vias without changing existing objects."""
import argparse
import collections
import json
from pathlib import Path

import pcbnew
from preroutes import SERVED_PADS


def position(point):
    return {"x": pcbnew.ToMM(point.x), "y": pcbnew.ToMM(point.y)}


def candidates(centre):
    # Pad-centred 0.05 mm grid, inside the 1.0 mm radius (including centre).
    offsets = sorted((x*x + y*y, x, y) for x in range(-20, 21)
                     for y in range(-20, 21) if x*x + y*y <= 400)
    step = pcbnew.FromMM(0.05)
    for _, x, y in offsets:
        yield pcbnew.VECTOR2I(centre.x + x*step, centre.y + y*step)


def obstacle(shape, gap):
    # SHAPE.BBox(gap) does not inflate in pcbnew 9.0.9; expand it here so the
    # quick point-in-box test cannot skip a real clearance collision.
    box = shape.BBox()
    return (shape, gap, box.GetLeft() - gap, box.GetTop() - gap,
            box.GetRight() + gap, box.GetBottom() + gap)


def nearby(obstacles, centre):
    reach = pcbnew.FromMM(1.0)
    return [o for o in obstacles if o[2] <= centre.x + reach
            and o[4] >= centre.x - reach and o[3] <= centre.y + reach
            and o[5] >= centre.y - reach]


def clear(point, obstacles):
    for shape, gap, left, top, right, bottom in obstacles:
        if (left <= point.x <= right and top <= point.y <= bottom
                and shape.Collide(point, gap)):
            return False
    return True


def add_vias(board):
    # Keep the new-via UUID stream separate from the SES import's seed 1.
    pcbnew.KIID.SeedGenerator(2)
    settings = board.GetDesignSettings()
    radius = pcbnew.FromMM(0.3)
    drill_radius = pcbnew.FromMM(0.15)
    layers = list(board.GetEnabledLayers().CuStack())
    pads = sorted(board.GetPads(), key=lambda p: (
        p.GetParentFootprint().GetReference(), p.GetNumber(), p.GetPosition().x,
        p.GetPosition().y, str(p.m_Uuid.AsString())))
    copper = list(board.GetTracks())
    vias = [item for item in copper if isinstance(item, pcbnew.PCB_VIA)]
    objects = pads + copper
    # Cache KiCad's effective geometry, including rounded/oval/rotated pads.
    by_net = collections.defaultdict(list)
    holes = []
    for item in objects:
        if item.HasDrilledHole():
            hole = item.GetEffectiveHoleShape()
            holes.append(obstacle(hole, drill_radius + settings.m_HoleClearance))
            # A different-net hole must also clear the new via's copper.
            by_net[item.GetNetCode()].append(obstacle(
                hole, radius + settings.m_MinClearance))
        for layer in layers:
            if item.IsOnLayer(layer):
                gap = radius + max(settings.m_MinClearance,
                                   item.GetOwnClearance(layer))
                by_net[item.GetNetCode()].append(obstacle(
                    item.GetEffectiveShape(layer), gap))
    # Filled signal-layer zones are copper obstacles too.  Plane fills are
    # regenerated around vias by the later fill step, so they are not treated
    # as fixed obstacles here.
    for zone in board.Zones():
        layer = zone.GetLayer()
        if not zone.IsFilled() or layer not in (pcbnew.F_Cu, pcbnew.B_Cu):
            continue
        polygon = zone.GetFilledPolysList(layer)
        if polygon.OutlineCount():
            by_net[zone.GetNetCode()].append(obstacle(
                polygon, radius + settings.m_MinClearance))
    edges = [obstacle(item.GetEffectiveShape(),
                      radius + settings.m_CopperEdgeClearance)
             for item in board.GetDrawings() if item.GetLayer() == pcbnew.Edge_Cuts]
    outline = pcbnew.SHAPE_POLY_SET()
    board.GetBoardPolygonOutlines(outline)
    targets = [p for p in pads if p.GetNetname() in ("GND", "PGND")]
    added = collections.Counter()
    failures = []
    skipped = 0
    skipped_pth = 0
    prerouted_skipped = 0
    for pad in targets:
        centre = pad.GetPosition()
        net = pad.GetNetCode()
        if (pad.GetParentFootprint().GetReference(), pad.GetNumber()) in SERVED_PADS:
            prerouted_skipped += 1
            continue
        # A plated through-hole pad already has a barrel connection to every
        # copper layer, so a second through via only adds a hole and can leave
        # a dangling via beside the pad.
        if pad.GetAttribute() == pcbnew.PAD_ATTRIB_PTH:
            skipped_pth += 1
            continue
        if any(v.GetNetCode() == net and
               (v.GetPosition().x-centre.x)**2 + (v.GetPosition().y-centre.y)**2
               <= pcbnew.FromMM(0.5)**2 for v in vias):
            skipped += 1
            continue
        obstacles = nearby(holes + edges + [o for n, group in by_net.items()
                                           if n != net for o in group], centre)
        pad_shape = pad.GetEffectiveShape(pcbnew.F_Cu)
        chosen = None
        tried = 0
        for point in candidates(centre):
            tried += 1
            # The via must still touch its own pad on F.Cu; an offset via that
            # clears everything but misses the pad is a dangling via.
            if not pad_shape.Collide(point, radius):
                continue
            if outline.Contains(point) and clear(point, obstacles):
                chosen = point
                break
        if chosen is None:
            failures.append({"reference": pad.GetParentFootprint().GetReference(),
                             "pad": pad.GetNumber(), "net": pad.GetNetname(),
                             "position": position(centre),
                             "nearest_point_tried": position(centre),
                             "nearest_legal_point_tried": None,
                             "candidates_tried": tried})
            continue
        via = pcbnew.PCB_VIA(board)
        via.SetPosition(chosen)
        via.SetViaType(pcbnew.VIATYPE_THROUGH)
        via.SetLayerPair(pcbnew.F_Cu, pcbnew.B_Cu)
        via.SetWidth(pcbnew.FromMM(0.6))
        via.SetDrill(pcbnew.FromMM(0.3))
        via.SetNetCode(net)
        board.Add(via)
        vias.append(via)
        added[pad.GetNetname()] += 1
        holes.append(obstacle(via.GetEffectiveHoleShape(),
                              drill_radius + settings.m_HoleClearance))
        for layer in layers:
            by_net[net].append(obstacle(via.GetEffectiveShape(layer),
                                       radius + settings.m_MinClearance))
    return {"pads": len(targets), "already_within_0_5_mm": skipped,
            "through_hole_skipped": skipped_pth,
            "prerouted_skipped": prerouted_skipped,
            "added": sum(added.values()), "added_by_net": dict(sorted(added.items())),
            "failures": len(failures), "plane_via_failures": failures}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--in', dest='input', required=True, type=Path)
    parser.add_argument('--out', required=True, type=Path)
    parser.add_argument('--report', required=True, type=Path)
    args = parser.parse_args()
    if args.input.resolve() == args.out.resolve():
        raise OSError('refusing in-place write')
    board = pcbnew.LoadBoard(str(args.input))
    if board is None:
        raise OSError('board load failed')
    report = add_vias(board)
    if not pcbnew.SaveBoard(str(args.out), board, True):
        raise OSError('board save failed')
    args.report.write_text(json.dumps(report, indent=2, sort_keys=True) + '\n')
    print(json.dumps(report, sort_keys=True), flush=True)


if __name__ == '__main__':
    main()
