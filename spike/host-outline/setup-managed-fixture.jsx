(function () {
    var project = app.project;
    if (!project.file || project.file.name !== "host-shape-native.aep") throw Error("Wrong project");
    for (var i = 1; i <= project.numItems; i++) {
        if (project.item(i).name === "HS_auto_base") throw Error("Managed fixture already exists");
    }
    var original = project.itemByID(1);
    if (!original || original.name !== "HS_shape_only") throw Error("Missing source fixture");
    var comp = original.duplicate();
    comp.name = "HS_auto_base";
    var layer = comp.layer(1);
    layer.name = "Renamed host";
    layer.property("ADBE Root Vectors Group").property(1).name = "Renamed vector group";
    var masks = layer.property("ADBE Mask Parade");
    if (masks.numProperties !== 0) throw Error("Unexpected source masks");
    var userMask = masks.addProperty("ADBE Mask Atom");
    userMask.name = "User mask control";
    userMask.maskMode = MaskMode.NONE;
    var shape = new Shape();
    shape.vertices = [[40,40],[80,40],[80,80],[40,80]];
    shape.closed = true;
    shape.inTangents = shape.outTangents = [[0,0],[0,0],[0,0],[0,0]];
    userMask.property("ADBE Mask Shape").setValue(shape);
    var effect = layer.property("ADBE Effect Parade").property(1);
    if (effect.matchName !== "DynamicFx Host Shape Probe" || effect.property(7).name !== "Manage coverage carrier") {
        throw Error("Wrong diagnostic parameter topology");
    }
    effect.property(4).setValue(true);
    effect.property(5).setValue(0);
    effect.property(7).setValue(true);
    return JSON.stringify({comp:comp.id, name:comp.name, layer:layer.id,
        masksBeforeIdle:masks.numProperties, parameters:effect.numProperties,
        userMask:shape.vertices, optedIn:effect.property(7).value});
})();
