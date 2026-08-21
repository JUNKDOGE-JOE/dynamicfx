"""Probe for the report 'copying the effect makes the original and the copy flicker'.
Fresh open of the prism sample -> baseline (repeated purged renders of one frame, log churn at
rest) -> copy/paste the DynamicFx effect from layer 2 onto another layer -> observe both
instances and the log for 15 s -> repeated purged renders -> short preview -> force a compile of
the copy (re-commit its Source expression) -> observe again. Writes flicker_report.txt."""
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
function describe(c){ var r=dfx(c), s=[]; for(var i=0;i<r.length;i++){ var ef=r[i].ef; var d=ef.property('Dispersion Distance'), st=ef.property('Color Strength'); s.push('L'+r[i].layer+' "'+ef.name+'" ['+ef.property(4).name+'] tok='+ef.property(5).value+' dist='+(d?d.value:'?')+' str='+(st?st.value:'?')); } return s.join(' || '); }
var c = app.project.itemByID(1);
"""
def luma_of(path):
    a = np.asarray(Image.open(path).convert("RGBA")).astype(np.float32) / 255
    return a, float((0.2126*a[...,0]+0.7152*a[...,1]+0.0722*a[...,2]).mean())
def render_n(tag, fr=120, n=5):
    arrs = []
    for i in range(n):
        path = f"{out_js}/{tag}_f{fr:05d}_{i}.png"
        if os.path.exists(path): os.remove(path)
        js(f"var c=app.project.itemByID(1); app.purge(PurgeTarget.ALL_CACHES); c.saveFrameToPng({fr}/c.frameRate, new File('{path}')); 'ok'")
        got = None
        for _ in range(600):
            if os.path.exists(path):
                try: got = luma_of(path); break
                except Exception: pass
            time.sleep(0.05)
        arrs.append(got)
    lumas = [None if g is None else round(g[1], 4) for g in arrs]
    ok = [g for g in arrs if g is not None]
    spread = max(float(np.abs(a[0]-ok[0][0]).max()) for a in ok) if len(ok) > 1 else 0.0
    say(f"    {tag}: {n} purged renders of f{fr}: luma={lumas} max|diff| between renders={spread:.4f}")
    return arrs
def observe(tag, seconds=15):
    say(f"    {tag}: observing {seconds}s")
    prev = None
    for s in range(seconds + 1):
        line = js(LIB + "counters() + ' :: ' + describe(c)")
        cnt = line.split(" :: ")[0]
        if line != prev: say(f"      +{s:2d}s {line}")
        prev = line
        time.sleep(1)

# --- A. fresh open -------------------------------------------------------------
src = glob.glob("E:/Code/AePlugin_Dynamicfx/BugSample/*.aep")[0]
dst = os.path.join(SP, "prism_flicker.aep"); shutil.copy(src, dst)
js("app.project.close(CloseOptions.DO_NOT_SAVE_CHANGES); 'closed'")
say("[A] open: " + js(f"app.open(new File('{dst.replace(chr(92), '/')}')); app.project.file.name + ' ' + app.version"))
time.sleep(4)
js("var c=app.project.itemByID(1); c.openInViewer(); c.time=4.0; app.purge(PurgeTarget.ALL_CACHES); 'ok'")
say("    " + js(LIB + "counters() + ' :: ' + describe(c)"))
observe("A-rest", 6)
render_n("A-baseline")

# --- B. copy/paste the effect from layer 2 onto the next AV layer -----------------
say("[B] copy/paste: " + js(LIB + r"""
c.openInViewer();
var inst=dfx(c), src=null; for(var i=0;i<inst.length;i++) if(inst[i].layer==2) src=inst[i];
for(var L=1;L<=c.numLayers;L++) c.layer(L).selected=false;
src.ef.selected=true;
var r='';
try { app.executeCommand(19); r+='copy ok; '; } catch(e){ r+='copy failed: '+e+'; '; }
src.ef.selected=false;
var target=null; for(var L=3;L<=c.numLayers;L++){ var ly=c.layer(L); if(ly.hasVideo && ly.property('ADBE Effect Parade')){ target=ly; break; } }
target.selected=true;
try { app.executeCommand(20); r+='paste ok; '; } catch(e){ r+='paste failed: '+e+'; '; }
var after=dfx(c);
r+'instances before='+inst.length+' after='+after.length+' target=L'+target.index+' "'+target.name+'"'
"""))
n_after = int(js(LIB + "dfx(c).length"))
if n_after < 3:
    say("    paste did not add an instance; falling back to layer.duplicate() of layer 2: " + js(LIB + "var d=c.layer(2).duplicate(); d.name='dup'; 'now '+dfx(c).length+' instances'"))
observe("B-after-copy", 15)
render_n("B-after-copy")
# short uninterrupted preview, stopped by toggling the play command
say("    preview 6s: " + js("var c=app.project.itemByID(1); c.openInViewer(); c.workAreaStart=3.0; c.workAreaDuration=4.0; c.time=3.0; app.executeCommand(10314); 'playing'"))
time.sleep(6)
js("app.executeCommand(10314); 'stopped'")
time.sleep(1)
say("    " + js(LIB + "counters() + ' :: ' + describe(c)"))
observe("B-after-preview", 6)

# --- C. force a compile of the copy (re-commit its Source expression) -------------
say("[C] recommit copy: " + js(LIB + r"""
var inst=dfx(c); var cp=inst[inst.length-1];
var srcp=cp.ef.property('Source'); var txt=srcp.expression; srcp.expression=txt;
'copy on L'+cp.layer+' recommitted ('+txt.length+' chars)'
"""))
observe("C-after-recommit", 15)
render_n("C-after-recommit")
say("    preview 6s: " + js("var c=app.project.itemByID(1); c.openInViewer(); c.time=3.0; app.executeCommand(10314); 'playing'"))
time.sleep(6)
js("app.executeCommand(10314); 'stopped'")
time.sleep(1)
observe("C-after-preview", 6)
render_n("C-final")
open(os.path.join(SP, "flicker_report.txt"), "w", encoding="utf-8").write("\n".join(report))
