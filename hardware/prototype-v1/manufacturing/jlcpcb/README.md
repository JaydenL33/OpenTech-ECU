# JLCPCB assembly mapping candidate

Not upload-ready: 140 BOM placements, 140 CPL placements, 140 unresolved rows (no verified LCSC mappings). Original CPL: 141; excluded A1 carrier: 1. XA1 remains one composite placeholder, requiring header breakdown before upload. A1 board-side sockets are missing from schematic BOM; no invented placements added. 123 SMT + 17 remaining THT footprint placements, all Top.

Coordinates preserve KiCad page/drill origin in mm: X positive, Y negative (Y up). Outline X 110.475–210.525, Y -137.025–-36.975. 139 origins inside; XA1 composite origin X=110.0 is outside and blocks upload. Other; PCB footprint layer F.Cu agrees with Top for every reference. No bottom placements; bottom drawing is mirrored and contains copper jumpers only. Rotations preserved from KiCad; assembler must visually confirm pin 1/polarity against top assembly drawing before acceptance. This checks origins, not full component body clearance or assembler rotation offsets.

Every CPL reference equals a BOM reference; no A1 module, JP6–JP11, holes or proto pads. Arduino Mega itself absent. Board-side A1 hardware needs a separately reviewed BOM/CPL amendment; XA1 composite origin is not an individual socket placement.
