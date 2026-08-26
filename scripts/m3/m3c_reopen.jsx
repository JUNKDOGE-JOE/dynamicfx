// M3 harness — C (fresh AE process): open the saved project and render
// IMMEDIATELY — no Compile click. The render clone must resolve through the
// snapshot (registry is empty in this process). Keyframes at t=0.4 must
// produce (51,51,0) at center: uv(0.5,0.5) * gain(0.4) * 255.
(function () {
    var OUT = ($.getenv("DFX_M3_OUT") || "E:/Code/AePlugin_Dynamicfx/scripts/out/m3/dev") + "/";
    new Folder(OUT).create();
    function logLine(s) {
        var f = new File(OUT + "m3c.log");
        f.encoding = "UTF-8";
        if (f.open("a")) { f.write(s + "\n"); f.close(); }
    }
    try {
        try { app.project.close(CloseOptions.DO_NOT_SAVE_CHANGES); } catch (eC) {}
        app.open(new File(OUT + "m3.aep"));
        var comp = null;
        for (var j = 1; j <= app.project.numItems; j++) {
            if (app.project.item(j) instanceof CompItem && app.project.item(j).name === "m3") {
                comp = app.project.item(j);
                break;
            }
        }
        var fx = comp.layer(1).property("ADBE Effect Parade").property(1);
        var keys = -1, tokenValue = -1;
        try { keys = fx.property(14).numKeys; } catch (e1) {}
        try { tokenValue = fx.property(6).value; } catch (e2) {}

        comp.openInViewer();
        comp.saveFrameToPng(0.4, new File(OUT + "m3c_reopen.png"));
        logLine("REOPEN numKeys=" + keys + " token_nonzero=" + (tokenValue > 0 ? 1 : 0));
        logLine("PNG saved m3c_reopen.png expect (51,51,0) via snapshot path, no Compile click");
        logLine("RESULT M3C numKeys=" + keys);
    } catch (e) {
        logLine("SCRIPT_ERROR " + String(e));
    }
    logLine("RESULT_DONE");
})();
