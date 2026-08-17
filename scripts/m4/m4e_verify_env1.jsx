// M4 harness — E: the one-pass envelope must render identically to the raw
// module; then stage a cyclic graph for the E6 diagnostic leg.
#include "m4_lib.jsxinc"
(function () {
    try {
        var comp = m4FindComp();
        var fx = m4Fx();
        comp.openInViewer();
        comp.saveFrameToPng(0, new File(m4Out() + "m4e_env1.png"));
        var status = "";
        try { status = fx.property(4).name; } catch (e1) {}
        m4Log("m4e.log", "ONE_PASS status=[" + status + "] png expect center (64,64,0)");

        var cyclic = "@dynamicfx 1\n@graph\npass a: b_out -> a_out\npass b: a_out -> b_out\npass c: a_out -> output\n@end\n" +
            "@pass a\n" + m4InvertPass() + "@endpass\n" +
            "@pass b\n" + m4InvertPass() + "@endpass\n" +
            "@pass c\n" + m4InvertPass() + "@endpass\n";
        fx.property(2).expression = "`" + cyclic + "`;0";
        m4Log("m4e.log", "CYCLIC set");
        m4Log("m4e.log", "RESULT M4E status=[" + status + "]");
    } catch (e) {
        m4Log("m4e.log", "SCRIPT_ERROR " + String(e));
    }
    m4Log("m4e.log", "RESULT_DONE");
})();
