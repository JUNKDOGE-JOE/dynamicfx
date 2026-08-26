// M7 baseline — TR-M7-001 benchmark matrix (audit 07) as ONE scheduleTask
// script. Requires the host to have been launched with DYNAMICFX_PERF=1
// (cold-start gate): every render then logs a perf: line; scenes are
// separated by SCENE begin/end epochs in m7bench.log.
#include "m7_lib.jsxinc"

function m7Guard(name, body) {
    try { body(); }
    catch (e) {
        m7Log("SCRIPT_ERROR " + name + " " + String(e) + " (line " + e.line + ")");
        m7Log("RESULT_DONE");
    }
}

// Readiness: an instance is renderable exactly when its StateToken stream
// (property 5) decodes to the Active state — the same word render clones
// resolve. The token is `(payload << 2) | state`, so Active is `% 4 === 1`;
// non-zero alone is NOT readiness, because every published diagnostic
// (state 0b10) is also non-zero — including E53 "publication pending",
// which is precisely the not-ready case this poll must not pass.
// Blind waits failed twice (idle-bridge latency; a modal blocking the
// window): idle hooks cannot fire while a script holds the main thread, so
// readiness must be reached by scheduleTask polling, never by $.sleep.
var M7_POLLS = 0;
function m7CountReady() {
    var names = ["m7grad720", "m7grad1080", "m7grad4k", "m7thermal720", "m7thermal4k", "m7temporal720", "m7multi720"];
    var ready = 0, total = 0;
    for (var c = 0; c < names.length; c++) {
        var comp = m7Find(names[c]);
        if (!comp) { continue; }
        for (var i = 1; i <= comp.numLayers; i++) {
            var fxs = comp.layer(i).property("ADBE Effect Parade");
            if (!fxs || fxs.numProperties < 1) { continue; }
            total++;
            try { if (fxs.property(2).property(5).value % 4 === 1) { ready++; } } catch (e) {}
        }
    }
    return { ready: ready, total: total };
}
function m7Poll() {
    m7Guard("poll", function () {
        var state = m7CountReady();
        M7_POLLS++;
        if (state.ready >= state.total && state.total >= 10) {
            m7Log("READY " + state.ready + "/" + state.total + " after " + M7_POLLS + " poll(s)");
            app.scheduleTask("m7Step1()", 1000, false);
        } else if (M7_POLLS > 90) {
            m7Log("SCRIPT_ERROR poll timeout: ready " + state.ready + "/" + state.total);
            m7Log("RESULT_DONE");
        } else {
            if (M7_POLLS % 5 === 1) { m7Log("WAIT ready " + state.ready + "/" + state.total); }
            app.scheduleTask("m7Poll()", 3000, false);
        }
    });
}

function m7Step1() {
    m7Guard("step1", function () {
        m7Log("STEP1 gradient scenes at bpc=8");
        m7RenderScene("m7grad720", "grad720_8");
        m7RenderScene("m7grad1080", "grad1080_8");
        m7RenderScene("m7grad4k", "grad4k_8");
        app.scheduleTask("m7Step2()", 1500, false);
    });
}
function m7Step2() {
    m7Guard("step2", function () {
        m7Log("STEP2 thermal scenes at bpc=8");
        m7RenderScene("m7thermal720", "thermal720_8");
        m7RenderScene("m7thermal4k", "thermal4k_8");
        app.scheduleTask("m7Step3()", 1500, false);
    });
}
function m7Step3() {
    m7Guard("step3", function () {
        m7Log("STEP3 temporal + multi at bpc=8");
        m7RenderScene("m7temporal720", "temporal720_8");
        m7RenderScene("m7multi720", "multi720_8");
        app.scheduleTask("m7Step4()", 1500, false);
    });
}
function m7Step4() {
    m7Guard("step4", function () {
        app.project.bitsPerChannel = 32;
        m7Log("STEP4 gradient scenes at bpc=32");
        m7RenderScene("m7grad720", "grad720_32");
        m7RenderScene("m7grad1080", "grad1080_32");
        m7RenderScene("m7grad4k", "grad4k_32");
        app.project.bitsPerChannel = 8;
        var aep = new File(m7Out() + "m7.aep");
        app.project.save(aep);
        m7Log("SAVED " + aep.fsName);
        m7Log("RESULT M7BENCH complete");
        m7Log("RESULT_DONE");
    });
}

(function () {
    new Folder(m7Out()).create();
    m7Guard("setup", function () {
        try { app.project.close(CloseOptions.DO_NOT_SAVE_CHANGES); } catch (eC) {}
        app.newProject();
        app.project.bitsPerChannel = 8;
        try { app.project.workingSpace = ""; } catch (eW) {}
        m7Log("SETUP bpc=" + app.project.bitsPerChannel + " ws=[" + app.project.workingSpace + "]");
        var grad = m7GradSource();
        m7NewScene("m7grad720", 1280, 720, grad, 1);
        m7NewScene("m7grad1080", 1920, 1080, grad, 1);
        m7NewScene("m7grad4k", 3840, 2160, grad, 1);
        var thermal = m7ThermalSource();
        m7NewScene("m7thermal720", 1280, 720, thermal, 1);
        m7NewScene("m7thermal4k", 3840, 2160, thermal, 1);
        m7NewScene("m7temporal720", 1280, 720, m7TemporalSource(16), 1);
        m7NewScene("m7multi720", 1280, 720, grad, 4);
        m7Log("SETUP comps ready; polling StateToken readiness");
        app.scheduleTask("m7Poll()", 5000, false);
    });
})();
