"""TR-GRP-001 - does After Effects re-match a saved project's effect parameter
streams by declaration index or by `PF_ParamDef.uu.id` when the plug-in's
parameter layout changes between saves?

The `after-effects` crate has always written a murmur3 id for every declared
parameter, so projects saved by released builds already carry whatever AE
persists of those ids. The spike build (worktree `AePlugin_Dynamicfx-spike-grp`,
detached at the 0.0.5 record commit) wraps the Float pool in one topic
("Floats"), which inserts a GROUP_START at the released Float 01 position and a
GROUP_END after Float 48 - shifting the floats by +1 and everything after them
by +2 while every pre-existing parameter keeps its id.

Phases are separate subcommands because the installed AEX is swapped between
them (AE must be closed for the swap):

  phase1   AE 2025 running the released 0.0.5: author baseline.aep with
           distinctive values, keyframes and an expression; dump baseline.json.
  quit     close the project without saving and quit AE.
  phase2   AE 2025 running the spike build: open baseline.aep, dump
           after_spike.json, close without saving.
  phase3   optional control with 0.0.5 restored: same dump to after_restore.json.
  verdict  compare the dumps and print the mechanism verdict.

Evidence: docs/audits/evidence/spike-20260826-param-group-matching/
"""
import json, os, sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, ".."))
import aemcp

REPO = os.path.abspath(os.path.join(HERE, "..", ".."))
EVID = os.path.join(REPO, "docs", "audits", "evidence", "spike-20260826-param-group-matching")
AEP = EVID.replace("\\", "/") + "/baseline.aep"

# Probe -> value set by phase1 and expected to survive the layout change on the
# parameter of the SAME NAME. Chosen to be unmistakable against the defaults
# (floats default 0, int 0, bool off, color white, point (50,50), angle 0,
# point3d (50,50,0), layer none, stops 2, stop pos 0).
PROBES = {
    "Float 01": 11.5,
    "Float 48": -3.25,
    "Int 01": 7.0,
    "Bool 01": 1.0,
    "Color 01": [0.1, 0.2, 0.3, 1.0],
    "Point 01": [30.0, 40.0],
    "Angle 01": 45.0,
    "Point 3D 01": [10.0, 20.0, 30.0],
    "Layer 01": 2.0,  # layer index of 'other'
    "Gradient 01 Stops": 5.0,
    "G01 Stop 01 Pos": 0.42,
}
KEYED = "Float 02"      # two keyframes: t=0 -> 1.25, t=1 -> 2.5
EXPRESSED = "Float 03"  # expression '0.123'

DUMP_JSX = r"""
(function(){
  function esc(s){ s=String(s); return s.replace(/\\/g,'\\\\').replace(/"/g,'\\"').replace(/\n/g,'\\n').replace(/\r/g,''); }
  function row(p, path){
    var t; try { t = Number(p.propertyValueType); } catch(e){ t = -1; }
    var v = 'null';
    try {
      if (t != PropertyValueType.NO_VALUE) {
        var val = p.value;
        if (val instanceof Array) { var a=[]; for (var i=0;i<val.length;i++) a.push(Number(val[i]).toFixed(6)); v='['+a.join(',')+']'; }
        else if (typeof val == 'number') v = Number(val).toFixed(6);
        else if (typeof val == 'boolean') v = val ? '1' : '0';
        else v = '"'+esc(val)+'"';
      }
    } catch(e) { v = '"ERR:'+esc(e.toString())+'"'; }
    var nk = 0; try { nk = p.numKeys || 0; } catch(e) { nk = -1; }
    var ex = ''; try { ex = p.expression || ''; } catch(e) { ex = 'ERR'; }
    var kk = '';
    if (nk > 0) { try { kk = ',"k0":'+Number(p.valueAtTime(0,true)).toFixed(6)+',"k1":'+Number(p.valueAtTime(1,true)).toFixed(6); } catch(e) { kk=''; } }
    return '{"path":"'+esc(path)+'","name":"'+esc(p.name)+'","mn":"'+esc(p.matchName)+'","t":'+t+',"v":'+v+',"nk":'+nk+',"ex":"'+esc(ex)+'"'+kk+'}';
  }
  function walk(pb, path, out){
    var isProp = false;
    try { isProp = (pb.propertyType == PropertyType.PROPERTY); } catch(e) { isProp = true; }
    if (isProp) { out.push(row(pb, path)); return; }
    out.push('{"path":"'+esc(path)+'","group":1,"name":"'+esc(pb.name)+'","mn":"'+esc(pb.matchName)+'","n":'+pb.numProperties+'}');
    for (var i=1;i<=pb.numProperties;i++) walk(pb.property(i), path+'/'+i, out);
  }
  var comp=null;
  for (var i=1;i<=app.project.numItems;i++){ var it=app.project.item(i); if (it instanceof CompItem && it.name=='grp'){ comp=it; break; } }
  if (!comp) return '{"error":"no comp grp"}';
  var lyr=null;
  for (var j=1;j<=comp.numLayers;j++){ if (comp.layer(j).name=='target'){ lyr=comp.layer(j); break; } }
  if (!lyr) return '{"error":"no layer target"}';
  var fx=lyr.property('ADBE Effect Parade').property(1);
  var out=[];
  for (var k=1;k<=fx.numProperties;k++) walk(fx.property(k), String(k), out);
  return '{"ae":"'+esc(app.version)+'","effect":"'+esc(fx.matchName)+'","top":'+fx.numProperties+',"props":['+out.join(',')+']}';
})()
"""

PHASE1_JSX = r"""
(function(){
  var proj = app.newProject();
  var comp = proj.items.addComp('grp', 100, 100, 1, 2, 25);
  comp.layers.addSolid([0,1,0], 'other', 100, 100, 1);
  comp.layers.addSolid([1,0,0], 'target', 100, 100, 1);
  var target = null;
  for (var j=1;j<=comp.numLayers;j++){ if (comp.layer(j).name=='target') target = comp.layer(j); }
  var fx = target.property('ADBE Effect Parade').addProperty('DynamicFx');
  function P(n){ return fx.property(n); }
  P('Float 01').setValue(11.5);
  P('Float 02').setValueAtTime(0, 1.25);
  P('Float 02').setValueAtTime(1, 2.5);
  P('Float 03').expression = '0.123';
  P('Float 48').setValue(-3.25);
  P('Int 01').setValue(7);
  P('Bool 01').setValue(1);
  P('Color 01').setValue([0.1, 0.2, 0.3, 1]);
  P('Point 01').setValue([30, 40]);
  P('Angle 01').setValue(45);
  P('Point 3D 01').setValue([10, 20, 30]);
  P('Layer 01').setValue(2);
  P('Gradient 01 Stops').setValue(5);
  P('G01 Stop 01 Pos').setValue(0.42);
  app.project.save(new File('%AEP%'));
  return 'saved:' + app.project.file.fsName;
})()
"""


def js(code, timeout_ms=120000):
    r = aemcp.exec_js(code, timeout_ms=timeout_ms)
    if not r.get("ok", False):
        raise SystemExit("exec failed: %s" % json.dumps(r, ensure_ascii=False)[:600])
    return r.get("result", "")


def dump_to(fname):
    raw = js(DUMP_JSX)
    data = json.loads(raw)
    os.makedirs(EVID, exist_ok=True)
    path = os.path.join(EVID, fname)
    # Recorded evidence is immutable: rerunning a phase against a different
    # installed build must never clobber the spike's files (it did once, on
    # 2026-08-26 — the 0.0.6 host pass overwrote after_spike.json/verdict.txt;
    # see the README's record-integrity note).
    if os.path.exists(path):
        raise SystemExit(f"refusing to overwrite recorded evidence: {path}")
    with open(path, "w", encoding="utf-8") as f:
        json.dump(data, f, indent=1, ensure_ascii=False)
    print("wrote %s (top-level rows: %s, ae %s)" % (path, data.get("top"), data.get("ae")))
    return data


def phase1():
    os.makedirs(EVID, exist_ok=True)
    print(js(PHASE1_JSX.replace("%AEP%", AEP)))
    dump_to("baseline.json")


def phase2():
    print(js("app.open(new File('%s')); 'opened'" % AEP))
    dump_to("after_spike.json")
    print(js("app.project.close(CloseOptions.DO_NOT_SAVE_CHANGES); 'closed'"))


def phase3():
    print(js("app.open(new File('%s')); 'opened'" % AEP))
    dump_to("after_restore.json")
    print(js("app.project.close(CloseOptions.DO_NOT_SAVE_CHANGES); 'closed'"))


def quit_ae():
    try:
        print(js("app.project.close(CloseOptions.DO_NOT_SAVE_CHANGES); app.quit(); 'quitting'", timeout_ms=15000))
    except SystemExit as e:
        # The panel dies with the process; a dropped connection here is the
        # expected shape of success.
        print("quit sent (%s)" % e)


def _by_name(data):
    out = {}
    for p in data["props"]:
        if p.get("group"):
            continue
        out.setdefault(p["name"], p)
    return out


def _close(a, b, tol=1e-4):
    if isinstance(a, list) and isinstance(b, list):
        return len(a) == len(b) and all(_close(x, y, tol) for x, y in zip(a, b))
    try:
        return abs(float(a) - float(b)) <= tol
    except (TypeError, ValueError):
        return a == b


def verdict():
    base = _by_name(json.load(open(os.path.join(EVID, "baseline.json"), encoding="utf-8")))
    after = _by_name(json.load(open(os.path.join(EVID, "after_spike.json"), encoding="utf-8")))
    lines, kept, lost = [], 0, 0
    for name, want in PROBES.items():
        b, a = base.get(name), after.get(name)
        if b is None or a is None:
            lines.append("MISSING %-18s baseline=%s after=%s" % (name, bool(b), bool(a)))
            lost += 1
            continue
        ok = _close(a["v"], want)
        kept += ok
        lost += not ok
        lines.append("%s %-18s want=%-22s after=%-22s (path %s -> %s, mn %s -> %s)"
                     % ("KEPT" if ok else "LOST", name, json.dumps(want), json.dumps(a["v"]),
                        b["path"], a["path"], b["mn"], a["mn"]))
    bk, ak = base.get(KEYED), after.get(KEYED)
    keys_ok = bool(ak) and ak.get("nk") == 2 and _close(ak.get("k0"), 1.25) and _close(ak.get("k1"), 2.5)
    lines.append("%s %-18s keyframes nk=%s k0=%s k1=%s (want 2, 1.25, 2.5)"
                 % ("KEPT" if keys_ok else "LOST", KEYED,
                    ak and ak.get("nk"), ak and ak.get("k0"), ak and ak.get("k1")))
    kept += keys_ok
    lost += not keys_ok
    be, ae_ = base.get(EXPRESSED), after.get(EXPRESSED)
    expr_ok = bool(ae_) and ae_.get("ex") == "0.123"
    lines.append("%s %-18s expression=%r (want '0.123')"
                 % ("KEPT" if expr_ok else "LOST", EXPRESSED, ae_ and ae_.get("ex")))
    kept += expr_ok
    lost += not expr_ok
    total = kept + lost
    if lost == 0:
        v = "ID_MATCH - every probe survived the index shift; AE re-matched streams by parameter id"
    elif kept == 0:
        v = "INDEX_MATCH_OR_RESET - no probe survived; grouping released parameters needs the approved one-time break"
    else:
        v = "MIXED - %d/%d probes survived; read the table before deciding" % (kept, total)
    report = "\n".join(lines) + "\n[VERDICT] " + v + "\n"
    print(report)
    with open(os.path.join(EVID, "verdict.txt"), "w", encoding="utf-8") as f:
        f.write(report)


def main():
    cmd = sys.argv[1] if len(sys.argv) > 1 else ""
    fn = {"phase1": phase1, "phase2": phase2, "phase3": phase3, "quit": quit_ae, "verdict": verdict}.get(cmd)
    if not fn:
        raise SystemExit("usage: tr_grp_001.py phase1|quit|phase2|phase3|verdict")
    fn()


if __name__ == "__main__":
    main()
