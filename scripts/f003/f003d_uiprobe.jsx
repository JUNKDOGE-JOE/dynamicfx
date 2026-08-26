// Gradient editor crash probe (ADR-0031 §7). Builds one instance with a
// gradient parameter and leaves it selected, so expanding the row in the
// Effect Controls panel is the only remaining step — the plug-in logs the
// frame rect it is handed on every draw.
#include "f003_lib.jsxinc"

(function () {
    var LOG = "f003d";
    try {
        try { app.project.close(CloseOptions.DO_NOT_SAVE_CHANGES); } catch (e) {}
        app.newProject();
        var comp = f003NewComp("f003d", 256, 64, 8);
        var solid = comp.layers.addSolid([0, 0, 0], "input", 256, 64, 1.0);

        var shader =
            "@dynamicfx 1\n@graph\npass main: input, ramp -> output\n@end\n@pass main\n" +
            "#version 450\n" +
            "// @param ramp label:\"Ramp\" hint:gradient\n" +
            "layout(location = 0) in vec2 v_uv;\n" +
            "layout(location = 0) out vec4 outColor;\n" +
            "layout(set = 0, binding = 0) uniform texture2D u_in;\n" +
            "layout(set = 0, binding = 1) uniform sampler u_s;\n" +
            "layout(set = 0, binding = 2) uniform FxUniforms {\n" +
            "    vec2 u_resolution;\n" +
            "    float u_time;\n" +
            "    float u_frame;\n" +
            "};\n" +
            "layout(set = 0, binding = 3) uniform texture2D u_ramp;\n" +
            "void main() {\n" +
            "    outColor = vec4(texture(sampler2D(u_ramp, u_s), vec2(v_uv.x, 0.5)).rgb, 1.0);\n" +
            "}\n" +
            "@endpass\n";

        var fx = solid.property("ADBE Effect Parade").addProperty("DynamicFx");
        fx.property(3).expression = f003Wrap(shader);

        // Select the layer so the Effect Controls panel shows this effect.
        solid.selected = true;
        var name = "<unread>";
        try { name = fx.property(132).name; } catch (e) { name = "ERR " + String(e); }
        f003Log(LOG, "READY gradient_slot=[" + name + "] numProperties=" + fx.numProperties);
        f003Log(LOG, "NEXT expand the DynamicFx twirl in Effect Controls to trigger a draw");
        f003Log(LOG, "RESULT_DONE");
    } catch (e) {
        f003Log(LOG, "SCRIPT_ERROR " + String(e) + " (line " + e.line + ")");
        f003Log(LOG, "RESULT_DONE");
    }
})();
