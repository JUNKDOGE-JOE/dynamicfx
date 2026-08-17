// M5 harness — X: color-management scripting probe. The 2025 host defaults
// new projects to the OCIO engine, which rejects bitsPerChannel=16 from
// scripting. Enumerate what this host exposes so the fixture scenarios can
// switch engines deterministically.
#include "m5_lib.jsxinc"
(function () {
    new Folder(m5Out()).create();
    var LOG = "m5x.log";
    function probe(name) {
        var t = "", v = "";
        try { t = typeof app.project[name]; } catch (e1) { t = "ERR " + e1; }
        try { v = String(app.project[name]); } catch (e2) { v = "ERR " + e2; }
        m5Log(LOG, "PROP " + name + " type=[" + t + "] value=[" + v + "]");
    }
    function tryBpc16(tag) {
        var ok = "";
        try { app.project.bitsPerChannel = 16; ok = "OK bpc=" + app.project.bitsPerChannel; }
        catch (e) { ok = "ERR " + e; }
        m5Log(LOG, "BPC16 " + tag + " -> " + ok);
    }
    try {
        try { app.project.close(CloseOptions.DO_NOT_SAVE_CHANGES); } catch (eC) {}
        app.newProject();
        var names = [
            "colorManagementMode", "colorEngine", "colorSettings",
            "ocioConfigName", "ocioConfigFile",
            "workingColorSpace", "displayColorSpace", "viewTransform",
            "workingSpace", "workingGamma", "linearizeWorkingSpace",
            "linearBlending", "bitsPerChannel", "expressionEngine"
        ];
        for (var i = 0; i < names.length; i++) { probe(names[i]); }
        m5Log(LOG, "GLOBALS ColorManagementMode=" + (typeof ColorManagementMode) +
            " CMSettings=" + (typeof CMSettings));
        tryBpc16("under-default-engine");

        // Try candidate engine switches, re-testing 16-bpc after each.
        var attempts = [
            ["colorManagementMode-3407179", function () { app.project.colorManagementMode = 3407179; }],
            ["colorManagementMode-0", function () { app.project.colorManagementMode = 0; }],
            ["colorManagementMode-1", function () { app.project.colorManagementMode = 1; }],
            ["colorManagementMode-2", function () { app.project.colorManagementMode = 2; }],
            ["ocioConfigName-empty", function () { app.project.ocioConfigName = ""; }],
            ["workingSpace-empty", function () { app.project.workingSpace = ""; }]
        ];
        for (var a = 0; a < attempts.length; a++) {
            var res = "";
            try { attempts[a][1](); res = "SET_OK"; } catch (eS) { res = "SET_ERR " + eS; }
            m5Log(LOG, "TRY " + attempts[a][0] + " -> " + res +
                " cmm=[" + (function(){ try { return String(app.project.colorManagementMode); } catch(e){ return "?"; } })() + "]");
            if (res === "SET_OK") { tryBpc16("after-" + attempts[a][0]); }
        }
        m5Log(LOG, "RESULT M5X done");
    } catch (e) {
        m5Log(LOG, "SCRIPT_ERROR " + String(e) + " (line " + e.line + ")");
    }
    m5Log(LOG, "RESULT_DONE");
})();
