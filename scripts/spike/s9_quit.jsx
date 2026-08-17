// M0 transport spike — S9: clean shutdown, no prompts.
(function () {
    var OUT = ($.getenv("DFX_SPIKE_OUT") || "E:/Code/AePlugin_Dynamicfx/scripts/out/spike/dev") + "/";
    new Folder(OUT).create();
    var f = new File(OUT + "s9.log");
    f.encoding = "UTF-8";
    if (f.open("a")) { f.write("QUIT requested\nRESULT_DONE\n"); f.close(); }
    try { app.project.close(CloseOptions.DO_NOT_SAVE_CHANGES); } catch (e) {}
    app.quit();
})();
