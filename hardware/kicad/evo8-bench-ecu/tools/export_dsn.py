#!/usr/bin/env python3
"""Export the fixed bench draft with plane-only inner layers and D5 margins."""
import argparse
from collections import Counter
from pathlib import Path

import pcbnew
from sexpr import Atom, child, children, dump, parse


def walk(node):
    if isinstance(node, list):
        yield node
        for value in node:
            yield from walk(value)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--board', required=True, type=Path)
    parser.add_argument('--out', required=True, type=Path)
    args = parser.parse_args()
    assert args.board.with_suffix('.kicad_pro').is_file(), 'missing sibling project'
    assert args.out.resolve() != args.board.resolve(), 'refusing in-place output'
    board = pcbnew.LoadBoard(str(args.board))
    footprints = list(board.GetFootprints())
    copper = list(board.GetTracks())
    counts = {
        'footprints': len(footprints),
        'front_footprints': sum(f.GetLayer() == pcbnew.F_Cu for f in footprints),
        'tracks': sum(not isinstance(t, pcbnew.PCB_VIA) for t in copper),
        'vias': sum(isinstance(t, pcbnew.PCB_VIA) for t in copper),
        'zones': len(board.Zones()),
        'nets': len(board.GetNetsByNetcode()),
    }
    print(counts, flush=True)
    assert counts == dict(footprints=305, front_footprints=305, tracks=24,
                          vias=13, zones=2, nets=199), 'draft precondition counts'
    pad_counts = Counter(str(p.GetNetname()) for f in footprints for p in f.Pads()
                         if p.GetNetCode() != 0)
    for item in copper:
        board.Remove(item)
    assert pcbnew.ExportSpecctraDSN(board, str(args.out)), 'DSN export returned False'
    tree = parse(args.out.read_text())
    structure = child(tree, 'structure')
    assert structure is not None, 'missing structure'
    # Apply A, B, C in the order specified by Section 3 Step 2.
    layers = children(structure, 'layer')
    for layer in layers:
        if layer[1] in ('In1.Cu', 'In2.Cu'):
            kind = child(layer, 'type')
            assert kind is not None, 'missing layer type'
            kind[1] = Atom('power')
    for node in walk(tree):
        if node and node[0] == 'clearance' and child(node, 'type') is None:
            if float(node[1]) < 205:
                node[1] = Atom('205')
    boundary = child(structure, 'boundary')
    assert boundary is not None, 'missing boundary'
    boundary[:] = parse('(boundary (path pcb 0 10500 -10500 339500 -10500 '
                        '339500 -259500 10500 -259500 10500 -10500))')
    assert len(layers) == 4, 'layer count'
    assert all(child(layer, 'type')[1] == 'power' for layer in layers
               if layer[1] in ('In1.Cu', 'In2.Cu')), 'inner layer type'
    for net, layer in [('GND', 'In1.Cu'), ('PGND', 'In2.Cu')]:
        assert any(p[1] == net and any(n and n[0] in ('polygon', 'rect', 'path')
                   and n[1] == layer for n in walk(p))
                   for p in children(structure, 'plane')), f'missing {net} plane'
    assert any('600:300' in str(n[1]) for n in walk(tree)
               if n and n[0] == 'padstack'), 'missing 600:300 padstack'
    assert sum(1 for n in walk(child(tree, 'placement'))
               if n and n[0] == 'place') == 305, 'placement count'
    network = child(tree, 'network')
    assert network is not None, 'missing network'
    exported_nets = {str(n[1]) for n in children(network, 'net')}
    multi = {name for name, count in pad_counts.items() if count >= 2}
    assert multi <= exported_nets, f'missing multi-pad nets: {sorted(multi - exported_nets)}'
    wiring = child(tree, 'wiring')
    assert wiring is not None, 'missing wiring'
    assert not any(n and n[0] == 'wire' for n in walk(wiring)), 'draft wires remain'
    args.out.write_text(dump(tree) + '\n')
    print({'layers': len(layers), 'placements': 305, 'multi_pad_nets': len(multi),
           'exported_nets': len(exported_nets), 'wires': 0}, flush=True)


if __name__ == '__main__':
    main()
