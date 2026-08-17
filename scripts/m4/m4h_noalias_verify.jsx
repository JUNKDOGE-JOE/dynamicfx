// M4 harness — H: render the no-alias frame for the A/B comparison.
#include "m4_lib.jsxinc"
(function () {
    try {
        var comp = m4FindComp();
        comp.openInViewer();
        comp.saveFrameToPng(0, new File(m4Out() + "m4h_noalias.png"));
        m4Log("m4h.log", "PNG m4h_noalias.png expect center (191,191,255) — identical to m4b");
        m4Log("m4h.log", "RESULT M4H rendered=1");
    } catch (e) {
        m4Log("m4h.log", "SCRIPT_ERROR " + String(e));
    }
    m4Log("m4h.log", "RESULT_DONE");
})();
