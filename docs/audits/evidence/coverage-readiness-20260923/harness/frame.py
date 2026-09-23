import json,re
import numpy as np
def canvas(text):
    matches=list(re.finditer(r'SMART_ALPHA id=0\nworld=(\d+)x(\d+) depth=32 origin=Point \{ h: (-?\d+), v: (-?\d+) \}\nalpha_words=(\[[^\n]*\])',text.replace('\r\n','\n')))
    if len(matches)!=1:raise RuntimeError('Expected one native frame, got '+str(len(matches)))
    m=matches[0];w,h,x,y=map(int,m.groups()[:4]);words=np.array(json.loads(m[5]),dtype=np.uint32).reshape(h,w)
    result=np.zeros((600,800),dtype=np.uint32);l,t,r,b=max(x,0),max(y,0),min(x+w,800),min(y+h,600)
    result[t:b,l:r]=words[t-y:b-y,l-x:r-x]
    return result
