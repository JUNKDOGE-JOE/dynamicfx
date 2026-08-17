// M1 harness — C / TR-M1-004 GUI leg: after the driver's idle window, read
// the Status parameter name (renamed by the plugin) and save the rendered
// frame. Numeric verification happens in the driver via check_png.py.
(function () {
    var OUT = ($.getenv("DFX_M1_OUT") || "E:/Code/AePlugin_Dynamicfx/scripts/out/m1/dev") + "/";
    new Folder(OUT).create();
    function logLine(s) {
        var f = new File(OUT + "m1c.log");
        f.encoding = "UTF-8";
        if (f.open("a")) { f.write(s + "\n"); f.close(); }
    }
    try {
        var comp = null;
        for (var j = 1; j <= app.project.numItems; j++) {
            if (app.project.item(j) instanceof CompItem && app.project.item(j).name === "m1") {
                comp = app.project.item(j);
                break;
            }
        }
        if (!comp) {
            logLine("SCRIPT_ERROR comp m1 not found");
            logLine("RESULT_DONE");
            return;
        }
        var fx = comp.layer(1).property("ADBE Effect Parade").property(1);
        // Render first: the status text set by the idle observer lands on
        // the next UI callback, which viewing/rendering provides.
        comp.openInViewer();
        comp.saveFrameToPng(0, new File(OUT + "m1c_gui.png"));
        logLine("GUI_PNG saved m1c_gui.png expect uv gradient (R=x, G=y, B=0, A=255)");

        var statusName = "";
        try { statusName = fx.property(4).name; } catch (eS) {}
        var tokenValue = -1;
        try { tokenValue = fx.property(5).value; } catch (eT) {}
        logLine("STATUS [" + statusName + "] token_nonzero=" + (tokenValue > 0 ? 1 : 0));
        logLine("RESULT M1C status=[" + statusName + "] token_nonzero=" + (tokenValue > 0 ? 1 : 0));
    } catch (e) {
        logLine("SCRIPT_ERROR " + String(e));
    }
    logLine("RESULT_DONE");
})();
