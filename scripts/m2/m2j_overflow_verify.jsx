// M2 harness — J: after the idle window, the 49-float definition must be
// atomically rejected: diagnostic in Status, token zeroed, frame passed
// through (solid 10,200,30).
(function () {
    var OUT = ($.getenv("DFX_M2_OUT") || "E:/Code/AePlugin_Dynamicfx/scripts/out/m2/dev") + "/";
    new Folder(OUT).create();
    function logLine(s) {
        var f = new File(OUT + "m2j.log");
        f.encoding = "UTF-8";
        if (f.open("a")) { f.write(s + "\n"); f.close(); }
    }
    try {
        var comp = null;
        for (var j = 1; j <= app.project.numItems; j++) {
            if (app.project.item(j) instanceof CompItem && app.project.item(j).name === "m2k") {
                comp = app.project.item(j);
                break;
            }
        }
        var fx = comp.layer(1).property("ADBE Effect Parade").property(1);

        comp.openInViewer();
        comp.saveFrameToPng(0, new File(OUT + "m2i_overflow.png"));

        var statusName = "", tokenValue = -1;
        try { statusName = fx.property(5).name; } catch (e1) {}
        try { tokenValue = fx.property(6).value; } catch (e2) {}
        logLine("OVERFLOW_STATUS [" + statusName + "] token=" + tokenValue);
        logLine("PNG saved m2i_overflow.png expect passthrough solid (10,200,30)");
        logLine("RESULT M2J status=[" + statusName + "] token=" + tokenValue);
    } catch (e) {
        logLine("SCRIPT_ERROR " + String(e));
    }
    logLine("RESULT_DONE");
})();
