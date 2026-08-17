// M0 transport spike — S5 / TR-M0-007: aerender parity project.
// A 1 MB committed expression on a Slider Control yields 37.5; the white
// solid's opacity follows it, so the frame pixel value proves the long
// expression evaluated. GUI renders a reference PNG now; the driver runs
// aerender on the saved project afterwards and compares numerically.
(function () {
    var OUT = ($.getenv("DFX_SPIKE_OUT") || "E:/Code/AePlugin_Dynamicfx/scripts/out/spike/dev") + "/";
    new Folder(OUT).create();
    function logLine(s) {
        var f = new File(OUT + "s5.log");
        f.encoding = "UTF-8";
        if (f.open("a")) { f.write(s + "\n"); f.close(); }
    }
    function body(n) {
        var s = "AERENDER PARITY PAYLOAD 0123456789 ABCDEFGHIJKLMNOPQRSTUVWXYZ\n";
        while (s.length < n) { s = s + s; }
        return s.substring(0, n);
    }
    try {
        try { app.project.close(CloseOptions.DO_NOT_SAVE_CHANGES); } catch (eC) {}
        app.newProject();
        var comp = app.project.items.addComp("spike_ar", 320, 240, 1.0, 1.0, 25);
        comp.bgColor = [0, 0, 0];
        var solid = comp.layers.addSolid([1, 1, 1], "white", 320, 240, 1.0, 1.0);
        var fx = solid.property("ADBE Effect Parade").addProperty("ADBE Slider Control");
        var payload = "/*" + body(1048576 - 9) + "*/37.5";
        fx.property(1).expression = payload;
        logLine("EXPR len=" + fx.property(1).expression.length +
                " immMatch=" + (fx.property(1).expression === payload ? 1 : 0) +
                " exprErr=" + fx.property(1).expressionError);
        solid.property("ADBE Transform Group").property("ADBE Opacity").expression = "effect(1)(1)";
        logLine("OPACITY expr set, value=" + solid.property("ADBE Transform Group").property("ADBE Opacity").value);

        comp.openInViewer();
        comp.saveFrameToPng(0, new File(OUT + "s5_gui.png"));
        logLine("GUI_PNG saved expect_gray=96 (255*0.375=95.6)");

        var rqItem = app.project.renderQueue.items.add(comp);
        rqItem.render = true;
        try { rqItem.timeSpanStart = 0; rqItem.timeSpanDuration = comp.frameDuration; } catch (eT) {}
        var om = rqItem.outputModule(1);
        logLine("OM_TEMPLATES " + om.templates.join(" | "));
        // Format is a read-only setSettings key; retarget via applyTemplate.
        // Match template names with ASCII-only regex against the ORIGINAL
        // names read from om.templates — non-ASCII string literals in this
        // file are mis-decoded by ExtendScript under -r. A localized name
        // like "带有 Alpha 的 TIFF 序列" still contains ASCII "TIFF".
        // ExtendScript under -r corrupts non-ASCII names even when they come
        // from om.templates (applyTemplate re-encodes to ANSI and drops the
        // CJK chars). "Photoshop" is the only ASCII-named still template here,
        // so it is the reliable lossless target. Output is a 16-bit PSD.
        var applied = "";
        for (var t = 0; t < om.templates.length; t++) {
            if (om.templates[t] === "Photoshop") { om.applyTemplate("Photoshop"); applied = "Photoshop"; break; }
        }
        try { om.file = new File(OUT + "s5_ar_[#####].psd"); } catch (eF) {}
        var fmt = ""; try { fmt = om.getSettings(GetSettingsFormat.STRING)["Format"]; } catch (eG) {}
        logLine("RQ applied=[" + applied + "] fmt=" + fmt);

        var aep = new File(OUT + "s5.aep");
        app.project.save(aep);
        logLine("SAVED " + aep.fsName);
    } catch (e) {
        logLine("ERROR " + e.toString() + " (line " + e.line + ")");
    }
    logLine("RESULT_DONE");
})();
