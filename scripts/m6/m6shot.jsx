// M6 harness — open the 8-bpc accumulator comp and select its layer so the
// Effect Controls panel shows the DynamicFx entry (the no-MFR-warning-icon
// screenshot evidence, ADR-0023 verification).
#include "m6_lib.jsxinc"
(function () {
    try {
        var comp = m6Find("m6acc8");
        comp.openInViewer();
        for (var i = 1; i <= comp.numLayers; i++) {
            comp.layer(i).selected = comp.layer(i).name === "input";
        }
        app.executeCommand(app.findMenuCommandId("Effect Controls DynamicFx"));
    } catch (e) {}
    try {
        // Fallback: the generic Effect Controls open command.
        app.executeCommand(2163);
    } catch (e2) {}
    m6Log("m6shot.log", "RESULT_DONE");
})();
