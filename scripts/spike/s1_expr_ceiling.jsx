// M0 transport spike — S1 / TR-M0-002: expression length ceiling.
// Sets progressively larger committed expressions on a built-in Slider
// Control and verifies immediate read-back. Cap 16 MB (logged, not silent).
(function () {
    var OUT = ($.getenv("DFX_SPIKE_OUT") || "E:/Code/AePlugin_Dynamicfx/scripts/out/spike/dev") + "/";
    new Folder(OUT).create();
    function logLine(s) {
        var f = new File(OUT + "s1.log");
        f.encoding = "UTF-8";
        if (f.open("a")) { f.write(s + "\n"); f.close(); }
    }
    // ASCII body with newlines, guaranteed free of "*/".
    var CHUNK = "ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789 abcdefghijklmnopqrstuvwxyz GLSL uv time resolution pass graph \n";
    function body(n) {
        var s = CHUNK;
        while (s.length < n) { s = s + s; }
        return s.substring(0, n);
    }
    function expr(total) { return "/*" + body(total - 5) + "*/1"; }
    function attempt(prop, size) {
        var s = expr(size);
        var t0 = (new Date()).getTime();
        var setOk = 1, err = "";
        try { prop.expression = s; } catch (e) { setOk = 0; err = e.toString(); }
        var ms = (new Date()).getTime() - t0;
        var match = 0, rbLen = -1, exprErr = "";
        if (setOk) {
            try {
                var rb = prop.expression;
                rbLen = rb.length;
                match = (rb === s) ? 1 : 0;
                exprErr = prop.expressionError;
            } catch (e2) { exprErr = "readback throws: " + e2.toString(); }
        }
        logLine("CEIL size=" + size + " setOk=" + setOk + " match=" + match + " rbLen=" + rbLen +
                " ms=" + ms + " err=" + err + " exprErr=" + exprErr);
        try { prop.expression = ""; } catch (e3) {}
        return setOk === 1 && match === 1;
    }
    try {
        try { app.project.close(CloseOptions.DO_NOT_SAVE_CHANGES); } catch (eC) {}
        app.newProject();
        var comp = app.project.items.addComp("spike_ceiling", 320, 240, 1.0, 1.0, 25);
        var solid = comp.layers.addSolid([1, 1, 1], "w", 320, 240, 1.0, 1.0);
        var fx = solid.property("ADBE Effect Parade").addProperty("ADBE Slider Control");
        var prop = fx.property(1);
        logLine("SETUP ok fxMatch=" + fx.matchName);

        var sizes = [1024, 4096, 16384, 65536, 262144, 1048576, 2097152, 4194304, 8388608, 16777216];
        var lastOk = 0, firstFail = 0;
        for (var i = 0; i < sizes.length; i++) {
            if (attempt(prop, sizes[i])) { lastOk = sizes[i]; }
            else { firstFail = sizes[i]; break; }
        }
        if (firstFail > 0) {
            var lo = lastOk, hi = firstFail;
            for (var b = 0; b < 6 && hi - lo > 1024; b++) {
                var mid = Math.floor((lo + hi) / 2 / 1024) * 1024;
                if (mid <= lo || mid >= hi) { break; }
                if (attempt(prop, mid)) { lo = mid; } else { hi = mid; }
            }
            logLine("BOUNDARY lastOk=" + lo + " firstFail=" + hi);
        } else {
            logLine("BOUNDARY lastOk=" + lastOk + " firstFail=none (cap 16MB reached, cap is explicit not a host limit)");
        }
    } catch (e) {
        logLine("ERROR " + e.toString() + " (line " + e.line + ")");
    }
    logLine("RESULT_DONE");
})();
