// M2 harness — G: after the rename idle window, the slot must carry the new
// label ("Volume"), keep both keyframes (alias inheritance), ignore the new
// default, and render the interpolated value.
(function () {
    var OUT = ($.getenv("DFX_M2_OUT") || "E:/Code/AePlugin_Dynamicfx/scripts/out/m2/dev") + "/";
    new Folder(OUT).create();
    function logLine(s) {
        var f = new File(OUT + "m2g.log");
        f.encoding = "UTF-8";
        if (f.open("a")) { f.write(s + "\n"); f.close(); }
    }
    try {
        var comp = null;
        for (var j = 1; j <= app.project.numItems; j++) {
            if (app.project.item(j) instanceof CompItem && app.project.item(j).name === "m2x") {
                comp = app.project.item(j);
                break;
            }
        }
        var fx = comp.layer(1).property("ADBE Effect Parade").property(1);

        comp.openInViewer();
        comp.saveFrameToPng(0.4, new File(OUT + "m2g_t04.png"));

        var slotName = "", keys = -1;
        try { slotName = fx.property(6).name; } catch (e1) {}
        try { keys = fx.property(6).numKeys; } catch (e2) {}
        logLine("AFTER_RENAME slot1=[" + slotName + "] numKeys=" + keys);
        logLine("PNG saved m2g_t04.png expect gray 128 (keyframes 0->1 at t=0.4, default 0.9 NOT applied)");
        logLine("RESULT M2G slot1=[" + slotName + "] numKeys=" + keys);
    } catch (e) {
        logLine("SCRIPT_ERROR " + String(e));
    }
    logLine("RESULT_DONE");
})();
