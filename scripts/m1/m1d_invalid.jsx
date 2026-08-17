// M1 harness — D: scripted write of an invalid source. The driver sleeps
// afterwards so the idle observer processes the transition; m1e verifies the
// diagnostic + pass-through.
(function () {
    var OUT = ($.getenv("DFX_M1_OUT") || "E:/Code/AePlugin_Dynamicfx/scripts/out/m1/dev") + "/";
    new Folder(OUT).create();
    function logLine(s) {
        var f = new File(OUT + "m1d.log");
        f.encoding = "UTF-8";
        if (f.open("a")) { f.write(s + "\n"); f.close(); }
    }
    try {
        var comp = null;
        for (var j = 1; j <= app.project.numItems; j++) {
            if (app.project.item(j) instanceof CompItem && app.project.item(j).name === "m1") {
                comp = app.project.item(j);
                break;
            }
        }
        var fx = comp.layer(1).property("ADBE Effect Parade").property(1);
        fx.property(2).expression = "`this is not glsl`;0";
        logLine("INVALID_EXPR set exprErr=[" + fx.property(2).expressionError + "]");
        logLine("RESULT M1D invalid_set=1");
    } catch (e) {
        logLine("SCRIPT_ERROR " + String(e));
    }
    logLine("RESULT_DONE");
})();
