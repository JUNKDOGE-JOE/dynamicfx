// Probe the state of the p05 test comp and dump to a text file.
(function () {
    var lines = [];
    lines.push("project items: " + app.project.items.length);
    var comp = null;
    for (var i = 1; i <= app.project.items.length; i++) {
        var it = app.project.items[i];
        if (it.name === "dynfx_p05_test") { comp = it; break; }
    }
    if (!comp) {
        lines.push("comp dynfx_p05_test NOT FOUND");
    } else {
        lines.push("comp found, layers: " + comp.layers.length);
        var solid = comp.layer(1);
        var fx = null;
        try { fx = solid.property("Effects").property("DynamicFx"); } catch (e) {
            lines.push("effect lookup error: " + e.toString());
        }
        if (fx) {
            lines.push("effect found, numProperties: " + fx.numProperties);
            var src = fx.property(2);
            lines.push("Source expr len: " + src.expression.length);
            lines.push("Source expr head: " + src.expression.substring(0, 40));
            lines.push("F01 value: " + fx.property(6).value);
            var st = fx.property(4);
            lines.push("Status name: " + st.name);
        } else {
            lines.push("DynamicFx effect NOT on layer");
        }
    }
    var f = new File("E:/Code/AePlugin_Dynamicfx/scripts/out/probe.txt");
    f.encoding = "UTF-8";
    if (f.open("w")) {
        f.write(lines.join("\n"));
        f.close();
    } else {
        lines.push("(file write blocked)");
    }
})();
