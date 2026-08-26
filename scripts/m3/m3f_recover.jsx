// M3 harness — F: after the idle window, the committed expression must have
// recovered the corrupted-snapshot project: recompiled, republished, and
// rendering exactly again (the snapshot is a carrier, never the authority).
(function () {
    var OUT = ($.getenv("DFX_M3_OUT") || "E:/Code/AePlugin_Dynamicfx/scripts/out/m3/dev") + "/";
    new Folder(OUT).create();
    function logLine(s) {
        var f = new File(OUT + "m3f.log");
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
        comp.openInViewer();
        comp.saveFrameToPng(0.4, new File(OUT + "m3f_recover.png"));
        var status = "", tokenValue = -1, keys = -1;
        try { status = fx.property(5).name; } catch (e1) {}
        try { tokenValue = fx.property(6).value; } catch (e2) {}
        try { keys = fx.property(14).numKeys; } catch (e3) {}
        logLine("RECOVERED status=[" + status + "] token_nonzero=" + (tokenValue > 0 ? 1 : 0) + " numKeys=" + keys);
        logLine("PNG saved m3f_recover.png expect (51,51,0) again");
        logLine("RESULT M3F token_nonzero=" + (tokenValue > 0 ? 1 : 0) + " numKeys=" + keys);
    } catch (e) {
        logLine("SCRIPT_ERROR " + String(e));
    }
    logLine("RESULT_DONE");
})();
