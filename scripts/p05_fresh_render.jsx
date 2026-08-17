// Fresh comp from scratch -> fresh render clones with hash=0 -> sidecar
// shader (posterize) + uncommitted slots -> annotation default u_levels=16.
(function () {
    var log = [];
    function say(s) { log.push(s); }
    function flush() {
        var f = new File("E:/Code/AePlugin_Dynamicfx/scripts/out/test_log.txt");
        f.encoding = "UTF-8";
        if (f.open("w")) { f.write(log.join("\n")); f.close(); }
    }
    try {
        var comp = app.project.items.addComp("dynfx_p05_fresh", 320, 240, 1.0, 1.0, 25);
        var solid = comp.layers.addSolid([0.3, 0.3, 0.3], "gray03", 320, 240, 1.0, 1.0);
        var fx = null, err = null;
        for (var t = 0; t < 3 && !fx; t++) {
            try { fx = solid.property("Effects").addProperty("DynamicFx"); }
            catch (e) { err = e; $.sleep(500); }
        }
        if (!fx) throw err;
        say("fx ok, props=" + fx.numProperties);

        app.purge(PurgeTarget.ALL_CACHES);
        comp.saveFrameToPng(0.5, new File("E:/Code/AePlugin_Dynamicfx/scripts/out/dynfx_p05.png"));
        say("png saved");
    } catch (e) {
        say("ERROR: " + e.toString() + " (line " + e.line + ")");
    }
    flush();
})();
