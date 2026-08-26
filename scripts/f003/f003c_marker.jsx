// TR-0015-001 host leg: the not-ready marker (E53 PublicationPending).
//
// The obligation that actually protects users is the *negative* one: an
// ordinary compile must never transit E53. A transient E53 would dirty the
// project for a sub-second state and make the marker cry wolf. So this leg
// polls the token densely across a normal compile and records every distinct
// word it ever held.
#include "f003_lib.jsxinc"

var F003C_SEEN = {};
var F003C_POLLS = 0;

(function () {
    var LOG = "f003c";
    try {
        try { app.project.close(CloseOptions.DO_NOT_SAVE_CHANGES); } catch (e) {}
        app.newProject();
        var comp = f003NewComp("f003c", 64, 64, 8);
        var solid = comp.layers.addSolid([0, 0, 0], "input", 64, 64, 1.0);

        var shader =
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
            "void main() { outColor = vec4(v_uv, 0.0, 1.0); }\n";

        var fx = solid.property("ADBE Effect Parade").addProperty("DynamicFx");
        // Record the pre-write word so "never authored" is in the evidence
        // alongside every later state.
        f003Log(LOG, "BEFORE_WRITE word=" + f003TokenState(fx).word);
        fx.property(3).expression = f003Wrap(shader);
        f003Log(LOG, "SHADER_WRITTEN");

        $.global.f003cState = { fx: fx, log: LOG };
        // First tick at 3 s: a `-r` script's globals are not reliably
        // reachable from `app.scheduleTask` until AE has finished with the
        // launch script. Measured 2026-08-15 — 4000 ms fired twice, 500 ms
        // and 120 ms never fired at all, with the setup logged and no error.
        app.scheduleTask("f003cPoll()", 3000, false);
    } catch (e) {
        f003Log("f003c", "SCRIPT_ERROR " + String(e) + " (line " + e.line + ")");
        f003Log("f003c", "RESULT_DONE");
    }
})();

function f003cPoll() {
    var s = $.global.f003cState;
    try {
        var t = f003TokenState(s.fx);
        if (!F003C_SEEN[t.word]) {
            F003C_SEEN[t.word] = 1;
            f003Log(s.log, "SAW word=" + t.word + " state=" + t.state + " payload=" + t.payload);
        }
        F003C_POLLS++;
        // ~24 s of dense polling: comfortably longer than the 1 s idle scan,
        // and long enough that a slow publication would still be caught.
        if (t.state === 1 || F003C_POLLS > 200) {
            var words = [];
            for (var w in F003C_SEEN) { if (F003C_SEEN.hasOwnProperty(w)) { words.push(w); } }
            // E53 encodes as (53 << 2) | 0b10 = 214.
            var sawPending = F003C_SEEN[214] ? 1 : 0;
            f003Log(s.log, "STATUS [" + f003Status(s.fx) + "]");
            f003Log(s.log, "POLLS " + F003C_POLLS + " distinct_words=[" + words.join(",") + "]");
            f003Log(s.log, "RESULT F003C ready=" + (t.state === 1 ? 1 : 0) + " saw_e53=" + sawPending);
            f003Log(s.log, "EXPECT ready=1 saw_e53=0 (an ordinary compile never transits E53)");
            f003Log(s.log, "RESULT_DONE");
            return;
        }
            app.scheduleTask("f003cPoll()", 400, false);
    } catch (e) {
        f003Log(s.log, "SCRIPT_ERROR " + String(e));
        f003Log(s.log, "RESULT_DONE");
    }
}

