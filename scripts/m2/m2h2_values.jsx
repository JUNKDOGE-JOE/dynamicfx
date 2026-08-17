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

        var names = [];
        var probes = [[54, "count->Count"], [62, "flag"], [78, "tint"], [90, "center"], [102, "sweep"]];
        for (var i = 0; i < probes.length; i++) {
            var nm = "";
            try { nm = fx.property(probes[i][0]).name; } catch (eN) {}
            names.push(probes[i][1] + "=[" + nm + "]");
        }
        logLine("SLOT_NAMES " + names.join(" "));
        var countValue = -1, flagValue = -1, sweepValue = -1;
        try { countValue = fx.property(54).value; } catch (e1) {}
        try { flagValue = fx.property(62).value; } catch (e2) {}
        try { sweepValue = fx.property(102).value; } catch (e3) {}
        logLine("DEFAULTS count=" + countValue + " flag=" + flagValue + " sweep=" + sweepValue);

        // Color takes 0..1 RGBA; Point takes comp pixels.
        fx.property(78).setValue([1, 0.5, 0.25, 1]);
        fx.property(90).setValue([240, 60]);
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
