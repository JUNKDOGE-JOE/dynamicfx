// M3 harness — H2: after the idle window the token must be corrected back
// to the real fingerprint word. Then write an invalid source for the undo
// leg.
(function () {
    var OUT = ($.getenv("DFX_M3_OUT") || "E:/Code/AePlugin_Dynamicfx/scripts/out/m3/dev") + "/";
    new Folder(OUT).create();
    function logLine(s) {
        var f = new File(OUT + "m3h2.log");
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
        var corrected = fx.property(5).value;
        logLine("CORRECTED token=" + corrected + " (must differ from 5 and be nonzero)");

        // Undo leg: commit an invalid source (this goes on the undo stack).
        app.beginUndoGroup("dfx invalid edit");
        fx.property(2).expression = "`broken source`;0";
        app.endUndoGroup();
        logLine("INVALID committed for the undo leg");
        logLine("RESULT M3H2 corrected=" + corrected);
    } catch (e) {
        logLine("SCRIPT_ERROR " + String(e));
    }
    logLine("RESULT_DONE");
})();
