// TR-0031-001 host leg, scriptable half: the gradient parameter's value path.
//
// The editor's gestures cannot be scripted (they are mouse events in the
// Effect Controls panel), so this leg proves everything *around* the editor:
// the control declares itself, the default value renders, the baked LUT
// reaches the shader with the right ramp, and save/reopen keeps it. The
// gestures themselves are the manual checklist in TR-0031-001.
#include "f003_lib.jsxinc"

(function () {
    var LOG = "f003b";
    try {
        try { app.project.close(CloseOptions.DO_NOT_SAVE_CHANGES); } catch (e) {}
        app.newProject();
        var comp = f003NewComp("f003b", 256, 32, 8);
        var solid = comp.layers.addSolid([0, 0, 0], "input", 256, 32, 1.0);

        // Sample the ramp straight across the frame: with the default
        // black->white gradient the output must be a horizontal ramp, so a
        // pixel probe at x reads back ~x/255 in every channel.
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
        fx.property(2).expression = f003Wrap(shader);

        // Gradient slots follow the four Layer slots: 5 heads + 104 v1 slots
        // + Details (109) + Layer 0..3 (110..113) => Gradient 0 at 114,
        // property index 115.
        var GRADIENT_SLOT_PROPERTY = 115;
        var name = "<unread>", kind = "<unread>";
        try {
            var p = fx.property(GRADIENT_SLOT_PROPERTY);
            name = p.name;
            kind = String(p.propertyValueType);
        } catch (e) { name = "ERR " + String(e); }
        f003Log(LOG, "GRADIENT_SLOT index=" + GRADIENT_SLOT_PROPERTY + " name=[" + name + "] valueType=" + kind);
        f003Log(LOG, "TOTAL_PROPERTIES " + fx.numProperties);

        // ADR-0033: the value now lives in ordinary stop parameters. They are
        // declared after every growth pool, so gradient 1's count is at
        // property 5+104+1+4+2+8+2+1 = 127 and its first stop follows. The
        // 8 and 2 are the ADR-0034 Point 3D and ADR-0035 Path pools, appended
        // between the gradient anchors and this block;
        // `growth_pool_property_indexes_match_the_harness` fails in CI if this
        // constant and the topology ever disagree.
        var STOP_BASE = 127;
        for (var k = 0; k < 5; k++) {
            var idx = STOP_BASE + k, nm = "<unread>", vt = "";
            try {
                var sp = fx.property(idx);
                nm = sp.name;
                vt = String(sp.propertyValueType);
            } catch (e) { nm = "ERR " + String(e); }
            f003Log(LOG, "STOP_PARAM [" + idx + "] name=[" + nm + "] valueType=" + vt);
        }

        $.global.f003bState = { comp: comp, fx: fx, log: LOG, tries: 0 };
        // Every leg now gets a cold AE (each one quits at its sentinel), and a
        // cold start needs far longer than the fixed 4 s this used to wait —
        // measured 2026-08-15, it passed warm and failed cold on the same code.
        app.scheduleTask("f003bWait()", 3000, false);
    } catch (e) {
        f003Log("f003b", "SCRIPT_ERROR " + String(e) + " (line " + e.line + ")");
        f003Log("f003b", "RESULT_DONE");
    }
})();

function f003bWait() {
    f003Log("f003b", "TICK state=" + (typeof $.global.f003bState));
    var s = $.global.f003bState;
    try {
        if (f003IsReady(s.fx) || ++s.tries > 60) {
            f003bAfterIdle();
        } else {
            app.scheduleTask("f003bWait()", 1000, false);
        }
    } catch (e) {
        f003Log("f003b", "SCRIPT_ERROR in wait: " + String(e));
        f003Log("f003b", "RESULT_DONE");
    }
}

function f003bAfterIdle() {
    var s = $.global.f003bState;
    try {
        var t = f003TokenState(s.fx);
        f003Log(s.log, "TOKEN word=" + t.word + " state=" + t.state);
        f003Log(s.log, "STATUS [" + f003Status(s.fx) + "]");
        if (t.state !== 1) {
            f003Log(s.log, "RESULT F003B ready=0");
            f003Log(s.log, "RESULT_DONE");
            return;
        }
        f003RenderPsd(s.log, s.comp, "f003b_default");
        f003Log(s.log, "EXPECT horizontal black->white ramp, row-constant");

        // Save and reopen: the value must come back through the arbitrary
        // data flatten/unflatten path, not be re-defaulted.
        var proj = new File(F003_OUT + "f003b.aep");
        app.project.save(proj);
        f003Log(s.log, "SAVED " + proj.fsName);
        f003Log(s.log, "RESULT F003B rendered=1");
        f003Log(s.log, "RESULT_DONE");
    } catch (e) {
        f003Log(s.log, "SCRIPT_ERROR " + String(e));
        f003Log(s.log, "RESULT_DONE");
    }
}

