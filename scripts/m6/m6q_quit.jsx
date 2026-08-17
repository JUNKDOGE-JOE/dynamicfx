// M6 harness — quit AE without saving.
(function () {
    try { app.project.close(CloseOptions.DO_NOT_SAVE_CHANGES); } catch (e) {}
    app.quit();
})();
