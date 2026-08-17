// M4 harness — B: verify the two-pass render (gain default 0.5: gen center
// (64,64,0) → inverted (191,191,255)), then stage the three-pass chain
// (gen → invert → invert = gen again, exercising physical-slot reuse).
#include "m4_lib.jsxinc"
(function () {
    try {
        var comp = m4FindComp();
        var fx = m4Fx();
        comp.openInViewer();
        comp.saveFrameToPng(0, new File(m4Out() + "m4b_two.png"));
        var status = "", slot1 = "";
        try { status = fx.property(4).name; } catch (e1) {}
        try { slot1 = fx.property(6).name; } catch (e2) {}
        m4Log("m4b.log", "TWO_PASS status=[" + status + "] slot1=[" + slot1 + "]");
        m4Log("m4b.log", "PNG m4b_two.png expect center (191,191,255), (32,120)=(242,191,255)");

        var three = "@dynamicfx 1\n@graph\npass gen: input -> a\npass inv1: a -> b\npass inv2: b -> output\n@end\n" +
            "@pass gen\n" + m4GenPass() + "@endpass\n" +
            "@pass inv1\n" + m4InvertPass() + "@endpass\n" +
            "@pass inv2\n" + m4InvertPass() + "@endpass\n";
        fx.property(2).expression = "`" + three + "`;0";
        m4Log("m4b.log", "THREE_PASS set");
        m4Log("m4b.log", "RESULT M4B status=[" + status + "]");
    } catch (e) {
        m4Log("m4b.log", "SCRIPT_ERROR " + String(e));
    }
    m4Log("m4b.log", "RESULT_DONE");
})();
