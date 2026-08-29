"""TR-CUI-001 - custom-UI crash bisection driver (AE 2025, warm panel session).

The 2026-08-15 gradient editor crashed AE on expand with zero log lines and no
custom-UI event delivered (TR-0031-001). catch_panics was already active, so
the death was an access violation, not a Rust panic - the missing instrument
was a crash dump. This driver runs the bisection legs with WER LocalDumps
armed (scripts/out/cui/cui_setup_elevated.ps1) and the probe's own event log
as the last-line-before-death marker.

Per-leg cycle (the probe AEX is locked while AE runs, so swaps need AE down):

  python scripts/cui/tr_cui_001.py install u1      # base|u1|u2|u2b|all
  python scripts/cui/tr_cui_001.py start
  python scripts/cui/tr_cui_001.py leg probe       # or: probe-muted | ecw | colorgrid
  python scripts/cui/tr_cui_001.py quit
  python scripts/cui/tr_cui_001.py forensics       # after any crash

Legs:
  probe        fresh project, one solid, apply DynamicFxProbe with the layer
               selected -> ECW renders the rows -> Draw path runs (or dies).
  probe-muted  U2a: apply with nothing selected, set Mute Draw, then select -
               separates "dies before any event" from "dies in the draw path".
  ecw          apply the patched upstream Custom_ECW_UI (standard-param canvas).
  colorgrid    apply the patched upstream ColorGrid (arb + custom UI).
"""
import hashlib
import json
import os
import subprocess
import sys
import time

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, ".."))
import aemcp

REPO = os.path.abspath(os.path.join(HERE, "..", ".."))
OUT = os.path.join(REPO, "scripts", "out", "cui")
DUMPS = os.path.join(OUT, "dumps")
LEGLOG = os.path.join(OUT, "legs.log")

AE_EXE = r"C:\Program Files\Adobe\Adobe After Effects 2025\Support Files\AfterFX.exe"
PLUG = r"C:\Program Files\Adobe\Adobe After Effects 2025\Support Files\Plug-ins"
PROBE_DST = os.path.join(PLUG, "DynamicFxProbe", "DynamicFxProbe.aex")
SAMPLES_DST = os.path.join(PLUG, "DynamicFxCuiSamples")
PROBE_SRC = os.path.join(REPO, "spike", "probe", "out")
SAMPLES_SRC = r"E:\Code\_refs\after-effects\target\release"
PROBE_LOG = os.path.join(os.environ.get("TEMP", r"C:\Windows\Temp"), "dynamicfx_probe.log")

VARIANTS = {
    "base": "DynamicFxProbe-base.aex",
    "u1": "DynamicFxProbe-u1.aex",
    "u2": "DynamicFxProbe-u2.aex",
    "u2b": "DynamicFxProbe-u2b.aex",
    "all": "DynamicFxProbe-u1-u2-u2b.aex",
    "u1nil": "DynamicFxProbe-u1nil.aex",
}
SAMPLES = {"Custom_ECW_UI.aex": "custom_ecw_ui.dll", "ColorGrid.aex": "colorgrid.dll"}


def sha256(path):
    h = hashlib.sha256()
    with open(path, "rb") as f:
        for chunk in iter(lambda: f.read(1 << 20), b""):
            h.update(chunk)
    return h.hexdigest().upper()


def ae_running():
    out = subprocess.run(
        ["tasklist", "/FI", "IMAGENAME eq AfterFX.exe", "/FO", "CSV", "/NH"],
        capture_output=True, text=True,
    ).stdout
    return "AfterFX.exe" in out


def record(entry):
    entry["t"] = time.strftime("%Y-%m-%dT%H:%M:%S")
    os.makedirs(OUT, exist_ok=True)
    with open(LEGLOG, "a", encoding="utf-8") as f:
        f.write(json.dumps(entry, ensure_ascii=False) + "\n")
    print(json.dumps(entry, ensure_ascii=False, indent=2))


def probe_log_size():
    try:
        return os.path.getsize(PROBE_LOG)
    except OSError:
        return 0


def probe_log_delta(offset):
    try:
        with open(PROBE_LOG, "r", encoding="utf-8", errors="replace") as f:
            f.seek(offset)
            return f.read()
    except OSError:
        return ""


def cmd_install(variant):
    if ae_running():
        sys.exit("AE is running; the loaded AEX cannot be swapped. Quit it first.")
    src = os.path.join(PROBE_SRC, VARIANTS[variant])
    data = open(src, "rb").read()
    with open(PROBE_DST, "wb") as f:
        f.write(data)
    installed = sha256(PROBE_DST)
    assert installed == sha256(src), "post-copy hash mismatch"
    record({"cmd": "install", "variant": variant, "sha256": installed})


def cmd_samples(state):
    if ae_running():
        sys.exit("AE is running; quit it before changing installed samples.")
    changed = {}
    for dst_name, src_name in SAMPLES.items():
        dst = os.path.join(SAMPLES_DST, dst_name)
        if state == "on":
            data = open(os.path.join(SAMPLES_SRC, src_name), "rb").read()
            with open(dst, "wb") as f:
                f.write(data)
            changed[dst_name] = sha256(dst)
        elif os.path.exists(dst):
            os.remove(dst)
            changed[dst_name] = "removed"
    record({"cmd": "samples", "state": state, "files": changed})


def cmd_start():
    if not ae_running():
        subprocess.Popen([AE_EXE], creationflags=subprocess.DETACHED_PROCESS)
    deadline = time.time() + 180
    while time.time() < deadline:
        try:
            if aemcp.health():
                record({"cmd": "start", "health": "ok", "port": aemcp.port()})
                return
        except SystemExit:
            pass
        if not ae_running() and time.time() > deadline - 150:
            record({"cmd": "start", "health": "AE_PROCESS_DIED_DURING_STARTUP"})
            return
        time.sleep(2)
    record({"cmd": "start", "health": "TIMEOUT (panel not reachable; recovery modal wedging startup?)"})


LEG_JSX = r"""
(function(){
  // Close-without-saving first: newProject() on a dirty project pops a save
  // modal, which wedges the panel's JSX bridge. Never open the ECW by menu
  // command here - findMenuCommandId against English names raises a modal
  // error box on this localized host; the docked panel renders on selection.
  try { app.project.close(CloseOptions.DO_NOT_SAVE_CHANGES); } catch(e) {}
  app.newProject();
  // Save immediately: an untitled project makes AE's auto-save pop a full
  // save browser (observed twice), which wedges the bridge like any modal.
  app.project.save(new File('E:/Code/AePlugin_Dynamicfx/scripts/out/cui/cui_session.aep'));
  var comp = app.project.items.addComp('cui', 200, 120, 1, 2, 25);
  comp.layers.addSolid([0.5,0.5,0.5], 'target', 200, 120, 1);
  comp.openInViewer();
  var lyr = comp.layer(1);
  lyr.selected = __PRESELECT__;
  var fx = lyr.property('ADBE Effect Parade').addProperty('__MATCH__');
  __MUTE__
  lyr.selected = true;
  return '{"applied":"' + fx.matchName + '","props":' + fx.numProperties + '}';
})()
"""

LEGS = {
    "probe": ("DynamicFxProbe", "true", ""),
    "probe-muted": ("DynamicFxProbe", "false", "fx.property('Mute Draw (U2a)').setValue(1);"),
    "ecw": ("ADBE Custom_ECW_UI", "true", ""),
    "colorgrid": ("ADBE ColorGrid", "true", ""),
}


def cmd_leg(name):
    match, preselect, mute = LEGS[name]
    offset = probe_log_size()
    jsx = LEG_JSX.replace("__MATCH__", match).replace("__PRESELECT__", preselect).replace("__MUTE__", mute)
    result = aemcp.exec_js(jsx, timeout_ms=30000)
    time.sleep(3)  # let ECW paint and the probe log flush before sampling
    alive = ae_running()
    record({
        "cmd": "leg", "leg": name, "exec": result, "ae_alive_after": alive,
        "probe_log_delta": probe_log_delta(offset)[-4000:],
    })


def cmd_quit():
    try:
        result = aemcp.exec_js(
            "try { app.project.close(CloseOptions.DO_NOT_SAVE_CHANGES); } catch(e) {} app.quit();",
            timeout_ms=15000,
        )
    except OSError as e:
        # AE tearing down closes the panel's socket mid-response; that IS the quit.
        result = {"ok": True, "note": f"connection dropped during quit ({e.__class__.__name__})"}
    time.sleep(4)
    record({"cmd": "quit", "exec": result, "ae_alive_after": ae_running()})


def cmd_forensics():
    ps = (
        "Get-WinEvent -FilterHashtable @{LogName='Application'; Id=1000} -MaxEvents 20 | "
        "Where-Object { $_.Message -match 'AfterFX' } | Select-Object -First 3 | "
        "ForEach-Object { '=== ' + $_.TimeCreated.ToString('s'); $_.Message }"
    )
    events = subprocess.run(["powershell", "-NoProfile", "-Command", ps], capture_output=True, text=True).stdout
    dumps = sorted(
        (os.path.join(DUMPS, f) for f in os.listdir(DUMPS)) if os.path.isdir(DUMPS) else [],
        key=os.path.getmtime, reverse=True,
    )[:3]
    record({
        "cmd": "forensics",
        "event_1000": events.strip()[-3000:] or "none found",
        "newest_dumps": [(d, os.path.getsize(d)) for d in dumps],
    })


def cmd_status():
    installed = sha256(PROBE_DST) if os.path.exists(PROBE_DST) else "ABSENT"
    variant = next((v for v, f in VARIANTS.items()
                    if os.path.exists(os.path.join(PROBE_SRC, f)) and sha256(os.path.join(PROBE_SRC, f)) == installed),
                   "UNKNOWN")
    health = None
    try:
        health = bool(aemcp.health())
    except SystemExit:
        health = False
    record({"cmd": "status", "ae_running": ae_running(), "panel": health,
            "installed_sha256": installed, "installed_variant": variant})


if __name__ == "__main__":
    args = sys.argv[1:]
    if not args:
        sys.exit(__doc__)
    cmd = args[0]
    if cmd == "install":
        cmd_install(args[1])
    elif cmd == "samples":
        cmd_samples(args[1])
    elif cmd == "start":
        cmd_start()
    elif cmd == "leg":
        cmd_leg(args[1])
    elif cmd == "quit":
        cmd_quit()
    elif cmd == "forensics":
        cmd_forensics()
    elif cmd == "status":
        cmd_status()
    else:
        sys.exit(f"unknown subcommand {cmd}")
