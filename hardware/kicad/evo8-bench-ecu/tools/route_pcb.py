#!/usr/bin/python3
"""Deterministic two-layer grid routing for the UNREVIEWED bench PCB.

No packages, pcbnew, schematic edits, placement edits, or design-rule edits.
The input is never overwritten. A nonzero result means the output is only a
partial routing attempt and MUST NOT be treated as a completed board.
KiCad DRC, zone filling, and independent review remain separate requirements.
"""

import argparse
from collections import defaultdict
from dataclasses import dataclass
import fnmatch
import heapq
import itertools
import json
import math
from pathlib import Path
import sys
import uuid

sys.dont_write_bytecode = True
from sexpr import child, children, dump, node, parse


GRID = 0.1
LAYERS = ("F.Cu", "B.Cu")
EPS = 1e-8
BIN = 2.0
NAMESPACE = uuid.UUID("343c648e-e194-5b48-bc11-178b461ec25f")


def number(value):
    return round(float(value), 6)


def xy(item, tag):
    return tuple(map(float, child(item, tag)[1:3]))


def box(x, y, hx, hy):
    return (x - hx, y - hy, x + hx, y + hy)


def overlap(a, b, gap):
    # Touching the exact clearance boundary is allowed; no rule is relaxed.
    return (a[0] < b[2] + gap - EPS and a[2] > b[0] - gap + EPS
            and a[1] < b[3] + gap - EPS and a[3] > b[1] - gap + EPS)


@dataclass(frozen=True)
class Rules:
    clearance: float
    width: float
    via: float
    drill: float


@dataclass(frozen=True)
class Terminal:
    net: int
    x: float
    y: float
    layers: tuple
    label: str


@dataclass(frozen=True)
class Obstacle:
    net: int
    layers: tuple
    bounds: tuple
    clearance: float
    # NPTH holes additionally constrain via drilling, even on the same net.
    hole: bool = False


class RuleBook:
    def __init__(self, source, board, names):
        project = source.with_suffix(".kicad_pro")
        data = json.loads(project.read_text()) if project.is_file() else {}
        settings = data.get("board", {}).get("design_settings", {})
        rules = settings.get("rules", {})
        self.minimum = float(rules.get("min_clearance", 0.2))
        self.edge = float(rules.get("min_copper_edge_clearance", 0.5))
        self.hole = float(rules.get("min_hole_clearance", 0.25))
        self.min_via = float(rules.get("min_via_diameter", 0.6))
        self.min_drill = float(rules.get("min_through_hole_diameter", 0.3))
        net_settings = data.get("net_settings", {})
        classes = {c["name"]: c for c in net_settings.get("classes", [])}
        assignments = dict(net_settings.get("netclass_assignments", {}))
        patterns = net_settings.get("netclass_patterns", [])
        # Older boards can store net classes directly in the PCB file.
        for item in children(board, "net_class"):
            values = {x[0]: float(x[1]) for x in item[3:]
                      if isinstance(x, list) and x[0] in
                      ("clearance", "trace_width", "via_dia", "via_drill")}
            classes[str(item[1])] = {
                {"trace_width": "track_width", "via_dia": "via_diameter"}.get(k, k): v
                for k, v in values.items()}
            assignments.update({str(n[1]): str(item[1])
                                for n in children(item, "add_net")})
        self.by_net = {}
        for net, name in names.items():
            class_name = assignments.get(name, "Default")
            matching = [p for p in patterns if fnmatch.fnmatchcase(name, p["pattern"])]
            if name not in assignments and matching:
                class_name = matching[0]["netclass"]
            cls = classes.get(class_name, {})
            width = 1.0 if net_order(name) == 0 else 0.5 if net_order(name) == 1 else 0.25
            self.by_net[net] = Rules(
                max(self.minimum, float(cls.get("clearance", self.minimum))),
                max(float(rules.get("min_track_width", 0.0)),
                    float(cls.get("track_width", width))),
                max(self.min_via, float(cls.get("via_diameter", 0.6))),
                max(self.min_drill, float(cls.get("via_drill", 0.3))))


def net_order(name):
    """Classify existing net names only; never assign or change a net."""
    if name.upper().endswith("GND"):
        return 1
    if (name.startswith("+") or name.startswith("VIN_")
            or name in ("VDDA", "USB_VBUS", "ACT_FEED")
            or name.endswith("_SUPPLY")):
        return 0
    return 2


class Geometry:
    """Conservative copper bounding boxes in a spatial index.

    Rectangular envelopes overestimate rounded pads/circular vias, never reduce
    clearance. Axis-aligned grid tracks permit exact swept-box collision tests.
    """
    def __init__(self, outline, rules):
        self.outline = outline
        self.rules = rules
        self.bins = defaultdict(list)
        self.search_gap = max(rules.hole, *(r.clearance for r in rules.by_net.values()))

    @staticmethod
    def bins_for(bounds):
        x0, y0, x1, y1 = bounds
        for x in range(math.floor(x0 / BIN), math.floor(x1 / BIN) + 1):
            for y in range(math.floor(y0 / BIN), math.floor(y1 / BIN) + 1):
                yield (x, y)

    def add(self, obstacle):
        for key in self.bins_for(obstacle.bounds):
            self.bins[key].append(obstacle)

    def free(self, net, layers, bounds, via=False):
        edge = self.rules.edge
        x0, y0, x1, y1 = self.outline
        if (bounds[0] < x0 + edge - EPS or bounds[1] < y0 + edge - EPS
                or bounds[2] > x1 - edge + EPS or bounds[3] > y1 - edge + EPS):
            return False
        clearance = self.rules.by_net[net].clearance
        # Query the largest required gap, including obstacles' net classes.
        search_gap = max(clearance, self.search_gap)
        expanded = (bounds[0] - search_gap, bounds[1] - search_gap,
                    bounds[2] + search_gap, bounds[3] + search_gap)
        seen = set()
        for key in self.bins_for(expanded):
            for obstacle in self.bins.get(key, ()):
                ident = id(obstacle)
                if ident in seen:
                    continue
                seen.add(ident)
                if not any(layer in obstacle.layers for layer in layers):
                    continue
                if obstacle.net == net and not obstacle.hole:
                    continue
                gap = max(clearance, obstacle.clearance)
                if via and obstacle.hole:
                    # Via copper checked against the hole envelope is stricter
                    # than checking only the via drill against hole clearance.
                    gap = max(gap, self.rules.hole)
                if overlap(bounds, obstacle.bounds, gap):
                    return False
        return True

    def track_free(self, net, layer, a, b):
        radius = self.rules.by_net[net].width / 2
        return self.free(net, (layer,),
                         (min(a[0], b[0]) - radius, min(a[1], b[1]) - radius,
                          max(a[0], b[0]) + radius, max(a[1], b[1]) + radius))

    def via_free(self, net, x, y):
        radius = self.rules.by_net[net].via / 2
        return self.free(net, LAYERS, box(x, y, radius, radius), via=True)


def outline_bounds(board):
    edges = [e for e in children(board, "gr_line")
             if child(e, "layer")[1] == "Edge.Cuts"]
    if any(child(e, "layer", [None, None])[1] == "Edge.Cuts"
           for tag in ("gr_arc", "gr_rect", "gr_poly", "gr_circle")
           for e in children(board, tag)):
        raise ValueError("only the draft's rectangular four-line outline is supported")
    points = [xy(e, tag) for e in edges for tag in ("start", "end")]
    if len(edges) != 4 or any(xy(e, "start")[0] != xy(e, "end")[0]
                              and xy(e, "start")[1] != xy(e, "end")[1] for e in edges):
        raise ValueError("expected a closed axis-aligned rectangular board outline")
    if any(points.count(p) != 2 for p in points) or len(set(points)) != 4:
        raise ValueError("board outline is not a closed rectangle")
    xs, ys = zip(*points)
    return min(xs), min(ys), max(xs), max(ys)


def collect(board, geometry):
    terminals = defaultdict(list)
    for footprint in children(board, "footprint"):
        at = child(footprint, "at")
        fx, fy = map(float, at[1:3])
        angle = math.radians(float(at[3]) if len(at) > 3 else 0)
        ref = next((p[2] for p in children(footprint, "property")
                    if p[1] == "Reference"), "?")
        for pad in children(footprint, "pad"):
            pad_layers = child(pad, "layers")[1:]
            layers = tuple(l for l in LAYERS if l in pad_layers or "*.Cu" in pad_layers)
            if not layers:
                continue
            if pad[3] not in ("circle", "oval", "rect", "roundrect"):
                raise ValueError(f"unsupported pad shape {pad[3]} on {ref}.{pad[1]}")
            px, py = xy(pad, "at")
            x = fx + px * math.cos(angle) + py * math.sin(angle)
            y = fy - px * math.sin(angle) + py * math.cos(angle)
            size = xy(pad, "size")
            pat = child(pad, "at")
            # KiCad serializes each pad's orientation in board coordinates.
            pa = math.radians(float(pat[3]) if len(pat) > 3 else 0)
            hx = (abs(size[0] * math.cos(pa)) + abs(size[1] * math.sin(pa))) / 2
            hy = (abs(size[0] * math.sin(pa)) + abs(size[1] * math.cos(pa))) / 2
            net = int(child(pad, "net", [None, 0])[1])
            hole = pad[2] == "np_thru_hole"
            clearance = float(child(pad, "clearance", [None, 0])[1])
            clearance = max(clearance, geometry.rules.by_net[net].clearance)
            geometry.add(Obstacle(net, layers, box(x, y, hx, hy), clearance, hole))
            if net and not hole:
                terminals[net].append(Terminal(net, number(x), number(y), layers,
                                              f"{ref}.{pad[1]}"))
    for segment in children(board, "segment"):
        layer = child(segment, "layer")[1]
        if layer not in LAYERS:
            raise ValueError("existing track uses an internal layer; refusing to discard it")
        a, b = xy(segment, "start"), xy(segment, "end")
        radius = float(child(segment, "width")[1]) / 2
        net = int(child(segment, "net")[1])
        geometry.add(Obstacle(net, (layer,),
                              (min(a[0], b[0]) - radius, min(a[1], b[1]) - radius,
                               max(a[0], b[0]) + radius, max(a[1], b[1]) + radius),
                              geometry.rules.by_net[net].clearance))
    for via in children(board, "via"):
        x, y = xy(via, "at")
        net = int(child(via, "net")[1])
        radius = float(child(via, "size")[1]) / 2
        geometry.add(Obstacle(net, LAYERS, box(x, y, radius, radius),
                              geometry.rules.by_net[net].clearance))
        if net:
            terminals[net].append(Terminal(net, x, y, LAYERS,
                                          "via:" + str(child(via, "uuid")[1])))
    return terminals


def ports(terminal, geometry):
    x, y = round(terminal.x / GRID), round(terminal.y / GRID)
    a, b = (terminal.x, terminal.y), (x * GRID, y * GRID)
    return [(x, y, LAYERS.index(layer)) for layer in terminal.layers
            if geometry.track_free(terminal.net, layer, a, b)]


def find_path(net, starts, goals, geometry):
    """Bounded deterministic A*, with increasing search windows.

    All vertices are on the 0.1 mm grid. Longer steps accelerate open areas;
    unit steps preserve access to narrow channels. Each swept track and via
    is checked against the complete existing obstacle index.
    """
    if not starts or not goals:
        return None
    goal_set = set(goals)
    gx, gy, _ = goals[0]
    x0, y0, x1, y1 = geometry.outline
    limits = (math.ceil(x0 / GRID), math.ceil(y0 / GRID),
              math.floor(x1 / GRID), math.floor(y1 / GRID))
    track_cache = {}
    via_cache = {}

    def estimate(state):
        x, y, layer = state
        return min(abs(x - g[0]) + abs(y - g[1])
                   + (0 if layer == g[2] else 80) for g in goals)

    for margin, max_expansions in ((50, 150000), (200, 300000), (None, 600000)):
        if margin is None:
            bounds = limits
        else:
            xs = [p[0] for p in starts + goals]
            ys = [p[1] for p in starts + goals]
            bounds = (max(limits[0], min(xs) - margin), max(limits[1], min(ys) - margin),
                      min(limits[2], max(xs) + margin), min(limits[3], max(ys) + margin))
        sequence = itertools.count()
        queue = []
        distance = {}
        previous = {}
        for start in starts:
            distance[start] = 0
            heapq.heappush(queue, (estimate(start), 0, next(sequence), start))
        expanded = 0
        while queue and expanded < max_expansions:
            _, cost, _, state = heapq.heappop(queue)
            if cost != distance.get(state):
                continue
            if state in goal_set:
                result = [state]
                while state in previous:
                    state = previous[state]
                    result.append(state)
                return list(reversed(result))
            expanded += 1
            x, y, layer = state
            # Prefer F.Cu horizontal, B.Cu vertical, with deterministic ties.
            directions = ((1, 0), (-1, 0), (0, 1), (0, -1))
            if layer == 1:
                directions = directions[2:] + directions[:2]
            candidates = []
            for step in (20, 5, 1):
                for dx, dy in directions:
                    nx, ny = x + step * dx, y + step * dy
                    if not (bounds[0] <= nx <= bounds[2] and bounds[1] <= ny <= bounds[3]):
                        continue
                    key = (x, y, nx, ny, layer)
                    if key not in track_cache:
                        track_cache[key] = geometry.track_free(
                            net, LAYERS[layer], (x * GRID, y * GRID), (nx * GRID, ny * GRID))
                    if track_cache[key]:
                        preferred = (dy == 0) if layer == 0 else (dx == 0)
                        candidates.append(((nx, ny, layer), step * (1.0 if preferred else 1.05)))
            if (x, y) not in via_cache:
                via_cache[x, y] = geometry.via_free(net, x * GRID, y * GRID)
            if via_cache[x, y]:
                candidates.append(((x, y, 1 - layer), 80))
            for neighbor, increment in candidates:
                new_cost = cost + increment
                if new_cost + EPS < distance.get(neighbor, math.inf):
                    distance[neighbor] = new_cost
                    previous[neighbor] = state
                    # Weighted A* limits search in this large prototype outline.
                    priority = new_cost + 1.15 * estimate(neighbor)
                    heapq.heappush(queue, (priority, new_cost, next(sequence), neighbor))
    return None


class Router:
    def __init__(self, geometry):
        self.geometry = geometry
        self.additions = []
        self.existing = set()

    def add(self, net, a, b, layer):
        a, b = tuple(map(number, a)), tuple(map(number, b))
        if a == b:
            return
        rule = self.geometry.rules.by_net[net]
        key = ("track", net, layer, tuple(sorted((a, b))), rule.width)
        if key in self.existing:
            return
        self.existing.add(key)
        self.additions.append(node("segment", node("start", *a), node("end", *b),
                                   node("width", rule.width), node("layer", layer),
                                   node("net", net),
                                   node("uuid", str(uuid.uuid5(NAMESPACE, repr(key))))))
        r = rule.width / 2
        self.geometry.add(Obstacle(net, (layer,),
                                  (min(a[0], b[0]) - r, min(a[1], b[1]) - r,
                                   max(a[0], b[0]) + r, max(a[1], b[1]) + r), rule.clearance))

    def via(self, net, point):
        x, y = map(number, point)
        rule = self.geometry.rules.by_net[net]
        key = ("via", net, x, y, rule.via, rule.drill)
        if key in self.existing:
            return
        self.existing.add(key)
        self.additions.append(node("via", node("at", x, y), node("size", rule.via),
                                   node("drill", rule.drill), node("layers", *LAYERS),
                                   node("net", net),
                                   node("uuid", str(uuid.uuid5(NAMESPACE, repr(key))))))
        self.geometry.add(Obstacle(net, LAYERS, box(x, y, rule.via / 2, rule.via / 2),
                                  rule.clearance))

    def connect(self, source, target):
        net = source.net
        path = find_path(net, ports(source, self.geometry), ports(target, self.geometry),
                         self.geometry)
        if path is None:
            return False
        # Merge collinear grid edges before writing, without changing geometry.
        points = [(x * GRID, y * GRID, LAYERS[layer]) for x, y, layer in path]
        self.add(net, (source.x, source.y), points[0][:2], points[0][2])
        run_start = points[0]
        prior = points[0]
        direction = None
        for current in points[1:]:
            if current[2] != prior[2]:
                self.add(net, run_start[:2], prior[:2], prior[2])
                self.via(net, prior[:2])
                run_start = current
                direction = None
            else:
                delta = (number(current[0] - prior[0]), number(current[1] - prior[1]))
                new_direction = (int(math.copysign(1, delta[0])) if delta[0] else 0,
                                 int(math.copysign(1, delta[1])) if delta[1] else 0)
                if direction is not None and new_direction != direction:
                    self.add(net, run_start[:2], prior[:2], prior[2])
                    run_start = prior
                direction = new_direction
            prior = current
        self.add(net, run_start[:2], prior[:2], prior[2])
        self.add(net, prior[:2], (target.x, target.y), prior[2])
        return True


def route(source, destination):
    if source.resolve() == destination.resolve():
        raise ValueError("output must differ from input; in-place routing is forbidden")
    if destination.exists() and source.samefile(destination):
        raise ValueError("output aliases input; in-place routing is forbidden")
    text = source.read_text()
    board = parse(text)
    names = {int(n[1]): str(n[2]) for n in children(board, "net")}
    rules = RuleBook(source, board, names)
    geometry = Geometry(outline_bounds(board), rules)
    terminals = collect(board, geometry)
    router = Router(geometry)
    failures = []
    for net in sorted(terminals, key=lambda n: (net_order(names[n]), names[n], n)):
        pending = sorted(terminals[net], key=lambda p: (p.x, p.y, p.label))
        connected = [pending.pop(0)]
        attempts = 0
        # Prim ordering connects nearest pads first; failed edges can retry
        # through another terminal as the net's spanning tree grows.
        while pending:
            pairs = sorted(((abs(a.x - b.x) + abs(a.y - b.y), i, j)
                            for i, a in enumerate(connected) for j, b in enumerate(pending)))
            success = False
            for _, i, j in pairs:
                attempts += 1
                if router.connect(connected[i], pending[j]):
                    connected.append(pending.pop(j))
                    success = True
                    break
                # Bound work for an obstructed net, deterministically.
                if attempts >= 3 * len(terminals[net]):
                    break
            if not success:
                failures.append((names[net], [p.label for p in pending]))
                break
        print(f"{names[net]}: {len(connected)}/{len(terminals[net])} terminals connected",
              flush=True)
    # Preserve original bytes for all existing objects and all warning labels.
    closing = text.rfind(")")
    rendered = "".join("\n  " + dump(item) for item in router.additions)
    destination.write_text(text[:closing] + rendered + "\n" + text[closing:])
    print(f"Added {len(router.additions)} copper objects; wrote {destination}", flush=True)
    if failures:
        print(f"INCOMPLETE: {len(failures)} unrouted nets:", file=sys.stderr)
        for name, pads in failures:
            print(f"  {name}: {', '.join(pads)}", file=sys.stderr)
        return 2
    print("Grid routing completed; KiCad DRC and zone fill checks are still required.")
    return 0


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("input", type=Path)
    parser.add_argument("output", type=Path)
    args = parser.parse_args()
    try:
        return route(args.input, args.output)
    except (ValueError, OSError, KeyError) as error:
        print(f"route_pcb: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
