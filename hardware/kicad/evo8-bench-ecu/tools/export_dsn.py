#!/usr/bin/env python3
"""Export the fixed bench draft with plane-only inner layers and D5 margins."""
import argparse
from collections import Counter
from pathlib import Path

import pcbnew
from preroutes import PREROUTES
from sexpr import Atom, child, children, dump, parse


def walk(node):
    if isinstance(node, list):
        yield node
        for value in node:
            yield from walk(value)


def point_mm(point):
    # Round to integer units; FromMM(130.95) truncates to 130949999.
    return tuple(round(value * pcbnew.FromMM(1)) for value in point)


def matches(item, route):
    if item.GetNetname() != route['net']:
        return False
    if route['layer'] == 'via':
        return (isinstance(item, pcbnew.PCB_VIA)
                and (item.GetPosition().x, item.GetPosition().y) == point_mm(route['position'])
                and item.GetWidth(pcbnew.F_Cu) == pcbnew.FromMM(route['size'])
                and item.GetDrillValue() == pcbnew.FromMM(route['drill'])
                and item.TopLayer() == pcbnew.F_Cu and item.BottomLayer() == pcbnew.B_Cu
                and item.GetViaType() == pcbnew.VIATYPE_THROUGH)
    if isinstance(item, pcbnew.PCB_VIA):
        return False
    endpoints = ((item.GetStart().x, item.GetStart().y),
                 (item.GetEnd().x, item.GetEnd().y))
    expected = (point_mm(route['start']), point_mm(route['end']))
    return (endpoints in (expected, expected[::-1])
            and item.GetLayerName() == route['layer']
            and item.GetWidth() == pcbnew.FromMM(route['width']))


def check_wiring(wiring):
    """Require the sixteen protected items, with their exact DSN geometry."""
    wires, vias = children(wiring, 'wire'), children(wiring, 'via')
    assert len(wires) == 11 and len(vias) == 6, 'pre-route DSN counts'
    actual = Counter()
    for item in wires + vias:
        assert children(item, 'type') == [[Atom('type'), Atom('protect')]], 'unprotected pre-route'
        net = child(item, 'net')
        assert net is not None, 'pre-route missing net'
        if item[0] == 'wire':
            path = child(item, 'path')
            assert path is not None and len(path) == 7, 'pre-route path geometry'
            actual[('wire', str(net[1]), str(path[1]),
                    *map(float, path[2:]))] += 1
        else:
            actual[('via', str(net[1]), str(item[1]),
                    float(item[2]), float(item[3]))] += 1
    expected = Counter()
    for route in PREROUTES:
        if route['layer'] == 'via':
            x, y = point_mm(route['position'])
            expected[('via', route['net'], 'Via[0-3]_600:300_um', x / 1000, -y / 1000)] += 1
        else:
            x1, y1 = point_mm(route['start'])
            x2, y2 = point_mm(route['end'])
            expected[('wire', route['net'], route['layer'],
                      pcbnew.FromMM(route['width']) / 1000,
                      x1 / 1000, -y1 / 1000, x2 / 1000, -y2 / 1000)] += 1
    assert actual == expected, f'pre-route DSN geometry: {actual - expected}; missing: {expected - actual}'


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
    kept = []
    for route in PREROUTES[:10]:
        found = [item for item in copper if matches(item, route)]
        assert len(found) == 1, f"draft pre-route {route['id']}: {len(found)} matches"
        kept.extend(found)
    assert len(kept) == 10, 'draft pre-route count'
    for item in copper:
        if item not in kept:
            board.Remove(item)
    for route in PREROUTES[10:]:
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
            item.SetWidth(pcbnew.FromMM(route['width']))
            item.SetLayer(board.GetLayerID(route['layer']))
        item.SetNetCode(board.FindNet(route['net']).GetNetCode())
        board.Add(item)
        kept.append(item)
    for item in kept:
        item.SetLocked(True)
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
    for item in children(wiring, 'wire') + children(wiring, 'via'):
        kind = child(item, 'type')
        if kind is None:
            item.append([Atom('type'), Atom('protect')])
        else:
            kind[:] = [Atom('type'), Atom('protect')]
    check_wiring(wiring)
    args.out.write_text(dump(tree) + '\n')
    print({'layers': len(layers), 'placements': 305, 'multi_pad_nets': len(multi),
           'exported_nets': len(exported_nets),
           'prerouted_wires': 11, 'prerouted_vias': 6}, flush=True)


if __name__ == '__main__':
    main()
