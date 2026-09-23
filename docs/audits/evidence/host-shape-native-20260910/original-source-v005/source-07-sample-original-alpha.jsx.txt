(function () {
    if (!app.project.file || app.project.file.name !== "host-shape-native.aep") {
        throw Error("Wrong disposable project");
    }
    var samples = [];
    var ids = [1, 90, 33, 17];
    for (var i = 0; i < ids.length; i++) {
        var comp = app.project.itemByID(ids[i]);
        if (!comp || comp.name.indexOf("HS_") !== 0) throw Error("Missing fixture");
        var layer = comp.layer(1);
        var effects = layer.property("ADBE Effect Parade");
        var label = "HS temporary original alpha sample";
        if (effects.property(label)) throw Error("Earlier sample is still present");
        var added = effects.addProperty("ADBE Color Control");
        added.name = label;
        try {
            var color = added.property(1);
            color.expression = "var a=thisLayer.sampleImage([210,490],[0.5,0.5],false,time)[3];" +
                "var b=thisLayer.sampleImage([400,300],[0.5,0.5],false,time)[3];" +
                "var c=thisLayer.sampleImage([50,50],[0.5,0.5],false,time)[3];[a,b,c,1]";
            var value = color.valueAtTime(0, false);
            samples.push({comp: comp.id, name: comp.name, adjustment: layer.adjustmentLayer,
                compTime: comp.time, seconds: 0, alpha: [value[0], value[1], value[2]],
                expressionError: color.expressionError});
        } finally {
            layer.property("ADBE Effect Parade").property(label).remove();
        }
    }
    app.project.save();
    return JSON.stringify({samples: samples, saved: !app.project.dirty});
})();
