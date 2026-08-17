// TR-0030-001 host leg: layer input parameters (`hint:layer`).
//
// Shape of the check: a shader that returns the referenced layer's pixels
// verbatim. If comp-space checkout works, the rendered frame equals the
// referenced layer's colour, not the effect layer's own — and a `None`
// selection must render transparent black instead of failing.
#include "f003_lib.jsxinc"

(function () {
    var LOG = "f003a";
    try {
        try { app.project.close(CloseOptions.DO_NOT_SAVE_CHANGES); } catch (e) {}
        app.newProject();
        var comp = f003NewComp("f003a", 160, 120, 8);

        // The referenced layer: a flat colour distinct from everything else.
        var source = comp.layers.addSolid([20 / 255, 180 / 255, 220 / 255], "sourceLayer", 160, 120, 1.0);
        // The effect layer, a different flat colour so a pass-through or a
        // self-read is immediately visible in the probe.
        var target = comp.layers.addSolid([200 / 255, 40 / 255, 40 / 255], "targetLayer", 160, 120, 1.0);

        var shader =
            "@dynamicfx 1\n@graph\npass main: input, side -> output\n@end\n@pass main\n" +
            "#version 450\n" +
            "// @param side label:\"Side Layer\" hint:layer\n" +
            "layout(location = 0) in vec2 v_uv;\n" +
            "layout(location = 0) out vec4 outColor;\n" +
            "layout(set = 0, binding = 0) uniform texture2D u_in;\n" +
            "layout(set = 0, binding = 1) uniform sampler u_s;\n" +
            "layout(set = 0, binding = 2) uniform FxUniforms {\n" +
            "    vec2 u_resolution;\n" +
            "    float u_time;\n" +
            "    float u_frame;\n" +
            "};\n" +
            "layout(set = 0, binding = 3) uniform texture2D u_side;\n" +
            "void main() {\n" +
            // Composite the side layer OVER the effect's own input rather than
            // returning it verbatim. Returning it verbatim made the two phases
            // indistinguishable in the render: an unassigned selector yields
            // transparent black, and transparent black over the source solid
            // sitting underneath in the comp looks exactly like a successful
            // read of that same solid. That is how a broken layer checkout
            // passed this leg on 2026-08-16 (`PF_CHECKOUT_LAYER` was handed the
            // declaration position instead of the AE parameter index and
            // answered BadCallbackParameter, silently, every frame).
            "    vec4 side = texture(sampler2D(u_side, u_s), v_uv);\n" +
            "    vec4 self = texture(sampler2D(u_in, u_s), v_uv);\n" +
            "    outColor = mix(self, vec4(side.rgb, 1.0), side.a);\n" +
            "}\n" +
            "@endpass\n";

        var fx = target.property("ADBE Effect Parade").addProperty("DynamicFx");
        fx.property(2).expression = f003Wrap(shader);
        f003Log(LOG, "SHADER_WRITTEN layers=" + comp.numLayers);

        // The Layer pool starts at declaration index 110 (5 heads + 104 v1
        // pool slots + Details at 109), so its property index is 111.
        var LAYER_SLOT_PROPERTY = 111;
        var slotName = "<unread>";
        try { slotName = fx.property(LAYER_SLOT_PROPERTY).name; } catch (e) {}
        f003Log(LOG, "LAYER_SLOT index=" + LAYER_SLOT_PROPERTY + " name=[" + slotName + "]");

        // Leave the selector at None for the first render: ADR-0030 §5 says
        // that binds transparent black and must never be an error.
        f003Log(LOG, "PHASE none_selected");
        $.global.f003aState = { comp: comp, fx: fx, source: source, slot: LAYER_SLOT_PROPERTY, log: LOG, tries: 0 };
        // Poll for readiness rather than waiting a fixed interval — the same
        // contract the public README teaches. A blind wait denies the idle
        // observer the main thread it needs; scheduleTask returns control.
        // First tick at 3 s: a `-r` script's globals are not reliably
        // reachable from `app.scheduleTask` until AE has finished with the
        // launch script. Measured 2026-08-15 — 4000 ms fired twice, 500 ms
        // and 120 ms never fired at all, with the setup logged and no error.
        app.scheduleTask("f003aWait()", 3000, false);
    } catch (e) {
        f003Log("f003a", "SCRIPT_ERROR " + String(e) + " (line " + e.line + ")");
        f003Log("f003a", "RESULT_DONE");
    }
})();

function f003aWait() {
    // Log before touching anything: if the scheduled call never arrives, the
    // absence of this line is the evidence. And never route the catch through
    // `s.log` — if `$.global` lost the state, that throws a second time and
    // the exception escapes silently, which is exactly how this leg stalled
    // with no error on 2026-08-15.
    f003Log("f003a", "TICK state=" + (typeof $.global.f003aState));
    var s = $.global.f003aState;
    try {
        if (f003IsReady(s.fx) || ++s.tries > 60) {
            f003aAfterIdle();
        } else {
            if (s.tries % 10 === 1) { f003Log(s.log, "WAIT tries=" + s.tries); }
                app.scheduleTask("f003aWait()", 1000, false);
        }
    } catch (e) {
        f003Log("f003a", "SCRIPT_ERROR in wait: " + String(e));
        f003Log("f003a", "RESULT_DONE");
    }
}

function f003aAfterIdle() {
    var s = $.global.f003aState;
    try {
        var t = f003TokenState(s.fx);
        f003Log(s.log, "TOKEN word=" + t.word + " state=" + t.state + " ready=" + (t.state === 1 ? 1 : 0));
        f003Log(s.log, "STATUS [" + f003Status(s.fx) + "]");
        if (t.state !== 1) {
            f003Log(s.log, "RESULT F003A ready=0 (definition never published)");
            f003Log(s.log, "RESULT_DONE");
            return;
        }
        f003RenderPsd(s.log, s.comp, "f003a_none");

        // Now point the selector at the source layer. Layer params take a
        // layer index within the comp (1-based, topmost first).
        var assigned = 0, err = "";
        try {
            s.fx.property(s.slot).setValue(s.source.index);
            assigned = 1;
        } catch (e) { err = String(e); }
        f003Log(s.log, "ASSIGN ok=" + assigned + " target_index=" + s.source.index + " err=" + err);
        app.scheduleTask("f003aFinish()", 3000, false);
    } catch (e) {
        f003Log(s.log, "SCRIPT_ERROR " + String(e));
        f003Log(s.log, "RESULT_DONE");
    }
}

function f003aFinish() {
    var s = $.global.f003aState;
    try {
        f003RenderPsd(s.log, s.comp, "f003a_assigned");
        // The two must DIFFER, and each must be the right one: unassigned
        // leaves the effect layer's own red untouched, assigned replaces it
        // with the referenced solid's cyan.
        f003Log(s.log, "EXPECT none=rgb(200,40,40) assigned=rgb(20,180,220)");
        f003Log(s.log, "RESULT F003A rendered=2");
        f003Log(s.log, "RESULT_DONE");
    } catch (e) {
        f003Log(s.log, "SCRIPT_ERROR " + String(e));
        f003Log(s.log, "RESULT_DONE");
    }
}

