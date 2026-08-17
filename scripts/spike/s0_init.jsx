// M0 transport spike — S0 host sanity (ADR-0009).
// Confirms scripted file writes work and records host identity.
(function () {
    var OUT = ($.getenv("DFX_SPIKE_OUT") || "E:/Code/AePlugin_Dynamicfx/scripts/out/spike/dev") + "/";
    new Folder(OUT).create();
    function logLine(s) {
        var f = new File(OUT + "s0.log");
        f.encoding = "UTF-8";
        if (f.open("a")) { f.write(s + "\n"); f.close(); }
    }
    try {
        logLine("HOST version=" + app.version + " build=" + app.buildName + " lang=" + app.isoLanguage);
        logLine("OS " + $.os);
        var dirtyType = "unavailable";
        try { dirtyType = typeof app.project.dirty; } catch (eD) { dirtyType = "throws:" + eD.toString(); }
        logLine("DIRTY_API typeof=" + dirtyType);
        var pref = "unreadable";
        try {
            pref = app.preferences.getPrefAsLong("Main Pref Section", "Pref_SCRIPTING_FILE_NETWORK_SECURITY");
        } catch (eP) { pref = "throws"; }
        logLine("SCRIPT_FILE_PREF " + pref);
        logLine("TEMP " + Folder.temp.fsName);
    } catch (e) {
        logLine("ERROR " + e.toString() + " (line " + e.line + ")");
    }
    logLine("RESULT_DONE");
})();
