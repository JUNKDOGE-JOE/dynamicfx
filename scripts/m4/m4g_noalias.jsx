// M4 harness — G (fresh AE process with DYNAMICFX_NO_ALIAS=1): the same
// two-pass envelope with aliasing disabled must probe identically to m4b
// (ADR-0020 §5 A/B obligation).
#include "m4_lib.jsxinc"
(function () {
    new Folder(m4Out()).create();
    try {
        try { app.project.close(CloseOptions.DO_NOT_SAVE_CHANGES); } catch (eC) {}
        app.newProject();
        app.project.bitsPerChannel = 8;
        try { app.project.workingSpace = ""; } catch (eW) {}
        var comp = app.project.items.addComp("m4", 320, 240, 1.0, 1.0, 25);
        comp.bgColor = [0, 0, 0];
        var solid = comp.layers.addSolid([10 / 255, 200 / 255, 30 / 255], "input", 320, 240, 1.0, 1.0);
        var fx = solid.property("ADBE Effect Parade").addProperty("DynamicFx");
        fx.property(3).expression = "`" + m4TwoPassEnvelope() + "`;0";
        m4Log("m4g.log", "NOALIAS two-pass set exprErr=[" + fx.property(3).expressionError + "]");
        m4Log("m4g.log", "RESULT M4G expr_set=1");
    } catch (e) {
        m4Log("m4g.log", "SCRIPT_ERROR " + String(e));
    }
    m4Log("m4g.log", "RESULT_DONE");
})();
