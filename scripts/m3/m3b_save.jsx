// M3 harness — B: after the idle window, verify the token published, add
// keyframes, stage the aerender queue (single frame at t=0.4), and save.
(function () {
    var OUT = ($.getenv("DFX_M3_OUT") || "E:/Code/AePlugin_Dynamicfx/scripts/out/m3/dev") + "/";
    new Folder(OUT).create();
    function logLine(s) {
        var f = new File(OUT + "m3b.log");
        f.encoding = "UTF-8";
        if (f.open("a")) { f.write(s + "\n"); f.close(); }
    }
    try {
        var comp = null;
        for (var j = 1; j <= app.project.numItems; j++) {
            if (app.project.item(j) instanceof CompItem && app.project.item(j).name === "m3") {
                comp = app.project.item(j);
                break;
            }
        }
        var fx = comp.layer(1).property("ADBE Effect Parade").property(1);

        var tokenValue = -1, slot1 = "";
        try { tokenValue = fx.property(5).value; } catch (e1) {}
        try { slot1 = fx.property(6).name; } catch (e2) {}
        logLine("PUBLISHED token_nonzero=" + (tokenValue > 0 ? 1 : 0) + " slot1=[" + slot1 + "]");

        var gain = fx.property(6);
        gain.setValueAtTime(0, 0.0);
        gain.setValueAtTime(0.8, 0.8);
        logLine("KEYFRAMES numKeys=" + gain.numKeys);

        var rqItem = app.project.renderQueue.items.add(comp);
        rqItem.render = true;
        try { rqItem.timeSpanStart = 0.4; rqItem.timeSpanDuration = comp.frameDuration; } catch (eT) {}
        var om = rqItem.outputModule(1);
        var applied = "";
        for (var t = 0; t < om.templates.length; t++) {
            if (om.templates[t] === "Photoshop") { om.applyTemplate("Photoshop"); applied = "Photoshop"; break; }
        }
        try { om.file = new File(OUT + "m3_ar_[#####].psd"); } catch (eF) {}
        logLine("RQ applied=[" + applied + "] span=0.4");

        var aep = new File(OUT + "m3.aep");
        app.project.save(aep);
        logLine("AEP saved m3.aep");
        logLine("RESULT M3B token_nonzero=" + (tokenValue > 0 ? 1 : 0) + " numKeys=" + gain.numKeys);
    } catch (e) {
        logLine("SCRIPT_ERROR " + String(e));
    }
    logLine("RESULT_DONE");
})();
