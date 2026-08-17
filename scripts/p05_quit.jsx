// Save the project to a scratch file and quit AE cleanly (no prompts).
(function () {
    var f = new File("E:/Code/AePlugin_Dynamicfx/scripts/out/dynfx_test.aep");
    app.project.save(f);
    var g = new File("E:/Code/AePlugin_Dynamicfx/scripts/out/test_log.txt");
    g.encoding = "UTF-8";
    if (g.open("w")) { g.write("project saved, quitting"); g.close(); }
    app.quit();
})();
