// M3 harness — Q: clean shutdown, no prompts.
(function () {
    var OUT = ($.getenv("DFX_M3_OUT") || "E:/Code/AePlugin_Dynamicfx/scripts/out/m3/dev") + "/";
    new Folder(OUT).create();
    var f = new File(OUT + "m3q.log");
    f.encoding = "UTF-8";
    if (f.open("a")) { f.write("QUIT requested\nRESULT_DONE\n"); f.close(); }
    try { app.project.close(CloseOptions.DO_NOT_SAVE_CHANGES); } catch (e) {}
    app.quit();
})();
