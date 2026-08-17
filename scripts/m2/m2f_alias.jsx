// M2 harness — F: verify the annotation label + rendered default, keyframe
// the slot, then rename the parameter (level → volume) WITH alias:level and
// a different default (0.9) that must NOT apply to the inherited binding.
(function () {
    var OUT = ($.getenv("DFX_M2_OUT") || "E:/Code/AePlugin_Dynamicfx/scripts/out/m2/dev") + "/";
    new Folder(OUT).create();
    function logLine(s) {
        var f = new File(OUT + "m2f.log");
        f.encoding = "UTF-8";
        if (f.open("a")) { f.write(s + "\n"); f.close(); }
    }
    try {
        var comp = null;
        for (var j = 1; j <= app.project.numItems; j++) {
            if (app.project.item(j) instanceof CompItem && app.project.item(j).name === "m2x") {
                comp = app.project.item(j);
                break;
            }
        }
        var fx = comp.layer(1).property("ADBE Effect Parade").property(1);

        var slotName = "", slotValue = -1;
        try { slotName = fx.property(6).name; } catch (e1) {}
        try { slotValue = fx.property(6).value; } catch (e2) {}
        logLine("ANNOTATED slot1=[" + slotName + "] value=" + slotValue);

        comp.openInViewer();
        comp.saveFrameToPng(0, new File(OUT + "m2e_default.png"));
        logLine("PNG saved m2e_default.png expect gray 128 (default 0.5, no stream touched)");

        // Keyframe, then rename with alias. The new default:0.9 must NOT
        // overwrite the inherited binding's keyframes.
        var level = fx.property(6);
        level.setValueAtTime(0, 0.0);
        level.setValueAtTime(0.8, 1.0);
        logLine("KEYFRAMES numKeys=" + level.numKeys);

        var src = "#version 450\n" +
            "layout(location = 0) in vec2 v_uv;\n" +
            "layout(location = 0) out vec4 outColor;\n" +
            "// @param volume label:\"Volume\" min:0 max:2 default:0.9 alias:level\n" +
            "layout(set = 0, binding = 2) uniform FxUniforms {\n" +
            "    vec2 u_resolution;\n" +
            "    float u_time;\n" +
            "    float u_frame;\n" +
            "    float volume;\n" +
            "};\n" +
            "void main() { outColor = vec4(volume, volume, volume, 1.0); }\n";
        fx.property(2).expression = "`" + src + "`;0";
        logLine("RENAME set exprErr=[" + fx.property(2).expressionError + "]");
        logLine("RESULT M2F slot1=[" + slotName + "] value=" + slotValue + " numKeys=" + level.numKeys);
    } catch (e) {
        logLine("SCRIPT_ERROR " + String(e));
    }
    logLine("RESULT_DONE");
})();
