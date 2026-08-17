// M2 harness — C: edit the source so a NEW parameter (`extra`) is declared
// BEFORE `gain`. Slots must follow stable IDs, not declaration order: gain
// keeps Float 01 (and its keyframes); extra takes the next free slot.
(function () {
    var OUT = ($.getenv("DFX_M2_OUT") || "E:/Code/AePlugin_Dynamicfx/scripts/out/m2/dev") + "/";
    new Folder(OUT).create();
    function logLine(s) {
        var f = new File(OUT + "m2c.log");
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
        var src = "#version 450\n" +
            "layout(location = 0) in vec2 v_uv;\n" +
            "layout(location = 0) out vec4 outColor;\n" +
            "layout(set = 0, binding = 2) uniform FxUniforms {\n" +
            "    vec2 u_resolution;\n" +
            "    float u_time;\n" +
            "    float u_frame;\n" +
            "    float extra;\n" +
            "    float gain;\n" +
            "};\n" +
            "void main() { outColor = vec4(gain + extra, gain + extra, gain + extra, 1.0); }\n";
        fx.property(2).expression = "`" + src + "`;0";
        logLine("EDIT set exprErr=[" + fx.property(2).expressionError + "]");
        logLine("RESULT M2C edit_set=1");
    } catch (e) {
        logLine("SCRIPT_ERROR " + String(e));
    }
    logLine("RESULT_DONE");
})();
