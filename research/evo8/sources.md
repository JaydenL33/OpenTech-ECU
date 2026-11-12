# Evo VIII research sources

Access date for everything below: **2026-11-12**. This is a research log, not an engineering
release. No source here is a substitute for the as-built vehicle.

## How the sources were accessed

- HTML pages were read directly or through a text-rendering proxy.
- PDFs were read as extracted text. Extracted text loses drawing labels, pin numbers printed
  inside connector symbols, and some table structure. Where a page number or section number is
  quoted, it is the printed page/section visible in the extracted text. **No page number has been
  invented.** OCR errors are possible and are flagged where they affect meaning.
- The Mitsubishi factory manuals are proprietary documents. They are cited for identification and
  research only. Do not redistribute them from this repository.

## Source key

| ID | Source | URL | What it is | Supports | Reliability | Licence / copyright |
|---|---|---|---|---|---|---|
| S1 | evoscan / evoscan-manuals (GitHub) | https://github.com/evoscan/evoscan-manuals | Index of Mitsubishi workshop manuals, wiring diagrams, TIMs, ROM files | Locating S2–S5, S8, S11, S20 | Medium (index); content quality varies | Repository says documents are historical workshop manuals offered for research/reference/AI training. Underlying Mitsubishi documents remain Mitsubishi copyright. |
| S2 | Mitsubishi Evo VIII ECU Connections (MLR) | https://evoscan.github.io/evoscan-manuals/wp-content/uploads/2022/02/Evo-8-ECU.pdf | 1-page community pinout sheet, 3-column/100-number layout | 3-plug Evo VIII function list; comparison only | Low–medium (community, marked "© grants and MLR") | Copyright MLR / author. Not a factory document. |
| S3 | Mitsubishi Evo VIII MR ECU Connections (MLR) | https://evoscan.github.io/evoscan-manuals/wp-content/uploads/2022/02/Evo-8-MR-ECU.pdf | 1-page community pinout sheet, same layout as S2, adds wastegate solenoid 2 | Evidence that an MR variant has a second wastegate solenoid; 3-plug layout | Low–medium (community) | Copyright MLR / author. |
| S4 | Mitsubishi Evo VIII 260 ECU Connections (MLR) | https://evoscan.github.io/evoscan-manuals/wp-content/uploads/2022/02/Evo-8-260-ECU.pdf | 1-page community pinout sheet, **4-column / 1-26, 31-46, 51-62, 71-92** | Independent corroboration of the 4-plug numbering and UK-market functions | Low–medium (community) | Copyright MLR / author. |
| S5 | Mitsubishi Evo VII ECU Connections (MLR) | https://evoscan.github.io/evoscan-manuals/wp-content/uploads/2022/02/Evo-7-ECU.pdf | 1-page community pinout sheet, 4-plug | Evo VII JDM 4-plug function list; shows SAS solenoid on pin 53, ACD/AYC on pin 87 | Low–medium (community) | Copyright MLR / author. |
| S6 | 2003 Lancer Evolution VIII Technical Information Manual (TIM) | https://www.tuningevo.club/manuals/EVO_8/2003_Lancer_Evo_8_Technical_Infomation_Manual.pdf | Factory new-model manual, 80 pages, January 2003, Mitsubishi Motors | Engine specs, 6MT W6MAA, ACD/AYC, immobiliser, control-system list, MDP/EGR deletion | **High** (factory), but JDM and 2003-specific | Mitsubishi Motors copyright. Research use only. |
| S7 | Lancer Evolution VIII MR Electrical Wiring Diagrams Supplement | https://sb952effb4c14a0dc.jimcontent.com/download/version/1704919146/module/9695237482/name/Mitsubishi%20Lancer%20Evolution%208%20Electrical.pdf | Factory electrical wiring supplement, 76 pages, February 2004, doc 1036K80 | **C-49 (35-pin), C-50 (28-pin), C-51 (30-pin) engine ECU connectors**; MR changes; wastegate solenoid 2; no CAN shown | **High** (factory), MR-specific | Mitsubishi Motors copyright. Research use only. |
| S8 | MMNA 2003 Lancer Evolution VIII Service Manual, Group 90 (Electrical Wiring) | https://evoscan.github.io/evoscan-manuals/manuals/Evo8-Manual-part1.zip → `GR00000400-90.pdf` | Factory USDM 2003 wiring diagrams, 128 pages | MFI circuit topology, fan control module, fuel pump relays/resistor, immobiliser; no separate igniter labelled in the extracted text | **High** (factory), **USDM-specific** | Mitsubishi/MMNA copyright. Research use only. |
| S9 | EvoEcu wiki — USDM VIII ECU Pinout | https://evoecu.logic.net/wiki/USDM_VIII_ECU_Pinout | Community wiki, last edited 2010, 4-plug pin tables | Full 4-plug pin functions; connector names C-121/C-119/C-117/C-115; 76 populated pins | Medium (community, detailed, consistent with S4/S11) | CC BY-SA 3.0. |
| S10 | EvoEcu wiki — Plugs, Pins, and Connectors | https://evoecu.logic.net/wiki/Plugs,_Pins,_and_Connectors | Community wiki, last edited 2011 | **Connector families and part numbers**: Tyco Multilock 040/070 for Evo VIII 4-plug; JAE MX7 for Evo IX / 3-plug; JAE MX5-E for ACD ECU | Medium (community, specific part numbers) | CC BY-SA 3.0. |
| S11 | Evo456 conversion to Evo78 ECU Pinout Diagrams (XLS) | https://evoscan.github.io/evoscan-manuals/manuals/Evo456-to-Evo78-ECU-Pinout.zip | Community conversion spreadsheet, dated 2007 | Evo 7 JDM 4-plug numbering 1-26/31-46/51-62/71-92; connector names 59/C121, 60/C119, 61/C117, 62/C115; injector resistance 2–3 Ω | Low–medium (community, conversion notes) | No licence stated. Author unknown. |
| S12 | Haltech — 4G63 Trigger Information | https://support.haltech.com/portal/en/kb/articles/4g63-trigger | Manufacturer technical article | Evo 4-on: 2x crank + cam Hall sensors, 12 V supply; Evo 8 MR–9 uses a different trigger configuration; Evo 9 cam pattern inverted | **High** (manufacturer, aftermarket) | Haltech copyright. Cited, not reproduced. |
| S13 | Haltech — 4G63 Engine | https://support.haltech.com/portal/en/kb/articles/4g63-engine | Manufacturer technical article | Waste-spark ignition; early external igniter vs later internal igniter; cam sensor on exhaust cam | **High** (manufacturer, aftermarket) | Haltech copyright. |
| S14 | MaxxECU — Mitsubishi EVO IV–VIII plugin | http://www.maxxecu.com/webhelp/wirings-pluginecu-mitsubishi_evo_4_to_8.html | Aftermarket ECU vendor documentation | Evo 7/8 share an adapter; pin 8 = A/C clutch relay, pin 32 = A/C fan high; PWM radiator fan; AYC/ACD not supported by adapter | Medium–high (vendor) | MaxxECU copyright. |
| S15 | Ecumaster — P&P adapter Mitsubishi Evolution 4 5 6 7 8 | https://www.ecumaster.com/wp/wp-content/uploads/2016/11/manual_evo4-8.pdf | Aftermarket ECU vendor manual, 12 pages | Evo 7/8 share adapter; MAF deleted for speed-density; factory IAT is in the MAF pre-turbo; additional intake-manifold IAT required | Medium–high (vendor) | Ecumaster copyright. |
| S16 | JustJDMImports — Mitsubishi EVO VIII CT9A ECU (MN132874) | https://justjdmimports.com/product/mitsubishi-evo-viii-ct9a-ecu/ | Used-parts listing with label and connector-side photographs | Candidate JDM Evo VIII 4-plug ECU **MN132874**; photo shows 4 connector bays | Low (seller listing) | Site content copyright. Photos used as evidence only. |
| S17 | DSMtuners — Evo 8 ECU part numbers | https://www.dsmtuners.com/threads/evo-8-ecu-part-numbers.336184 | Forum thread (search result; not read in full) | MN132874 = 2003 Evo 8 RS/GSR; MN191022 = 2004 GSR (USDM list) | Low (forum) | Forum terms; cited as a lead only. |
| S18 | Spoolinup — ECU connectors/pins | https://www.spoolinup.com/ECU-_p_72.html | Vendor page (search result; not read in full) | "Evo 8's with a 3 connector ecu use the Evo 9 ECU Pins" — confirms two Evo VIII ECU architectures | Medium (vendor) | Vendor copyright. |
| S19 | Mitsubishi M.U.T.-III Owner's Manual | https://mitsubishitechinfo.com/epacarb/ver_30_NAS_M.U.T.3MANUAL.pdf | Factory diagnostic tool manual (search result; **not downloaded in full**) | MUT-III supports K-line and CAN; 12-pin and 16-pin vehicle connectors | Medium–high (factory), but not fully read | Mitsubishi copyright. Not relied on for pin-level claims. |
| S20 | Evo VII TIM, Section 2 (Power Train) — ACD/AYC | https://evoscan.github.io/evoscan-manuals/wp-content/uploads/simple-file-list/Manuals/Evo7/TIM_1/G2.pdf | Factory new-model manual section, Evo VII | ACD/AYC architecture: 4WD-ECU, ABS-ECU wheel speeds, TPS, G sensors, steering sensor, engine idle information; no CAN mentioned | **High** (factory), Evo VII-specific; Evo VIII TIM says system is carried over with Super AYC and one terminal change | Mitsubishi copyright. Research use only. |
| S21 | Speeduino 4G63 decoder (local reference tree) | `/home/jlee/OpenTech-ECU/speeduino/src/decoders/4g63.cpp` | GPL-2.0 reference implementation retained in this repository | Assumes 4 crank edges per revolution at alternating 70°/110° spacing plus cam sync; useful as a hypothesis generator only | Low for OEM truth (implementation, not documentation) | GPL-2.0. |
| S22 | Autodoc / OEM parts listing — ignition coil MD362907 | https://www.autodoc.parts/car-parts/oem/md362907 ; https://mitsubishi.oempartsonline.com/oem-parts/mitsubishi-ignition-coil-md362907 | Aftermarket parts catalogue entries (search results; pages not read in full) | MD362907-type coil is 3-pin and listed for 1997–2007 Mitsubishi applications | Low–medium (catalogue fitment, not a factory drawing) | Vendor copyright. Used only to characterise the coil connector, not the drive type. |

## Sources that were *not* obtained

| Source | Why it matters | Status |
|---|---|---|
| JDM Evo VIII factory **wiring diagram** for the exact 2003 GSR | Would settle pin numbers and JDM market differences | Not found in accessible public sources. S7 is the **MR** supplement; S8 is **USDM**. |
| JDM Evo VIII factory **service manual** Group 13A/16 with terminal voltage tables | Would give pin-level electrical characteristics | Not obtained. The accessible MMNA Group 13A extract is general description only. |
| ECU internal schematic / immobiliser protocol | Needed to assess immobiliser integration | Not public. |
| ACD/AYC 4WD-ECU pinout for Evo VIII | Needed to assess stand-alone integration | Not obtained. S20 covers Evo VII architecture only. |
| Connector manufacturer drawing for the exact JDM harness | Needed for a production adapter | S10 gives candidate part numbers for the USDM/Evo IX families; JDM verification pending. |

## Reliability rules used in the audit

1. A factory document for a *different* market or *different* Evo generation is **not** evidence
   for this car's pin numbers. It is only a cross-check.
2. Two independent community sources that agree raise a pin to **Probable**, never to **Verified**.
3. A single community source is **Unverified**, regardless of how confident it sounds.
4. Photographs of a used part prove what that part looks like, not what the owner's car has.
5. Any claim that could cause an electrical or engine fault defaults to **Unknown** until measured.
