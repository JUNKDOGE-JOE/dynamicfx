// M3 harness — E (fresh AE process): open the deliberately corrupted copy.
// The snapshot must fail closed (SnapshotCorrupt); the first render may pass
// through. m3f verifies the expression path recovers after the idle window.
(function () {
    var OUT = ($.getenv("DFX_M3_OUT") || "E:/Code/AePlugin_Dynamicfx/scripts/out/m3/dev") + "/";
    new Folder(OUT).create();
    function logLine(s) {
        var f = new File(OUT + "m3e.log");
        f.encoding = "UTF-8";
        if (f.open("a")) { f.write(s + "\n"); f.close(); }
    }
    try {
        try { app.project.close(CloseOptions.DO_NOT_SAVE_CHANGES); } catch (eC) {}
        app.open(new File(OUT + "m3_corrupt.aep"));
        var comp = null;
        for (var j = 1; j <= app.project.numItems; j++) {
            if (app.project.item(j) instanceof CompItem && app.project.item(j).name === "m3") {
                comp = app.project.item(j);
                break;
            }
        }
        var fx = comp.layer(1).property("ADBE Effect Parade").property(1);
        comp.openInViewer();
        comp.saveFrameToPng(0.4, new File(OUT + "m3e_corrupt.png"));
        var status = "";
        try { status = fx.property(5).name; } catch (e1) {}
        logLine("CORRUPT_OPEN status=[" + status + "] (first frame may be passthrough)");
        logLine("RESULT M3E opened=1");
    } catch (e) {
        logLine("SCRIPT_ERROR " + String(e));
    }
    logLine("RESULT_DONE");
})();
