// M3 harness — H4: after the post-undo idle window, rendering must have
// converged back to the keyframed shader, and a save must leave the project
// clean even though the observer keeps running (ADR-0015 §3 / TR-M0-005).
(function () {
    var OUT = ($.getenv("DFX_M3_OUT") || "E:/Code/AePlugin_Dynamicfx/scripts/out/m3/dev") + "/";
    new Folder(OUT).create();
    function logLine(s) {
        var f = new File(OUT + "m3h4.log");
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
        var token = fx.property(6).value;
        comp.openInViewer();
        comp.saveFrameToPng(0.4, new File(OUT + "m3h4_undo.png"));
        logLine("CONVERGED token_nonzero=" + (token > 0 ? 1 : 0) +
            " png expect (51,51,0) again");

        // Dirty leg: save, then report dirty after this script ends; the
        // driver's idle window follows and m3h5 (quit) is not needed — read
        // dirty here after save as the baseline, the observer has already
        // been running throughout this session.
        var aep = new File(OUT + "m3_session4.aep");
        app.project.save(aep);
        var dirtyAfterSave = "unknown";
        try { dirtyAfterSave = String(app.project.dirty); } catch (eD) {}
        logLine("SAVED dirty_after_save=" + dirtyAfterSave);
        logLine("RESULT M3H4 token_nonzero=" + (token > 0 ? 1 : 0) + " dirty_after_save=" + dirtyAfterSave);
    } catch (e) {
        logLine("SCRIPT_ERROR " + String(e));
    }
    logLine("RESULT_DONE");
})();
