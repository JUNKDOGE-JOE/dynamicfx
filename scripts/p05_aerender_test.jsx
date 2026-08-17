// P0.5 isolated test, run inside aerender.exe (fresh process):
// fresh instance, NO expression -> hash 0 -> sidecar posterize shader ->
// uncommitted slots -> annotation default u_levels=16 -> 0.3 gray -> 0.25.
(function () {
    var log = [];
    function say(s) { log.push(s); }
    function flush() {
        var f = new File("E:/Code/AePlugin_Dynamicfx/scripts/out/test_log.txt");
        f.encoding = "UTF-8";
        if (f.open("w")) { f.write(log.join("\n")); f.close(); }
    }
    try {
        var comp = app.project.items.addComp("dynfx_p05_iso", 320, 240, 1.0, 1.0, 25);
        var solid = comp.layers.addSolid([0.3, 0.3, 0.3], "gray03", 320, 240, 1.0, 1.0);
        var fx = null, err = null;
        for (var t = 0; t < 3 && !fx; t++) {
            try { fx = solid.property("Effects").addProperty("DynamicFx"); }
            catch (e) { err = e; $.sleep(500); }
        }
        if (!fx) throw err;
        say("fx ok, props=" + fx.numProperties);
        comp.openInViewer();
        comp.saveFrameToPng(0.5, new File("E:/Code/AePlugin_Dynamicfx/scripts/out/dynfx_p05.png"));
        say("png saved");
    } catch (e) {
        say("ERROR: " + e.toString() + " (line " + e.line + ")");
    }
    flush();
})();
