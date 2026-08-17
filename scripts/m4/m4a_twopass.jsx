// M4 harness — A: two-pass envelope (gradient generator → invert). The idle
// observer must parse the grammar, compile both passes, and publish.
//! include m4_lib
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
        fx.property(2).expression = "`" + m4TwoPassEnvelope() + "`;0";
        m4Log("m4a.log", "TWO_PASS set exprErr=[" + fx.property(2).expressionError + "]");
        m4Log("m4a.log", "RESULT M4A expr_set=1");
    } catch (e) {
        m4Log("m4a.log", "SCRIPT_ERROR " + String(e));
    }
    m4Log("m4a.log", "RESULT_DONE");
})();
