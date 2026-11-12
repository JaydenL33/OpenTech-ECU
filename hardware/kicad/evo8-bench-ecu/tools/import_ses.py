#!/usr/bin/env python3
"""Import routing only, preserving the exact draft footprint placement."""
import argparse
from pathlib import Path
import tempfile

import pcbnew
from preroutes import PREROUTES


def without_placement(text):
    """Remove root-child placement expressions with a quoted-string aware scan."""
    depth = 0
    quoted = escaped = False
    start = None
    spans = []
    for i, char in enumerate(text):
        if quoted:
            if escaped:
                escaped = False
            elif char == '\\':
                escaped = True
            elif char == '"':
                quoted = False
            continue
        if char == '"':
            quoted = True
        elif char == '(':
            if depth == 1:
                tag = text[i + 1:].lstrip().split(None, 1)[0].split(')', 1)[0]
                if tag == 'placement':
                    start = i
            depth += 1
        elif char == ')':
            depth -= 1
            assert depth >= 0, 'unbalanced SES'
            if depth == 1 and start is not None:
                spans.append((start, i + 1))
                start = None
    assert depth == 0 and not quoted, 'unbalanced SES'
    for begin, end in reversed(spans):
        text = text[:begin] + text[end:]
    return text


def placement(board):
    return {str(f.m_Uuid.AsString()): (f.GetReference(), f.GetPosition().x,
            f.GetPosition().y, f.GetOrientation().AsDegrees(), f.GetLayer())
            for f in board.GetFootprints()}


def point_mm(point):
    return tuple(pcbnew.FromMM(value) for value in point)


def same_location(item, route):
    if route['layer'] == 'via':
        return (isinstance(item, pcbnew.PCB_VIA)
                and (item.GetPosition().x, item.GetPosition().y) == point_mm(route['position']))
    if isinstance(item, pcbnew.PCB_VIA):
        return False
    endpoints = ((item.GetStart().x, item.GetStart().y),
                 (item.GetEnd().x, item.GetEnd().y))
    expected = (point_mm(route['start']), point_mm(route['end']))
    return endpoints in (expected, expected[::-1])


def matches(item, route):
    if not same_location(item, route) or item.GetNetname() != route['net']:
        return False
    if route['layer'] == 'via':
        return (item.GetWidth(pcbnew.F_Cu) == pcbnew.FromMM(route['size'])
                and item.GetDrillValue() == pcbnew.FromMM(route['drill'])
                and item.TopLayer() == pcbnew.F_Cu and item.BottomLayer() == pcbnew.B_Cu
                and item.GetViaType() == pcbnew.VIATYPE_THROUGH)
    return (item.GetLayerName() == route['layer']
            and item.GetWidth() == pcbnew.FromMM(route['width']))


def keep_preroutes(board):
    from_ses = added = 0
    for route in PREROUTES:
        found = [item for item in board.GetTracks() if same_location(item, route)]
        assert len(found) <= 1, f"duplicate pre-route {route['id']}: {len(found)}"
        if found:
            item = found[0]
            assert matches(item, route), f"pre-route geometry or net differs: {route['id']}"
            from_ses += 1
        else:
            if route['layer'] == 'via':
                item = pcbnew.PCB_VIA(board)
                item.SetPosition(pcbnew.VECTOR2I(*point_mm(route['position'])))
                item.SetViaType(pcbnew.VIATYPE_THROUGH)
                item.SetLayerPair(pcbnew.F_Cu, pcbnew.B_Cu)
                item.SetWidth(pcbnew.FromMM(route['size']))
                item.SetDrill(pcbnew.FromMM(route['drill']))
            else:
                item = pcbnew.PCB_TRACK(board)
                item.SetStart(pcbnew.VECTOR2I(*point_mm(route['start'])))
                item.SetEnd(pcbnew.VECTOR2I(*point_mm(route['end'])))
                item.SetLayer(board.GetLayerID(route['layer']))
                item.SetWidth(pcbnew.FromMM(route['width']))
            item.SetNetCode(board.FindNet(route['net']).GetNetCode())
            board.Add(item)
            added += 1
        item.SetLocked(True)
    for route in PREROUTES:
        found = [item for item in board.GetTracks() if same_location(item, route)]
        assert len(found) == 1 and matches(found[0], route), f"pre-route missing or changed: {route['id']}"
        assert found[0].IsLocked(), f"pre-route unlocked: {route['id']}"
    assert from_ses + added == 12, 'pre-route count'
    return from_ses, added


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--board', required=True, type=Path)
    parser.add_argument('--ses', required=True, type=Path)
    parser.add_argument('--out', required=True, type=Path)
    args = parser.parse_args()
    assert args.out.resolve() != args.board.resolve(), 'refusing in-place write'
    pcbnew.KIID.SeedGenerator(1)
    board = pcbnew.LoadBoard(str(args.board))
    before = placement(board)
    assert len(before) == 305, 'footprint count'
    # Keep the removed SWIG proxies alive until the import and save finish.
    draft_copper = list(board.GetTracks())
    for item in draft_copper:
        board.Remove(item)
    with tempfile.NamedTemporaryFile(mode='w', suffix='.ses', dir=args.out.parent,
                                     encoding='utf-8') as copy:
        copy.write(without_placement(args.ses.read_text()))
        copy.flush()
        assert pcbnew.ImportSpecctraSES(board, copy.name), 'SES import returned False'
    assert placement(board) == before, 'footprint placement changed'
    preroutes_from_ses, preroutes_added = keep_preroutes(board)
    segments = vias = 0
    for item in board.GetTracks():
        if isinstance(item, pcbnew.PCB_VIA):
            vias += 1
            assert item.TopLayer() == pcbnew.F_Cu and item.BottomLayer() == pcbnew.B_Cu, 'via span'
            assert item.GetViaType() == pcbnew.VIATYPE_THROUGH, 'via type'
            assert item.GetWidth() == pcbnew.FromMM(0.6), 'via size'
            assert item.GetDrillValue() == pcbnew.FromMM(0.3), 'via drill'
        else:
            segments += 1
            assert item.GetLayer() in (pcbnew.F_Cu, pcbnew.B_Cu), 'track on inner layer'
    assert pcbnew.SaveBoard(str(args.out), board, True), 'board save failed'
    print({'footprints_unchanged': len(before), 'segments': segments, 'vias': vias,
           'preroutes_from_ses': preroutes_from_ses,
           'preroutes_added': preroutes_added}, flush=True)


if __name__ == '__main__':
    main()
