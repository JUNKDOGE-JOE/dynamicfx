// Position-zeroing repro: a text layer placed off-center, with a simple
// tint shader applied DIRECTLY on the text layer (tight buffer + origin),
// plus a small offset solid. Render with and without the effect.
(function () {
    var OUT = "C:/Users/A/AppData/Local/Temp/claude/E--Code-AePlugin-Dynamicfx/fbefc838-0280-4b84-9c35-f140b6366b08/scratchpad/thermal/";
    function log(s) {
        var f = new File(OUT + "repro.log");
        f.encoding = "UTF-8";
        if (f.open("a")) { f.write(s + "\n"); f.close(); }
    }
    var TINT =
        "#version 450\n" +
        "layout(location = 0) in vec2 v_uv;\n" +
        "layout(location = 0) out vec4 outColor;\n" +
        "layout(set = 0, binding = 0) uniform texture2D u_in;\n" +
        "layout(set = 0, binding = 1) uniform sampler u_s;\n" +
        "layout(set = 0, binding = 2) uniform FxUniforms {\n" +
        "    vec2 u_resolution;\n" +
        "    float u_time;\n" +
        "    float u_frame;\n" +
        "};\n" +
        "void main() {\n" +
        "    vec4 c = texture(sampler2D(u_in, u_s), v_uv);\n" +
        "    outColor = vec4(c.a, c.a * 0.3, c.a * 0.8, c.a);\n" +
        "}\n";
    try {
        var comp = app.project.items.addComp("posRepro", 720, 480, 1.0, 3.0, 25);
        comp.bgColor = [0.08, 0.08, 0.08];
        var bg = comp.layers.addSolid([0.08, 0.08, 0.08], "bg", 720, 480, 1.0, 3.0);

        // Reference text WITHOUT effect, upper-left area.
        var t1 = comp.layers.addText("REF");
        var td1 = t1.property("Source Text").value;
        td1.fontSize = 90;
        td1.fillColor = [0.4, 0.9, 0.4];
        t1.property("Source Text").setValue(td1);
        t1.property("Position").setValue([160, 140]);

        // Text WITH the effect, lower-right area — if origins are ignored,
        // this one's pixels land in the wrong place.
        var t2 = comp.layers.addText("FX");
        var td2 = t2.property("Source Text").value;
        td2.fontSize = 90;
        td2.fillColor = [1, 1, 1];
        t2.property("Source Text").setValue(td2);
        t2.property("Position").setValue([520, 340]);
        var fx2 = t2.property("ADBE Effect Parade").addProperty("DynamicFx");
        fx2.property(2).expression = "`" + TINT + "`;0";

        // Small offset solid WITH effect.
        var s3 = comp.layers.addSolid([1, 0.2, 0.2], "smallSolid", 160, 120, 1.0, 3.0);
        s3.property("Position").setValue([560, 120]);
        var fx3 = s3.property("ADBE Effect Parade").addProperty("DynamicFx");
        fx3.property(2).expression = "`" + TINT + "`;0";

        comp.openInViewer();
        log("REPRO built");
    } catch (e) {
        log("SCRIPT_ERROR " + String(e) + " (line " + e.line + ")");
    }
    log("RESULT_DONE");
})();
