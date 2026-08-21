"""Clean reproduction of the dropped-frame report on the prism sample (AE 2026, DynamicFX 0.0.4):
purge -> interrupted-preview loop -> sample the work area from cache -> re-sample the dips
(persistence) -> purge and re-sample (recovery). Writes mf_report.txt."""
import os, re, time, json
import numpy as np
from PIL import Image
import aemcp
SP = os.path.dirname(os.path.abspath(__file__))
SAMPLES = os.path.join(SP, "mf_samples"); os.makedirs(SAMPLES, exist_ok=True)
PREVIEW = os.path.join(SP, "mf_preview"); os.makedirs(PREVIEW, exist_ok=True)
report = []
def say(s): print(s, flush=True); report.append(s)
def js(code, timeout=60000):
    r = aemcp.exec_js(code, timeout_ms=timeout)
    if not r.get("ok"): raise SystemExit("exec failed: " + json.dumps(r)[:400])
    return r.get("result")
def log_state():
    # (count of 'checkout failed' lines, total lines, epoch stamps of the checkout-failed lines)
    res = js("var f=new File(Folder.temp.fsName+'/dynamicfx.log'); f.encoding='UTF-8'; f.open('r'); var n=0,m=0,st=[]; while(!f.eof){var l=f.readln(); m++; if(l.indexOf('checkout failed')>=0){n++; st.push(l.substring(0,l.indexOf(']')+1)+' '+l.substring(l.indexOf('failed:')+8));}} f.close(); n+'|'+m+'|'+st.slice(-12).join(';')")
    n, m, st = res.split("|", 2)
    return int(n), int(m), st
def luma_of(path):
    a = np.asarray(Image.open(path).convert("RGBA")).astype(np.float32) / 255
    return float((0.2126*a[...,0]+0.7152*a[...,1]+0.0722*a[...,2]).mean())
def grab(fr, folder, tag, purge=False):
    path = f"{folder}/f_{fr:05d}{tag}.png".replace("\\", "/")
    if os.path.exists(path): os.remove(path)
    js("var c=app.project.itemByID(1); " + ("app.purge(PurgeTarget.ALL_CACHES); " if purge else "") + f"c.saveFrameToPng({fr}/c.frameRate, new File('{path}')); 'ok'")
    for _ in range(600):
        if os.path.exists(path):
            try: return luma_of(path)
            except Exception: pass
        time.sleep(0.05)
    return None

say(f"host {js('app.version')} | project {js('app.project.file.name')} | {time.strftime('%Y-%m-%d %H:%M:%S')} local")
js("app.purge(PurgeTarget.ALL_CACHES); 'purged'")
n0, m0, _ = log_state(); say(f"[0] purged; log: {n0} checkout-failed / {m0} lines")

# 1. interrupted preview loop (8 cycles of play -> move CTI), work area 3.0s+4.0s = frames 90..209
done = os.path.join(PREVIEW, "done.txt")
if os.path.exists(done): os.remove(done)
say("[1] " + js(open(os.path.join(SP, "mf_preview_interrupt.jsx"), encoding="utf-8").read()))
t0 = time.time()
while not os.path.exists(done) and time.time() - t0 < 60: time.sleep(0.5)
say(open(done, encoding="utf-8").read() if os.path.exists(done) else "preview loop did not finish in 60 s")
time.sleep(2)
n1, m1, st1 = log_state(); say(f"    log after preview: {n1} checkout-failed (+{n1-n0}) / {m1} lines; new stamps: {st1}")

# 2. sample the work area from whatever AE now holds (no purge)
t0 = time.time(); lum = {}
for fr in range(90, 210): lum[fr] = grab(fr, SAMPLES, "")
say(f"[2] sampled 120 frames (no purge) in {time.time()-t0:.0f}s; failures={[f for f,v in lum.items() if v is None]}")
n2, m2, _ = log_state(); say(f"    log after sampling: {n2} checkout-failed (+{n2-n1} during sampling) / {m2} lines")
frames = sorted(lum); dips = []
for i, fr in enumerate(frames):
    nb = [lum[f] for f in frames[max(0,i-3):i] + frames[i+1:i+4] if lum[f] is not None]
    if lum[fr] is None or not nb: continue
    med = sorted(nb)[len(nb)//2]
    if med - lum[fr] > 0.006: dips.append((fr, lum[fr], med))
say(f"    dips (luma > 0.006 below neighbour median): {len(dips)} -> " + ", ".join(f"f{fr}: {l:.4f} vs {m:.4f} ({l-m:+.4f})" for fr, l, m in dips))

# 3. persistence: re-read the dipped frames from cache twice; then purge and re-render
say(f"{'frame':>6} {'sampled':>8} {'reread':>8} {'purged':>8}")
for fr, l, m in dips:
    again = grab(fr, SAMPLES, "_reread")
    fresh = grab(fr, SAMPLES, "_purged", purge=True)
    say(f"{fr:>6} {l:8.4f} {again if again is None else round(again,4):>8} {fresh if fresh is None else round(fresh,4):>8}")
n3, m3, _ = log_state(); say(f"[3] log at end: {n3} checkout-failed / {m3} lines")
open(os.path.join(SP, "mf_report.txt"), "w", encoding="utf-8").write("\n".join(report))
