function alphaBound(contents) {
    var total = 0;
    function child(group, matchName) {
        try { return group(matchName); } catch (e) { return null; }
    }
    for (var i = 1; i <= contents.numProperties; i++) {
        var item = contents(i);
        var nested = child(item, "ADBE Vectors Group");
        if (nested !== null) {
            total += alphaBound(nested);
            continue;
        }
        var opacity = child(item, "ADBE Vector Fill Opacity");
        if (opacity === null) opacity = child(item, "ADBE Vector Stroke Opacity");
        if (opacity !== null) {
            var value = opacity.value / 100;
            if (!isFinite(value) || value < 0 || value > 1) return 1;
            total += value;
        } else if (child(item, "ADBE Vector Shape") === null &&
                   child(item, "ADBE Vector Rect Size") === null &&
                   child(item, "ADBE Vector Ellipse Size") === null &&
                   child(item, "ADBE Vector Star Type") === null) {
            return 1;
        }
    }
    return Math.min(1, total);
}
var maximumAlpha = 1;
try { maximumAlpha = alphaBound(thisLayer.content); } catch (e) { maximumAlpha = 1; }
