// M1 harness — A / TR-M1-003 host leg: single addProperty("DynamicFx"),
// ADR-0013 topology shape, Language defaults to GLSL (position 1, the only
// menu entry), and popup persistence across save/reopen.
(function () {
    var OUT = ($.getenv("DFX_M1_OUT") || "E:/Code/AePlugin_Dynamicfx/scripts/out/m1/dev") + "/";
    new Folder(OUT).create();
    function logLine(s) {
        var f = new File(OUT + "m1a.log");
        f.encoding = "UTF-8";
        if (f.open("a")) { f.write(s + "\n"); f.close(); }
    }
    try {
        try { app.project.close(CloseOptions.DO_NOT_SAVE_CHANGES); } catch (eC) {}
        app.newProject();
        app.project.bitsPerChannel = 8;
        var ws0 = "";
        try { ws0 = app.project.workingSpace; } catch (eW0) {}
        // Empty string disables color management. Never assign "None": AE
        // stores it as a literal profile NAME and later raises a modal
        // "profile None is missing" error on project load.
        try { app.project.workingSpace = ""; } catch (eW2) {}
        var ws1 = "";
        try { ws1 = app.project.workingSpace; } catch (eW3) {}
        logLine("PROJECT bpc=" + app.project.bitsPerChannel + " ws_before=[" + ws0 + "] ws_after=[" + ws1 + "]");

        var comp = app.project.items.addComp("m1", 320, 240, 1.0, 1.0, 25);
        comp.bgColor = [0, 0, 0];
        var solid = comp.layers.addSolid([10 / 255, 200 / 255, 30 / 255], "input", 320, 240, 1.0, 1.0);

        var addOk = 0, addErr = "";
        var fx = null;
        try {
            fx = solid.property("ADBE Effect Parade").addProperty("DynamicFx");
            addOk = 1;
        } catch (eA) { addErr = String(eA); }
        if (!fx) {
            logLine("RESULT M1A add_ok=0 err=" + addErr);
            logLine("RESULT_DONE");
            return;
        }

        var props = fx.numProperties;
        var names = [];
        for (var i = 1; i <= Math.min(props, 6); i++) {
            try { names.push(i + ":" + fx.property(i).name); } catch (eN) { names.push(i + ":ERR"); }
        }
        var langValue = -1;
        try { langValue = fx.property(1).value; } catch (eL) {}

        // The v1 menu is exactly ["GLSL"]: position 2 must be out of range.
        var popupReject = 0;
        try { fx.property(1).setValue(2); } catch (eP) { popupReject = 1; }
        var langAfterReject = -1;
        try { langAfterReject = fx.property(1).value; } catch (eL2) {}

        logLine("TOPOLOGY add_ok=" + addOk + " props=" + props +
            " heads=[" + names.join(" | ") + "]" +
            " lang_value=" + langValue +
            " popup_reject=" + popupReject +
            " lang_after_reject=" + langAfterReject);

        var aep = new File(OUT + "m1a.aep");
        app.project.save(aep);
        app.project.close(CloseOptions.DO_NOT_SAVE_CHANGES);
        app.open(aep);

        var comp2 = null;
        for (var j = 1; j <= app.project.numItems; j++) {
            if (app.project.item(j) instanceof CompItem && app.project.item(j).name === "m1") {
                comp2 = app.project.item(j);
                break;
            }
        }
        var reopenProps = -1, reopenLang = -1, reopenName = "";
        if (comp2 && comp2.numLayers >= 1) {
            var fx2 = comp2.layer(1).property("ADBE Effect Parade").property(1);
            if (fx2) {
                reopenProps = fx2.numProperties;
                try { reopenLang = fx2.property(1).value; } catch (eR) {}
                try { reopenName = fx2.property(1).name; } catch (eR2) {}
            }
        }
        logLine("REOPEN props=" + reopenProps + " lang_value=" + reopenLang + " lang_name=" + reopenName);
        logLine("RESULT M1A add_ok=" + addOk + " props=" + props +
            " lang_value=" + langValue + " popup_reject=" + popupReject +
            " reopen_props=" + reopenProps + " reopen_lang=" + reopenLang);
    } catch (e) {
        logLine("SCRIPT_ERROR " + String(e));
    }
    logLine("RESULT_DONE");
})();
