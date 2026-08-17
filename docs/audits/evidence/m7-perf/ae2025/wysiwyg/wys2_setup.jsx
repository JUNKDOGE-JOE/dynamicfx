(function () {
    var OUT = "C:/Users/A/AppData/Local/Temp/claude/E--Code-AePlugin-Dynamicfx/fbefc838-0280-4b84-9c35-f140b6366b08/scratchpad/m7opt/";
    var comp = app.project.items.addComp("wys2", 320, 240, 1.0, 2.0, 25);
    comp.bgColor = [0, 0, 0];
    // GRAY input solid: passthrough=gray, shader=BLUE, empty=black — 3-way.
    var solid = comp.layers.addSolid([0.5, 0.5, 0.5], "input", 320, 240, 1.0, 2.0);
    var fx = solid.property("ADBE Effect Parade").addProperty("DynamicFx");
    var src = "@dynamicfx 1\n@graph\npass main: input -> output\n@end\n@pass main\n" +
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
        "void main() {\n" +
        "    vec4 base = texture(sampler2D(u_in, u_s), v_uv);\n" +
        "    outColor = vec4(0.0, 0.0, 1.0, 1.0) + base * 0.0;\n" +
        "}\n" +
        "@endpass\n";
    fx.property(2).expression = "`" + src + "`;0";
    solid.selected = false;
    // Synchronous render INSIDE this script: the idle observer cannot have
    // run yet, so this frame is rendered by the UNCOMPILED instance and
    // enters the frame cache as passthrough gray.
    comp.saveFrameToPng(0, new File(OUT + "wys2_precompile.png"));
    comp.openInViewer();
    return "committed BLUE + synchronous pre-compile render saved";
})();
