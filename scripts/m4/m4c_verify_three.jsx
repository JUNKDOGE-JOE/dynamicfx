// M4 harness — C: the three-pass double-invert must equal the plain
// generator (center (64,64,0)); then stage the raw single-pass generator
// for the envelope/raw identity comparison.
#include "m4_lib.jsxinc"
(function () {
    try {
        var comp = m4FindComp();
        var fx = m4Fx();
        comp.openInViewer();
        comp.saveFrameToPng(0, new File(m4Out() + "m4c_three.png"));
        var status = "";
        try { status = fx.property(5).name; } catch (e1) {}
        m4Log("m4c.log", "THREE_PASS status=[" + status + "] png expect center (64,64,0)");

        fx.property(3).expression = "`" + m4GenPass() + "`;0";
        m4Log("m4c.log", "RAW set");
        m4Log("m4c.log", "RESULT M4C status=[" + status + "]");
    } catch (e) {
        m4Log("m4c.log", "SCRIPT_ERROR " + String(e));
    }
    m4Log("m4c.log", "RESULT_DONE");
})();
