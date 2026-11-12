#!/usr/bin/env python3
"""Fill the existing zones using the sibling KiCad project rules."""
import argparse
from pathlib import Path

import pcbnew


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--in', dest='input', required=True, type=Path)
    parser.add_argument('--out', required=True, type=Path)
    args = parser.parse_args()
    assert args.input.resolve() != args.out.resolve(), 'refusing in-place write'
    assert args.input.with_suffix('.kicad_pro').is_file(), 'missing sibling project'
    board = pcbnew.LoadBoard(str(args.input))
    assert pcbnew.ZONE_FILLER(board).Fill(board.Zones()), 'zone fill returned False'
    assert pcbnew.SaveBoard(str(args.out), board, True), 'board save failed'
    print({'zones': len(board.Zones()), 'filled': True}, flush=True)


if __name__ == '__main__':
    main()
