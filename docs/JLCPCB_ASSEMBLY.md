# JLCPCB assembly review — 2026-11-13

Incomplete sourcing package; no upload, quote, DFM acceptance or H3/H4 approval obtained.

JLCPCB supports mixed SMT/THT assembly according to https://jlcpcb.com/capabilities/pcb-assembly-capabilities and https://jlcpcb.com/help/article/pcb-assembly-faqs . This does not establish acceptance of these specific parts.

Assembly intent: place 123 top SMT components and solder the board-side THT connectors, socket, switch and MAP sensor through a contracted assembly service. Owner hand placement is not an acceptable completion path. Arduino Mega and Pololu DRV8825 carrier are customer-supplied plug-ins, not reflow placements. Arduino Mega is not a purchased item in this board BOM. A1 needs separate board-side socket/header sourcing; XA1 represents the shield mating headers, not the Mega itself. IC3 is a purchased DIP-8 socket; its plug-in trigger conditioner needs separate identification. JP6–JP11 are bare-board copper features, no purchased parts or assembly operation. Preserve their existing bridges.

Missing verified library mappings must use exact-part sourcing or a confirmed consignment route; availability and fees are unresolved. Do not approve a substitute for power, analogue, trigger or output circuitry merely because its pins fit.

Existing review inputs (relative to hardware/prototype-v1): manufacturing/gerbers/* and manufacturing/drill/v0.4.4d.drl; manufacturing/assembly/v0.4.4d-bom.csv and v0.4.4d-pos.csv. These are NOT yet upload-ready JLC files. Produce a filtered BOM with Comment, Designator, Footprint and LCSC Part #; CPL needs Designator, Mid X, Mid Y, Layer, Rotation in mm. Exclude modules, copper jumpers, holes and proto pads; include the separately sourced board-side headers. Verify coordinates, bottom convention, rotations and pin 1 against assembly drawings before upload. Guide: https://jlcpcb.com/help/article/advice-for-bom-and-cpl-files-preparation . No converted files are released here.

DFM questions for reviewer and assembler:

- Confirm actual minimum annulus against the project 0.05 mm rule and fabrication capability.
- Quote actual 100.05 × 100.05 mm outline; do not assume a 100 × 100 promotion or trim the design.
- Confirm adjacent J1/J5/J6/J8 housing end tolerances, solder access and installed fit.
- Confirm mixed SMT/THT process, selective/manual labour, module sockets, Mega header stack, MAP mounting and complete assembly without owner soldering.
- Select J4 keyed IDC40 header and real mating ribbon socket as a pair; verify notch and pin-1 triangle against PCB pad 1 and cable stripe using drawings and continuity. No vehicle connector or pin mapping is specified.
