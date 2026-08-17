// M0 transport spike — S2 / TR-M0-003: long-expression save/reopen fidelity.
// Four payload variants (ASCII, hostile punctuation, CRLF line endings,
// Unicode) at 4KB/256KB/1MB. Byte-exact comparison after save + reopen.
(function () {
    var OUT = ($.getenv("DFX_SPIKE_OUT") || "E:/Code/AePlugin_Dynamicfx/scripts/out/spike/dev") + "/";
    new Folder(OUT).create();
    function logLine(s) {
        var f = new File(OUT + "s2.log");
        f.encoding = "UTF-8";
        if (f.open("a")) { f.write(s + "\n"); f.close(); }
    }
    function grow(token, n) {
        var s = token;
        while (s.length < n) { s = s + s; }
        return s.substring(0, n);
    }
    // Variant bodies. None may contain "*/".
    var TOKENS = {
        A: "ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789 abcdefghijklmnopqrstuvwxyz\n",
        B: "AB\"'`\\{}();#@$%^&|<>?,.!~[]-_=+ \t^ * x / y //line ;;\n",
        C: "LINEENDING TEST 0123456789 ABCDEFGHIJ\r\n",
        D: "\u4e2d\u6587\u6ce8\u91ca\u6d4b\u8bd5 \u5168\u89d2\uff21\uff22\uff23 \ud83c\udfa8 \u00e9\u00fc\u00f1\n"
    };
    function buildBody(variant, n) {
        if (variant === "D") {
            var t = TOKENS.D, s = "";
            while (s.length + t.length <= n) { s = s.length ? s + s : t; }
            if (!s.length) { s = t; }
            while (s.length + t.length <= n) { s = s + t; }
            return s; // whole tokens only: never split a surrogate pair
        }
        return grow(TOKENS[variant], n);
    }
    function firstDiff(a, b) {
        var n = Math.min(a.length, b.length);
        var cap = Math.min(n, 200000);
        for (var i = 0; i < cap; i++) {
            if (a.charCodeAt(i) !== b.charCodeAt(i)) { return i; }
        }
        if (a.length !== b.length && cap === n) { return n; }
        return cap < n ? -2 : -1; // -2: identical within scan cap; -1: identical
    }
    function findProp() {
        var comp = null;
        for (var i = 1; i <= app.project.items.length; i++) {
            if (app.project.items[i].name === "spike_rt") { comp = app.project.items[i]; break; }
        }
        if (!comp) { throw new Error("spike_rt comp not found after reopen"); }
        return comp.layer(1).property("ADBE Effect Parade").property(1).property(1);
    }
    try {
        try { app.project.close(CloseOptions.DO_NOT_SAVE_CHANGES); } catch (eC) {}
        app.newProject();
        var comp = app.project.items.addComp("spike_rt", 320, 240, 1.0, 1.0, 25);
        var solid = comp.layers.addSolid([1, 1, 1], "w", 320, 240, 1.0, 1.0);
        solid.property("ADBE Effect Parade").addProperty("ADBE Slider Control");
        var prop = findProp();
        logLine("SETUP ok");

        var variants = ["A", "B", "C", "D"];
        var sizes = [4096, 262144, 1048576];
        for (var v = 0; v < variants.length; v++) {
            for (var s = 0; s < sizes.length; s++) {
                var variant = variants[v], size = sizes[s];
                var payload = "/*" + buildBody(variant, size - 5) + "*/1";
                var line = "RT var=" + variant + " size=" + payload.length;
                try {
                    prop.expression = payload;
                    var imm = prop.expression;
                    line += " immMatch=" + (imm === payload ? 1 : 0);
                    if (imm !== payload) {
                        line += " immLen=" + imm.length + " immDiff=" + firstDiff(payload, imm);
                    }
                    var aep = new File(OUT + "s2_rt_" + variant + ".aep");
                    app.project.save(aep);
                    app.open(aep);
                    prop = findProp();
                    var rb = prop.expression;
                    line += " reopenMatch=" + (rb === payload ? 1 : 0);
                    if (rb !== payload) {
                        line += " rbLen=" + rb.length + " rbDiff=" + firstDiff(payload, rb);
                        // Stability probe: does the host's own normalized text
                        // survive a second round trip unchanged?
                        prop.expression = rb;
                        app.project.save(aep);
                        app.open(aep);
                        prop = findProp();
                        var rb2 = prop.expression;
                        line += " stabilized=" + (rb2 === rb ? 1 : 0);
                    }
                } catch (e1) {
                    line += " ERROR=" + e1.toString();
                    try { prop = findProp(); } catch (e2) {}
                }
                logLine(line);
                try { prop.expression = ""; } catch (e3) {}
            }
        }
    } catch (e) {
        logLine("ERROR " + e.toString() + " (line " + e.line + ")");
    }
    logLine("RESULT_DONE");
})();
