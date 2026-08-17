// M1 harness — E: after the idle window, verify the invalid-source
// diagnostic and pass-through frame, then restore the gradient and stage the
// aerender project + render queue.
(function () {
    var OUT = ($.getenv("DFX_M1_OUT") || "E:/Code/AePlugin_Dynamicfx/scripts/out/m1/dev") + "/";
    new Folder(OUT).create();
    function logLine(s) {
        var f = new File(OUT + "m1e.log");
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
        var fx = comp.layer(1).property("ADBE Effect Parade").property(1);
        // Render first so the diagnostic text lands via the UI callback.
        comp.openInViewer();
        comp.saveFrameToPng(0, new File(OUT + "m1d_invalid.png"));
        logLine("INVALID_PNG saved m1d_invalid.png expect passthrough solid (10,200,30)");
        var statusName = "";
        try { statusName = fx.property(4).name; } catch (eS) {}
        var tokenValue = -1;
        try { tokenValue = fx.property(5).value; } catch (eT) {}
        logLine("INVALID_STATUS [" + statusName + "] token=" + tokenValue);

        // Restore the gradient for the aerender leg and stage the queue.
        var src = "#version 450\n" +
            "layout(location = 0) in vec2 v_uv;\n" +
            "layout(location = 0) out vec4 outColor;\n" +
            "layout(set = 0, binding = 2) uniform FxUniforms {\n" +
            "    vec2 u_resolution;\n" +
            "    float u_time;\n" +
            "    float u_frame;\n" +
            "};\n" +
            "void main() { outColor = vec4(v_uv, 0.0, 1.0); }\n";
        fx.property(2).expression = "`" + src + "`;0";

        var rqItem = app.project.renderQueue.items.add(comp);
        rqItem.render = true;
        try { rqItem.timeSpanStart = 0; rqItem.timeSpanDuration = comp.frameDuration; } catch (eT2) {}
        var om = rqItem.outputModule(1);
        var applied = "";
        for (var t = 0; t < om.templates.length; t++) {
            if (om.templates[t] === "Photoshop") { om.applyTemplate("Photoshop"); applied = "Photoshop"; break; }
        }
        try { om.file = new File(OUT + "m1_ar_[#####].psd"); } catch (eF) {}
        logLine("RQ applied=[" + applied + "]");
        var aep = new File(OUT + "m1_ar.aep");
        app.project.save(aep);
        logLine("AEP saved m1_ar.aep");
        logLine("RESULT M1E status=[" + statusName + "] rq=[" + applied + "]");
    } catch (e) {
        logLine("SCRIPT_ERROR " + String(e));
    }
    logLine("RESULT_DONE");
})();
