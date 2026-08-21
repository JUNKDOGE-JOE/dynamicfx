"""Repeat purge -> interrupted preview -> cache sample N times; report the dip count
each round. A deterministic fix yields 0 every round regardless of interrupt timing."""
import os, time, glob
import numpy as np
from PIL import Image
import aemcp
SP = os.path.dirname(os.path.abspath(__file__))
def js(code, t=90000):
    r = aemcp.exec_js(code, timeout_ms=t)
    if not r.get("ok"): raise SystemExit("exec failed: "+str(r)[:400])
    return r.get("result")
def luma(path):
    a=np.asarray(Image.open(path).convert("RGBA")).astype(np.float32)/255
    return float((0.2126*a[...,0]+0.7152*a[...,1]+0.0722*a[...,2]).mean())
def grab(folder,fr):
    p=f"{folder}/f_{fr:05d}.png".replace("\\","/")
    if os.path.exists(p): os.remove(p)
    js(f"var c=app.project.itemByID(1); c.saveFrameToPng({fr}/c.frameRate, new File('{p}')); 'ok'")
    for _ in range(800):
        if os.path.exists(p):
            try: return luma(p)
            except Exception: pass
        time.sleep(0.05)
    return None
def cf():
    return js("var f=new File(Folder.temp.fsName+'/dynamicfx.log'); f.encoding='UTF-8'; f.open('r'); var n=0; while(!f.eof){if(f.readln().indexOf('checkout failed')>=0)n++;} f.close(); ''+n")
for rnd in range(1,3):
    folder=os.path.join(SP,f"round{rnd}"); os.makedirs(folder,exist_ok=True)
    js("var c=app.project.itemByID(1); c.openInViewer(); app.purge(PurgeTarget.ALL_CACHES); 'ok'")
    done=os.path.join(SP,"mf_preview","done.txt")
    if os.path.exists(done): os.remove(done)
    cf0=cf()
    js(open(os.path.join(SP,"mf_preview_interrupt.jsx"),encoding="utf-8").read())
    t0=time.time()
    while not os.path.exists(done) and time.time()-t0<60: time.sleep(0.5)
    time.sleep(2)
    lum={fr:grab(folder,fr) for fr in range(90,210)}
    fr_s=sorted(lum); dips=[]
    for i,fr in enumerate(fr_s):
        nb=[lum[f] for f in fr_s[max(0,i-3):i]+fr_s[i+1:i+4] if lum[f] is not None]
        if lum[fr] is None or not nb: continue
        med=sorted(nb)[len(nb)//2]
        if med-lum[fr]>0.006: dips.append(fr)
    print(f"round {rnd}: checkout-failed {cf0}->{cf()}, dips={len(dips)} {dips}", flush=True)
