#!/usr/bin/env python3
"""Import routing only, preserving the exact draft footprint placement."""
import argparse
from pathlib import Path
import tempfile

import pcbnew


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
    for item in list(board.GetTracks()):
        board.Remove(item)
    with tempfile.NamedTemporaryFile(mode='w', suffix='.ses', dir=args.out.parent,
                                     encoding='utf-8') as copy:
        copy.write(without_placement(args.ses.read_text()))
        copy.flush()
        assert pcbnew.ImportSpecctraSES(board, copy.name), 'SES import returned False'
    assert placement(board) == before, 'footprint placement changed'
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
    print({'footprints_unchanged': len(before), 'segments': segments, 'vias': vias}, flush=True)


if __name__ == '__main__':
    main()
