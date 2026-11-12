# UNREVIEWED bench-prototype PCB tools

Bench prototype only. UNREVIEWED, NOT FOR FABRICATION. The routing script does
not establish manufacturing suitability, safety, EMC performance or vehicle
compatibility. Preserve the existing warnings on the board and in its outputs.

## Deterministic grid router

Run from the repository root; no packages or `pcbnew` are required:

```sh
cd hardware/kicad/evo8-bench-ecu
/usr/bin/python3 tools/route_pcb.py evo8-bench-ecu.kicad_pcb /tmp/routed-a.kicad_pcb
/usr/bin/python3 tools/route_pcb.py evo8-bench-ecu.kicad_pcb /tmp/routed-b.kicad_pcb
cmp /tmp/routed-a.kicad_pcb /tmp/routed-b.kicad_pcb
```

The router uses a 0.1 mm grid within the board outline, F.Cu and B.Cu, and
through vias when it changes layers. It routes power, then ground, then signal
nets using existing net names and pad positions. Conservative rectangular
obstacle envelopes preserve clearance around copper from other nets. Existing
tracks, vias, zones, footprints, outline, labels and rules are preserved.

Rules and net classes come from the input PCB and its sibling `.kicad_pro`.
The current project's Default net class specifies 0.25 mm tracks, 0.2 mm
clearance, 0.6 mm vias and 0.3 mm drills; those class values override the spec's
fallback widths. Without a class width, the defaults are 1.0 mm power, 0.5 mm
ground and 0.25 mm signal. Without a clearance setting, the default is 0.2 mm.

Output must differ from the input, including hard links and symbolic links.
The script never replaces the source PCB. A failure to route returns exit 2,
lists the remaining net names and terminals, and leaves a partial output for
inspection. Never promote a partial output to the source board. Determinism
applies to identical PCB and project inputs. UUIDs derive from route geometry;
no clock or random seed is used.

## KiCad DRC

Run from `hardware/kicad/evo8-bench-ecu`:

```sh
KC=/tmp/opencode/evo-hardware/.toolchain/root/usr/bin/kicad-cli
export LD_LIBRARY_PATH=/tmp/opencode/evo-hardware/.toolchain/root/usr/lib/x86_64-linux-gnu
$KC pcb drc --format json --severity-all --output reports/drc.json evo8-bench-ecu.kicad_pcb
/usr/bin/python3 -c "import json,collections;d=json.load(open('reports/drc.json'));print(collections.Counter((v['severity'],v['type']) for v in d['violations']),len(d['unconnected_items']))"
```

The CLI can return exit 0 while reporting violations. Read the JSON counts.
Zone filling and clearance/dangling-via cleanup are separate Phase B steps;
this script does not claim those checks pass. The prototype requires independent
review, verification of footprint libraries, and a zone-fill check.
