(function () {
    if (!app.project.file || app.project.file.name !== "host-shape-native.aep") throw Error("Wrong project");
    var ids = [132, 146], records = [];
    for (var n = 0; n < ids.length; n++) {
        var comp = app.project.itemByID(ids[n]);
        if (!comp || comp.name.indexOf("HS_expression_coverage_") !== 0) throw Error("Wrong fixture");
        var paths = [];
        function walk(group) {
            for (var i = 1; i <= group.numProperties; i++) {
                var p = group.property(i);
                if (p.matchName === "ADBE Vector Shape") paths.push(p);
                else if (p.propertyType !== PropertyType.PROPERTY) walk(p);
            }
        }
        walk(comp.layer(1).property("ADBE Root Vectors Group"));
        if (paths.length !== 1 || paths[0].numKeys !== 2) throw Error("Unexpected animated shape");
        var p = paths[0], shape = p.keyValue(1), vertices = shape.vertices;
        var before = []; for (var j = 0; j < vertices.length; j++) before.push([vertices[j][0], vertices[j][1]]);
        records.push({comp: comp.id, before: before, time: p.keyTime(1)});
        vertices[0] = [vertices[0][0] + 60, vertices[0][1] + 30];
        shape.vertices = vertices;
        p.setValueAtKey(1, shape);
    }
    return JSON.stringify(records);
})();
