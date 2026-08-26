// M3 harness — H1: deliberately tear the StateToken (scripted overwrite
// with a wrong-but-well-formed Active word). The render clone must detect
// the registry miss, fall back to the snapshot, and still render exactly;
// the idle observer must correct the token afterwards (ADR-0015 §2).
(function () {
    var OUT = ($.getenv("DFX_M3_OUT") || "E:/Code/AePlugin_Dynamicfx/scripts/out/m3/dev") + "/";
    new Folder(OUT).create();
    function logLine(s) {
        var f = new File(OUT + "m3h1.log");
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
        // Undo the m3g layer edits so session 4 continues from a clean comp.
        while (comp.numLayers > 1) { comp.layer(1).remove(); }
        var fx = comp.layer(1).property("ADBE Effect Parade").property(1);
        var before = fx.property(6).value;
        // (1<<2)|1 = 5.0: a well-formed Active token whose fingerprint (1)
        // matches nothing in the registry.
        fx.property(6).setValue(5);
        logLine("TORN before=" + before + " written=5");
        comp.openInViewer();
        comp.saveFrameToPng(0.4, new File(OUT + "m3h1_torn.png"));
        logLine("PNG saved m3h1_torn.png expect (51,51,0) — snapshot wins over the torn token");
        logLine("RESULT M3H1 before=" + before);
    } catch (e) {
        logLine("SCRIPT_ERROR " + String(e));
    }
    logLine("RESULT_DONE");
})();
