// M3 harness — G (session 4): duplicate-instance isolation. Two copies of
// the effect must hold independent parameter state: layer 1 keeps its
// keyframes (t=0.4 → 0.4), the duplicate gets a static 0.9.
(function () {
    var OUT = ($.getenv("DFX_M3_OUT") || "E:/Code/AePlugin_Dynamicfx/scripts/out/m3/dev") + "/";
    new Folder(OUT).create();
    function logLine(s) {
        var f = new File(OUT + "m3g.log");
        f.encoding = "UTF-8";
        if (f.open("a")) { f.write(s + "\n"); f.close(); }
    }
    try {
        try { app.project.close(CloseOptions.DO_NOT_SAVE_CHANGES); } catch (eC) {}
        app.open(new File(OUT + "m3.aep"));
        var comp = null;
        for (var j = 1; j <= app.project.numItems; j++) {
            if (app.project.item(j) instanceof CompItem && app.project.item(j).name === "m3") {
                comp = app.project.item(j);
                break;
            }
        }
        var layer1 = comp.layer(1);
        var layer2 = layer1.duplicate();
        // Give the duplicate a static value: strip inherited keyframes first.
        var gain2 = layer2.property("ADBE Effect Parade").property(1).property(6);
        while (gain2.numKeys > 0) { gain2.removeKey(1); }
        gain2.setValue(0.9);
        var gain1 = layer1.property("ADBE Effect Parade").property(1).property(6);
        logLine("DUP layer1_keys=" + gain1.numKeys + " layer2_keys=" + gain2.numKeys +
            " layer2_value=" + gain2.value);

        comp.openInViewer();
        layer2.enabled = false;
        comp.saveFrameToPng(0.4, new File(OUT + "m3g_layer1.png"));
        layer2.enabled = true;
        layer1.enabled = false;
        comp.saveFrameToPng(0.4, new File(OUT + "m3g_layer2.png"));
        layer1.enabled = true;
        logLine("PNGS layer1 expect (51,51,0); layer2 expect (115,115,0)");
        logLine("RESULT M3G l1_keys=" + gain1.numKeys + " l2_value=" + gain2.value);
    } catch (e) {
        logLine("SCRIPT_ERROR " + String(e));
    }
    logLine("RESULT_DONE");
})();
