(function () {
    if (parseFloat(app.version) < 26.5) throw Error("Liquid Glass requires After Effects 26.5 or newer.");
    var comp = app.project && app.project.activeItem;
    if (!(comp instanceof CompItem)) throw Error("Open a composition and select its shape layers.");
    var layers = comp.selectedLayers;
    if (!layers.length) throw Error("Select one or more shape layers.");
    for (var i = 0; i < layers.length; i++) {
        if (!(layers[i] instanceof ShapeLayer) || layers[i].locked)
            throw Error("Select unlocked shape layers only.");
    }
    var sourceFile = new File(new File($.fileName).parent.fsName + "/liquid-glass.glsl");
    sourceFile.encoding = "UTF-8";
    if (!sourceFile.open("r")) throw Error("Keep liquid-glass.glsl beside this script.");
    var shader = sourceFile.read();
    sourceFile.close();
    if (shader.indexOf("@dynamicfx 1") !== 0 || shader.indexOf("hint:coverage") < 0)
        throw Error("The adjacent Liquid Glass source is invalid.");
    app.beginUndoGroup("Apply Liquid Glass");
    try {
        for (var i = 0; i < layers.length; i++) {
            var layer = layers[i], effects = layer.property("ADBE Effect Parade"), effect = null;
            for (var j = 1; j <= effects.numProperties; j++) {
                var candidate = effects.property(j);
                if (candidate.matchName === "DynamicFx" && candidate.name === "Liquid Glass") {
                    if (effect) throw Error("More than one Liquid Glass effect on " + layer.name);
                    effect = candidate;
                }
            }
            if (!effect) effect = effects.addProperty("DynamicFx");
            effect.name = "Liquid Glass";
            var language = null, source = null;
            for (var j = 1; j <= effect.numProperties; j++) {
                var property = effect.property(j);
                if (property.name === "Language") language = property;
                if (property.name === "Source" || property.name === "Source (use expression)") source = property;
            }
            if (!language || !source) throw Error("The installed DynamicFX parameter layout is unsupported.");
            language.setValue(1);
            source.expression = "`" + shader + "`;0";
            source.expressionEnabled = true;
            layer.adjustmentLayer = true;
        }
    } finally {
        app.endUndoGroup();
    }
})();
