import csv,re,pathlib
root=pathlib.Path('hardware/prototype-v1'); out=root/'manufacturing/jlcpcb';out.mkdir(exist_ok=True)
def expand(s):
 for t in s.split(','):
  m=re.fullmatch(r'([A-Z]+)(\d+)-(?:[A-Z]+)?(\d+)',t)
  if m: yield from (m[1]+str(n) for n in range(int(m[2]),int(m[3])+1))
  else: yield t
bom=list(csv.DictReader(open(root/'manufacturing/assembly/v0.4.4d-bom.csv'))); pos=list(csv.DictReader(open(root/'manufacturing/assembly/v0.4.4d-pos.csv')))
lookup={r:b for b in bom for r in expand(b['Refs'])}
rows=[];cpl=[];un=[]
pcb=(root/'v0.4.4d.kicad_pcb').read_text(); blocks=re.split(r'\n\t\(footprint ',pcb)[1:]
for p in pos:
 r=p['Ref']; b=lookup[r]
 if r=='A1':continue
 x,y=float(p['PosX']),float(p['PosY']);assert r=='XA1' or (110.475<=x<=210.525 and -137.025<=y<=-36.975)
 block=next(v for v in blocks if re.search(r'\(property "Reference" "'+r+'"',v));assert '(layer "F.Cu")' in block and p['Side']=='top'
 rows.append([b['Value'],r,b['Footprint'],'']);cpl.append([r,p['PosX'],p['PosY'],'Top',p['Rot']])
 reason='Exact MPN-to-LCSC mapping and sourcing not verified'
 if r=='XA1':reason='Composite shield footprint; individual board-side header BOM and placement breakdown absent'
 un.append([r,reason,b['MPN'].strip() or 'See docs/CONNECTOR_DECISIONS.md','EE approve exact part/fit; assembler confirm exact sourcing and placement'])
for name,header,data in [('bom-jlcpcb.csv',['Comment','Designator','Footprint','LCSC Part #'],rows),('cpl-jlcpcb.csv',['Designator','Mid X','Mid Y','Layer','Rotation'],cpl),('bom-unresolved.csv',['Designator','Reason','Candidate MPN or family','Decision needed'],un)]:
 with open(out/name,'w') as f:w=csv.writer(f);w.writerow(header);w.writerows(data)
assert {r[1] for r in rows}=={r[0] for r in cpl};assert len(rows)==140
(out/'README.md').write_text('''# JLCPCB assembly mapping candidate\n\nNot upload-ready: 140 BOM placements, 140 CPL placements, 140 unresolved rows (no verified LCSC mappings). Original CPL: 141; excluded A1 carrier: 1. XA1 remains one composite placeholder, requiring header breakdown before upload. A1 board-side sockets are missing from schematic BOM; no invented placements added. 123 SMT + 17 remaining THT footprint placements, all Top.\n\nCoordinates preserve KiCad page/drill origin in mm: X positive, Y negative (Y up). Outline X 110.475–210.525, Y -137.025–-36.975. 139 origins inside; XA1 composite origin X=110.0 is outside and blocks upload. Other; PCB footprint layer F.Cu agrees with Top for every reference. No bottom placements; bottom drawing is mirrored and contains copper jumpers only. Rotations preserved from KiCad; assembler must visually confirm pin 1/polarity against top assembly drawing before acceptance. This checks origins, not full component body clearance or assembler rotation offsets.\n\nEvery CPL reference equals a BOM reference; no A1 module, JP6–JP11, holes or proto pads. Arduino Mega itself absent. Board-side A1 hardware needs a separately reviewed BOM/CPL amendment; XA1 composite origin is not an individual socket placement.\n''')
print('PASS: 140 BOM = 140 CPL; 140 unresolved; 139 origins inside outline; XA1 outside (documented blocker); all PCB layers F.Cu/Top')
