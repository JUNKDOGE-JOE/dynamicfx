// M3 harness — H3: confirm the invalid source took effect (diagnostic +
// pass-through), then undo it. The observer must converge back to the
// working shader without any Compile click (ADR-0015 §3).
(function () {
    var OUT = ($.getenv("DFX_M3_OUT") || "E:/Code/AePlugin_Dynamicfx/scripts/out/m3/dev") + "/";
    new Folder(OUT).create();
    function logLine(s) {
        var f = new File(OUT + "m3h3.log");
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
        var tokenDuringInvalid = fx.property(5).value;
        comp.openInViewer();
        comp.saveFrameToPng(0.4, new File(OUT + "m3h3_invalid.png"));
        logLine("INVALID_STATE token=" + tokenDuringInvalid +
            " png expect passthrough solid (10,200,30)");
        // The plugin's AEGP token write lands one undo entry of its own
        // (measured host behavior: AEGP_SetStreamValue is always undoable),
        // so press Undo repeatedly until the user's expression edit is
        // reached — like a real user holding Ctrl+Z.
        var presses = 0, exprAfterUndo = "";
        for (var u = 0; u < 4; u++) {
            app.executeCommand(16); // Edit > Undo
            presses++;
            try { exprAfterUndo = fx.property(2).expression.substring(0, 30); } catch (e1) {}
            if (exprAfterUndo.indexOf("broken") === -1) { break; }
        }
        logLine("UNDO pressed=" + presses + " exprHead=[" + exprAfterUndo + "]");
        logLine("RESULT M3H3 token_during_invalid=" + tokenDuringInvalid);
    } catch (e) {
        logLine("SCRIPT_ERROR " + String(e));
    }
    logLine("RESULT_DONE");
})();
