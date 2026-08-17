// M3 harness — D: after the idle window in the reopened project, the slot
// UI must be restored (label "gain") and the frame must still be exact.
(function () {
    var OUT = ($.getenv("DFX_M3_OUT") || "E:/Code/AePlugin_Dynamicfx/scripts/out/m3/dev") + "/";
    new Folder(OUT).create();
    function logLine(s) {
        var f = new File(OUT + "m3d.log");
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
        var slot1 = "", status = "", keys = -1;
        try { slot1 = fx.property(6).name; } catch (e1) {}
        try { status = fx.property(4).name; } catch (e2) {}
        try { keys = fx.property(6).numKeys; } catch (e3) {}
        comp.openInViewer();
        comp.saveFrameToPng(0.4, new File(OUT + "m3d_t04.png"));
        logLine("RESTORED slot1=[" + slot1 + "] status=[" + status + "] numKeys=" + keys);
        logLine("RESULT M3D slot1=[" + slot1 + "] numKeys=" + keys);
    } catch (e) {
        logLine("SCRIPT_ERROR " + String(e));
    }
    logLine("RESULT_DONE");
})();
