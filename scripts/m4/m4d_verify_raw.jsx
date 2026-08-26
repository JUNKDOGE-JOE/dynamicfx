// M4 harness — D: render the raw single-pass generator, then stage the SAME
// module as a one-pass envelope — the two must probe identically (ADR-0018
// verification obligation).
#include "m4_lib.jsxinc"
(function () {
    try {
        var comp = m4FindComp();
        var fx = m4Fx();
        comp.openInViewer();
        comp.saveFrameToPng(0, new File(m4Out() + "m4d_raw.png"));
        m4Log("m4d.log", "RAW png expect center (64,64,0)");

        var one = "@dynamicfx 1\n@graph\npass main: input -> output\n@end\n" +
            "@pass main\n" + m4GenPass() + "@endpass\n";
        fx.property(3).expression = "`" + one + "`;0";
        m4Log("m4d.log", "ONE_PASS_ENVELOPE set");
        m4Log("m4d.log", "RESULT M4D staged=1");
    } catch (e) {
        m4Log("m4d.log", "SCRIPT_ERROR " + String(e));
    }
    m4Log("m4d.log", "RESULT_DONE");
})();
