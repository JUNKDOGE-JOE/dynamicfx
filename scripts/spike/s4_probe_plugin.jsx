// M0 transport spike — S4 / TR-M0-004 + TR-M0-006.
// Drives the throwaway DynamicFxProbe plugin.
//
// TR-M0-004 (sequence transport capacity): the plugin's flatten() emits a
// checksummed blob of DFX_PROBE_KB kilobytes (env, read at process start,
// so this run must be a COLD AE launch). save writes it into the .aep;
// reopen runs unflatten() which verifies magic+length+crc. Evidence is in
// the plugin log (SEQ_FLATTEN env_kb / SEQ_UNFLATTEN crc_ok) plus the .aep
// size delta observed here. Driving size through the environment sidesteps
// the confirmed host behavior that scripted setValue() never reaches the
// plugin as a committed parameter change.
//
// TR-M0-006 (popup menu mutation): the plugin attempts set_options(5 items)
// + PF_UpdateParamUI once per process at first UpdateParamsUi. Success is
// judged by whether setValue past the original 1..4 range is accepted and
// whether the popup name changed.
//
// Plugin-side evidence: %TEMP%\dynamicfx_probe.log. Host-side view: s4.log.
(function () {
    var OUT = ($.getenv("DFX_SPIKE_OUT") || "E:/Code/AePlugin_Dynamicfx/scripts/out/spike/dev") + "/";
    new Folder(OUT).create();
    function logLine(s) {
        var f = new File(OUT + "s4.log");
        f.encoding = "UTF-8";
        if (f.open("a")) { f.write(s + "\n"); f.close(); }
    }
    function dirtyStr() {
        try { return String(app.project.dirty); } catch (e) { return "n/a"; }
    }
    function findComp(name) {
        for (var i = 1; i <= app.project.items.length; i++) {
            if (app.project.items[i].name === name) { return app.project.items[i]; }
        }
        return null;
    }
    function probeFx() {
        var comp = findComp("spike_probe");
        if (!comp) { throw new Error("spike_probe comp not found"); }
        return comp.layer(1).property("ADBE Effect Parade").property(1);
    }
    try {
        logLine("ENV DFX_PROBE_KB=" + ($.getenv("DFX_PROBE_KB") || "(unset)"));
        try { app.project.close(CloseOptions.DO_NOT_SAVE_CHANGES); } catch (eC) {}
        app.newProject();
        var comp = app.project.items.addComp("spike_probe", 320, 240, 1.0, 1.0, 25);
        var solid = comp.layers.addSolid([0.5, 0.5, 0.5], "gray05", 320, 240, 1.0, 1.0);
        var fx = null, err = null;
        for (var t = 0; t < 3 && !fx; t++) {
            try { fx = solid.property("ADBE Effect Parade").addProperty("DynamicFxProbe"); }
            catch (eA) { err = eA; $.sleep(500); }
        }
        if (!fx) { throw (err || new Error("addProperty(DynamicFxProbe) failed")); }
        $.sleep(1500); // allow the one-shot popup mutation to run
        logLine("PROBE props=" + fx.numProperties);
        var kbIdx = -1, popupIdx = -1, statusIdx = -1;
        for (var i = 1; i <= fx.numProperties; i++) {
            var p = fx.property(i);
            if (p.name === "Payload KB") { kbIdx = i; }
            if (p.name.indexOf("Popup Probe") === 0) { popupIdx = i; }
            if (p.name.indexOf("Probe Status") === 0 || p.name.indexOf("Probe:") === 0) { statusIdx = i; }
        }
        logLine("IDX kb=" + kbIdx + " popup=" + popupIdx + " status=" + statusIdx);
        if (popupIdx < 0) { throw new Error("popup param not located"); }

        // ---- TR-M0-006: is the popup menu now larger than the setup 4? ----
        logLine("POPUP name=[" + fx.property(popupIdx).name + "] value=" + fx.property(popupIdx).value);
        var set5 = "accepted";
        try { fx.property(popupIdx).setValue(5); } catch (e5) { set5 = e5.toString(); }
        logLine("POPUP setValue(5)=" + set5 + " (accepted => menu grew to 5; range error => menu unchanged at 4)");
        try { fx.property(popupIdx).setValue(2); } catch (e2) {}

        // ---- TR-M0-004: sequence transport at DFX_PROBE_KB ----
        var aep = new File(OUT + "s4.aep");
        var t0 = (new Date()).getTime();
        app.project.save(aep);
        var saveMs = (new Date()).getTime() - t0;
        var aepBytes = aep.length;
        logLine("SEQ_SAVE aepBytes=" + aepBytes + " saveMs=" + saveMs +
                " (compare vs env_kb; plugin log SEQ_FLATTEN has real byte count)");

        app.open(aep);
        $.sleep(500);
        logLine("SEQ_REOPEN done (plugin log SEQ_UNFLATTEN has magic/len/crc verdict)");

        // ---- render must not raise a modal error at project bit depth ----
        var comp2 = findComp("spike_probe");
        comp2.openInViewer();
        try {
            comp2.saveFrameToPng(0, new File(OUT + "s4_render.png"));
            logLine("RENDER_PNG ok dirty=" + dirtyStr());
        } catch (eR) {
            logLine("RENDER_PNG ERROR=" + eR.toString());
        }
        logLine("DONE_MAIN");
    } catch (e) {
        logLine("ERROR " + e.toString() + " (line " + e.line + ")");
    }
    logLine("RESULT_DONE");
})();
