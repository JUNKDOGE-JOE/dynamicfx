// TR-0037-001 host leg: pool valid range (ADR-0037, public issue #5).
//
// The defect: a Float slot registered `valid 0..1` at PARAMS_SETUP put a
// ceiling of 1.0 (and a floor of 0.0) under every float parameter and a
// ceiling of 10 under every integer, whatever `@param min:/max:` declared —
// `PF_UpdateParamUI` cannot change the valid range, and After Effects clamps
// the rendered value to it. The fix registers a wide valid range and treats
// the annotation as the SLIDER range.
//
// Shape of the check: one shader declares a float `min:2 max:200`, a float
// `min:-1 max:1` and an int `min:0 max:100`, and paints them into R/G/B with
// encodings whose expected 8-bit values are exact integers. Two renders:
//   1. defaults only (the annotation `default:` written by the idle observer)
//   2. after `setValue` on all three streams
// Every expected value is far from what the OLD artifact produced (1.0 -> R 1,
// 0.0 -> G 128, 10 -> B 26), so a stale binary cannot pass by accident.
//
// Then `examples/thermal.glsl` renders once at its defaults — its `glow`
// default is 1.2, so its palette had never been seen on a host before this.
//
// Property indexes (declaration index + 1): Float slot 0 = 6, Float slot 1 =
// 7, Integer slot 0 = 54. `growth_pool_property_indexes_match_the_harness`
// (src/host/params.rs) pins them against the topology.
#include "f003_lib.jsxinc"

(function () {
    var LOG = "f003h";
    try {
        try { app.project.close(CloseOptions.DO_NOT_SAVE_CHANGES); } catch (e) {}
        app.newProject();
        var comp = f003NewComp("f003h", 160, 120, 8);
        var solid = comp.layers.addSolid([0, 0, 0], "target", 160, 120, 1.0);

        // R = wide / 200      -> default 40 -> 0.2  -> 51 ; set 150 -> 0.75 -> 191
        // G = (neg + 1) / 2   -> default -0.6 -> 0.2 -> 51 ; set 0.2 -> 0.6 -> 153
        // B = count / 100     -> default 60 -> 0.6  -> 153; set 80 -> 0.8 -> 204
        var shader =
            "@dynamicfx 1\n@graph\npass main: input -> output\n@end\n@pass main\n" +
            "#version 450\n" +
            "// @param wide label:\"Wide\" min:2 max:200 default:40\n" +
            "// @param neg label:\"Neg\" min:-1 max:1 default:-0.6\n" +
            "// @param count label:\"Count\" min:0 max:100 default:60\n" +
            "layout(location = 0) in vec2 v_uv;\n" +
            "layout(location = 0) out vec4 outColor;\n" +
            "layout(set = 0, binding = 0) uniform texture2D u_in;\n" +
            "layout(set = 0, binding = 1) uniform sampler u_s;\n" +
            "layout(set = 0, binding = 2) uniform FxUniforms {\n" +
            "    vec2 u_resolution;\n" +
            "    float u_time;\n" +
            "    float u_frame;\n" +
            "    float wide;\n" +
            "    float neg;\n" +
            "    int count;\n" +
            "};\n" +
            "void main() {\n" +
            "    outColor = vec4(wide / 200.0, (neg + 1.0) * 0.5, float(count) / 100.0, 1.0);\n" +
            "}\n" +
            "@endpass\n";

        var fx = solid.property("ADBE Effect Parade").addProperty("DynamicFx");
        fx.property(2).expression = f003Wrap(shader);
        f003Log(LOG, "SHADER_WRITTEN");

        $.global.f003hState = {
            comp: comp, fx: fx, log: LOG, tries: 0,
            WIDE: 6, NEG: 7, COUNT: 54
        };
        app.scheduleTask("f003hWait()", 3000, false);
    } catch (e) {
        f003Log("f003h", "SCRIPT_ERROR " + String(e) + " (line " + e.line + ")");
        f003Log("f003h", "RESULT_DONE");
    }
})();

function f003hWait() {
    var s = $.global.f003hState;
    try {
        if (f003IsReady(s.fx) || ++s.tries > 60) {
            f003hAfterIdle();
        } else {
            app.scheduleTask("f003hWait()", 1000, false);
        }
    } catch (e) {
        f003Log("f003h", "SCRIPT_ERROR in wait: " + String(e));
        f003Log("f003h", "RESULT_DONE");
    }
}

// The scripting-visible range of a slot. Whether this reflects the declared
// `min:/max:` (the host's UI copy honouring the valid_* write) or the wide
// registered range is measured here, not assumed — the render is what the
// decision is about, this line only records the typing courtesy.
function f003hRange(fx, index) {
    try {
        var p = fx.property(index);
        return "name=[" + p.name + "] min=" + p.minValue + " max=" + p.maxValue + " value=" + p.value;
    } catch (e) { return "ERR " + String(e); }
}

function f003hAfterIdle() {
    var s = $.global.f003hState;
    try {
        var t = f003TokenState(s.fx);
        f003Log(s.log, "TOKEN word=" + t.word + " state=" + t.state);
        f003Log(s.log, "STATUS [" + f003Status(s.fx) + "]");
        if (t.state !== 1) {
            f003Log(s.log, "RESULT F003H ready=0 (definition never published)");
            f003Log(s.log, "RESULT_DONE");
            return;
        }
        // Give the idle observer one more window to write the annotation
        // defaults into the fresh bindings before reading them back.
        app.scheduleTask("f003hDefaults()", 3000, false);
    } catch (e) {
        f003Log(s.log, "SCRIPT_ERROR " + String(e));
        f003Log(s.log, "RESULT_DONE");
    }
}

function f003hDefaults() {
    var s = $.global.f003hState;
    try {
        f003Log(s.log, "RANGE wide  " + f003hRange(s.fx, s.WIDE));
        f003Log(s.log, "RANGE neg   " + f003hRange(s.fx, s.NEG));
        f003Log(s.log, "RANGE count " + f003hRange(s.fx, s.COUNT));

        // Render 1: the annotation defaults, untouched. 40 and 60 are above
        // the old ceilings (1.0 / 10) and -0.6 is below the old floor.
        f003RenderPsd(s.log, s.comp, "f003h_defaults");
        f003Log(s.log, "EXPECT f003h_defaults rgb(51,51,153) from wide=40 neg=-0.6 count=60");
        f003Log(s.log, "EXPECT_OLD_ARTIFACT rgb(1,128,26) (clamped 1.0 / 0.0 / 10)");

        // Drive all three streams. The typing courtesy is recorded but does
        // not gate: a rejected out-of-range setValue is a UI fact, not a
        // render fact.
        var err = "";
        var ok = 0;
        try { s.fx.property(s.WIDE).setValue(150); ok++; } catch (e) { err += " wide:" + String(e); }
        try { s.fx.property(s.NEG).setValue(0.2); ok++; } catch (e) { err += " neg:" + String(e); }
        try { s.fx.property(s.COUNT).setValue(80); ok++; } catch (e) { err += " count:" + String(e); }
        f003Log(s.log, "ASSIGN ok=" + ok + "/3 wide=150 neg=0.2 count=80 err=[" + err + "]");
        var bound = "accepted";
        try { s.fx.property(s.WIDE).setValue(0.3); s.fx.property(s.WIDE).setValue(150); }
        catch (e) { bound = "rejected: " + String(e); }
        f003Log(s.log, "TYPING_BOUND wide=0.3 (below declared min 2) -> " + bound);

        app.scheduleTask("f003hSet()", 2000, false);
    } catch (e) {
        f003Log(s.log, "SCRIPT_ERROR " + String(e));
        f003Log(s.log, "RESULT_DONE");
    }
}

function f003hSet() {
    var s = $.global.f003hState;
    try {
        f003Log(s.log, "RANGE_AFTER wide  " + f003hRange(s.fx, s.WIDE));
        f003Log(s.log, "RANGE_AFTER neg   " + f003hRange(s.fx, s.NEG));
        f003Log(s.log, "RANGE_AFTER count " + f003hRange(s.fx, s.COUNT));
        f003RenderPsd(s.log, s.comp, "f003h_set");
        f003Log(s.log, "EXPECT f003h_set rgb(191,153,204) from wide=150 neg=0.2 count=80");
        f003Log(s.log, "EXPECT_OLD_ARTIFACT rgb(1,128,26)");

        // Thermal at its shipped defaults — first host sighting of the
        // intended palette (glow default 1.2 was clamped to 1.0 before).
        var here = new File($.fileName).parent;            // scripts/f003
        var thermal = new File(here.parent.parent.fsName + "/examples/thermal.glsl");
        var src = "";
        if (thermal.exists && thermal.open("r")) { thermal.encoding = "UTF-8"; src = thermal.read(); thermal.close(); }
        if (src === "") {
            f003Log(s.log, "THERMAL_MISSING " + thermal.fsName);
            f003Log(s.log, "RESULT F003H rendered=2 thermal=0");
            f003Log(s.log, "RESULT_DONE");
            return;
        }
        var comp2 = f003NewComp("f003h_thermal", 320, 180, 8);
        var solid2 = comp2.layers.addSolid([0.1, 0.1, 0.1], "thermal", 320, 180, 1.0);
        var fx2 = solid2.property("ADBE Effect Parade").addProperty("DynamicFx");
        fx2.property(2).expression = f003Wrap(src);
        f003Log(s.log, "THERMAL_WRITTEN bytes=" + src.length);
        s.comp2 = comp2; s.fx2 = fx2; s.tries = 0;
        app.scheduleTask("f003hThermalWait()", 3000, false);
    } catch (e) {
        f003Log(s.log, "SCRIPT_ERROR " + String(e));
        f003Log(s.log, "RESULT_DONE");
    }
}

function f003hThermalWait() {
    var s = $.global.f003hState;
    try {
        if (f003IsReady(s.fx2) || ++s.tries > 60) {
            var t = f003TokenState(s.fx2);
            f003Log(s.log, "THERMAL_TOKEN word=" + t.word + " state=" + t.state + " status=[" + f003Status(s.fx2) + "]");
            // Glow Radius is thermal's first float (default 1.2): the slot
            // must now hold 1.2, not 1.0.
            f003Log(s.log, "THERMAL_RANGE glow " + f003hRange(s.fx2, s.WIDE));
            f003RenderPsd(s.log, s.comp2, "f003h_thermal");
            f003Log(s.log, "EXPECT f003h_thermal VISUAL: the intended palette (deep/main/light body, hot core, rim, glow) at glow=1.2 — evidence PSD, no numeric gate");
            f003Log(s.log, "RESULT F003H rendered=3");
            f003Log(s.log, "RESULT_DONE");
        } else {
            app.scheduleTask("f003hThermalWait()", 1000, false);
        }
    } catch (e) {
        f003Log(s.log, "SCRIPT_ERROR in thermal wait: " + String(e));
        f003Log(s.log, "RESULT_DONE");
    }
}
