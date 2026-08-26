// M2 harness — E: annotated shader. The idle observer must apply the
// annotation label to the slot, write the default VALUE into the fresh
// binding via AEGP, and the frame must render at the default before any
// stream is touched (the defaults-before-committed-streams exit criterion).
(function () {
    var OUT = ($.getenv("DFX_M2_OUT") || "E:/Code/AePlugin_Dynamicfx/scripts/out/m2/dev") + "/";
    new Folder(OUT).create();
    function logLine(s) {
        var f = new File(OUT + "m2e.log");
        f.encoding = "UTF-8";
        if (f.open("a")) { f.write(s + "\n"); f.close(); }
    }
    try {
        try { app.project.close(CloseOptions.DO_NOT_SAVE_CHANGES); } catch (eC) {}
        app.newProject();
        app.project.bitsPerChannel = 8;
        try { app.project.workingSpace = ""; } catch (eW) {}

        var comp = app.project.items.addComp("m2x", 320, 240, 1.0, 1.0, 25);
        comp.bgColor = [0, 0, 0];
        var solid = comp.layers.addSolid([10 / 255, 200 / 255, 30 / 255], "input", 320, 240, 1.0, 1.0);
        var fx = solid.property("ADBE Effect Parade").addProperty("DynamicFx");

        var src = "#version 450\n" +
            "layout(location = 0) in vec2 v_uv;\n" +
            "layout(location = 0) out vec4 outColor;\n" +
            "// @param level label:\"Master Level\" min:0 max:2 default:0.5\n" +
            "layout(set = 0, binding = 2) uniform FxUniforms {\n" +
            "    vec2 u_resolution;\n" +
            "    float u_time;\n" +
            "    float u_frame;\n" +
            "    float level;\n" +
            "};\n" +
            "void main() { outColor = vec4(level, level, level, 1.0); }\n";
        fx.property(3).expression = "`" + src + "`;0";
        logLine("EXPR set exprErr=[" + fx.property(3).expressionError + "]");
        logLine("RESULT M2E expr_set=1");
    } catch (e) {
        logLine("SCRIPT_ERROR " + String(e));
    }
    logLine("RESULT_DONE");
})();
