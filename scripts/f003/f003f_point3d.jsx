// TR-0034-001 host leg: Point 3D parameters (`hint:point3d`).
//
// Shape of the check: one shader declares BOTH an un-annotated `vec3` (which
// ADR-0026 keeps a colour) and a `hint:point3d` `vec3`. The render encodes the
// point's three components into the three channels, so one PNG says whether
// x/y arrived normalized to the frame and z arrived in pixels (ADR-0034 §3) —
// and the parameter list says whether the two vec3s produced two different AE
// controls, which is the whole decision.
#include "f003_lib.jsxinc"

(function () {
    var LOG = "f003f";
    try {
        try { app.project.close(CloseOptions.DO_NOT_SAVE_CHANGES); } catch (e) {}
        app.newProject();
        var comp = f003NewComp("f003f", 160, 120, 8);
        var solid = comp.layers.addSolid([0, 0, 0], "target", 160, 120, 1.0);

        // r = point.x, g = point.y, b = point.z / 100. With the point driven to
        // a known place below, each channel is a direct readout of one
        // component, so the encoding is legible from the pixels alone.
        var shader =
            "@dynamicfx 1\n@graph\npass main: input -> output\n@end\n@pass main\n" +
            "#version 450\n" +
            "// @param probe label:\"Probe\" hint:point3d\n" +
            "// @param tint label:\"Tint\"\n" +
            "layout(location = 0) in vec2 v_uv;\n" +
            "layout(location = 0) out vec4 outColor;\n" +
            "layout(set = 0, binding = 0) uniform texture2D u_in;\n" +
            "layout(set = 0, binding = 1) uniform sampler u_s;\n" +
            "layout(set = 0, binding = 2) uniform FxUniforms {\n" +
            "    vec2 u_resolution;\n" +
            "    float u_time;\n" +
            "    float u_frame;\n" +
            "    vec3 probe;\n" +
            "    vec3 tint;\n" +
            "};\n" +
            "void main() {\n" +
            "    outColor = vec4(probe.x, probe.y, probe.z / 100.0, 1.0) * vec4(tint, 1.0);\n" +
            "}\n" +
            "@endpass\n";

        var fx = solid.property("ADBE Effect Parade").addProperty("DynamicFx");
        fx.property(3).expression = f003Wrap(shader);
        f003Log(LOG, "SHADER_WRITTEN");

        // The Point 3D pool starts at declaration index 116 (5 heads + 104 v1
        // slots + Details at 109 + 4 Layer + 2 Gradient), so property 117.
        // `growth_pool_property_indexes_match_the_harness` fails in CI if this
        // and the topology ever disagree.
        var POINT3D_SLOT_PROPERTY = 117;
        $.global.f003fState = {
            comp: comp, fx: fx, slot: POINT3D_SLOT_PROPERTY, log: LOG, tries: 0
        };
        app.scheduleTask("f003fWait()", 3000, false);
    } catch (e) {
        f003Log("f003f", "SCRIPT_ERROR " + String(e) + " (line " + e.line + ")");
        f003Log("f003f", "RESULT_DONE");
    }
})();

function f003fWait() {
    f003Log("f003f", "TICK state=" + (typeof $.global.f003fState));
    var s = $.global.f003fState;
    try {
        if (f003IsReady(s.fx) || ++s.tries > 60) {
            f003fAfterIdle();
        } else {
            app.scheduleTask("f003fWait()", 1000, false);
        }
    } catch (e) {
        f003Log("f003f", "SCRIPT_ERROR in wait: " + String(e));
        f003Log("f003f", "RESULT_DONE");
    }
}

function f003fAfterIdle() {
    var s = $.global.f003fState;
    try {
        var t = f003TokenState(s.fx);
        f003Log(s.log, "TOKEN word=" + t.word + " state=" + t.state);
        f003Log(s.log, "STATUS [" + f003Status(s.fx) + "]");
        if (t.state !== 1) {
            f003Log(s.log, "RESULT F003F ready=0 (definition never published)");
            f003Log(s.log, "RESULT_DONE");
            return;
        }

        // The decision under test, in one line: the two vec3s must have become
        // two DIFFERENT AE controls. 6613 is THREE_D_SPATIAL, 6415 is COLOR.
        // If both read the same, `hint:point3d` did nothing.
        var probe = "<unread>", probeType = "", tint = "<unread>", tintType = "";
        try {
            var p = s.fx.property(s.slot);
            probe = p.name;
            probeType = String(p.propertyValueType);
        } catch (e) { probe = "ERR " + String(e); }
        // The Color pool starts at declaration index 5+48+8+16 = 77, property 78.
        try {
            var c = s.fx.property(86);
            tint = c.name;
            tintType = String(c.propertyValueType);
        } catch (e) { tint = "ERR " + String(e); }
        f003Log(s.log, "POINT3D_SLOT index=" + s.slot + " name=[" + probe + "] valueType=" + probeType);
        f003Log(s.log, "COLOR_SLOT index=78 name=[" + tint + "] valueType=" + tintType);
        f003Log(s.log, "DISTINCT_CONTROLS " + (probeType !== tintType ? 1 : 0));

        // Drive the point somewhere arithmetically obvious: quarter width,
        // half height, z = 50. Expected render, if ADR-0034 §3 holds, is
        // r = 40/160 = 0.25 -> 64, g = 60/120 = 0.5 -> 128, b = 50/100 -> 128.
        var assigned = 0, err = "";
        try {
            s.fx.property(s.slot).setValue([40, 60, 50]);
            assigned = 1;
        } catch (e) { err = String(e); }
        f003Log(s.log, "ASSIGN ok=" + assigned + " value=40,60,50 err=" + err);

        // A keyframe on the same stream: one control, one animatable stream is
        // the reason this kind exists rather than three floats.
        var keyed = 0;
        try {
            var p2 = s.fx.property(s.slot);
            p2.setValueAtTime(0, [40, 60, 50]);
            p2.setValueAtTime(1, [120, 30, 0]);
            keyed = p2.numKeys;
        } catch (e) { err = String(e); }
        f003Log(s.log, "KEYS " + keyed + " err=" + err);

        app.scheduleTask("f003fFinish()", 3000, false);
    } catch (e) {
        f003Log(s.log, "SCRIPT_ERROR " + String(e));
        f003Log(s.log, "RESULT_DONE");
    }
}

function f003fFinish() {
    var s = $.global.f003fState;
    try {
        // Re-read the labels after a full idle window. On the first read the
        // colour slot carried the shader's own name and the Point 3D slot
        // still carried its declared default, from the SAME read — either AE
        // refreshes a THREE_D_SPATIAL stream's name later than a colour's, or
        // the growth-pool rename never reaches the stream. This line is what
        // tells the two apart (2026-08-16).
        var late = "<unread>", lateColor = "<unread>";
        try { late = s.fx.property(s.slot).name; } catch (e) { late = "ERR " + String(e); }
        try { lateColor = s.fx.property(86).name; } catch (e) {}
        f003Log(s.log, "LATE_NAMES point3d=[" + late + "] color=[" + lateColor + "]");
        f003RenderPsd(s.log, s.comp, "f003f_point3d");
        f003Log(s.log, "EXPECT frame0 rgb(64,128,128) from x=40/160 y=60/120 z=50/100");
        f003Log(s.log, "EXPECT frame1 differs (the stream is keyframed)");
        f003Log(s.log, "RESULT F003F rendered=1");
        f003Log(s.log, "RESULT_DONE");
    } catch (e) {
        f003Log(s.log, "SCRIPT_ERROR " + String(e));
        f003Log(s.log, "RESULT_DONE");
    }
}
