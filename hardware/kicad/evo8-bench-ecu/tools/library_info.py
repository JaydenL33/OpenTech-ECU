from pathlib import Path
from sexpr import *
import copy,sys
ROOT=Path(__file__).resolve().parents[4]
LIB=ROOT/'.toolchain/root/usr/share/kicad/symbols'
cache={}
def symbol(libid):
    lib,name=libid.split(':')
    if lib not in cache:
        cache[lib]={x[1]:x for x in children(parse((LIB/(lib+'.kicad_sym')).read_text()),'symbol')}
    src=copy.deepcopy(cache[lib][name])
    ext=child(src,'extends')
    if ext:
        base=symbol(lib+':'+ext[1]); old=base[1].split(':')[-1]
        base[1]=name
        props={p[1] for p in children(src,'property')}
        base=[x for x in base if not (isinstance(x,list) and x[0]=='property' and x[1] in props)]
        for sub in children(base,'symbol'):sub[1]=sub[1].replace(old+'_',name+'_',1)
        src=base+[x for x in src[2:] if not (isinstance(x,list) and x[0]=='extends')]
    src[1]=name
    return src
def pins(sym,unit=1):
    out=[]
    for sub in children(sym,'symbol'):
        u,b=map(int,sub[1].rsplit('_',2)[1:])
        if u not in (0,unit) or b not in (0,1):continue
        for p in children(sub,'pin'):
            a=child(p,'at')
            out.append(dict(num=child(p,'number')[1],name=child(p,'name')[1],x=float(a[1]),y=float(a[2]),angle=int(a[3]),type=p[1]))
    return out
if __name__=='__main__':
    for n in sys.argv[1:]:
        s=symbol(n)
        print(n,[(p['num'],p['name'],p['x'],p['y']) for p in pins(s)])
        print('FP',child(next(p for p in children(s,'property') if p[1]=='Footprint'),'at') if False else [(p[1],p[2]) for p in children(s,'property') if p[1] in ('Footprint','Datasheet')])
