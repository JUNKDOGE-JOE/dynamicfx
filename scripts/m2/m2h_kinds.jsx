// M2 harness — H: the multi-kind fixture shader (int/bool/color/point/angle
// in one block, spatially partitioned into five vertical bands). Pins the
// value-encoding semantics of every v1 kind (ADR-0013 §8).
(function () {
    var OUT = ($.getenv("DFX_M2_OUT") || "E:/Code/AePlugin_Dynamicfx/scripts/out/m2/dev") + "/";
    new Folder(OUT).create();
    function logLine(s) {
        var f = new File(OUT + "m2h.log");
        f.encoding = "UTF-8";
        if (f.open("a")) { f.write(s + "\n"); f.close(); }
    }
    try {
        try { app.project.close(CloseOptions.DO_NOT_SAVE_CHANGES); } catch (eC) {}
        app.newProject();
        app.project.bitsPerChannel = 8;
        try { app.project.workingSpace = ""; } catch (eW) {}

        var comp = app.project.items.addComp("m2k", 320, 240, 1.0, 1.0, 25);
        comp.bgColor = [0, 0, 0];
        var solid = comp.layers.addSolid([10 / 255, 200 / 255, 30 / 255], "input", 320, 240, 1.0, 1.0);
        var fx = solid.property("ADBE Effect Parade").addProperty("DynamicFx");

        var src = "#version 450\n" +
            "layout(location = 0) in vec2 v_uv;\n" +
            "layout(location = 0) out vec4 outColor;\n" +
            "// @param count label:\"Count\" min:0 max:10 default:3\n" +
            "// @param flag hint:bool default:1\n" +
            "// @param sweep hint:angle default:90\n" +
            "layout(set = 0, binding = 2) uniform FxUniforms {\n" +
            "    vec2 u_resolution;\n" +
            "    float u_time;\n" +
            "    float u_frame;\n" +
            "    int count;\n" +
            "    int flag;\n" +
            "    vec3 tint;\n" +
            "    vec2 center;\n" +
            "    float sweep;\n" +
            "};\n" +
            "void main() {\n" +
            "    float x = v_uv.x;\n" +
            "    vec3 c;\n" +
            "    if (x < 0.2)      c = vec3(float(count) / 10.0);\n" +
            "    else if (x < 0.4) c = vec3(flag != 0 ? 1.0 : 0.0);\n" +
            "    else if (x < 0.6) c = tint;\n" +
            "    else if (x < 0.8) c = vec3(center, 0.0);\n" +
            "    else              c = vec3(sweep / 360.0);\n" +
            "    outColor = vec4(c, 1.0);\n" +
            "}\n";
        fx.property(3).expression = "`" + src + "`;0";
        logLine("EXPR set exprErr=[" + fx.property(3).expressionError + "]");
        logLine("RESULT M2H expr_set=1");
    } catch (e) {
        logLine("SCRIPT_ERROR " + String(e));
    }
    logLine("RESULT_DONE");
})();
