// M2 harness — D: after the idle window, verify slots followed stable IDs
// across the source edit (gain kept Float 01 + keyframes; extra took
// Float 02) and the interpolated frame still renders through the new
// definition.
(function () {
    var OUT = ($.getenv("DFX_M2_OUT") || "E:/Code/AePlugin_Dynamicfx/scripts/out/m2/dev") + "/";
    new Folder(OUT).create();
    function logLine(s) {
        var f = new File(OUT + "m2d.log");
        f.encoding = "UTF-8";
        if (f.open("a")) { f.write(s + "\n"); f.close(); }
    }
    try {
        var comp = null;
        for (var j = 1; j <= app.project.numItems; j++) {
            if (app.project.item(j) instanceof CompItem && app.project.item(j).name === "m2") {
                comp = app.project.item(j);
                break;
            }
        }
        var fx = comp.layer(1).property("ADBE Effect Parade").property(1);

        comp.openInViewer();
        comp.saveFrameToPng(0.4, new File(OUT + "m2d_t04.png"));

        var slot1 = "", slot2 = "", keys = -1;
        try { slot1 = fx.property(6).name; } catch (e1) {}
        try { slot2 = fx.property(7).name; } catch (e2) {}
        try { keys = fx.property(6).numKeys; } catch (e3) {}
        logLine("AFTER_EDIT slot1=[" + slot1 + "] slot2=[" + slot2 + "] slot1_numKeys=" + keys);
        logLine("PNG saved m2d_t04.png expect gray 102 (gain keyframes intact, extra=0)");
        logLine("RESULT M2D slot1=[" + slot1 + "] slot2=[" + slot2 + "] numKeys=" + keys);
    } catch (e) {
        logLine("SCRIPT_ERROR " + String(e));
    }
    logLine("RESULT_DONE");
})();
