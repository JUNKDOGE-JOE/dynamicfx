// M4 harness — Q: clean shutdown, no prompts.
#include "m4_lib.jsxinc"
(function () {
    new Folder(m4Out()).create();
    m4Log("m4q.log", "QUIT requested\nRESULT_DONE");
    try { app.project.close(CloseOptions.DO_NOT_SAVE_CHANGES); } catch (e) {}
    app.quit();
})();
