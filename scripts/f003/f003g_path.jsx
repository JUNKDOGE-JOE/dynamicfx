// TR-0035-001 host leg: path parameters (`hint:path`).
//
// Shape of the check: a mask is drawn on the effect layer at known vertex
// positions, and the shader paints one channel from `textureSize` (the vertex
// count, ADR-0035 §4) and the others from vertex 0's normalized position
// (§3). One PNG then answers three questions at once — did the count arrive,
// did the position arrive, and did it arrive normalized.
//
// The unassigned selector is rendered FIRST, because §5's "renders rather than
// fails" is the obligation that costs a user their project when it is wrong.
#include "f003_lib.jsxinc"

(function () {
    var LOG = "f003g";
    try {
        try { app.project.close(CloseOptions.DO_NOT_SAVE_CHANGES); } catch (e) {}
        app.newProject();
        var comp = f003NewComp("f003g", 160, 120, 8);
        var solid = comp.layers.addSolid([0, 0, 0], "target", 160, 120, 1.0);

        // r = vertex 0 x, g = vertex 0 y, b = vertexCount / 16.
        // texelFetch, not texture(): ADR-0035's cost note — an Rgba32Float
        // texture is only filterable where the adapter offers
        // FLOAT32_FILTERABLE, and a vertex is a value to read exactly, not to
        // interpolate between.
        var shader =
            "@dynamicfx 1\n@graph\npass main: input, outline -> output\n@end\n@pass main\n" +
            "#version 450\n" +
            "// @param outline label:\"Outline\" hint:path\n" +
            "layout(location = 0) in vec2 v_uv;\n" +
            "layout(location = 0) out vec4 outColor;\n" +
            "layout(set = 0, binding = 0) uniform texture2D u_in;\n" +
            "layout(set = 0, binding = 1) uniform sampler u_s;\n" +
            "layout(set = 0, binding = 2) uniform FxUniforms {\n" +
            "    vec2 u_resolution;\n" +
            "    float u_time;\n" +
            "    float u_frame;\n" +
            "};\n" +
            "layout(set = 0, binding = 3) uniform texture2D u_path;\n" +
            "void main() {\n" +
            "    ivec2 size = textureSize(sampler2D(u_path, u_s), 0);\n" +
            "    vec4 v0 = texelFetch(sampler2D(u_path, u_s), ivec2(0, 0), 0);\n" +
            "    outColor = vec4(v0.x, v0.y, float(size.x) / 16.0, 1.0);\n" +
            "}\n" +
            "@endpass\n";

        var fx = solid.property("ADBE Effect Parade").addProperty("DynamicFx");
        fx.property(3).expression = f003Wrap(shader);

        // A closed rectangle mask with corners at known pixel positions. AE
        // reports N segments and N+1 vertices for a closed path, so a
        // four-corner rectangle should read back as 5.
        var maskAdded = 0, maskErr = "";
        try {
            var mask = solid.property("ADBE Mask Parade").addProperty("ADBE Mask Atom");
            var shape = new Shape();
            shape.vertices = [[40, 30], [120, 30], [120, 90], [40, 90]];
            shape.closed = true;
            mask.property("ADBE Mask Shape").setValue(shape);
            // NONE, so the mask never mattes the layer away — it is being read
            // as data here, not used as a mask.
            mask.maskMode = MaskMode.NONE;
            maskAdded = 1;
        } catch (e) { maskErr = String(e); }
        f003Log(LOG, "MASK ok=" + maskAdded + " err=" + maskErr);

        // The Path pool starts at declaration index 124 (5 heads + 104 v1 slots
        // + Details at 109 + 4 Layer + 2 Gradient + 8 Point 3D), so property
        // 125. `growth_pool_property_indexes_match_the_harness` fails in CI if
        // this and the topology ever disagree.
        var PATH_SLOT_PROPERTY = 125;
        $.global.f003gState = {
            comp: comp, fx: fx, slot: PATH_SLOT_PROPERTY, log: LOG, tries: 0
        };
        app.scheduleTask("f003gWait()", 3000, false);
    } catch (e) {
        f003Log("f003g", "SCRIPT_ERROR " + String(e) + " (line " + e.line + ")");
        f003Log("f003g", "RESULT_DONE");
    }
})();

function f003gWait() {
    f003Log("f003g", "TICK state=" + (typeof $.global.f003gState));
    var s = $.global.f003gState;
    try {
        if (f003IsReady(s.fx) || ++s.tries > 60) {
            f003gAfterIdle();
        } else {
            app.scheduleTask("f003gWait()", 1000, false);
        }
    } catch (e) {
        f003Log("f003g", "SCRIPT_ERROR in wait: " + String(e));
        f003Log("f003g", "RESULT_DONE");
    }
}

function f003gAfterIdle() {
    var s = $.global.f003gState;
    try {
        var t = f003TokenState(s.fx);
        f003Log(s.log, "TOKEN word=" + t.word + " state=" + t.state);
        f003Log(s.log, "STATUS [" + f003Status(s.fx) + "]");
        if (t.state !== 1) {
            f003Log(s.log, "RESULT F003G ready=0 (definition never published)");
            f003Log(s.log, "RESULT_DONE");
            return;
        }

        var name = "<unread>", vt = "";
        try {
            var p = s.fx.property(s.slot);
            name = p.name;
            vt = String(p.propertyValueType);
        } catch (e) { name = "ERR " + String(e); }
        f003Log(s.log, "PATH_SLOT index=" + s.slot + " name=[" + name + "] valueType=" + vt);

        // §5 first: unassigned must render, not fail.
        f003RenderPsd(s.log, s.comp, "f003g_none");
        f003Log(s.log, "EXPECT none: rgb(0,0,16) — one zero vertex, count 1 of 16");

        // Then point the selector at the mask. A Path param's value is the
        // 1-based index of the layer's path; 0 is NONE.
        var assigned = 0, err = "";
        try {
            s.fx.property(s.slot).setValue(1);
            assigned = 1;
        } catch (e) { err = String(e); }
        f003Log(s.log, "ASSIGN ok=" + assigned + " value=1 err=" + err);
        app.scheduleTask("f003gFinish()", 3000, false);
    } catch (e) {
        f003Log(s.log, "SCRIPT_ERROR " + String(e));
        f003Log(s.log, "RESULT_DONE");
    }
}

function f003gFinish() {
    var s = $.global.f003gState;
    try {
        f003RenderPsd(s.log, s.comp, "f003g_assigned");
        f003Log(s.log, "EXPECT assigned: rgb(64,64,80) — vertex0 (40/160, 30/120), 5 vertices of 16");
        f003Log(s.log, "RESULT F003G rendered=2");
        f003Log(s.log, "RESULT_DONE");
    } catch (e) {
        f003Log(s.log, "SCRIPT_ERROR " + String(e));
        f003Log(s.log, "RESULT_DONE");
    }
}
