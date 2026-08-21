"""Continuation of copy_flicker_repro.py on the same open project (3 instances after the paste):
C) force a compile of the copy by re-committing Source (property index 2); D) edit a parameter on
the copy, then on the original; E) short preview; F) duplicate layer 2 (Ctrl+D route);
G) reopen the sample fresh and identify, for each cached bad frame of the missing-frame run,
which instance's output was missing (layer-hidden decomposition). Appends to flicker_report.txt."""
import os, time, json, glob, shutil
import numpy as np
from PIL import Image
import aemcp
SP = os.path.dirname(os.path.abspath(__file__))
OUT = os.path.join(SP, "flicker"); os.makedirs(OUT, exist_ok=True)
out_js = OUT.replace("\\", "/")
report = []
def say(s): print(s, flush=True); report.append(s)
def js(code, timeout=90000):
    r = aemcp.exec_js(code, timeout_ms=timeout)
    if not r.get("ok"): raise SystemExit("exec failed: " + json.dumps(r, ensure_ascii=False)[:600])
    return r.get("result")
LIB = r"""
function counters(){ var f=new File(Folder.temp.fsName+'/dynamicfx.log'); f.encoding='UTF-8'; f.open('r'); var n=0,tok=0,ui=0,pipe=0,res=0,st=0,cf=0,reb=0; while(!f.eof){var l=f.readln(); n++; if(l.indexOf('idle state token updated')>=0)tok++; if(l.indexOf('idle slot ui applied')>=0)ui++; if(l.indexOf('pipelines built')>=0)pipe++; if(l.indexOf('resolved from process registry')>=0)res++; if(l.indexOf('status:')>=0)st++; if(l.indexOf('checkout failed')>=0)cf++; if(l.indexOf('rebuilt')>=0)reb++;} f.close(); return 'lines='+n+' tok='+tok+' ui='+ui+' pipe='+pipe+' res='+res+' status='+st+' rebuilt='+reb+' cf='+cf; }
function dfx(c){ var r=[]; for(var L=1;L<=c.numLayers;L++){ var fx=c.layer(L).property('ADBE Effect Parade'); if(!fx) continue; for(var e=1;e<=fx.numProperties;e++) if(fx.property(e).matchName=='DynamicFx') r.push({layer:L, ef:fx.property(e)}); } return r; }
function describe(c){ var r=dfx(c), s=[]; for(var i=0;i<r.length;i++){ var ef=r[i].ef; var d=ef.property('Dispersion Distance'), st=ef.property('Color Strength'); s.push('L'+r[i].layer+' ['+ef.property(4).name+'] tok='+ef.property(5).value+' dist='+(d?d.value:'?')+' str='+(st?st.value:'?')); } return s.join(' || '); }
var c = app.project.itemByID(1);
"""
def luma_of(path):
    a = np.asarray(Image.open(path).convert("RGBA")).astype(np.float32) / 255
    return a, float((0.2126*a[...,0]+0.7152*a[...,1]+0.0722*a[...,2]).mean())
def grab(path, fr, setup="", purge=True):
    path = path.replace("\\", "/")
    if os.path.exists(path): os.remove(path)
    js(LIB + setup + (" app.purge(PurgeTarget.ALL_CACHES);" if purge else "") + f" c.saveFrameToPng({fr}/c.frameRate, new File('{path}')); 'ok'")
    for _ in range(600):
        if os.path.exists(path):
            try: return luma_of(path)
            except Exception: pass
        time.sleep(0.05)
    return None
def render_n(tag, fr=120, n=5):
    arrs = [grab(f"{out_js}/{tag}_f{fr:05d}_{i}.png", fr) for i in range(n)]
    ok = [g for g in arrs if g is not None]
    spread = max(float(np.abs(a[0]-ok[0][0]).max()) for a in ok) if len(ok) > 1 else 0.0
    say(f"    {tag}: {n} purged renders of f{fr}: luma={[None if g is None else round(g[1],4) for g in arrs]} max|diff| between renders={spread:.4f}")
def observe(tag, seconds=15):
    say(f"    {tag}: observing {seconds}s"); prev = None
    for s in range(seconds + 1):
        line = js(LIB + "counters() + ' :: ' + describe(c)")
        if line != prev: say(f"      +{s:2d}s {line}")
        prev = line; time.sleep(1)
def preview(seconds=6):
    js("var c=app.project.itemByID(1); c.openInViewer(); c.workAreaStart=3.0; c.workAreaDuration=4.0; c.time=3.0; app.executeCommand(10314); 'playing'")
    time.sleep(seconds); js("app.executeCommand(10314); 'stopped'"); time.sleep(1)

say("[C] state: " + js(LIB + "app.project.file.name + ' :: ' + counters() + ' :: ' + describe(c)"))
say("[C] recommit copy: " + js(LIB + r"""var inst=dfx(c); var cp=inst[inst.length-1]; var srcp=cp.ef.property(2); var txt=srcp.expression; srcp.expression=txt; 'copy on L'+cp.layer+' "'+srcp.name+'" recommitted ('+txt.length+' chars)'"""))
observe("C-after-recommit", 15)
render_n("C-after-recommit")
say("[D] edit copy param: " + js(LIB + r"""var inst=dfx(c); var cp=inst[inst.length-1]; var p=cp.ef.property('Dispersion Distance'); p.setValue(0.3); 'copy dist='+p.value"""))
observe("D-after-copy-edit", 8)
say("[D] edit original param: " + js(LIB + r"""var inst=dfx(c); var o=null; for(var i=0;i<inst.length;i++) if(inst[i].layer==2) o=inst[i]; var p=o.ef.property('Dispersion Distance'); p.setValue(0.2); 'orig dist='+p.value"""))
observe("D-after-orig-edit", 8)
render_n("D-after-edits")
say("[E] preview 6s"); preview(6)
observe("E-after-preview", 6)
say("[F] duplicate layer 2: " + js(LIB + r"""var d=c.layer(2).duplicate(); d.name='solid (dup)'; 'now '+dfx(c).length+' instances; dup at L'+d.index"""))
observe("F-after-duplicate", 15)
render_n("F-after-duplicate")
say("[F] preview 6s"); preview(6)
observe("F-after-preview", 6)

# G. decomposition of the cached bad frames on a fresh open of the sample
dst = os.path.join(SP, "prism_flicker.aep")
js("app.project.close(CloseOptions.DO_NOT_SAVE_CHANGES); 'closed'")
say("[G] reopen: " + js(f"app.open(new File('{dst.replace(chr(92), '/')}')); app.project.file.name"))
time.sleep(4)
js("var c=app.project.itemByID(1); c.openInViewer(); 'ok'")
say("    " + js(LIB + "counters() + ' :: ' + describe(c)"))
DEC = os.path.join(SP, "decomp"); os.makedirs(DEC, exist_ok=True); dec_js = DEC.replace("\\", "/")
variants = {"none_hidden": "", "L1_hidden": "c.layer(1).enabled=false;", "L2_hidden": "c.layer(2).enabled=false;", "both_hidden": "c.layer(1).enabled=false; c.layer(2).enabled=false;"}
say(f"    {'frame':>6} {'bad luma':>9} | " + " | ".join(f"{v:>12}" for v in variants) + "   (mean |diff| vs the cached bad frame; lower = closer)")
for fr in (111, 153, 154, 181):
    bad, bl = luma_of(os.path.join(SP, "mf_samples", f"f_{fr:05d}.png"))
    cells = []; best = None
    for v, setup in variants.items():
        got = grab(f"{dec_js}/f{fr:05d}_{v}.png", fr, setup + " ")
        js(LIB + "c.layer(1).enabled=true; c.layer(2).enabled=true; 'restored'")
        if got is None: cells.append("timeout"); continue
        d = float(np.abs(got[0] - bad).mean()); cells.append(f"{d:.5f}({got[1]:.4f})")
        if best is None or d < best[1]: best = (v, d)
    say(f"    {fr:>6} {bl:9.4f} | " + " | ".join(f"{x:>12}" for x in cells) + f"   -> closest: {best[0] if best else '?'}")
open(os.path.join(SP, "flicker_report.txt"), "a", encoding="utf-8").write("\n" + "\n".join(report))
