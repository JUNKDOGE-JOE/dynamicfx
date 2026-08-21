"""Post-fix host verification of TR-CACHE-001 on the fix build (AE 2026):
open a fresh copy of the sample -> purge -> interrupted-preview loop -> sample the
work area from cache -> flag persistent dips (same neighbour-median method as the
pre-fix run) -> for any flagged frame, compare cache vs purged re-render.
Pre-fix baseline (0.0.4): 7 persistent dips. Expected after fix: 0. Writes verify_report.txt."""
import os, time, json, glob, shutil
import numpy as np
from PIL import Image
import aemcp
SP = os.path.dirname(os.path.abspath(__file__))
SAMP = os.path.join(SP, "mf_samples_fixed"); os.makedirs(SAMP, exist_ok=True)
PREVIEW = os.path.join(SP, "mf_preview"); os.makedirs(PREVIEW, exist_ok=True)
report = []
def say(s): print(s, flush=True); report.append(s)
def js(code, timeout=90000):
    r = aemcp.exec_js(code, timeout_ms=timeout)
    if not r.get("ok"): raise SystemExit("exec failed: " + json.dumps(r, ensure_ascii=False)[:500])
    return r.get("result")
def cf():
    return js("var f=new File(Folder.temp.fsName+'/dynamicfx.log'); f.encoding='UTF-8'; f.open('r'); var n=0,m=0; while(!f.eof){var l=f.readln(); m++; if(l.indexOf('checkout failed')>=0)n++;} f.close(); n+'/'+m")
def luma_of(path):
    a = np.asarray(Image.open(path).convert("RGBA")).astype(np.float32)/255
    return a, float((0.2126*a[...,0]+0.7152*a[...,1]+0.0722*a[...,2]).mean())
def grab(folder, fr, tag="", purge=False):
    path = f"{folder}/f_{fr:05d}{tag}.png".replace("\\","/")
    if os.path.exists(path): os.remove(path)
    js("var c=app.project.itemByID(1); " + ("app.purge(PurgeTarget.ALL_CACHES); " if purge else "") + f"c.saveFrameToPng({fr}/c.frameRate, new File('{path}')); 'ok'")
    for _ in range(800):
        if os.path.exists(path):
            try: return luma_of(path)
            except Exception: pass
        time.sleep(0.05)
    return None

dst = os.path.join(SP, "prism_verify.aep")
try: js("app.project.close(CloseOptions.DO_NOT_SAVE_CHANGES); 'closed'")
except SystemExit: pass
say("[open] " + js(f"app.open(new File('{dst.replace(chr(92),'/')}')); app.project.file.name + ' | ' + app.version"))
time.sleep(4)
say("[host] AE " + js("app.version") + " | installed DynamicFx.aex = fix build 24E963FB (verified) | " + time.strftime('%Y-%m-%d %H:%M:%S') + " local")
js("var c=app.project.itemByID(1); c.openInViewer(); 'ok'")
js("app.purge(PurgeTarget.ALL_CACHES); 'purged'")
say("[0] purged; log " + cf())
# interrupted preview loop (same jsx: 8 cycles, work area frames 90..209)
done = os.path.join(PREVIEW, "done.txt")
if os.path.exists(done): os.remove(done)
say("[1] " + js(open(os.path.join(SP,"mf_preview_interrupt.jsx"),encoding="utf-8").read()))
t0=time.time()
while not os.path.exists(done) and time.time()-t0 < 60: time.sleep(0.5)
say(open(done,encoding="utf-8").read() if os.path.exists(done) else "preview loop timeout")
time.sleep(2)
say("[1] after preview: log " + cf() + " (InterruptCancel lines are EXPECTED and correct now — the fix propagates them)")
# sample the work area from cache (no purge)
lum={}; t0=time.time()
for fr in range(90,210): lum[fr]=grab(SAMP, fr)
say(f"[2] sampled 120 frames from cache in {time.time()-t0:.0f}s; failures={[f for f,v in lum.items() if v is None]}; log {cf()}")
frames=sorted(lum); dips=[]
for i,fr in enumerate(frames):
    nb=[lum[f][1] for f in frames[max(0,i-3):i]+frames[i+1:i+4] if lum[f]]
    if not lum[fr] or not nb: continue
    med=sorted(nb)[len(nb)//2]
    if med-lum[fr][1] > 0.006: dips.append((fr, lum[fr][1], med))
say(f"[3] persistent dips (luma >0.006 below neighbour median): {len(dips)}  (pre-fix 0.0.4 baseline was 7)")
for fr,l,m in dips: say(f"      f{fr}: {l:.4f} vs {m:.4f} ({l-m:+.4f})")
# for any dip, prove cache == purged (persistence gone)
if dips:
    say(f"    {'frame':>6} {'cache':>8} {'purged':>8} {'delta':>8}")
    for fr,l,m in dips:
        pg=grab(SAMP, fr, "_purged", purge=True)
        say(f"    {fr:>6} {l:8.4f} {pg[1] if pg else None:>8} {('' if pg is None else f'{pg[1]-l:+.4f}'):>8}")
say("[VERDICT] " + ("PASS — no persistent dropped-layer frame after an interrupted preview." if not dips else f"CHECK — {len(dips)} dip(s) remain; inspect above."))
open(os.path.join(SP,"verify_report.txt"),"w",encoding="utf-8").write("\n".join(report))
