// M2 harness — A: scripted shader with one user parameter (`float gain`).
// The idle observer must compile it, bind gain to Float slot 0, publish the
// token, AND apply slot names/visibility via AEGP (no UI callback exists on
// this path).
(function () {
    var OUT = ($.getenv("DFX_M2_OUT") || "E:/Code/AePlugin_Dynamicfx/scripts/out/m2/dev") + "/";
    new Folder(OUT).create();
    function logLine(s) {
        var f = new File(OUT + "m2a.log");
        f.encoding = "UTF-8";
        if (f.open("a")) { f.write(s + "\n"); f.close(); }
    }
    try {
        try { app.project.close(CloseOptions.DO_NOT_SAVE_CHANGES); } catch (eC) {}
        app.newProject();
        app.project.bitsPerChannel = 8;
        try { app.project.workingSpace = ""; } catch (eW) {}

        var comp = app.project.items.addComp("m2", 320, 240, 1.0, 1.0, 25);
        comp.bgColor = [0, 0, 0];
        var solid = comp.layers.addSolid([10 / 255, 200 / 255, 30 / 255], "input", 320, 240, 1.0, 1.0);
        var fx = solid.property("ADBE Effect Parade").addProperty("DynamicFx");

        var src = "#version 450\n" +
            "layout(location = 0) in vec2 v_uv;\n" +
            "layout(location = 0) out vec4 outColor;\n" +
            "layout(set = 0, binding = 2) uniform FxUniforms {\n" +
            "    vec2 u_resolution;\n" +
            "    float u_time;\n" +
            "    float u_frame;\n" +
            "    float gain;\n" +
            "};\n" +
            "void main() { outColor = vec4(gain, gain, gain, 1.0); }\n";
        fx.property(3).expression = "`" + src + "`;0";
        logLine("EXPR set len=" + fx.property(3).expression.length +
            " exprErr=[" + fx.property(3).expressionError + "]");
        logLine("RESULT M2A expr_set=1");
    } catch (e) {
        logLine("SCRIPT_ERROR " + String(e));
    }
    logLine("RESULT_DONE");
})();
