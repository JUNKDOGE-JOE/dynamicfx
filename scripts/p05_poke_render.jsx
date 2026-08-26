// Purge caches to force a real re-render, then save a frame.
(function () {
    var log = [];
    function say(s) { log.push(s); }
    function flush() {
        var f = new File("E:/Code/AePlugin_Dynamicfx/scripts/out/test_log.txt");
        f.encoding = "UTF-8";
        if (f.open("w")) { f.write(log.join("\n")); f.close(); }
    }
    try {
        var comp = null;
        for (var i = 1; i <= app.project.items.length; i++) {
            if (app.project.items[i].name === "dynfx_p05_test") { comp = app.project.items[i]; break; }
        }
        if (!comp) throw new Error("comp not found");
        var fx = comp.layer(1).property("Effects").property("DynamicFx");
        say("expr len: " + fx.property(2).expression.length);

        app.purge(PurgeTarget.ALL_CACHES);
        say("caches purged");

        comp.saveFrameToPng(0.5, new File("E:/Code/AePlugin_Dynamicfx/scripts/out/dynfx_p05.png"));
        say("png saved at t=0.5");
    } catch (e) {
        say("ERROR: " + e.toString() + " (line " + e.line + ")");
    }
    flush();
})();
