// M6 harness — TR-M6-001 windowed re-simulation (ADR-0025) as ONE
// scheduleTask script. Every frame is self-contained: value at frame F =
// min(F+1, W) * step, in ANY evaluation order. The probe plan reads frames
// SHUFFLED on purpose; the RQ render is the MFR-concurrent leg.
#include "m6_lib.jsxinc"

var M6LOG = "m6all.log";

function m6Guard(name, body) {
    try { body(); }
    catch (e) {
        m6Log(M6LOG, "SCRIPT_ERROR " + name + " " + String(e) + " (line " + e.line + ")");
        m6Log(M6LOG, "RESULT_DONE");
    }
}

function m6ReadAt(compName, frame) {
    var comp = m6Find(compName);
    var lay = null;
    for (var i = 1; i <= comp.numLayers; i++) {
        if (comp.layer(i).name === "probes") { lay = comp.layer(i); break; }
    }
    var t = frame * comp.frameDuration;
    var v = "ERR";
    try { v = lay.effect("acc")(1).valueAtTime(t, false).toFixed(9); }
    catch (e) { v = "ERR " + String(e); }
    m6Log(M6LOG, "PROBE " + compName + " frame " + frame + " = " + v);
}

function m6Step1() {
    m6Guard("step1", function () {
        m6ArmProbe("m6acc8");
        m6ArmProbe("m6acc32");
        app.scheduleTask("m6Step2()", 2000, false);
    });
}
function m6Step2() {
    m6Guard("step2", function () {
        app.project.bitsPerChannel = 8;
        m6Log(M6LOG, "STEP2 shuffled reads at bpc=8 (W=16)");
        // Deliberately shuffled: ramp, plateau, backwards, repeats — every
        // value must be exact regardless (ADR-0025 §2).
        var frames = [5, 0, 12, 3, 20, 8, 30, 12, 1, 15, 16, 7];
        for (var i = 0; i < frames.length; i++) { m6ReadAt("m6acc8", frames[i]); }
        app.scheduleTask("m6Step3()", 2000, false);
    });
}
function m6Step3() {
    m6Guard("step3", function () {
        app.project.bitsPerChannel = 32;
        m6Log(M6LOG, "STEP3 shuffled reads at bpc=32 (W=16)");
        var frames = [10, 2, 0, 25, 6, 15, 40, 6];
        for (var i = 0; i < frames.length; i++) { m6ReadAt("m6acc32", frames[i]); }
        app.scheduleTask("m6Step4()", 2000, false);
    });
}
function m6Step4() {
    m6Guard("step4", function () {
        // Recompile with a smaller window: the plateau must follow W.
        var fx = m6Fx(m6Find("m6acc32"));
        fx.property(2).expression = "`" + m6AccSource("1.0/64.0", 8) + "`;0";
        m6Log(M6LOG, "STEP4 recompiled with @window 8");
        app.scheduleTask("m6Step5()", 14000, false);
    });
}
function m6Step5() {
    m6Guard("step5", function () {
        m6Log(M6LOG, "STEP5 post-recompile reads (W=8)");
        m6ReadAt("m6acc32", 20);
        m6ReadAt("m6acc32", 3);
        // MFR leg: RQ render of the untouched comp, frames 0..24.
        app.project.bitsPerChannel = 8;
        try { app.purge(PurgeTarget.ALL_CACHES); } catch (eP) {}
        var rq = app.project.renderQueue;
        var item = rq.items.add(m6Find("m6rq"));
        item.render = true;
        try {
            item.timeSpanStart = 0;
            item.timeSpanDuration = 25 * m6Find("m6rq").frameDuration;
        } catch (eT) {}
        var om = item.outputModule(1);
        var applied = "";
        for (var t = 0; t < om.templates.length; t++) {
            if (om.templates[t] === "Photoshop") { om.applyTemplate("Photoshop"); applied = "Photoshop"; break; }
        }
        om.file = new File(m6Out() + "m6_rq_[#####].psd");
        m6Log(M6LOG, "RQ template=[" + applied + "]");
        try {
            rq.render();
            m6Log(M6LOG, "RENDERED rq status=" + String(item.status));
        } catch (eX) { m6Log(M6LOG, "RENDER_ERR " + String(eX)); }
        try { item.remove(); } catch (eR) {}
        var aep = new File(m6Out() + "m6.aep");
        app.project.save(aep);
        m6Log(M6LOG, "SAVED " + aep.fsName);
        m6Log(M6LOG, "RESULT M6ALL complete");
        m6Log(M6LOG, "RESULT_DONE");
    });
}

(function () {
    new Folder(m6Out()).create();
    m6Guard("setup", function () {
        try { app.project.close(CloseOptions.DO_NOT_SAVE_CHANGES); } catch (eC) {}
        app.newProject();
        app.project.bitsPerChannel = 8;
        try { app.project.workingSpace = ""; } catch (eW) {}
        m6Log(M6LOG, "SETUP bpc=" + app.project.bitsPerChannel + " ws=[" + app.project.workingSpace + "]");
        m6NewAccComp("m6acc8", "4.0/255.0");
        m6NewAccComp("m6acc32", "1.0/64.0");
        m6NewAccComp("m6rq", "4.0/255.0");
        m6Log(M6LOG, "SETUP comps ready; idle window scheduled");
        app.scheduleTask("m6Step1()", 15000, false);
    });
})();
