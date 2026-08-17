// M0 transport spike — S3 / TR-M0-005: undo/redo and project-dirty behavior.
// Part 1: scripted expression writes on a built-in Slider Control.
// Part 2 (indicative only, prototype DynamicFx): how plugin-published state
// (Status rename, SourceChannel write) interacts with undo. Prototype
// behavior does not verify the target rewrite; it maps the host mechanism.
(function () {
    var OUT = ($.getenv("DFX_SPIKE_OUT") || "E:/Code/AePlugin_Dynamicfx/scripts/out/spike/dev") + "/";
    new Folder(OUT).create();
    function logLine(s) {
        var f = new File(OUT + "s3.log");
        f.encoding = "UTF-8";
        if (f.open("a")) { f.write(s + "\n"); f.close(); }
    }
    function dirtyStr() {
        try { return String(app.project.dirty); } catch (e) { return "n/a"; }
    }
    try {
        try { app.project.close(CloseOptions.DO_NOT_SAVE_CHANGES); } catch (eC) {}
        app.newProject();
        var comp = app.project.items.addComp("spike_undo", 320, 240, 1.0, 1.0, 25);
        var solid = comp.layers.addSolid([1, 1, 1], "w", 320, 240, 1.0, 1.0);
        var fx = solid.property("ADBE Effect Parade").addProperty("ADBE Slider Control");
        var prop = fx.property(1);
        var aep = new File(OUT + "s3.aep");
        app.project.save(aep);
        logLine("DIRTY afterSave=" + dirtyStr());

        app.beginUndoGroup("spike-A");
        prop.expression = "111";
        app.endUndoGroup();
        logLine("SET_A expr=" + prop.expression + " dirty=" + dirtyStr());

        app.beginUndoGroup("spike-B");
        prop.expression = "222";
        app.endUndoGroup();
        logLine("SET_B expr=" + prop.expression + " dirty=" + dirtyStr());

        app.executeCommand(16); // Undo
        logLine("UNDO1 expr=" + prop.expression + " dirty=" + dirtyStr());
        app.executeCommand(16);
        logLine("UNDO2 expr=[" + prop.expression + "] enabled=" + prop.expressionEnabled + " dirty=" + dirtyStr());
        app.executeCommand(17); // Redo
        logLine("REDO1 expr=" + prop.expression + " dirty=" + dirtyStr());

        app.project.save(aep);
        logLine("DIRTY afterSave2=" + dirtyStr());

        // ---- Part 2: prototype DynamicFx (indicative host-mechanism data) ----
        var solid2 = comp.layers.addSolid([0.3, 0.3, 0.3], "gray03", 320, 240, 1.0, 1.0);
        var fxD = null, errD = null;
        for (var t = 0; t < 3 && !fxD; t++) {
            try { fxD = solid2.property("ADBE Effect Parade").addProperty("DynamicFx"); }
            catch (eA) { errD = eA; $.sleep(500); }
        }
        if (!fxD) {
            logLine("PROTO unavailable: " + (errD ? errD.toString() : "unknown"));
        } else {
            logLine("PROTO props=" + fxD.numProperties);
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
            var chan = function () {
                try { return fxD.numProperties >= 39 ? String(fxD.property(39).value) : "n/a"; }
                catch (e39) { return "err"; }
            };
            var protoState = function (tag) {
                logLine(tag + " exprLen=" + fxD.property(1).expression.length +
                        " status=[" + fxD.property(3).name + "]" +
                        " f01=[" + fxD.property(5).name + "]" +
                        " chan=" + chan() + " dirty=" + dirtyStr());
            };
            app.beginUndoGroup("spike-proto-src");
            fxD.property(1).expression = "`" + glsl + "`;0";
            app.endUndoGroup();
            $.sleep(2500); // idle observer compile window
            protoState("PROTO after-compile");

            app.project.save(aep);
            logLine("PROTO afterSave dirty=" + dirtyStr());
            $.sleep(1500); // does idle republication re-dirty a saved project?
            logLine("PROTO idleDirty=" + dirtyStr());

            app.executeCommand(16);
            $.sleep(800);
            protoState("PROTO_UNDO1");
            app.executeCommand(16);
            $.sleep(800);
            protoState("PROTO_UNDO2");
            app.executeCommand(17);
            $.sleep(800);
            protoState("PROTO_REDO1");
        }
    } catch (e) {
        logLine("ERROR " + e.toString() + " (line " + e.line + ")");
    }
    logLine("RESULT_DONE");
})();
