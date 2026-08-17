// M4 harness — F: the cyclic graph must fail closed — E6 with a line number
// in Status, token in the Invalid state, byte-exact pass-through.
#include "m4_lib.jsxinc"
(function () {
    try {
        var comp = m4FindComp();
        var fx = m4Fx();
        comp.openInViewer();
        comp.saveFrameToPng(0, new File(m4Out() + "m4f_bad.png"));
        var status = "", token = -1;
        try { status = fx.property(4).name; } catch (e1) {}
        try { token = fx.property(5).value; } catch (e2) {}
        m4Log("m4f.log", "CYCLIC status=[" + status + "] token=" + token);
        m4Log("m4f.log", "PNG m4f_bad.png expect passthrough solid (10,200,30)");
        m4Log("m4f.log", "RESULT M4F status=[" + status + "] token=" + token);
    } catch (e) {
        m4Log("m4f.log", "SCRIPT_ERROR " + String(e));
    }
    m4Log("m4f.log", "RESULT_DONE");
})();
