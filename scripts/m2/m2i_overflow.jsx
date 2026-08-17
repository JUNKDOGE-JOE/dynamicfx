// M2 harness — I: a 49-float shader must overflow the 48-slot Float pool and
// reject the WHOLE definition atomically (host leg of the pool-overflow exit
// criterion). Verification happens in m2j after the idle window.
(function () {
    var OUT = ($.getenv("DFX_M2_OUT") || "E:/Code/AePlugin_Dynamicfx/scripts/out/m2/dev") + "/";
    new Folder(OUT).create();
    function logLine(s) {
        var f = new File(OUT + "m2i.log");
        f.encoding = "UTF-8";
        if (f.open("a")) { f.write(s + "\n"); f.close(); }
    }
    try {
        var comp = null;
        for (var j = 1; j <= app.project.numItems; j++) {
            if (app.project.item(j) instanceof CompItem && app.project.item(j).name === "m2k") {
                comp = app.project.item(j);
                break;
            }
        }
        var fx = comp.layer(1).property("ADBE Effect Parade").property(1);

        var members = "";
        for (var i = 0; i < 49; i++) { members += "    float f" + i + ";\n"; }
        var src = "#version 450\n" +
            "layout(location = 0) in vec2 v_uv;\n" +
            "layout(location = 0) out vec4 outColor;\n" +
            "layout(set = 0, binding = 2) uniform FxUniforms {\n" +
            "    vec2 u_resolution;\n" +
            "    float u_time;\n" +
            "    float u_frame;\n" +
            members +
            "};\n" +
            "void main() { outColor = vec4(f0, f1, f48, 1.0); }\n";
        fx.property(2).expression = "`" + src + "`;0";
        logLine("OVERFLOW_EXPR set (49 floats) exprErr=[" + fx.property(2).expressionError + "]");
        logLine("RESULT M2I overflow_set=1");
    } catch (e) {
        logLine("SCRIPT_ERROR " + String(e));
    }
    logLine("RESULT_DONE");
})();
