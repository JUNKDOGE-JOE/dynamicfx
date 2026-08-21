"""TR-BIND-002 host harness (ADR-0038): two instances of ONE source whose BindingPlans differ.

A is *migrated* (compiled at v1 with p1..p3 / a1 / texB, then re-committed at v2 which inserts
p0, a0, texA in front), B is *fresh* (addProperty + the same v2 text). Each instance gets its own
distinguishable values (4 floats, 2 angles, 2 layer inputs) and is rendered alone; the four
quadrants of the output encode (p0,p1,p2) / (p3,a0/360,a1/360) / texA / texB, so one PNG per
instance proves every stream AND the layer wiring resolved through that instance's own plan.
Both compile orders run (A-then-B, B-then-A) on distinct source text (distinct fingerprints);
--reopen adds a same-session save/close/reopen pass and an aerender pass (fresh process).

Driven through the ae-mcp panel /exec channel of a warm AE session (scripts/aemcp.py).
Writes PNGs + report.txt under scripts/out/bind/<run>/ ; exit code 0 = every assertion held.
"""
import os, sys, time, json, argparse, glob, subprocess
import numpy as np
from PIL import Image

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.dirname(HERE))
import aemcp  # noqa: E402

# The plug-in logs to Rust's std::env::temp_dir(), which prefers TMP over TEMP on Windows.
LOG = os.path.join(os.environ.get("TMP") or os.environ.get("TEMP") or os.path.expanduser("~"), "dynamicfx.log")
AERENDER = r"C:\Program Files\Adobe\Adobe After Effects %s\Support Files\aerender.exe"
# property indices: Float i -> 6+i, Angle i -> 102+i, Layer i -> 111+i (checked live by --smoke)
F = lambda i: 6 + i
ANG = lambda i: 102 + i
LAY = lambda i: 111 + i
TOL = 3  # 8-bpc


def src(version, tag):
    v2 = version == 2
    graph = "pass main: input, texA, texB -> output" if v2 else "pass main: input, texB -> output"
    params = ["// @param p0 label:\"p0\" min:0 max:1"] if v2 else []
    params += ["// @param p1 label:\"p1\" min:0 max:1", "// @param p2 label:\"p2\" min:0 max:1",
               "// @param p3 label:\"p3\" min:0 max:1"]
    params += (["// @param a0 label:\"a0\" hint:angle"] if v2 else []) + ["// @param a1 label:\"a1\" hint:angle"]
    params += (["// @param texA hint:layer"] if v2 else []) + ["// @param texB hint:layer"]
    fields = (["    float p0;"] if v2 else []) + ["    float p1;", "    float p2;", "    float p3;"]
    fields += (["    float a0;"] if v2 else []) + ["    float a1;"]
    tex = (["layout(set = 0, binding = 3) uniform texture2D texA;",
            "layout(set = 0, binding = 4) uniform texture2D texB;"] if v2
           else ["layout(set = 0, binding = 3) uniform texture2D texB;"])
    body = [
        "    vec3 tb = texture(sampler2D(texB, u_s), v_uv).rgb;",
        "    vec3 ta = texture(sampler2D(texA, u_s), v_uv).rgb;" if v2 else "    vec3 ta = vec3(0.0);",
        "    vec3 q0 = " + ("vec3(p0, p1, p2);" if v2 else "vec3(0.0, p1, p2);"),
        "    vec3 q1 = " + ("vec3(p3, a0 / 360.0, a1 / 360.0);" if v2 else "vec3(p3, 0.0, a1 / 360.0);"),
        "    vec3 c = (v_uv.y < 0.5) ? ((v_uv.x < 0.5) ? q0 : q1) : ((v_uv.x < 0.5) ? ta : tb);",
        "    outColor = vec4(c, 1.0);",
    ]
    return "\n".join([
        "@dynamicfx 1", "@graph", graph, "@end", "@pass main", "#version 450",
        "// tr-bind-002 " + tag + " v" + str(version), *params,
        "layout(location = 0) in vec2 v_uv;", "layout(location = 0) out vec4 outColor;",
        "layout(set = 0, binding = 0) uniform texture2D u_in;",
        "layout(set = 0, binding = 1) uniform sampler u_s;",
        "layout(set = 0, binding = 2) uniform FxUniforms {",
        "    vec2 u_resolution;", "    float u_time;", "    float u_frame;",
        *fields, "};", *tex, "void main() {", *body, "}", "@endpass"]) + "\n"


PRELUDE = r"""
function CMP(){ for (var i=1;i<=app.project.numItems;i++){ var it=app.project.item(i); if (it instanceof CompItem && it.name==='bind') return it; } return null; }
function LYR(n){ var c=CMP(); for (var i=1;i<=c.numLayers;i++){ if (c.layer(i).name===n) return c.layer(i); } return null; }
function FX(n){ return LYR(n).property('ADBE Effect Parade').property(1); }
function NAMES(n, idx){ var fx=FX(n); var o=[]; for (var i=0;i<idx.length;i++){ o.push(idx[i]+'='+fx.property(idx[i]).name); } return o.join('|'); }
"""
report = []


def say(s):
    print(s, flush=True)
    report.append(s)


def js(code, timeout=60000):
    r = aemcp.exec_js(PRELUDE + code, timeout_ms=timeout)
    if not r.get("ok"):
        raise SystemExit("exec failed: " + json.dumps(r, ensure_ascii=False)[:800])
    return r.get("result")


# Log lines that must stay at zero on a correct build (a clone adopted something other than its own
# instance's entry, or passed through) versus lines that are merely reported.
STRICT_ZERO = ["resolved by latest entry", "adopting latest entry", "not this plan; passing through",
               "missed registry"]
REPORTED = ["resolved from process registry", "via lineage", "rebuilt from snapshot",
            "idle slot ui applied", "pipelines built", "without this instance's artifact"]
LOG_KEYS = REPORTED + STRICT_ZERO


def log_counts():
    try:
        text = open(LOG, encoding="utf-8", errors="replace").read()
    except OSError:
        return {k: None for k in LOG_KEYS}
    return {k: text.count(k) for k in LOG_KEYS}


def log_delta(label, c0, c1):
    """Report every counter; fail the leg if a strict-zero line appeared (or the log is unreadable)."""
    ok = True
    parts = []
    for k in LOG_KEYS:
        if c0[k] is None or c1[k] is None:
            parts.append("%s ?" % k)
            ok = False
            continue
        parts.append("%s %d->%d" % (k, c0[k], c1[k]))
        if k in STRICT_ZERO and c1[k] != c0[k]:
            ok = False
    say("  log deltas (%s): %s%s" % (label, ", ".join(parts), "" if ok else "   <-- STRICT line moved or log unreadable"))
    return ok


def new_project():
    js("try{app.project.close(CloseOptions.DO_NOT_SAVE_CHANGES);}catch(e){} app.newProject(); app.project.bitsPerChannel=8; 'ok'")
    js("""var c=app.project.items.addComp('bind',320,240,1.0,1.0,25); c.bgColor=[0,0,0];
        var cols={red:[1,0,0],green:[0,1,0],blue:[0,0,1],yellow:[1,1,0]};
        for (var n in cols){ var s=c.layers.addSolid(cols[n],n,320,240,1.0,1.0); s.enabled=false; }
        c.layers.addSolid([0.5,0.5,0.5],'B',320,240,1.0,1.0).enabled=false;
        c.layers.addSolid([0.5,0.5,0.5],'A',320,240,1.0,1.0).enabled=false;
        c.openInViewer(); 'ok'""")


def token(n):
    return float(js("FX('%s').property(5).value" % n))


def status(n):
    return js("FX('%s').property(4).name" % n)


def add_fx(n):
    js("LYR('%s').property('ADBE Effect Parade').addProperty('DynamicFx'); 'ok'" % n)


def commit(n, source):
    expr = "`" + source + "`;0"
    js("FX('%s').property(2).expression = %s; 'ok'" % (n, json.dumps(expr)))


def wait_compiled(n, prev_token, timeout=20):
    t0 = time.time()
    while time.time() - t0 < timeout:
        v = token(n)
        if v % 4 == 1 and v != prev_token:
            return v, status(n)
        time.sleep(0.25)
    raise SystemExit("%s did not compile in %ss (token=%s status=%s)" % (n, timeout, token(n), status(n)))


def names(n):
    idx = [F(0), F(1), F(2), F(3), ANG(0), ANG(1), LAY(0), LAY(1)]
    return js("NAMES('%s', [%s])" % (n, ",".join(str(i) for i in idx)))


def set_values(n, by_index):
    stmts = "".join("fx.property(%d).setValue(%s);" % (i, json.dumps(v)) for i, v in by_index.items())
    js("var fx=FX('%s'); %s 'ok'" % (n, stmts))


def layer_index(name):
    return int(js("LYR('%s').index" % name))


def wait_idle(seconds):
    time.sleep(seconds)


def load_rgb(path):
    return np.asarray(Image.open(path).convert("RGB")).astype(int)


def render(n, path):
    path = path.replace("\\", "/")
    if os.path.exists(path):
        os.remove(path)
    js("LYR('A').enabled=(%s); LYR('B').enabled=(%s); app.purge(PurgeTarget.ALL_CACHES); "
       "CMP().saveFrameToPng(0.5, new File('%s')); 'ok'"
       % ("true" if n == "A" else "false", "true" if n == "B" else "false", path))
    for _ in range(400):
        if os.path.exists(path):
            try:
                return load_rgb(path)
            except Exception:
                pass  # PNG still being written
        time.sleep(0.05)
    raise SystemExit("render of %s never appeared: %s" % (n, path))


def quadrants(a):
    h, w = a.shape[:2]
    pts = {"TL": (h // 4, w // 4), "TR": (h // 4, 3 * w // 4), "BL": (3 * h // 4, w // 4), "BR": (3 * h // 4, 3 * w // 4)}
    return {k: tuple(int(x) for x in a[y, x]) for k, (y, x) in pts.items()}


def expect_rgb(vals):
    return tuple(int(round(v * 255)) for v in vals)


COLORS = {"red": (1, 0, 0), "green": (0, 1, 0), "blue": (0, 0, 1), "yellow": (1, 1, 0)}
VALUES = {
    "A": dict(p0=0.10, p1=0.20, p2=0.30, p3=0.40, a0=36.0, a1=72.0, texA="red", texB="green"),
    "B": dict(p0=0.50, p1=0.60, p2=0.70, p3=0.80, a0=180.0, a1=252.0, texA="blue", texB="yellow"),
}
# slot tables each instance must end up with (property index -> param id)
PLAN_MIGRATED = {F(0): "p1", F(1): "p2", F(2): "p3", F(3): "p0", ANG(0): "a1", ANG(1): "a0", LAY(0): "texB", LAY(1): "texA"}
PLAN_FRESH = {F(0): "p0", F(1): "p1", F(2): "p2", F(3): "p3", ANG(0): "a0", ANG(1): "a1", LAY(0): "texA", LAY(1): "texB"}
PLAN = {"A": PLAN_MIGRATED, "B": PLAN_FRESH}


def apply_values(n):
    v = VALUES[n]
    by_index = {}
    for idx, pid in PLAN[n].items():
        val = v[pid]
        by_index[idx] = layer_index(val) if pid.startswith("tex") else val
    set_values(n, by_index)


def expected_quadrants(n):
    v = VALUES[n]
    return {"TL": expect_rgb((v["p0"], v["p1"], v["p2"])),
            "TR": expect_rgb((v["p3"], v["a0"] / 360, v["a1"] / 360)),
            "BL": expect_rgb(COLORS[v["texA"]]), "BR": expect_rgb(COLORS[v["texB"]])}


def check_quadrants(label, n, a):
    """One instance's rendered frame against its own values and layer wiring."""
    q = quadrants(a)
    e = expected_quadrants(n)
    say("  render %s %s: quadrants %s" % (label, n, q))
    ok = True
    for k in ("TL", "TR", "BL", "BR"):
        good = all(abs(g - x) <= TOL for g, x in zip(q[k], e[k]))
        say("    %-30s got=%-15s expect=%-15s %s" % ("%s %s %s" % (label, n, k), q[k], e[k], "OK" if good else "MISMATCH"))
        ok &= good
    return ok


def verify_names(n, nm):
    got = dict(kv.split("=", 1) for kv in nm.split("|"))
    ok = True
    for idx, pid in PLAN[n].items():
        if pid.startswith("tex"):
            continue  # AE keeps host names on Layer controls (reference.md); wiring is proven by the render
        if got.get(str(idx)) != pid:
            ok = False
            say("    %s: slot %d is %r, expected %r" % (n, idx, got.get(str(idx)), pid))
    say("  %s slot names %s" % (n, "match its own plan" if ok else "DO NOT match its own plan"))
    return ok


def make_A(tag):
    """A: compile v1, then re-commit v2 -> migrated plan."""
    add_fx("A")
    commit("A", src(1, tag))
    t1, s1 = wait_compiled("A", 0)
    say("  A v1 compiled: %s" % s1)
    commit("A", src(2, tag))
    t2, s2 = wait_compiled("A", t1)
    say("  A v2 compiled: %s" % s2)
    wait_idle(2.5)
    nm = names("A")
    say("  A names after migration: " + nm)
    apply_values("A")
    return nm


def make_B(tag):
    add_fx("B")
    commit("B", src(2, tag))
    t, s = wait_compiled("B", 0)
    say("  B v2 compiled: %s" % s)
    wait_idle(2.5)
    nm = names("B")
    say("  B names (fresh): " + nm)
    apply_values("B")
    return nm


def run_order(order, outdir, tag=None):
    tag = tag or "order-" + order
    say("=== compile order %s (%s) ===" % (order, tag))
    new_project()
    c0 = log_counts()
    if order == "AB":
        first, first_names = "A", make_A(tag)
        make_B(tag)
    else:
        first, first_names = "B", make_B(tag)
        make_A(tag)
    wait_idle(3.0)  # let the idle observer take a few ticks over both instances
    ok = True
    after = {n: names(n) for n in ("A", "B")}
    for n in ("A", "B"):
        say("  %s names after both compiled: %s" % (n, after[n]))
        ok &= verify_names(n, after[n])
    if after[first] != first_names:
        say("  !! slot names of %s CHANGED after the other instance compiled (idle observer applied a foreign plan)" % first)
        ok = False
    for n in ("A", "B"):
        for rnd in range(2):
            ok &= check_quadrants("%s r%d" % (tag, rnd), n, render(n, os.path.join(outdir, "%s_%s_r%d.png" % (tag, n, rnd))))
    ok &= log_delta(tag, c0, log_counts())
    return ok


def run_reopen(outdir, year):
    """Same-session save/close/reopen (Resetup path, warm registry), then aerender (fresh
    process, cold registry). Assumes the order-AB project is still open. No Compile is pressed."""
    say("=== reopen leg (same session) ===")
    aep = os.path.join(outdir, "bind_AB.aep").replace("\\", "/")
    c0 = log_counts()
    js("app.project.save(new File('%s')); app.project.close(CloseOptions.DO_NOT_SAVE_CHANGES); 'ok'" % aep)
    js("app.open(new File('%s')); CMP().openInViewer(); 'ok'" % aep)
    wait_idle(4.0)
    ok = True
    for n in ("A", "B"):
        nm = names(n)
        say("  %s names after reopen: %s" % (n, nm))
        ok &= verify_names(n, nm)
    for n in ("A", "B"):
        ok &= check_quadrants("reopen", n, render(n, os.path.join(outdir, "reopen_%s.png" % n)))
    c1 = log_counts()
    ok &= log_delta("reopen", c0, c1)

    say("=== aerender leg (fresh process, cold registry) ===")
    # comp 'bind' renders A alone, its duplicate 'bind2' renders B alone (the duplicate carries
    # copies of both instances, i.e. the copy/paste flow, each with its own snapshot).
    applied = js("""var c=CMP(); LYR('A').enabled=true; LYR('B').enabled=false;
        var d=c.duplicate(); d.name='bind2';
        for (var i=1;i<=d.numLayers;i++){ if (d.layer(i).name==='A') d.layer(i).enabled=false; if (d.layer(i).name==='B') d.layer(i).enabled=true; }
        var rq=app.project.renderQueue; while (rq.numItems>0) rq.item(1).remove();
        var comps=[c,d]; var applied=[];
        for (var k=0;k<2;k++){ var it=rq.items.add(comps[k]); it.timeSpanStart=0.5; it.timeSpanDuration=comps[k].frameDuration;
            var om=it.outputModule(1); var done='';
            for (var t=0;t<om.templates.length;t++){ if (om.templates[t]==='PNG Sequence'){ om.applyTemplate('PNG Sequence'); done='PNG Sequence'; break; } }
            if (!done){ for (var t2=0;t2<om.templates.length;t2++){ if (om.templates[t2]==='Photoshop'){ om.applyTemplate('Photoshop'); done='Photoshop'; break; } } }
            om.file=new File('%s/aer_'+comps[k].name+'_[#####]'); applied.push(done); }
        app.project.save(); applied.join(',')""" % outdir.replace("\\", "/"))
    say("  output module templates applied: [%s]" % applied)
    if not applied or "" in applied.split(","):
        say("  no still-image output template (PNG Sequence / Photoshop) on this host; aerender leg cannot be evaluated")
        return False
    exe = AERENDER % year
    if not os.path.exists(exe):
        say("  aerender not found: %s; aerender leg cannot be evaluated" % exe)
        return False
    for f in glob.glob(os.path.join(outdir, "aer_*")):
        os.remove(f)
    t0 = time.time()
    proc = subprocess.run([exe, "-project", aep.replace("/", "\\")], capture_output=True, text=True,
                          encoding="utf-8", errors="replace", timeout=600)
    open(os.path.join(outdir, "aerender_stdout.txt"), "w", encoding="utf-8").write(proc.stdout + "\n" + proc.stderr)
    say("  aerender exit=%s in %.0fs" % (proc.returncode, time.time() - t0))
    for n, comp in (("A", "bind"), ("B", "bind2")):
        files = sorted(glob.glob(os.path.join(outdir, "aer_%s_*" % comp)))
        if not files:
            say("  aerender output for %s missing" % comp)
            ok = False
            continue
        ok &= check_quadrants("aerender", n, load_rgb(files[0]))
    ok &= log_delta("aerender", c1, log_counts())
    return ok


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--out", default=None)
    ap.add_argument("--orders", default="AB,BA")
    ap.add_argument("--smoke", action="store_true", help="only project + A migration, print names")
    ap.add_argument("--reopen", action="store_true", help="after the orders, run order AB again and the reopen + aerender legs")
    ap.add_argument("--year", type=int, default=2026)
    args = ap.parse_args()
    outdir = args.out or os.path.join(HERE, "..", "out", "bind", time.strftime("%Y%m%d_%H%M%S"))
    os.makedirs(outdir, exist_ok=True)
    say("[host] AE %s | %s local | log %%TEMP%%\\%s" % (js("app.version"), time.strftime("%Y-%m-%d %H:%M:%S"), os.path.basename(LOG)))
    all_ok = True
    try:
        if args.smoke:
            new_project()
            add_fx("A")
            say("names before binding: " + names("A").replace("|", " | "))
            js("LYR('A').property('ADBE Effect Parade').property(1).remove(); 'ok'")
            make_A("smoke")
            say("status A: " + status("A"))
            check_quadrants("smoke", "A", render("A", os.path.join(outdir, "smoke_A.png")))
            return 0
        for order in args.orders.split(","):
            all_ok &= run_order(order, outdir)
        if args.reopen:
            all_ok &= run_order("AB", outdir, tag="reopen-src")
            all_ok &= run_reopen(outdir, args.year)
        say("[VERDICT] " + ("PASS — every instance resolved its own values and layer wiring in every leg"
                            if all_ok else "FAIL — see MISMATCH / CHANGED / STRICT lines above"))
    except SystemExit as e:
        say("[ABORTED] %s" % e)
        all_ok = False
    finally:
        # A failing or aborted run must still leave its evidence behind.
        open(os.path.join(outdir, "report.txt"), "w", encoding="utf-8").write("\n".join(report))
        say("report: " + outdir)
    return 0 if all_ok else 1


if __name__ == "__main__":
    sys.exit(main())
