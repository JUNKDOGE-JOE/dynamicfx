// M3 harness — H5: after another idle window past the save, the project
// must still be clean — idle republication of unchanged state never
// re-dirties a saved project (ADR-0015 §3, extending TR-M0-005 to target
// code).
(function () {
    var OUT = ($.getenv("DFX_M3_OUT") || "E:/Code/AePlugin_Dynamicfx/scripts/out/m3/dev") + "/";
    new Folder(OUT).create();
    function logLine(s) {
        var f = new File(OUT + "m3h5.log");
        f.encoding = "UTF-8";
        if (f.open("a")) { f.write(s + "\n"); f.close(); }
    }
    try {
        var dirty = "unknown";
        try { dirty = String(app.project.dirty); } catch (eD) {}
        logLine("IDLE_LATER dirty=" + dirty + " (expect false)");
        logLine("RESULT M3H5 dirty=" + dirty);
    } catch (e) {
        logLine("SCRIPT_ERROR " + String(e));
    }
    logLine("RESULT_DONE");
})();
