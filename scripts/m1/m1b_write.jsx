// M1 harness — B / TR-M1-004 setup: scripted write of a raw ABI v1 GLSL
// gradient into the Source expression. Scripted writes never arrive as
// UserChangedParam (TR-M0-005), so this leg exercises the idle observer:
// the driver sleeps afterwards while AE idles and the plugin observes,
// compiles, and publishes the session token.
(function () {
    var OUT = ($.getenv("DFX_M1_OUT") || "E:/Code/AePlugin_Dynamicfx/scripts/out/m1/dev") + "/";
    new Folder(OUT).create();
    function logLine(s) {
        var f = new File(OUT + "m1b.log");
        f.encoding = "UTF-8";
        if (f.open("a")) { f.write(s + "\n"); f.close(); }
    }
    try {
        try { app.project.close(CloseOptions.DO_NOT_SAVE_CHANGES); } catch (eC) {}
        app.newProject();
        app.project.bitsPerChannel = 8;
        // Empty string disables color management ("None" would be stored as
        // a literal missing-profile name and raise a modal error).
        try { app.project.workingSpace = ""; } catch (eW2) {}

        var comp = app.project.items.addComp("m1", 320, 240, 1.0, 1.0, 25);
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
            "};\n" +
            "void main() { outColor = vec4(v_uv, 0.0, 1.0); }\n";
        var expr = "`" + src + "`;0";
        fx.property(2).expression = expr;

        logLine("EXPR set len=" + fx.property(2).expression.length +
            " match=" + (fx.property(2).expression === expr ? 1 : 0) +
            " exprErr=[" + fx.property(2).expressionError + "]");
        logLine("STATUS_NAME_NOW [" + fx.property(4).name + "]");
        logLine("RESULT M1B expr_set=1");
    } catch (e) {
        logLine("SCRIPT_ERROR " + String(e));
    }
    logLine("RESULT_DONE");
})();
