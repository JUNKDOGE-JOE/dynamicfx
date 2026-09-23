(function () {
    if (!app.project || app.project.numItems !== 0 || app.project.dirty || app.project.file) {
        throw Error("Expected an empty, unsaved project; refusing to replace it");
    }
    var out = new Folder("E:/Code/AePlugin_Dynamicfx/scripts/out/host-shape-native-20260910");
    if (!out.exists) throw Error("Output directory is missing");
    var shaderFile = new File("E:/Code/AePlugin_Dynamicfx/spike/host-outline/input-alpha-probe.glsl");
    if (!shaderFile.open("r")) throw Error("Probe shader is missing");
    var shader = shaderFile.read(); shaderFile.close();
    var available = false;
    var effects = app.effects;
    for (var e = 0; e < effects.length; e++) {
        if (effects[e].matchName === "DynamicFx Host Shape Probe") available = true;
    }
    if (!available) throw Error("Diagnostic effect was not registered by AE");
    app.project.bitsPerChannel = 16;
    function comp(name) { return app.project.items.addComp(name, 800, 600, 1, 2, 25); }
    function triangle() {
        var s = new Shape(); s.vertices = [[200,150],[600,150],[400,500]];
        s.inTangents = [[0,0],[0,0],[0,0]]; s.outTangents = [[0,0],[0,0],[0,0]]; s.closed = true;
        return s;
    }
    function background(c) { c.layers.addSolid([0.2,0.3,0.4], "opaque background", 800, 600, 1, 2); }
    function shape(c) {
        var l = c.layers.addShape(); l.name = "host";
        var g = l.property("ADBE Root Vectors Group").addProperty("ADBE Vector Group");
        g.name = "triangle";
        var v = g.property("ADBE Vectors Group");
        v.addProperty("ADBE Vector Shape - Group").property("ADBE Vector Shape").setValue(triangle());
        v.addProperty("ADBE Vector Graphic - Fill").property("ADBE Vector Fill Color").setValue([1,1,1,1]);
        l.property("ADBE Transform Group").property("ADBE Anchor Point").setValue([0,0]);
        l.property("ADBE Transform Group").property("ADBE Position").setValue([0,0]);
        return l;
    }
    function mask(l, animated) {
        var m = l.property("ADBE Mask Parade").addProperty("ADBE Mask Atom"); m.name = "mask control";
        var p = m.property("ADBE Mask Shape");
        if (animated) {
            var a = triangle(), b = triangle();
            b.vertices = [[150,100],[650,180],[390,520]];
            b.inTangents = [[-30,40],[-60,-20],[70,0]];
            b.outTangents = [[80,-30],[10,60],[-60,0]];
            p.setValueAtTime(0, a); p.setValueAtTime(1, b);
        } else p.setValue(triangle());
    }
    function probe(l) {
        var p = l.property("ADBE Effect Parade").addProperty("DynamicFx Host Shape Probe");
        p.property(4).setValue(1);
        return p;
    }
    var rows = [];
    var c = comp("HS_shape_only"); background(c); var l = shape(c); l.adjustmentLayer = true; var p = probe(l);
    var controls = [];
    for (var n = 1; n <= p.numProperties; n++) {
        controls.push({index:n,name:p.property(n).name,type:String(p.property(n).propertyValueType)});
    }
    rows.push({name:c.name,id:c.id});
    c = comp("HS_mask_only"); background(c); l = c.layers.addSolid([1,1,1],"host",800,600,1,2);
    mask(l, true); l.adjustmentLayer = true; probe(l); rows.push({name:c.name,id:c.id});
    c = comp("HS_shape_and_mask"); background(c); l = shape(c); mask(l, false); l.adjustmentLayer = true;
    probe(l); rows.push({name:c.name,id:c.id});
    c = comp("HS_ordinary_input"); background(c); l = shape(c); l.adjustmentLayer = true;
    var fx = l.property("ADBE Effect Parade").addProperty("DynamicFx");
    for (n = 1; n <= fx.numProperties; n++) {
        if (fx.property(n).name.indexOf("Source") === 0) fx.property(n).expression = "`" + shader + "`;0";
    }
    rows.push({name:c.name,id:c.id});
    c = comp("HS_reference_coverage"); shape(c); rows.push({name:c.name,id:c.id});
    app.project.save(new File(out.fsName + "/host-shape-native.aep"));
    return JSON.stringify({version:app.version,build:app.buildNumber,bpc:app.project.bitsPerChannel,
        project:app.project.file.fsName,comps:rows,probeControls:controls});
})()
