// M2 harness — B: after the idle window, verify the bound slot's AEGP-applied
// label, keyframe it, and render three frames across the interpolation.
(function () {
    var OUT = ($.getenv("DFX_M2_OUT") || "E:/Code/AePlugin_Dynamicfx/scripts/out/m2/dev") + "/";
    new Folder(OUT).create();
    function logLine(s) {
        var f = new File(OUT + "m2b.log");
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

        // Slot labels applied by the idle observer via AEGP: Float 01 (prop 6)
        // must now be "gain"; Float 02 (prop 7) keeps its default name and is
        // hidden (hidden state is not scripting-visible; the plugin log's
        // "idle slot ui applied" line is the visibility evidence).
        var slot1 = "", slot2 = "", status = "";
        try { slot1 = fx.property(6).name; } catch (e1) {}
        try { slot2 = fx.property(7).name; } catch (e2) {}
        try { status = fx.property(4).name; } catch (e3) {}
        logLine("SLOTS slot1=[" + slot1 + "] slot2=[" + slot2 + "] status=[" + status + "]");

        // Keyframe the bound slot: 0.0 at t=0, 0.8 at t=0.8s (frame 20).
        var gain = fx.property(6);
        gain.setValueAtTime(0, 0.0);
        gain.setValueAtTime(0.8, 0.8);
        logLine("KEYFRAMES numKeys=" + gain.numKeys);

        comp.openInViewer();
        comp.saveFrameToPng(0, new File(OUT + "m2b_t0.png"));
        comp.saveFrameToPng(0.4, new File(OUT + "m2b_t04.png"));
        comp.saveFrameToPng(0.8, new File(OUT + "m2b_t08.png"));
        logLine("PNGS saved t0/t04/t08 expect gray 0 / 102 / 204");
        logLine("RESULT M2B slot1=[" + slot1 + "] numKeys=" + gain.numKeys);
    } catch (e) {
        logLine("SCRIPT_ERROR " + String(e));
    }
    logLine("RESULT_DONE");
})();
