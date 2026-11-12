#!/usr/bin/env python3
"""Compare protected board data and report routing and zone-fill counts."""
import argparse
from collections import Counter
import json
from pathlib import Path
import re

from sexpr import Atom, child, children, parse


def canonical(value):
    if isinstance(value, list):
        return [canonical(x) for x in value]
    if isinstance(value, Atom) and re.fullmatch(r'[+-]?(?:\d+(?:\.\d*)?|\.\d+)(?:[eE][+-]?\d+)?', value):
        return round(float(value), 6)
    return str(value)


def field(node, tag):
    value = child(node or [], tag)
    return canonical(value[1:]) if value is not None else None


def at(node):
    value = field(node, 'at')
    assert value is not None and len(value) in (2, 3), 'invalid at'
    return value + [0] if len(value) == 2 else value


def ordered(items):
    return sorted(items, key=lambda x: json.dumps(x, sort_keys=True))


def protected(board):
    footprints = []
    for fp in children(board, 'footprint'):
        reference = [p[2] for p in children(fp, 'property') if p[1] == 'Reference']
        assert len(reference) == 1, 'missing or duplicate reference'
        pads = []
        for pad in children(fp, 'pad'):
            pads.append({'number_type_shape': canonical(pad[1:4]), 'at': at(pad),
                         **{tag: field(pad, tag) for tag in
                            ('size', 'drill', 'layers', 'net')}})
        footprints.append({'reference': reference[0], 'library': fp[1],
                           'at': at(fp), 'layer': field(fp, 'layer'),
                           'pads': ordered(pads)})
    zones = []
    for zone in children(board, 'zone'):
        fill = child(zone, 'fill', [])
        zones.append({**{tag: field(zone, tag) for tag in
                         ('net', 'net_name', 'layer', 'min_thickness')},
                      'polygons': ordered(canonical(children(zone, 'polygon'))),
                      'clearance': field(child(zone, 'connect_pads', []), 'clearance'),
                      'thermal_gap': field(fill, 'thermal_gap'),
                      'thermal_bridge_width': field(fill, 'thermal_bridge_width')})
    edges = []
    for line in children(board, 'gr_line'):
        if field(line, 'layer') == ['Edge.Cuts']:
            edges.append({tag: field(line, tag) for tag in ('start', 'end')})
    return {'nets': ordered(canonical(children(board, 'net'))),
            'footprints': ordered(footprints), 'edge_cuts': ordered(edges),
            'zones': ordered(zones)}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('draft', type=Path)
    parser.add_argument('routed', type=Path)
    args = parser.parse_args()
    draft = parse(args.draft.read_text())
    routed = parse(args.routed.read_text())
    before, after = protected(draft), protected(routed)
    mismatches = []
    for category in before:
        if before[category] != after[category]:
            mismatches.append({'category': category, 'draft': before[category],
                               'routed': after[category]})
    segments = Counter({layer: 0 for layer in ('F.Cu', 'In1.Cu', 'In2.Cu', 'B.Cu')})
    segments.update(field(s, 'layer')[0] for s in children(routed, 'segment'))
    zones = [{'net': field(z, 'net_name')[0], 'layer': field(z, 'layer')[0],
              'filled_polygon': len(children(z, 'filled_polygon'))}
             for z in children(routed, 'zone')]
    result = {'segments_per_layer': dict(segments),
              'vias': len(children(routed, 'via')), 'zones': zones,
              'mismatches': mismatches}
    print(json.dumps(result, indent=2))
    return int(bool(mismatches) or segments['In1.Cu'] > 0 or segments['In2.Cu'] > 0
               or any(z['filled_polygon'] == 0 for z in zones))


if __name__ == '__main__':
    raise SystemExit(main())
