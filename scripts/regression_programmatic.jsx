// Safe in-process regression for P0 addProperty + P1 idle compilation.
//
// Run from an already-open After Effects via File > Scripts > Run Script
// File. The script creates one scratch comp, calls addProperty exactly once,
// writes only the Source expression, then returns immediately. It never
// retries, renders, purges caches, saves, quits AE, or launches another host.
(function () {
    var lines = [];
    var outDir = new Folder("E:/Code/AePlugin_Dynamicfx/scripts/out");
    var outFile = new File(outDir.fsName + "/regression_programmatic.txt");

    function say(message) {
        lines.push(message);
        $.writeln(message);
    }

    function flush() {
        if (!outDir.exists) {
            outDir.create();
        }
        outFile.encoding = "UTF-8";
        if (outFile.open("w")) {
            outFile.write(lines.join("\n"));
            outFile.close();
        }
    }

    if (!app.project) {
        say("FAIL: no open project");
        flush();
        return;
    }

    app.beginUndoGroup("DynamicFx programmatic regression");
    try {
        var stamp = (new Date()).getTime();
        var comp = app.project.items.addComp(
            "DynamicFx Regression " + stamp,
            320,
            240,
            1.0,
            2.0,
            25
        );
        var solid = comp.layers.addSolid(
            [0.25, 0.25, 0.25],
            "DynamicFx Regression Solid",
            320,
            240,
            1.0,
            2.0
        );
        var effects = solid.property("Effects");
        var before = effects.numProperties;
        var returned = null;
        var addError = null;

        // Deliberately exactly one call. An exception after insertion is a
        // failure and must never be followed by another addProperty call.
        try {
            returned = effects.addProperty("DynamicFx");
        } catch (error) {
            addError = error;
        }

        effects = solid.property("Effects");
        var after = effects.numProperties;
        var inserted = after === before + 1;
        var reacquired = inserted ? effects.property(after) : null;
        var returnedReadable = false;
        var reacquiredReadable = false;

        try {
            returnedReadable = returned !== null && returned.numProperties > 0;
        } catch (_) {}
        try {
            reacquiredReadable = reacquired !== null && reacquired.numProperties > 0;
        } catch (_) {}

        var addPassed = addError === null && inserted && returnedReadable && reacquiredReadable;
        say("P0_ADDPROPERTY=" + (addPassed ? "PASS" : "FAIL"));
        say("effects_before=" + before + " effects_after=" + after);
        say("exception=" + (addError === null ? "none" : addError.toString()));
        say("returned_readable=" + returnedReadable);
        say("reacquired_readable=" + reacquiredReadable);

        if (!inserted || !reacquiredReadable) {
            say("P1_IDLE=NOT_RUN (no readable inserted effect)");
            flush();
            return;
        }

        var shader = [
            "#version 450",
            "layout(location = 0) in vec2 v_uv;",
            "layout(location = 0) out vec4 outColor;",
            "layout(set = 0, binding = 0) uniform texture2D u_input;",
            "layout(set = 0, binding = 1) uniform sampler u_sampler;",
            "// @param u_mix float 0.0 1.0 0.5 Mix",
            "layout(set = 0, binding = 2) uniform FxUniforms {",
            "    vec2 u_resolution;",
            "    float u_time;",
            "    float u_mix;",
            "};",
            "void main() {",
            "    vec4 c = texture(sampler2D(u_input, u_sampler), v_uv);",
            "    outColor = vec4(mix(c.rgb, 1.0 - c.rgb, u_mix), c.a);",
            "}"
        ].join("\n");

        // Source is effect property 1. This is the only P1 mutation; do not
        // click Compile, touch another value, render, purge, or sleep here.
        reacquired.property(1).expression = "`" + shader + "`;0";
        say("P1_IDLE=PENDING_LOG");
        say("source_expression_written=true");
        say("expected_log=idle expression update: ... status=GLSL OK main=true");
        say("scratch_comp=" + comp.name);
    } catch (error) {
        say("FAIL: " + error.toString() + " (line " + error.line + ")");
    } finally {
        app.endUndoGroup();
        flush();
    }
})();
