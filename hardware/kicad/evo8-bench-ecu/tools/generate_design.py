#!/usr/bin/env python3
"""Generate the bench schematic and its explicit connectivity manifest.

All symbols and footprints originate in the KiCad 9 standard libraries.
This script is design source, not an electrical validation tool.
"""
from pathlib import Path
import copy,csv,json,uuid,collections,shutil
from sexpr import *
from library_info import symbol,pins,LIB,ROOT

OUT=Path(__file__).resolve().parents[1]
NAME='evo8-bench-ecu'
def uid(s):return str(uuid.uuid5(uuid.NAMESPACE_URL,'opentech-bench-revA/'+s))
ROOT_ID=uid('root')
SHEETS=[('power_protection','Power protection and hardware inhibit'),('mcu','STM32H743 MCU'),
 ('injector_drivers','Four bench injector sinks'),('ignition_outputs','Logic ignition to external igniters'),
 ('sensor_trigger','Analog and Hall trigger conditioning'),('can_debug','CAN USB and SWD'),
 ('aux_outputs','Auxiliary and relay sinks')]
parts=[]; instances=[]; notes=collections.defaultdict(list); counters={}
RFP='Resistor_SMD:R_0805_2012Metric'
CFP='Capacitor_SMD:C_0805_2012Metric'
SO14='Package_SO:SOIC-14_3.9x8.7mm_P1.27mm'
SO8='Package_SO:SOIC-8_3.9x4.9mm_P1.27mm'
def effects(sz=1.0,hide=False):
    return node('effects',node('font',node('size',sz,sz)),*([node('hide',Atom('yes'))] if hide else []))
def prop(k,v,x=0,y=0,hide=True):return node('property',k,v,node('at',x,y,0),effects(1,hide))
def newref(sheet,prefix):
    base=100*(1+[a for a,b in SHEETS].index(sheet)); key=(sheet,prefix)
    counters[key]=counters.get(key,base-1)+1
    return prefix+str(counters[key])
def add(sheet,libid,value,conn,fp=None,mpn='',maker='',note='',ref=None,unit=1,xy=None):
    sym=symbol(libid)
    if ref is None:
        prefix=next(p[2] for p in children(sym,'property') if p[1]=='Reference')
        ref=newref(sheet,prefix)
    conn={str(k):v for k,v in conn.items()}
    existing=next((p for p in parts if p['ref']==ref),None)
    if existing:
        existing['pins'].update(conn)
        part=existing
    else:
        if fp is None:fp=next(p[2] for p in children(sym,'property') if p[1]=='Footprint')
        assert fp or libid=='power:PWR_FLAG',(libid,value)
        part=dict(ref=ref,sheet=sheet,libid=libid,value=value,pins=conn,footprint=fp,mpn=mpn,
                  manufacturer=maker,note=note,uuid=uid(ref),dnp=False)
        parts.append(part)
    ps=pins(sym,unit)
    assert set(p['num'] for p in ps)<=set(conn), (ref,unit,set(p['num'] for p in ps)-set(conn))
    instances.append(dict(part=part,unit=unit,xy=xy,uuid=uid(ref+':'+str(unit))))
    return ref
def R(s,value,a,b,mpn=None,fp=RFP,note='',ref=None):
    codes={'0':'0R','10':'10R','22':'22R','100':'100R','1k':'1K','2.2k':'2K2','4.7k':'4K7',
           '10k':'10K','22k':'22K','33k':'33K','47k':'47K','68k':'68K','100k':'100K',
           '150k':'150K','220k':'220K','1M':'1M','45.3k':'45K3','120':'120R'}
    return add(s,'Device:R',value,{1:a,2:b},fp,mpn or ('RC0805FR-07'+codes[value]+'L'),'Yageo',note,ref)
def C(s,value,a,b='GND',note='',fp=CFP,mpn=None):
    choices={'100n':('GRM21BR71H104KA01L','Murata'),'1u':('GRM21BR71H105KA12L','Murata'),
             '2.2u':('GRM21BR71C225KA12L','Murata'),'10u':('GRM21BR61A106KE19L','Murata'),
             '10n':('GRM21BR71H103KA01L','Murata'),'1n':('GRM21B5C1H102JA01L','Murata'),
             '27p':('GRM21A5C1H270JA01D','Murata')}
    m,maker=choices[value]
    return add(s,'Device:C',value,{1:a,2:b},fp,mpn or m,maker,note)
def diode(s,value,k,a,fp='Diode_SMD:D_SMA',mpn=None,note=''):
    return add(s,'Device:D_Schottky' if value=='SS34' else 'Device:D_Zener',value,{1:k,2:a},fp,mpn or value,'Vishay' if value=='SS34' else 'Littelfuse',note)
def tp(s,net,description='',ground='GND'):
    return add(s,'Connector:TestPoint',net,{1:net},'TestPoint:TestPoint_THTPad_D2.0mm_Drill1.0mm','5010','Keystone',description or ('Measure '+net+' relative to '+ground))
def terminal(s,label,nets):
    n=len(nets)
    mpns={2:'1715721',3:'1715734',4:'1715747',5:'1715750',6:'1715763',8:'1715789'}
    return add(s,'Connector_Generic:Conn_01x%02d'%n,label,dict(enumerate(nets,1)),
      'TerminalBlock_Phoenix:TerminalBlock_Phoenix_MKDS-1,5-%d-5.08_1x%02d_P5.08mm_Horizontal'%(n,n),
      mpns[n],'Phoenix Contact','Board-local bench connector; pin numbers are NOT OEM cavities')
def fuse(s,a,b,value='500mA',mpn='0467.500NR'):
    return add(s,'Device:Fuse',value,{1:a,2:b},'Fuse:Fuse_1206_3216Metric',mpn,'Littelfuse','Fast fuse; verify interrupt rating and coordination')
def and4(s,channels,voltage='+5V',ahct=True):
    # 74LS08 is the standard quad-AND drawing/pinout. Actual fitted logic is AHCT or HC.
    ref=None
    for u,(a,b,y) in enumerate(channels,1):
        nums=[(1,2,3),(4,5,6),(9,10,8),(12,13,11)][u-1]
        ref=add(s,'74xx:74LS08','SN74AHCT08DR' if ahct else 'SN74HC08DR',dict(zip(nums,(a,b,y))),SO14,
           'SN74AHCT08DR' if ahct else 'SN74HC08DR','Texas Instruments',
           'Standard quad AND pinout; fit the specified logic family',ref,unit=u)
    add(s,'74xx:74LS08','SN74AHCT08DR' if ahct else 'SN74HC08DR',{14:voltage,7:'GND'},SO14,ref=ref,unit=5)
    C(s,'100n',voltage,note='Local bypass for '+ref)
    return ref

def make_power():
    s='power_protection'
    terminal(s,'J_PWR 12V BENCH ONLY',['VIN_BENCH','PGND'])
    fuse(s,'VIN_BENCH','VIN_FUSED','2A','0467002.NR')
    add(s,'Transistor_FET:IRF4905','IRF4905PBF',{1:'REV_GATE',2:'VIN_FUSED',3:'VIN_REVERSE_OK'},mpn='IRF4905PBF',maker='Infineon',note='Reverse protection: D=input S=protected; not ideal-diode backfeed blocking')
    R(s,'100k','REV_GATE','PGND')
    diode(s,'BZT52C12','VIN_REVERSE_OK','REV_GATE','Diode_SMD:D_SOD-123',note='12V gate-source clamp',mpn='BZT52C12-7-F')
    diode(s,'SMBJ12A','VIN_REVERSE_OK','PGND','Diode_SMD:D_SMB',note='Bench transient clamp only; no load-dump qualification')
    C(s,'100n','VIN_REVERSE_OK','PGND')
    add(s,'Power_Management:TPS2592xx','TPS259260DRCR',
        {1:'EFUSE_SLEW',2:'EFUSE_UVLO',3:'VIN_REVERSE_OK',4:'VIN_REVERSE_OK',5:'VIN_REVERSE_OK',
         6:'+12V_PROTECTED',7:'+12V_PROTECTED',8:'+12V_PROTECTED',9:None,10:'EFUSE_ILIM',11:'PGND'},
        mpn='TPS259260DRCR',maker='Texas Instruments',note='12V variant, latched thermal shutdown; nominal 2.1A limit')
    R(s,'45.3k','EFUSE_ILIM','PGND');C(s,'1n','EFUSE_SLEW','PGND')
    R(s,'47k','VIN_REVERSE_OK','EFUSE_UVLO');R(s,'10k','EFUSE_UVLO','PGND')
    C(s,'1u','+12V_PROTECTED','PGND')
    # Logic regulator input/output capacitors: electrolytic ESR is intentional for LM2596.
    add(s,'Device:C_Polarized','220u 35V',{1:'+12V_PROTECTED',2:'PGND'},
        'Capacitor_THT:CP_Radial_D8.0mm_P3.50mm','UHE1V221MPD','Nichicon','Input ripple/temperature review required')
    add(s,'Regulator_Switching:LM2596S-5','LM2596S-5.0/NOPB',{1:'+12V_PROTECTED',2:'BUCK_SW',3:'PGND',4:'+5V',5:'PGND'},
        mpn='LM2596S-5.0/NOPB',maker='Texas Instruments')
    diode(s,'SS34','BUCK_SW','PGND')
    add(s,'Device:L','33uH',{1:'BUCK_SW',2:'+5V'},'Inductor_SMD:L_Bourns_SRR1260','SRR1260-330M','Bourns','Check saturation and ripple at bench load')
    add(s,'Device:C_Polarized','330u 25V',{1:'+5V',2:'PGND'},
        'Capacitor_THT:CP_Radial_D8.0mm_P3.50mm','UHE1E331MPD','Nichicon','Buck stability/ESR requires scope validation')
    C(s,'100n','+5V');C(s,'10u','+5V')
    add(s,'Regulator_Linear:LD1117S33TR_SOT223','LD1117S33TR',{1:'GND',2:'+3V3',3:'+5V'},mpn='LD1117S33TR',maker='STMicroelectronics',note='Budget <=350mA; thermal copper still incomplete')
    C(s,'10u','+3V3');C(s,'100n','+3V3')
    R(s,'0','GND','PGND',mpn='RC2512JK-070RL',fp='Resistor_SMD:R_2512_6332Metric',note='Single logic/power return star; never route injector current through it')
    R(s,'0','AGND','GND',note='Analog star beside MCU VSSA; no actuator return')
    # Remote dual-pole normally-closed E-stop: the high-current pole is physically in series.
    R(s,'1k','+3V3','ESTOP_SEND')
    terminal(s,'J_ESTOP EXTERNAL NC CONTACTS',['+12V_PROTECTED','ACT_FEED','ESTOP_SEND','ESTOP_OK'])
    R(s,'100k','ESTOP_OK','GND')
    add(s,'Transistor_FET:IRF4905','IRF4905PBF',{1:'ACT_PWR_GATE',2:'+12V_ACT',3:'ACT_FEED'},
        mpn='IRF4905PBF',maker='Infineon',note='Actuator high-side switch; NC E-stop precedes source')
    R(s,'100k','ACT_PWR_GATE','ACT_FEED')
    diode(s,'BZT52C12','ACT_FEED','ACT_PWR_GATE','Diode_SMD:D_SOD-123',mpn='BZT52C12-7-F')
    R(s,'10k','ACT_PWR_GATE','ACT_PWR_COLLECTOR')
    add(s,'Transistor_BJT:MMBT3904','MMBT3904LT1G',{1:'ACT_PWR_BASE',2:'PGND',3:'ACT_PWR_COLLECTOR'},
        mpn='MMBT3904LT1G',maker='onsemi')
    R(s,'4.7k','ACT_EN','ACT_PWR_BASE');R(s,'100k','ACT_PWR_BASE','PGND')
    C(s,'1u','+12V_ACT','PGND');R(s,'1k','+12V_ACT','PGND',note='Actuator rail discharge; 0.16W at12.6V')
    # Wired open-drain reset from rail supervisor, watchdog and SWD/reset switch.
    add(s,'Power_Supervisor:TPS3808DBV','TPS3808G33DBVR',{1:'BROWN_RESET_N',2:'GND',3:'+3V3',4:None,5:'+3V3',6:'+3V3'},
        mpn='TPS3808G33DBVR',maker='Texas Instruments',note='3.07V threshold; CT open gives20ms nominal reset release')
    R(s,'10k','BROWN_RESET_N','+3V3')
    add(s,'Device:D_Schottky','BAT54WS',{1:'BROWN_RESET_N',2:'NRST'},'Diode_SMD:D_SOD-323','BAT54WS-7-F','Diodes Incorporated',
        'Reset isolation: anode NRST, cathode supervisor; verify low level after diode drop')
    C(s,'100n','+3V3')
    add(s,'Power_Supervisor:TPS3430WDRC','TPS3430WDRCR',
        {1:'+3V3',2:None,3:'GND',4:None,5:'GND',6:'GND',7:'WD_KICK',8:'NRST',9:None,10:'+3V3',11:'GND'},
        mpn='TPS3430WDRCR',maker='Texas Instruments',note='CWD/CRST open; SET0=SET1=0; service falling edge every35ms')
    C(s,'100n','+3V3');R(s,'100k','WD_KICK','GND');R(s,'10k','NRST','+3V3')
    # Reset/E-stop clear a hardware arm latch, independent of firmware.
    and4(s,[('NRST','ESTOP_OK','ARM_CLEAR_N'),('ARMED','MCU_ENABLE','ENABLE_REQ'),
            ('ENABLE_REQ','ARM_CLEAR_N','ACT_EN'),('GND','GND',None)],'+3V3',False)
    ref=add(s,'74xx:74HC74','SN74HC74DR',{1:'ARM_CLEAR_N',2:'+3V3',3:'ARM_CLOCK',4:'+3V3',5:'ARMED',6:None},SO14,'SN74HC74DR','Texas Instruments',unit=1)
    add(s,'74xx:74HC74','SN74HC74DR',{13:'GND',12:'GND',11:'GND',10:'+3V3',9:None,8:None},SO14,ref=ref,unit=2)
    add(s,'74xx:74HC74','SN74HC74DR',{14:'+3V3',7:'GND'},SO14,ref=ref,unit=3)
    C(s,'100n','+3V3')
    add(s,'Switch:SW_Push','ARM BENCH',{1:'+3V3',2:'ARM_CLOCK'},'Button_Switch_THT:SW_PUSH_6mm','B3F-1000','Omron','Press after valid watchdog service; contact bounce cannot defeat asynchronous clear')
    R(s,'100k','ARM_CLOCK','GND');R(s,'100k','MCU_ENABLE','GND');R(s,'100k','ACT_EN','GND')
    for net in ['VIN_BENCH','+12V_PROTECTED','+12V_ACT','+5V','+3V3','NRST','ARMED','ACT_EN','PGND','GND','ESTOP_OK']:
        tp(s,net)
    for net in ['VIN_BENCH','VIN_REVERSE_OK','PGND','AGND','+5V','+12V_ACT','ACT_FEED']:
        add(s,'power:PWR_FLAG','PWR_FLAG',{1:net},'',ref='#FLG'+str(100+len(parts)))
    notes[s]=['10.0-12.6V regulated bench source ONLY. Set external current limit before applying power.',
      'J_ESTOP pins1-2: external NC power contact; pins3-4: independent NC logic contact. No jumpers fitted.',
      'ACT_EN = ARMED AND MCU_ENABLE AND NRST AND ESTOP_OK. Reset/E-stop clears arm latch.',
      'TPS3430: falling edges every35ms; guaranteed service interval25.9-46.8ms; reset170-230ms.',
      'PWR_FLAG marks intended source nets for ERC; it does not verify voltage, current, or safe operation.']

MCU_MAP={
 'PA0':('CRANK_DIG','TIM2_CH1 AF1 input capture'), 'PA1':('CAM_DIG','TIM2_CH2 AF1 input capture'),
 'PA2':('UART_TX','USART2_TX AF7'), 'PA3':('UART_RX','USART2_RX AF7'),
 'PA4':('ADC_AN1','ADC12_INP18'), 'PA5':('ADC_AN2','ADC12_INP19'),
 'PA6':('ADC_AN3','ADC12_INP3'), 'PA7':('ADC_AN4','ADC12_INP7'),
 'PA9':('USB_VBUS_SENSE','GPIO VBUS sense; see USB firmware requirements'),
 'PA11':('USB_DM','USB_OTG_FS_DM AF10'), 'PA12':('USB_DP','USB_OTG_FS_DP AF10'),
 'PA13':('SWDIO','SWD AF0'), 'PA14':('SWCLK','SWD AF0'), 'PB3':('SWO','TRACE_SWO AF0'),
 'PB8':('CAN_RX','FDCAN1_RX AF9, classic CAN only'), 'PB9':('CAN_TX','FDCAN1_TX AF9'),
 'PE9':('INJ1_CMD','TIM1_CH1 AF1'), 'PE11':('INJ2_CMD','TIM1_CH2 AF1'),
 'PE13':('INJ3_CMD','TIM1_CH3 AF1'), 'PE14':('INJ4_CMD','TIM1_CH4 AF1'),
 'PD12':('IGN1_CMD','TIM4_CH1 AF2'), 'PD13':('IGN2_CMD','TIM4_CH2 AF2'),
 'PD14':('IGN3_CMD','TIM4_CH3 AF2'), 'PD15':('IGN4_CMD','TIM4_CH4 AF2'),
 'PC6':('AUX1_CMD','TIM3_CH1 AF2'), 'PC7':('AUX2_CMD','TIM3_CH2 AF2'),
 'PC8':('AUX3_CMD','TIM3_CH3 AF2'), 'PC9':('AUX4_CMD','TIM3_CH4 AF2'),
 'PB0':('RELAY_CMD','GPIO push-pull active high'), 'PB1':('MCU_ENABLE','GPIO active high; firmware must default low'),
 'PB2':('WD_KICK','GPIO software falling edge heartbeat; do not use autonomous PWM'),
 'PC0':('ADC_IS1','ADC123_INP10'), 'PC1':('ADC_IS2','ADC123_INP11'),
 'PC4':('ADC_IS3','ADC12_INP4'), 'PC5':('ADC_IS4','ADC12_INP8'),
 'PC13':('ARMED','GPIO latch state input'), 'PD2':('ESTOP_OK','GPIO E-stop state input')}
def make_mcu():
    s='mcu';sym=symbol('MCU_ST_STM32H7:STM32H743VITx');conn={}
    fixed={'NRST':'NRST','BOOT0':'BOOT0','VREF+':'VDDA','PH0':'HSE_IN','PH1':'HSE_OUT',
           'VBAT':'+3V3','VDD':'+3V3','VSS':'GND','VSSA':'AGND','VDDA':'VDDA'}
    for p in pins(sym):
        conn[p['num']]=MCU_MAP[p['name']][0] if p['name'] in MCU_MAP else fixed.get(p['name'])
        if p['name']=='VCAP':conn[p['num']]='VCAP'+p['num']
    add(s,'MCU_ST_STM32H7:STM32H743VITx','STM32H743VIT6',conn,mpn='STM32H743VIT6',maker='STMicroelectronics',ref='U200',xy=(100.33,130.81))
    for pin in ['11','27','50','75','100']:
        C(s,'100n','+3V3',note='Place beside U200 VDD pin'+pin)
    C(s,'100n','+3V3',note='VBAT local bypass');C(s,'10u','+3V3',note='MCU bulk')
    C(s,'2.2u','VCAP48',note='U200 pin48 core LDO output; no other load')
    C(s,'2.2u','VCAP73',note='U200 pin73 core LDO output; no other load')
    R(s,'10','+3V3','VDDA',note='Analog rail RC isolator; verify drop at ADC load')
    C(s,'1u','VDDA','AGND');C(s,'100n','VDDA','AGND')
    R(s,'100k','BOOT0','GND')
    add(s,'Jumper:Jumper_2_Open','BOOT ROM',{1:'+3V3',2:'BOOT0'},'Connector_PinHeader_2.54mm:PinHeader_1x02_P2.54mm_Vertical','61300211121','Wurth Elektronik','Normally open; bootloader is not authenticated')
    add(s,'Switch:SW_Push','RESET',{1:'NRST',2:'GND'},'Button_Switch_THT:SW_PUSH_6mm','B3F-1000','Omron')
    C(s,'10n','NRST')
    add(s,'Device:Crystal_GND24','8MHz CL18pF',{1:'HSE_IN',2:'GND',3:'HSE_OUT',4:'GND'},
        'Crystal:Crystal_SMD_3225-4Pin_3.2x2.5mm','ABM8-8.000MHZ-B2-T','Abracon','Load caps27p assume4.5pF stray; oscillator gain/startup untested')
    C(s,'27p','HSE_IN');C(s,'27p','HSE_OUT')
    for net in ['VCAP48','VCAP73','VDDA','BOOT0','WD_KICK']:tp(s,net)
    add(s,'power:PWR_FLAG','PWR_FLAG',{1:'VDDA'},'',ref='#FLG200')
    for i in range(4):add(s,'Mechanical:MountingHole','M3 NPTH',{},'MountingHole:MountingHole_3.2mm_M3',note='Nonconductive mounting hardware',ref='H'+str(i+1))
    notes[s]=['MCU pinout is from the standard KiCad STM32H743VITx LQFP100 symbol; fitted ordering code STM32H743VIT6.',
      'VCAP48 and VCAP73 each have a2.2uF ceramic to GND. Firmware must select internal core LDO mode.',
      'No application firmware is delivered. Pin-map AF assignments require board firmware and scope proof.',
      'Start with all outputs low, service the watchdog, then deliberately assert MCU_ENABLE. Debug halt must stop heartbeat.']

def make_injectors():
    s='injector_drivers'
    and4(s,[(f'INJ{i}_CMD','ACT_EN',f'INJ{i}_DRIVE') for i in range(1,5)])
    for i in range(1,5):
        n=f'INJ{i}';R(s,'100k',n+'_CMD','GND');R(s,'100',n+'_DRIVE',n+'_GATE')
        R(s,'10k',n+'_GATE',n+'_SHUNT')
        add(s,'Transistor_FET:IRLZ44N','IRLZ44NPBF',{1:n+'_GATE',2:n+'_OUT',3:n+'_SHUNT'},
            mpn='IRLZ44NPBF',maker='Infineon',note='Bench saturated sink only; no peak/hold or fast overcurrent comparator')
        R(s,'0.22',n+'_SHUNT','PGND','WSL2512R2200FEA','Resistor_SMD:R_2512_6332Metric','Kelvin test pads; 0.22ohm1%1W bench shunt')
        fuse(s,'+12V_ACT',n+'_SUPPLY')
        diode(s,'SS34',n+'_SUPPLY',n+'_OUT',note='Slow recirculating flyback; not calibrated injector closing')
        diode(s,'SMBJ24A',n+'_OUT','PGND','Diode_SMD:D_SMB',note='Secondary drain overshoot clamp; scope validation required')
        R(s,'1k',n+'_SHUNT','ADC_IS'+str(i));C(s,'10n','ADC_IS'+str(i))
        terminal(s,n+' DUMMY LOAD ONLY',[n+'_SUPPLY',n+'_OUT'])
        tp(s,n+'_OUT',f'Drain transient, PGND reference; do not use a logic probe','PGND')
        tp(s,n+'_SHUNT',f'Kelvin high: I = V/0.22ohm; <=55mV at250mA','PGND')
        tp(s,'PGND',n+' Kelvin low for shunt measurement','PGND')
        tp(s,n+'_GATE','Gate referenced to local source/shunt')
    notes[s]=['Four low-side saturated BENCH channels. <=250mA/channel; <=1A combined injector current.',
      'Use47ohm >=10W resistor loads first. Optional enclosed dummy inductors <=10mH, <=250mA only after review.',
      'SS34 recirculation is deliberately slow; flyback energy and closing time are NOT suitable engine evidence.',
      '0.22ohm shunts have filtered ADC taps and paired Kelvin test pads. No fast short-circuit shutdown.',
      'Do not connect low-impedance Evo injectors, fuel, real coils or an OEM harness.']

def make_ignition():
    s='ignition_outputs'
    and4(s,[(f'IGN{i}_CMD','ACT_EN',f'IGN{i}_BUF') for i in range(1,5)])
    for i in range(1,5):
        n=f'IGN{i}';R(s,'100k',n+'_CMD','GND');R(s,'100',n+'_BUF',n+'_LOGIC');R(s,'10k',n+'_LOGIC','GND')
        diode(s,'BZT52C5V1',n+'_LOGIC','GND','Diode_SMD:D_SOD-123',mpn='BZT52C5V1-7-F',note='Logic connector clamp, not a coil flyback clamp')
        terminal(s,n+' EXTERNAL IGNITER',[n+'_LOGIC','GND']);tp(s,n+'_LOGIC','0/5V active-high external-igniter command; <=2mA load')
    terminal(s,'IGNITER BENCH POWER',['+12V_ACT','PGND'])
    notes[s]=['0/5V positive logic, <=2mA per output, to EXTERNAL igniters only. NEVER connect a coil primary.',
      'SN74AHCT08 accepts3.3V TTL inputs and forces every ignition command low when ACT_EN=0.',
      'External igniter supply must come from switched+12V_ACT. No separate always-on supply.',
      'Igniter polarity, dwell and Evo compatibility are unverified. Start with logic probes or LEDs only.']

def make_sensors():
    s='sensor_trigger'
    R(s,'2.2k','+5V','CLAMP_3V0')
    add(s,'Reference_Voltage:LM4040DBZ-3','LM4040D30IDBZR',{1:'CLAMP_3V0',2:'AGND',3:None},
        mpn='LM4040D30IDBZR',maker='Texas Instruments',note='3.0V shunt absorbs positive fault clamp current; not ADC VREF')
    C(s,'100n','CLAMP_3V0','AGND')
    for i in range(1,5):
        n=f'AN{i}';terminal(s,n+' 0-5V BENCH',[n+'_INPUT','AGND'])
        R(s,'22k',n+'_INPUT',n+'_DIV');R(s,'33k',n+'_DIV','AGND')
        add(s,'Diode:BAT54S','BAT54S-7-F',{1:'AGND',2:'CLAMP_3V0',3:n+'_DIV'},mpn='BAT54S-7-F',maker='Diodes Incorporated',note='Lower clamp to AGND, upper to absorbing3.0V shunt')
        R(s,'1k',n+'_DIV','ADC_'+n);C(s,'10n','ADC_'+n,'AGND');tp(s,'ADC_'+n, '0-3.0V nominal for0-5V input; source impedance14.2kohm')
    # Noninverting Schmitt comparator; reference ~1V, 22k:1M positive feedback.
    R(s,'33k','+5V','TRIG_REF');R(s,'10k','TRIG_REF','AGND');C(s,'100n','TRIG_REF','AGND')
    cmpref=None
    for i,n in enumerate(['CRANK','CAM'],1):
        terminal(s,n+' 0-5V HALL ONLY',[n+'_INPUT','AGND'])
        R(s,'22k',n+'_INPUT',n+'_DIV');R(s,'33k',n+'_DIV','AGND')
        add(s,'Diode:BAT54S','BAT54S-7-F',{1:'AGND',2:'CLAMP_3V0',3:n+'_DIV'},mpn='BAT54S-7-F',maker='Diodes Incorporated')
        C(s,'1n',n+'_DIV','AGND');R(s,'1M',n+'_DIG',n+'_DIV')
        # LM2903: plus3/5, minus2/6, out1/7.
        nums=(3,2,1) if i==1 else (5,6,7)
        cmpref=add(s,'Comparator:LM2903','LM2903BDR',dict(zip(nums,(n+'_DIV','TRIG_REF',n+'_DIG'))),SO8,
            'LM2903BDR','Texas Instruments','Open-collector output with3.3V pullup; single-ended Hall/function generator only',cmpref,unit=i)
        R(s,'4.7k',n+'_DIG','+3V3');tp(s,n+'_DIG','3.3V comparator-conditioned timer capture')
    add(s,'Comparator:LM2903','LM2903BDR',{8:'+5V',4:'AGND'},SO8,ref=cmpref,unit=3)
    C(s,'100n','+5V','AGND');tp(s,'CLAMP_3V0');tp(s,'TRIG_REF')
    terminal(s,'SENSOR 5V BENCH',['+5V','AGND'])
    notes[s]=['Analog divider22k/33k: gain0.600; ADC0-3.0V for input0-5V. ADC source14.2kohm requires long sampling time.',
      'BAT54S clamps to AGND and dedicated absorbing LM4040 3.0V shunt. No phantom powering of MCU rail.',
      'Continuous accidental +/-12.6V inputs are resistor-limited design cases, NOT tested survival ratings.',
      'Trigger inputs: single-ended push-pull0-5V Hall/function generator only. No onboard Hall pullup, no VR interface.',
      'Positive comparator feedback1M gives approx72mV input hysteresis; input RC approx13us; measure edge delay.',
      'Signal sources and USB must be disconnected while board is unpowered. Sensor rail shares logic buck current limit.']

def make_comms():
    s='can_debug'
    add(s,'Interface_CAN_LIN:TCAN332','TCAN332DR',{1:'CAN_TX',2:'GND',3:'+3V3',4:'CAN_RX',5:None,6:'CAN_L',7:'CAN_H',8:None},
        SO8,'TCAN332DR','Texas Instruments','3.3V classic CAN; not an Evo OEM bus interface')
    C(s,'100n','+3V3');R(s,'10k','CAN_TX','+3V3',note='Recessive TX during MCU reset')
    add(s,'Power_Protection:NUP2105L','NUP2105LT1G',{1:'CAN_H',2:'CAN_L',3:'GND'},mpn='NUP2105LT1G',maker='onsemi')
    R(s,'120','CAN_H','CAN_TERM');add(s,'Jumper:Jumper_2_Open','CAN TERMINATION',{1:'CAN_TERM',2:'CAN_L'},
        'Connector_PinHeader_2.54mm:PinHeader_1x02_P2.54mm_Vertical','61300211121','Wurth Elektronik','Fit shunt only at bus endpoint')
    terminal(s,'CAN BENCH',['CAN_H','CAN_L','GND'])
    add(s,'Connector:USB_B_Micro','USB DEVICE SELF POWERED',{1:'USB_VBUS',2:'USB_CONN_DM',3:'USB_CONN_DP',4:None,5:'GND',6:'GND'},
        'Connector_USB:USB_Micro-B_Amphenol_10118194-0001LF_Horizontal','10118194-0001LF','Amphenol','VBUS does not power board; do not attach USB until board powered')
    add(s,'Power_Protection:USBLC6-2SC6','USBLC6-2SC6',{1:'USB_CONN_DM',6:'USB_CONN_DM',3:'USB_CONN_DP',4:'USB_CONN_DP',5:'USB_VBUS',2:'GND'},mpn='USBLC6-2SC6',maker='STMicroelectronics')
    R(s,'22','USB_CONN_DM','USB_DM');R(s,'22','USB_CONN_DP','USB_DP')
    R(s,'68k','USB_VBUS','USB_VBUS_SENSE');R(s,'100k','USB_VBUS_SENSE','GND')
    C(s,'100n','USB_VBUS');tp(s,'USB_VBUS_SENSE')
    add(s,'Connector_Generic:Conn_02x05_Odd_Even','CORTEX SWD 10PIN',
        {1:'+3V3',2:'SWDIO',3:'GND',4:'SWCLK',5:'GND',6:'SWO',7:None,8:None,9:'GND',10:'NRST'},
        'Connector_PinHeader_1.27mm:PinHeader_2x05_P1.27mm_Vertical','20021121-00010T4LF','Amphenol','Pin1 target VREF output, not power input; pin7 key, pin8 NC')
    R(s,'100k','SWCLK','GND');R(s,'100k','SWDIO','+3V3')
    terminal(s,'UART 3V3',['UART_TX','UART_RX','GND']);R(s,'100k','UART_RX','+3V3')
    for net in ['CAN_TX','CAN_RX','SWDIO','SWCLK']:tp(s,net)
    notes[s]=['CAN is3.3V classic CAN on TCAN332.120ohm termination is switchable and normally open.',
      'USB is self-powered FS device only. No VBUS-to+5V connection. Enable pullup only with valid VBUS and rails.',
      'USB/CAN ESD routing and differential pair impedance/matching remain incomplete on the placed PCB.',
      'SWD target3.3V is a voltage reference; debug probe must not feed target power. Headers are not isolated.',
      'Board GND, USB host and bench instruments share a common reference; no isolation is provided.']

def make_aux():
    s='aux_outputs'
    and4(s,[(f'AUX{i}_CMD','ACT_EN',f'AUX{i}_DRIVE') for i in range(1,5)])
    and4(s,[('RELAY_CMD','ACT_EN','RELAY_DRIVE'),('GND','GND',None),('GND','GND',None),('GND','GND',None)])
    for n in ['AUX1','AUX2','AUX3','AUX4','RELAY']:
        R(s,'100k',n+'_CMD','GND');R(s,'100',n+'_DRIVE',n+'_GATE');R(s,'10k',n+'_GATE','PGND')
        add(s,'Transistor_FET:IRLZ44N','IRLZ44NPBF',{1:n+'_GATE',2:n+'_OUT',3:'PGND'},mpn='IRLZ44NPBF',maker='Infineon',note='Bench sink <=100mA; no per-channel electronic current protection')
        fuse(s,'+12V_ACT',n+'_SUPPLY');diode(s,'SS34',n+'_SUPPLY',n+'_OUT')
        terminal(s,n+' BENCH LOAD',[n+'_SUPPLY',n+'_OUT']);tp(s,n+'_OUT',ground='PGND')
    notes[s]=['Four AUX plus one RELAY low-side sinks.<=100mA/channel, <=500mA combined. Active-high commands.',
      'Every channel has an AHCT AND gate, gate-source10k pulldown,500mA branch fuse and SS34 flyback diode.',
      'Use resistor loads first. The RELAY connector drives an external low-current12V relay coil, not contact/load power.',
      'No onboard relay contacts, fuel-pump/fan motors, stepper bridge, knock circuit or wideband heater driver.']

def write_sheets():
    # Preserve only used standard symbol definitions and their inheritance bases, under original library IDs.
    used=collections.defaultdict(set)
    for p in parts:used[p['libid'].split(':')[0]].add(p['libid'].split(':')[1])
    local=OUT/'libraries/symbols'
    for lib,names in used.items():
        src=parse((LIB/(lib+'.kicad_sym')).read_text());defs={x[1]:x for x in children(src,'symbol')}
        pending=list(names)
        while pending:
            ext=child(defs[pending.pop()],'extends')
            if ext and ext[1] not in names:names.add(ext[1]);pending.append(ext[1])
        # Flatten standard inheritance so library load order is deterministic and matches embedded symbols.
        (local/(lib+'.kicad_sym')).write_text(dump(node('kicad_symbol_lib',node('version',20241209),node('generator','kicad_symbol_editor'),*[symbol(lib+':'+n) for n in sorted(names)]))+'\n')
    (OUT/'sym-lib-table').write_text('(sym_lib_table\n'+''.join(f' (lib (name "{l}")(type "KiCad")(uri "${{KIPRJMOD}}/libraries/symbols/{l}.kicad_sym")(options "")(descr "KiCad 9 standard subset"))\n' for l in sorted(used))+')\n')
    for idx,(s,title) in enumerate(SHEETS,2):
        sheetid=uid(s);its=[i for i in instances if i['part']['sheet']==s]
        libids=sorted({i['part']['libid'] for i in its})
        embedded=[]
        for libid in libids:
            sym=symbol(libid);sym[1]=libid;embedded.append(sym)
        sch=node('kicad_sch',node('version',20250114),node('generator','eeschema'),node('uuid',sheetid),node('paper','A1'),
           node('title_block',node('title','OpenTech-ECU / '+title),node('date','2026-11-12'),node('rev','A-UNREVIEWED'),
             node('company','BENCH PROTOTYPE - NOT AUTOMOTIVE QUALIFIED'),node('comment',1,'NOT a factory-Evo replacement. NO vehicle or real fuel/spark connection.')),
           node('lib_symbols',*embedded))
        for j,t in enumerate(notes[s]):
            e=effects(1.2);e.append(node('justify',Atom('left')))
            sch.append(node('text',t,node('at',20,17+j*4,0),e,node('uuid',uid(s+'note'+str(j)))))
        # Label-oriented readable component matrix; pins have actual wires and global nets.
        # MCU is kept intact at the left; its support network occupies the right.
        auto=0
        for it in its:
            p=it['part'];sym=symbol(p['libid']);unit=it['unit'];ps=pins(sym,unit)
            if it['xy']:x,y=it['xy']
            else:
                if s=='mcu':x=241.30+(auto%7)*73.66;y=71.12+(auto//7)*43.18
                else:x=50.80+(auto%10)*73.66;y=76.20+(auto//10)*43.18
                auto+=1
            # All coordinates on1.27mm grid; symbol local Y runs up, sheet Y runs down.
            inst=node('symbol',node('lib_id',p['libid']),node('at',x,y,0),node('unit',unit),
              node('in_bom',Atom('no' if p['ref'].startswith('#') else 'yes')),
              node('on_board',Atom('no' if p['ref'].startswith('#') else 'yes')),
              node('dnp',Atom('no')),node('uuid',it['uuid']),
              prop('Reference',p['ref'],x,y-12.7,False),prop('Value',p['value'],x,y-10.16,False),
              prop('Footprint',p['footprint']),prop('Datasheet',next((z[2] for z in children(sym,'property') if z[1]=='Datasheet'),'')),
              prop('MPN',p['mpn']),prop('Manufacturer',p['manufacturer']),prop('Design_Note',p['note']),
              node('instances',node('project',NAME,node('path','/'+ROOT_ID+'/'+sheetid,node('reference',p['ref']),node('unit',unit)))))
            for pin in ps:inst.append(node('pin',pin['num'],node('uuid',uid(p['ref']+'pin'+pin['num']))))
            sch.append(inst)
            done=set()
            for pin in ps:
                px=round(x+pin['x'],4);py=round(y-pin['y'],4);net=p['pins'][pin['num']]
                if (px,py) in done:continue
                done.add((px,py))
                if net is None:
                    sch.append(node('no_connect',node('at',px,py),node('uuid',uid(p['ref']+'nc'+pin['num']))));continue
                angle=pin['angle'];dx,dy={0:(-5.08,0),180:(5.08,0),90:(0,5.08),270:(0,-5.08)}[angle]
                ex,ey=round(px+dx,4),round(py+dy,4)
                sch.append(node('wire',node('pts',node('xy',px,py),node('xy',ex,ey)),node('stroke',node('width',0),node('type',Atom('default'))),node('uuid',uid(it['uuid']+'w'+pin['num']))))
                # Global labels are sufficient to connect hierarchy; root explicitly documents this convention.
                labelangle={0:0,180:180,90:270,270:90}[angle]
                sch.append(node('global_label',net,node('shape',Atom('passive')),node('at',ex,ey,labelangle),
                   effects(0.9),node('uuid',uid(it['uuid']+'l'+pin['num'])),prop('Intersheetrefs','${INTERSHEET_REFS}',ex,ey)))
        (OUT/(s+'.kicad_sch')).write_text(dump(sch)+'\n')
    root=node('kicad_sch',node('version',20250114),node('generator','eeschema'),node('uuid',ROOT_ID),node('paper','A3'),
      node('title_block',node('title','OpenTech-ECU / Evo VIII future development / BENCH ONLY'),node('date','2026-11-12'),node('rev','A-UNREVIEWED'),
       node('company','UNREVIEWED BENCH PROTOTYPE - NOT AUTOMOTIVE QUALIFIED'),node('comment',1,'NOT a vehicle plug-in. No OEM connector or OEM cavity assignments.')),
      node('lib_symbols'))
    for j,t in enumerate(['UNREVIEWED BENCH PROTOTYPE. Placed PCB is PARTIALLY ROUTED; DO NOT FABRICATE.',
       '12V regulated supply; dummy loads only. No firmware, no vehicle, no fuel or real ignition coils.',
       'Global net labels connect the seven hierarchical sheets. References and connector pins are board-local.',
       'Read README.md, docs/DESIGN_REVIEW.md and reports before any assembly or powered work.']):
        e=effects(1.5);e.append(node('justify',Atom('left')))
        root.append(node('text',t,node('at',25,20+j*6,0),e,node('uuid',uid('rootnote'+str(j)))))
    for i,(s,title) in enumerate(SHEETS):
        x=30+(i%3)*125;y=65+(i//3)*60
        root.append(node('sheet',node('at',x,y),node('size',110,35),node('stroke',node('width',0.254),node('type',Atom('default'))),
          node('fill',node('color',0,0,0,0)),node('uuid',uid(s)),
          prop('Sheetname',title,x,y-2,False),prop('Sheetfile',s+'.kicad_sch',x,y+37,False),
          node('instances',node('project',NAME,node('path','/'+ROOT_ID,node('page',str(i+2)))))))
    root.append(node('sheet_instances',node('path','/',node('page','1'))))
    (OUT/(NAME+'.kicad_sch')).write_text(dump(root)+'\n')

def write_manifest():
    outparts=[p for p in parts if not p['ref'].startswith('#')]
    (OUT/'design.json').write_text(json.dumps(dict(project=NAME,root_uuid=ROOT_ID,sheets=SHEETS,parts=outparts),indent=2)+'\n')
    with (OUT/'BOM.csv').open('w',newline='') as f:
        w=csv.writer(f);w.writerow(['Reference','Quantity','Value','Manufacturer','MPN','Footprint','Sheet','Assembly','Design note'])
        for p in outparts:w.writerow([p['ref'],1,p['value'],p['manufacturer'],p['mpn'],p['footprint'],p['sheet'],'FIT' if p['mpn'] else 'MECHANICAL / PCB FEATURE',p['note']])
    with (OUT/'pin_map.csv').open('w',newline='') as f:
        w=csv.writer(f);w.writerow(['Reference','Pin','Pin name','Net','Function / constraint','OEM assignment'])
        for p in outparts:
            if p['ref']=='U200' or p['ref'].startswith('J'):
                sym=symbol(p['libid']);names={a['num']:a['name'] for a in pins(sym)}
                for num,net in sorted(p['pins'].items(),key=lambda x:int(x[0])):
                    pn=names.get(num,'');function=MCU_MAP.get(pn,('',p['note']))[1]
                    w.writerow([p['ref'],num,pn,net or 'NC',function,'NONE - BENCH ONLY'])
    with (OUT/'test_points.csv').open('w',newline='') as f:
        w=csv.writer(f);w.writerow(['Reference','Net','Purpose / measurement'])
        for p in outparts:
            if p['ref'].startswith('TP'):w.writerow([p['ref'],p['pins']['1'],p['note']])
    # Project config with explicit fabrication defaults. No ERC/DRC exclusions.
    pro={'meta':{'filename':NAME+'.kicad_pro','version':1},'board':{'design_settings':{
      'defaults':{'board_outline_line_width':0.05,'copper_line_width':0.25,'copper_text_size_h':1.5,'copper_text_size_v':1.5},
      'rules':{'min_clearance':0.2,'min_track_width':0.15,'min_via_diameter':0.6,'min_through_hole_diameter':0.3,'min_hole_clearance':0.25,'min_copper_edge_clearance':0.5},
      'track_widths':[0,0.2,0.25,0.5,1.0,1.5],'via_dimensions':[{'diameter':0.6,'drill':0.3}],
      'drc_exclusions':[], 'meta':{'version':2}}},
      'net_settings':{'meta':{'version':4},'classes':[{'name':'Default','clearance':0.2,'track_width':0.25,'via_diameter':0.6,'via_drill':0.3,'diff_pair_width':0.2,'diff_pair_gap':0.2,'diff_pair_via_gap':0.25,
        'bus_width':12,'line_style':0,'microvia_diameter':0.3,'microvia_drill':0.1,'wire_width':6,'schematic_color':[0,0,0,0],'pcb_color':[0,0,0,0]}],
       'netclass_assignments':{},'netclass_patterns':[]},
      'schematic':{'annotate_start_num':0,'legacy_lib_dir':'','legacy_lib_list':[],'meta':{'version':1}},'text_variables':{'PROTOTYPE_STATUS':'UNREVIEWED BENCH ONLY'}}
    (OUT/(NAME+'.kicad_pro')).write_text(json.dumps(pro,indent=2)+'\n')

if __name__=='__main__':
    make_power();make_mcu();make_injectors();make_ignition();make_sensors();make_comms();make_aux()
    # Verify every logical symbol pin is represented, and all fitted devices have a source footprint.
    for p in parts:
        if p['footprint']:
            l,n=p['footprint'].split(':');assert (ROOT/'.toolchain/root/usr/share/kicad/footprints'/(l+'.pretty')/(n+'.kicad_mod')).exists(),p['footprint']
    write_sheets();write_manifest()
    print('Generated',len(parts),'symbols;',len([p for p in parts if not p['ref'].startswith('#')]),'board components;',len(instances),'symbol units')
