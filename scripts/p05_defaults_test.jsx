// P0.5 test on the existing dynfx_p05_test comp (effect already applied).
(function () {
    var log = [];
    function say(s) { log.push(s); }
    function flush() {
        var f = new File("E:/Code/AePlugin_Dynamicfx/scripts/out/test_log.txt");
        f.encoding = "UTF-8";
        if (f.open("w")) { f.write(log.join("\n")); f.close(); }
    }
    try {
        var comp = null;
        for (var i = 1; i <= app.project.items.length; i++) {
            if (app.project.items[i].name === "dynfx_p05_test") { comp = app.project.items[i]; break; }
        }
        if (!comp) throw new Error("comp not found");
        var fx = comp.layer(1).property("Effects").property("DynamicFx");
        say("fx ok, props=" + fx.numProperties);

        var glsl =
            "#version 450\n" +
            "layout(location = 0) in vec2 v_uv;\n" +
            "layout(location = 0) out vec4 outColor;\n" +
            "layout(set = 0, binding = 0) uniform texture2D u_input;\n" +
            "layout(set = 0, binding = 1) uniform sampler u_sampler;\n" +
            "// @param u_levels float 1.0 32.0 16.0 Levels\n" +
            "layout(set = 0, binding = 2) uniform FxUniforms {\n" +
            "    vec2 u_resolution;\n" +
            "    float u_time;\n" +
            "    float u_levels;\n" +
            "};\n" +
            "void main() {\n" +
            "    vec4 c = texture(sampler2D(u_input, u_sampler), v_uv);\n" +
            "    outColor = vec4(floor(c.rgb * u_levels) / u_levels, 1.0);\n" +
            "}\n";

        var p1 = fx.property(1);
        say("p1 name=" + p1.name + " matchName=" + p1.matchName);
        p1.expression = "`" + glsl + "`;0";
        say("expr set, readback len=" + p1.expression.length);

        var p5 = fx.property(5);
        say("p5 name=" + p5.name);
        p5.setValue(16.0);
        say("p5 set, readback=" + p5.value);

        comp.saveFrameToPng(0.0, new File("E:/Code/AePlugin_Dynamicfx/scripts/out/dynfx_p05.png"));
        say("png saved");
    } catch (e) {
        say("ERROR: " + e.toString() + " (line " + e.line + ")");
    }
    flush();
})();
