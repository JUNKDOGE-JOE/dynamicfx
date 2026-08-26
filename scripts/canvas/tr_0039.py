"""TR-0039 host legs - canvas expansion (ADR-0039) on the installed batch build.

All comps are 1024x1024 with the content centred, so leg renders and their
padded-precomp references diff pixel-to-pixel with no alignment step:

  L1 undeclared twin, plain 256 solid      -> clipped at the layer rect
  L2 undeclared twin + GrowBounds 256      -> matches the 256-padded reference
  L3 declared reach-ring (reach 160)       -> matches the 160-padded reference
  L4 declared reach 64 + GrowBounds 256    -> bounded by 64 (author wins)
  L5 declared, reach keyframed 0 -> 200    -> t=0 clipped, t=1 escaped

The shader is the shipped `examples/reach-ring.glsl`; the "twin" is the same
bytes with ` hint:canvas` stripped (the pinned no-hint variant from
`reach_ring_example_compiles_and_declares_the_canvas`).

Run on a live AE (ae-mcp panel) with the 0.0.6 batch build installed:
  python scripts/canvas/tr_0039.py all [--out DIR]
"""
import argparse, json, os, sys, time

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, ".."))
import aemcp  # noqa: E402

import numpy as np  # noqa: E402
from PIL import Image  # noqa: E402

REPO = os.path.abspath(os.path.join(HERE, "..", ".."))
LAYER = 256          # solid edge
COMP = 1024          # every comp/render frame
REACH = 160          # the example's declared default
TIGHT = 64           # L4's author boundary
GROW = 256           # GrowBounds pixels
# Grouped-topology property indexes (ADR-0040; verified live before use).
# ADR-0041 layout: Setup group adds two rows above everything.
IDX_SOURCE, IDX_STATUS, IDX_TOKEN, IDX_REACH, IDX_COLOR = 3, 5, 6, 11, 83

report = []


def say(s):
    print(s, flush=True)
    report.append(s)


def js(code, timeout=90000):
    r = aemcp.exec_js(code, timeout_ms=timeout)
    if not r.get("ok"):
        raise SystemExit("exec failed: " + json.dumps(r, ensure_ascii=False)[:800])
    return r.get("result")


def sources():
    with open(os.path.join(REPO, "examples", "reach-ring.glsl"), encoding="utf-8") as f:
        declared = f.read()
    twin = declared.replace(" hint:canvas", "")
    assert twin != declared
    return declared, twin


def setup():
    js("try{app.project.close(CloseOptions.DO_NOT_SAVE_CHANGES);}catch(e){} app.newProject(); app.project.bitsPerChannel=8; 'ok'")
    js("""
      function mk(name, size){
        var c = app.project.items.addComp(name, size, size, 1.0, 2, 25);
        c.bgColor = [0, 0, 0];
        c.layers.addSolid([1, 1, 1], 'white', %d, %d, 1.0, 2);
        return c;
      }
      var cvs = mk('cvs', %d);
      var pad160 = mk('pad160src', %d);
      var pad256 = mk('pad256src', %d);
      function wrap(name, src){
        var c = app.project.items.addComp(name, %d, %d, 1.0, 2, 25);
        c.bgColor = [0, 0, 0];
        c.layers.add(src);
        return c;
      }
      wrap('ref160', pad160);
      wrap('ref256', pad256);
      cvs.openInViewer();
      'ok'
    """ % (LAYER, LAYER, COMP, LAYER + 2 * REACH, LAYER + 2 * GROW, COMP, COMP))


def fx_js(comp):
    return ("(function(){ var c=null; for (var i=1;i<=app.project.numItems;i++){ var it=app.project.item(i);"
            " if (it instanceof CompItem && it.name=='%s') c=it; }"
            " return c.layer(1).property('ADBE Effect Parade'); })()" % comp)


def ensure_fx(comp):
    js("var fp=%s; if (fp.numProperties==0 || fp.property(fp.numProperties).matchName!='DynamicFx')"
       " fp.addProperty('DynamicFx'); 'ok'" % fx_js(comp))


def fx_prop(comp, index):
    return "%s.property(%s).property(%d)" % (fx_js(comp), "'DynamicFx'", index)


def assert_indexes(comp):
    got = js("var fx=%s.property('DynamicFx');"
             "[fx.property(%d).name, fx.property(%d).name, fx.property(%d).name].join('|')"
             % (fx_js(comp), IDX_SOURCE, IDX_REACH, IDX_COLOR))
    name_src, name_reach, name_color = got.split("|")
    assert "Source" in name_src, got
    # An unbound slot shows its pool name; once the shader binds it shows the label.
    assert name_reach.startswith(("Float 01", "Reach")), got
    assert name_color.startswith(("Color 01", "Halo")), got
    say("  index check (%s): %s" % (comp, got))


def commit(comp, source, timeout=25):
    prev = float(js("%s.value" % fx_prop(comp, IDX_TOKEN)))
    expr = "`" + source + "`;0"
    js("%s.expression = %s; 'ok'" % (fx_prop(comp, IDX_SOURCE), json.dumps(expr)))
    t0 = time.time()
    while time.time() - t0 < timeout:
        v = float(js("%s.value" % fx_prop(comp, IDX_TOKEN)))
        if v % 4 == 1 and v != prev:
            say("  %s compiled: %s" % (comp, js("%s.name" % fx_prop(comp, IDX_STATUS))))
            return
        time.sleep(0.3)
    raise SystemExit("%s did not compile (status=%s)" % (comp, js("%s.name" % fx_prop(comp, IDX_STATUS))))


GROW_MATCH = {"name": None}


def grow_bounds(comp, on):
    if not on:
        js("""var fp=%s;
          for (var i=fp.numProperties;i>=1;i--){ try{ if (/Grow ?Bounds/i.test(fp.property(i).matchName) || /Grow ?Bounds/i.test(fp.property(i).name)) fp.property(i).remove(); }catch(e){} }
          'ok'""" % fx_js(comp))
        return True
    if GROW_MATCH["name"] is None:
        # Some installed-effect descriptors throw on displayName access;
        # a bare loop dies mid-scan, so every read is guarded.
        GROW_MATCH["name"] = js("""
          var pick=''; var n=app.effects.length;
          for (var j=0;j<n;j++){
            try { var d=app.effects[j].displayName;
                  if (/Grow ?Bounds/i.test(d)) { pick=app.effects[j].matchName; break; } } catch(e){}
          }
          pick
        """)
        say("  grow_bounds matchName: %r" % GROW_MATCH["name"])
    if not GROW_MATCH["name"]:
        say("  grow_bounds unavailable: no matching installed effect")
        return False
    props = js("""
      var fp=%s; fp.addProperty(%s).moveTo(1);
      var e=fp.property(1);
      var names=[]; for (var i=1;i<=e.numProperties;i++){ try{ names.push(e.property(i).name); }catch(err){ names.push('?'); } }
      names.join('|')
    """ % (fx_js(comp), json.dumps(GROW_MATCH["name"])))
    say("  grow_bounds params: %s" % props)
    target = next((p for p in props.split("|") if "pixel" in p.lower()), None)
    if target is None:
        say("  grow_bounds unavailable: no pixels-like parameter in [%s]" % props)
        grow_bounds(comp, False)
        return False
    js("""var fp=%s;
      for (var i=1;i<=fp.numProperties;i++){
        try { if (/Grow ?Bounds/i.test(fp.property(i).matchName) || /Grow ?Bounds/i.test(fp.property(i).name)) { fp.property(i).property(%s).setValue(%d); break; } } catch(e){}
      }
      'ok'""" % (fx_js(comp), json.dumps(target), GROW))
    return True


def render(comp, path, t=0.5):
    path = path.replace("\\", "/")
    if os.path.exists(path):
        os.remove(path)
    js("app.purge(PurgeTarget.ALL_CACHES); (function(){ for (var i=1;i<=app.project.numItems;i++){ var it=app.project.item(i);"
       " if (it instanceof CompItem && it.name=='%s') it.saveFrameToPng(%s, new File('%s')); } })(); 'ok'" % (comp, t, path))
    for _ in range(600):
        if os.path.exists(path):
            try:
                return np.asarray(Image.open(path).convert("RGBA")).astype(int)
            except Exception:
                pass
        time.sleep(0.05)
    raise SystemExit("render never appeared: " + path)


def bbox(img, thr=8):
    a = img[:, :, 3]
    ys, xs = np.nonzero(a > thr)
    if len(xs) == 0:
        return None
    return int(xs.min()), int(ys.min()), int(xs.max()) + 1, int(ys.max()) + 1


def check_bbox(tag, img, max_reach, min_reach=None):
    lo = (COMP - LAYER) // 2
    hi = lo + LAYER
    b = bbox(img)
    say("  %s bbox=%s layer=[%d..%d] allowed reach=%s" % (tag, b, lo, hi, max_reach))
    if b is None:
        say("  !! %s rendered nothing" % tag)
        return False
    x0, y0, x1, y1 = b
    reach = max(lo - x0, lo - y0, x1 - hi, y1 - hi, 0)
    ok = reach <= max_reach + 2
    if min_reach is not None:
        ok &= reach >= min_reach
    if not ok:
        say("  !! %s reach=%d outside [%s..%s+2]" % (tag, reach, min_reach, max_reach))
    return ok


def compare(tag, a, b, tol_mean=2.0):
    diff = np.abs(a - b)
    mean = float(diff.mean())
    peak = int(diff.max())
    say("  %s mean|diff|=%.3f peak=%d (tolerance mean<=%.1f)" % (tag, mean, peak, tol_mean))
    return mean <= tol_mean


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("cmd", nargs="?", default="all")
    ap.add_argument("--out", default=os.path.join(REPO, "docs", "audits", "evidence",
                                                  "hostpass-20260826-006", "canvas"))
    args = ap.parse_args()
    os.makedirs(args.out, exist_ok=True)
    declared, twin = sources()
    ok = True

    say("[host] AE %s" % js("app.version"))
    setup()
    for comp in ("cvs", "ref160", "ref256"):
        ensure_fx(comp)
    assert_indexes("cvs")

    # References: the twin on padded precomps (the manual workaround).
    commit("ref160", twin)
    commit("ref256", twin)
    ref160 = render("ref160", os.path.join(args.out, "ref160.png"))
    ref256 = render("ref256", os.path.join(args.out, "ref256.png"))
    ok &= check_bbox("ref160", ref160, REACH, min_reach=REACH // 2)
    ok &= check_bbox("ref256", ref256, REACH, min_reach=REACH // 2)

    # L1: undeclared on the plain solid stays inside the layer rect.
    commit("cvs", twin)
    l1 = render("cvs", os.path.join(args.out, "L1-undeclared-plain.png"))
    ok &= check_bbox("L1", l1, 0)

    # L2: undeclared + GrowBounds == the padded reference (the released
    # no-op becomes the feature's positive test).
    if grow_bounds("cvs", True):
        l2 = render("cvs", os.path.join(args.out, "L2-undeclared-growbounds.png"))
        ok &= check_bbox("L2", l2, REACH, min_reach=REACH // 2)
        ok &= compare("L2 vs ref256", l2, ref256)
        grow_bounds("cvs", False)
    else:
        say("  L2 BLOCKED: no GrowBounds effect installed on this host")

    # L3: declared == the padded reference.
    commit("cvs", declared)
    l3 = render("cvs", os.path.join(args.out, "L3-declared-160.png"))
    ok &= check_bbox("L3", l3, REACH, min_reach=REACH // 2)
    ok &= compare("L3 vs ref160", l3, ref160)

    # L4: the declared boundary beats an upstream expander.
    js("%s.setValue(%d); 'ok'" % (fx_prop("cvs", IDX_REACH), TIGHT))
    if grow_bounds("cvs", True):
        l4 = render("cvs", os.path.join(args.out, "L4-declared-64-growbounds.png"))
        ok &= check_bbox("L4", l4, TIGHT)
        grow_bounds("cvs", False)
    else:
        say("  L4 BLOCKED: no GrowBounds effect installed on this host")

    # L5: a keyframed canvas parameter.
    js("var p=%s; p.setValueAtTime(0, 0); p.setValueAtTime(1, 200); 'ok'" % fx_prop("cvs", IDX_REACH))
    l5a = render("cvs", os.path.join(args.out, "L5-t0.png"), t=0.0)
    l5b = render("cvs", os.path.join(args.out, "L5-t1.png"), t=1.0)
    ok &= check_bbox("L5 t=0", l5a, 0)
    ok &= check_bbox("L5 t=1", l5b, 200, min_reach=100)

    say("[VERDICT] " + ("PASS - every canvas leg behaved per ADR-0039"
                        if ok else "FAIL - see !! lines above"))
    with open(os.path.join(args.out, "report.txt"), "w", encoding="utf-8") as f:
        f.write("\n".join(report) + "\n")
    return 0 if ok else 1


if __name__ == "__main__":
    sys.exit(main())
