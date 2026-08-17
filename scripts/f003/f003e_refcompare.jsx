// Side-by-side parameter dump: the reference gradient effect (bfxMapRamp)
// against DynamicFx. The question this answers is the one that reading a
// stripped binary cannot: what does a *working* gradient control look like
// from the AE side — property type, group shape, sub-properties?
#include "f003_lib.jsxinc"

(function () {
    var LOG = "f003e";
    function dump(fx, label) {
        f003Log(LOG, "=== " + label + " matchName=[" + fx.matchName + "] numProperties=" + fx.numProperties);
        for (var i = 1; i <= fx.numProperties; i++) {
            var p, line;
            try {
                p = fx.property(i);
            } catch (e) {
                f003Log(LOG, "  [" + i + "] <unreadable> " + String(e));
                continue;
            }
            line = "  [" + i + "] name=[" + p.name + "] match=[" + p.matchName + "]";
            // propertyType distinguishes a real property from an indexed
            // group; propertyValueType is the value kind AE assigned.
            try { line += " type=" + String(p.propertyType); } catch (e) {}
            try { line += " valueType=" + String(p.propertyValueType); } catch (e) {}
            try { line += " numProps=" + String(p.numProperties); } catch (e) {}
            try { line += " canVary=" + String(p.canVaryOverTime); } catch (e) {}
            f003Log(LOG, line);
        }
    }

    try {
        try { app.project.close(CloseOptions.DO_NOT_SAVE_CHANGES); } catch (e) {}
        app.newProject();
        var comp = f003NewComp("f003e", 320, 120, 8);
        var solid = comp.layers.addSolid([0, 0, 0], "input", 320, 120, 1.0);

        var refFx = null, refErr = "";
        // Try both the English and Chinese display names, plus the match name.
        // Match name decoded straight from the reference PiPL (eMNA).
        var candidates = ["bfx Map Ramp"];
        for (var c = 0; c < candidates.length && !refFx; c++) {
            try { refFx = solid.property("ADBE Effect Parade").addProperty(candidates[c]); }
            catch (e) { refErr = String(e); }
        }
        if (refFx) {
            dump(refFx, "REFERENCE bfxMapRamp");
        } else {
            f003Log(LOG, "REFERENCE not applicable: " + refErr);
            f003Log(LOG, "available effect names must be checked in the Effects panel");
        }

        var ours = solid.property("ADBE Effect Parade").addProperty("DynamicFx");
        // Only the gradient slots matter here; the pool head is already known.
        f003Log(LOG, "=== OURS DynamicFx numProperties=" + ours.numProperties);
        for (var j = 111; j <= Math.min(119, ours.numProperties); j++) {
            var q, l;
            try { q = ours.property(j); } catch (e) { continue; }
            l = "  [" + j + "] name=[" + q.name + "]";
            try { l += " type=" + String(q.propertyType); } catch (e) {}
            try { l += " valueType=" + String(q.propertyValueType); } catch (e) {}
            try { l += " numProps=" + String(q.numProperties); } catch (e) {}
            f003Log(LOG, l);
        }
        f003Log(LOG, "RESULT_DONE");
    } catch (e) {
        f003Log(LOG, "SCRIPT_ERROR " + String(e) + " (line " + e.line + ")");
        f003Log(LOG, "RESULT_DONE");
    }
})();
