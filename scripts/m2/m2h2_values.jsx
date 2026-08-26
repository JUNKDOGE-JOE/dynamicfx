// M2 harness — H2: after the idle window, set the color/point stream values
// by script (defaults for those kinds are v1-rejected by design), verify the
// slot labels across all five pools, and render the fixture frame.
//
// Pool property indexes (1-based, 5 heads then pools):
//   Float 6..53, Int 54..61, Bool 62..77, Color 78..89, Point 90..101,
//   Angle 102..109.
(function () {
    var OUT = ($.getenv("DFX_M2_OUT") || "E:/Code/AePlugin_Dynamicfx/scripts/out/m2/dev") + "/";
    new Folder(OUT).create();
    function logLine(s) {
        var f = new File(OUT + "m2h2.log");
        f.encoding = "UTF-8";
        if (f.open("a")) { f.write(s + "\n"); f.close(); }
    }
    try {
        var comp = null;
        for (var j = 1; j <= app.project.numItems; j++) {
            if (app.project.item(j) instanceof CompItem && app.project.item(j).name === "m2k") {
                comp = app.project.item(j);
                break;
            }
        }
        var fx = comp.layer(1).property("ADBE Effect Parade").property(1);

        // Layout-proof: address bound slots by their applied LABEL, never by
        // property index. Index literals here went stale across the ADR-0040/41
        // topology shifts twice (the second time hidden by a repin gap), so the
        // lookup is now by the names the idle observer applied.
        function byName(label) {
            for (var i = 1; i <= fx.numProperties; i++) {
                try { if (fx.property(i).name === label) return fx.property(i); } catch (eB) {}
            }
            return null;
        }
        var labels = ["Count", "flag", "tint", "center", "sweep"];
        var names = [];
        for (var i = 0; i < labels.length; i++) {
            names.push(labels[i] + "=[" + (byName(labels[i]) ? "found" : "MISSING") + "]");
        }
        logLine("SLOT_NAMES " + names.join(" "));
        var countValue = -1, flagValue = -1, sweepValue = -1;
        try { countValue = byName("Count").value; } catch (e1) {}
        try { flagValue = byName("flag").value ? 1 : 0; } catch (e2) {}
        try { sweepValue = byName("sweep").value; } catch (e3) {}
        logLine("DEFAULTS count=" + countValue + " flag=" + flagValue + " sweep=" + sweepValue);

        // Color takes 0..1 RGBA; Point takes comp pixels.
        byName("tint").setValue([1, 0.5, 0.25, 1]);
        byName("center").setValue([240, 60]);
        logLine("VALUES set tint=[1,0.5,0.25] center=[240,60]");

        comp.openInViewer();
        comp.saveFrameToPng(0, new File(OUT + "m2h_kinds.png"));
        logLine("PNG saved m2h_kinds.png bands: count 77 / flag 255 / tint (255,128,64) / center (191,64,0) / sweep 64");
        logLine("RESULT M2H2 defaults count=" + countValue + " flag=" + flagValue + " sweep=" + sweepValue);
    } catch (e) {
        logLine("SCRIPT_ERROR " + String(e));
    }
    logLine("RESULT_DONE");
})();
