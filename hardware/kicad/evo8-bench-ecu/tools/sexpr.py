"""Small S-expression reader/writer for KiCad source generation; no dependencies."""
import re
class Atom(str):
    pass
def parse(s):
    tokens = re.findall(r'"(?:\\.|[^"\\])*"|[()]|[^\s()]+', s)
    stack=[]; root=None
    for t in tokens:
        if t=='(':
            v=[]
            if stack: stack[-1].append(v)
            else: root=v
            stack.append(v)
        elif t==')': stack.pop()
        else:
            if t.startswith('"'):
                v=t[1:-1].replace('\\"','"').replace('\\n','\n').replace('\\\\','\\')
            else: v=Atom(t)
            stack[-1].append(v)
    assert not stack
    return root
def dump(v, level=0):
    if isinstance(v,list):
        if not any(isinstance(x,list) for x in v):
            return '('+' '.join(dump(x) for x in v)+')'
        out='('+dump(v[0])
        for x in v[1:]:
            out+= ('\n'+'  '*(level+1) if isinstance(x,list) else ' ')+dump(x,level+1)
        return out+')'
    if isinstance(v,Atom): return str(v)
    if isinstance(v,(int,float)): return str(v)
    return '"'+str(v).replace('\\','\\\\').replace('"','\\"').replace('\n','\\n')+'"'
def node(tag,*args):return [Atom(tag),*args]
def children(v,tag): return [x for x in v if isinstance(x,list) and x and x[0]==tag]
def child(v,tag,default=None):return next(iter(children(v,tag)),default)
