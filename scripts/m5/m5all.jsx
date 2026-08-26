// M5 harness — the whole TR-M5-001 fixture as ONE script driven by
// app.scheduleTask. A single -r launch sidesteps AE's "second script"
// rejection dialogs entirely; the gaps between steps are real idle time
// for the observer.
//
// Numeric evidence comes from sampleImage expression probes (Slider
// Controls on a "probes" null, read back post-expression by the script):
// depth-independent, float64 in the log, no output-module depth fight.
// PSD renders remain only as visible artifacts (the OM Depth key is
// read-only on this host and truncates files to 8-bit — recorded).
#include "m5_lib.jsxinc"

var M5LOG = "m5all.log";

// point tags -> [x,y] pixel coordinates (sampled at pixel centers).
// Sliders are created bare at setup; the expressions are armed in step1.
// Any slider evaluated before the idle observer compiles caches the black
// pass-through result, and that cached entry outlives token writes,
// expression re-assignment, and depth round-trips (all measured) — so no
// expression may exist before compilation.
function m5AddProbes(comp, points, postEffect) {
    var nul = comp.layers.addNull();
    nul.name = postEffect ? "probes" : "probes_src";
    for (var tag in points) {
        if (!points.hasOwnProperty(tag)) { continue; }
        var fx = nul.property("ADBE Effect Parade").addProperty("ADBE Slider Control");
        fx.name = tag;
    }
}
function m5ArmProbes(compName, layerName, points, postEffect) {
    var comp = m5Find(compName);
    var lay = null;
    for (var i = 1; i <= comp.numLayers; i++) {
        if (comp.layer(i).name === layerName) { lay = comp.layer(i); break; }
    }
    for (var tag in points) {
        if (!points.hasOwnProperty(tag)) { continue; }
        var p = points[tag];
        // sampleImage points: integer coordinates are pixel CENTERS
        // (measured: +0.5 offsets bilinear-blend adjacent columns).
        lay.effect(tag)(1).expression =
            'thisComp.layer("input").sampleImage([' + p[0] + ',' + p[1] +
            '],[0.49,0.49],' + (postEffect ? 'true' : 'false') + ',time)[' + p[2] + ']';
    }
}
function m5ReadProbes(logName, layerName, tags, realComp) {
    var comp = m5Find(realComp || logName);
    var lay = null;
    for (var i = 1; i <= comp.numLayers; i++) {
        if (comp.layer(i).name === layerName) { lay = comp.layer(i); break; }
    }
    for (var t = 0; t < tags.length; t++) {
        var v = "ERR";
        try { v = lay.effect(tags[t])(1).valueAtTime(0, false).toFixed(9); }
        catch (e) { v = "ERR " + String(e); }
        m5Log(M5LOG, "PROBE " + logName + " " + tags[t] + " bpc" + app.project.bitsPerChannel + " = " + v);
    }
}
function m5Statuses(tag) {
    var names = ["m5ramp", "m5chain", "m5hdr", "m5hdrchain", "m5alpha"];
    for (var i = 0; i < names.length; i++) {
        var s = "";
        try { s = m5Fx(m5Find(names[i])).property(4).name; } catch (e) {}
        m5Log(M5LOG, "STATUS " + tag + " " + names[i] + " [" + s + "]");
    }
}
function m5Guard(name, body) {
    try { body(); }
    catch (e) {
        m5Log(M5LOG, "SCRIPT_ERROR " + name + " " + String(e) + " (line " + e.line + ")");
        m5Log(M5LOG, "RESULT_DONE");
    }
}

// Ramp probes: p9/p10 are the staircase pair that collapses to one value
// under any 8-bit remnant (round(v*255) maps both to 8); white is the
// self-check reference. Kept away from the frame edge so ROI request
// rects stay inside the layer.
var RAMP_PTS = { p100: [100, 120, 0], p9: [9, 120, 0], p10: [10, 120, 0], white: [160, 15, 0] };
var RAMP_TAGS = ["p100", "p9", "p10", "white"];
var HDR_PTS = { over: [160, 30, 0], neg: [160, 90, 0], one: [160, 150, 0], ramp: [100, 210, 0] };
var HDR_TAGS = ["over", "neg", "one", "ramp"];
var ALPHA_PTS = { rec: [160, 120, 0], white: [160, 15, 0] };
var ALPHA_SRC_PTS = { srcR: [160, 120, 0], srcA: [160, 120, 3] };

function m5Step1() {
    m5Guard("step1", function () {
        // Setup-time slider evaluations (pre-compile, at 16-bpc) are cached
        // per-property and survive token writes, expression re-assignment,
        // and same-block bpc toggles (all measured). Only a REAL depth
        // transition — separate evaluation passes between the two sets —
        // invalidates them, so the 16-bpc reads happen in step1b after this
        // step parks the project at 8-bpc.
        app.project.bitsPerChannel = 16;
        m5Log(M5LOG, "STEP1 arm probes at bpc=" + app.project.bitsPerChannel);
        m5Statuses("step1");
        // Arm all probe expressions now — strictly after the idle compile,
        // so the first evaluation of every slider samples a real render.
        m5ArmProbes("m5ramp", "probes", RAMP_PTS, true);
        m5ArmProbes("m5chain", "probes", RAMP_PTS, true);
        m5ArmProbes("m5hdr", "probes", HDR_PTS, true);
        m5ArmProbes("m5hdrchain", "probes", HDR_PTS, true);
        m5ArmProbes("m5alpha", "probes", ALPHA_PTS, true);
        m5ArmProbes("m5alpha", "probes_src", ALPHA_SRC_PTS, false);
        app.scheduleTask("m5Step1b()", 1500, false);
    });
}
function m5Step1b() {
    m5Guard("step1b", function () {
        app.project.bitsPerChannel = 16;
        m5Log(M5LOG, "STEP1B bpc=" + app.project.bitsPerChannel + " ws=[" + app.project.workingSpace + "]");
        m5ReadProbes("m5ramp", "probes", RAMP_TAGS);
        m5ReadProbes("m5chain", "probes", RAMP_TAGS);
        // Single-pass 16-bpc evidence rides the HDR generator: the bands
        // also pin the ADR-0022 boundary clamp (2.0 -> 1.0, -0.5 -> 0).
        m5ReadProbes("m5hdr", "probes", HDR_TAGS);
        m5RenderPSD(M5LOG, m5Find("m5chain"), "m5_chain16", "Trillions of Colors");
        app.scheduleTask("m5Step2()", 3000, false);
    });
}
function m5Step2() {
    m5Guard("step2", function () {
        app.project.bitsPerChannel = 32;
        m5Log(M5LOG, "STEP2 bpc=" + app.project.bitsPerChannel);
        m5ReadProbes("m5hdr", "probes", HDR_TAGS);
        m5ReadProbes("m5hdrchain", "probes", HDR_TAGS);
        // Visible artifact only (8-bit preview; the numeric evidence is the
        // probe log). The Photoshop OM fails with a modal on this host when
        // fed 32-bpc frames (RQ item "失败", measured), so no PSD here.
        try {
            m5Find("m5hdr").saveFrameToPng(0, new File(m5Out() + "m5_hdr32_preview.png"));
            m5Log(M5LOG, "PNG m5_hdr32_preview saved");
        } catch (eP) { m5Log(M5LOG, "PNG_ERR " + String(eP)); }
        app.scheduleTask("m5Step3()", 3000, false);
    });
}
function m5Step3() {
    m5Guard("step3", function () {
        app.project.bitsPerChannel = 8;
        m5Log(M5LOG, "STEP3 bpc=" + app.project.bitsPerChannel);
        m5ReadProbes("m5ramp", "probes", RAMP_TAGS);
        app.scheduleTask("m5Step4()", 2000, false);
    });
}
function m5Step4() {
    m5Guard("step4", function () {
        var depths = [8, 16, 32];
        for (var i = 0; i < depths.length; i++) {
            app.project.bitsPerChannel = depths[i];
            m5ReadProbes("m5alpha", "probes", ["rec", "white"]);
            m5ReadProbes("m5alpha", "probes_src", ["srcR", "srcA"]);
        }
        app.scheduleTask("m5Step5()", 2000, false);
    });
}
function m5Step5() {
    m5Guard("step5", function () {
        app.project.bitsPerChannel = 32;
        try { app.project.workingSpace = ""; } catch (eU) {}
        m5Log(M5LOG, "COLOR unmanaged ws=[" + app.project.workingSpace + "]");
        m5ReadProbes("m5hdr", "probes", HDR_TAGS);

        // ACES ICC names raise a modal "profile missing (83::0)" dialog on
        // this host (measured; OCIO ACES is engine-level and unscriptable) —
        // the managed leg uses the universally installed sRGB profile.
        var candidates = ["sRGB IEC61966-2.1"];
        var stuck = "";
        for (var i = 0; i < candidates.length && stuck === ""; i++) {
            try {
                app.project.workingSpace = candidates[i];
                if (app.project.workingSpace !== "" && app.project.workingSpace !== "None") {
                    stuck = app.project.workingSpace;
                }
            } catch (eM) {}
        }
        m5Log(M5LOG, "COLOR managed ws=[" + stuck + "]");
        if (stuck !== "") { m5ReadProbes("m5hdr", "probes", HDR_TAGS); }
        try { app.project.workingSpace = ""; } catch (eR) {}
        m5Log(M5LOG, "RESULT M5ALL complete");
        m5Log(M5LOG, "RESULT_DONE");
    });
}

(function () {
    new Folder(m5Out()).create();
    m5Guard("setup", function () {
        try { app.project.close(CloseOptions.DO_NOT_SAVE_CHANGES); } catch (eC) {}
        app.newProject();
        app.project.bitsPerChannel = 16;
        try { app.project.workingSpace = ""; } catch (eW) {}
        m5Log(M5LOG, "SETUP bpc=" + app.project.bitsPerChannel + " ws=[" + app.project.workingSpace + "]");

        m5AddProbes(m5NewShaderComp("m5ramp", m5RampWhite()), RAMP_PTS, true);
        m5AddProbes(m5NewShaderComp("m5chain", m5Chain(m5RampWhite())), RAMP_PTS, true);
        m5AddProbes(m5NewShaderComp("m5hdr", m5Hdr()), HDR_PTS, true);
        m5AddProbes(m5NewShaderComp("m5hdrchain", m5Chain(m5Hdr())), HDR_PTS, true);

        var src = app.project.items.addComp("m5src", 320, 240, 1.0, 1.0, 25);
        var red = src.layers.addSolid([1, 0, 0], "red", 320, 240, 1.0, 1.0);
        red.property("ADBE Transform Group").property("ADBE Opacity").setValue(50);
        var alpha = app.project.items.addComp("m5alpha", 320, 240, 1.0, 1.0, 25);
        var lay = alpha.layers.add(src);
        lay.name = "input";
        var fx = lay.property("ADBE Effect Parade").addProperty("DynamicFx");
        fx.property(3).expression = "`" + m5AlphaProbe() + "`;0";
        m5AddProbes(alpha, ALPHA_PTS, true);
        m5AddProbes(alpha, ALPHA_SRC_PTS, false);

        m5Log(M5LOG, "SETUP comps ready; idle window scheduled");
        app.scheduleTask("m5Step1()", 15000, false);
    });
})();
