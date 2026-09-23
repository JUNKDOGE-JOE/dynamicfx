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
var W = thisLayer.width, H = thisLayer.height, source = thisLayer;
var queries = 0;
function block(node, x, y, w, h) {
    return node.alpha === null ? node.points : (node.alpha === 0 ? [] : [[x, y], [w + node.alpha / 4, h]]);
}
function scan(x, y, w, h) {
    if (w * h <= 64) {
        if (++queries > 32768) throw new Error("Coverage probe sampling budget exceeded");
        var a = source.sampleImage([x + (w - 1) / 2, y + (h - 1) / 2], [w / 2, h / 2], false, time)[3];
        if (!isFinite(a) || a < 0 || a > 1) throw new Error("Coverage alpha out of range");
        if (a === 0 || a === maximumAlpha || (w === 1 && h === 1)) return {alpha: a};
    }
    var left, right, points;
    if (w >= h && w > 1) {
        var wl = Math.floor(w / 2);
        left = scan(x, y, wl, h); right = scan(x + wl, y, w - wl, h);
        if (left.alpha !== null && left.alpha === right.alpha) return {alpha: left.alpha};
        points = block(left, x, y, wl, h).concat(block(right, x + wl, y, w - wl, h));
    } else {
        var ht = Math.floor(h / 2);
        left = scan(x, y, w, ht); right = scan(x, y + ht, w, h - ht);
        if (left.alpha !== null && left.alpha === right.alpha) return {alpha: left.alpha};
        points = block(left, x, y, w, ht).concat(block(right, x, y + ht, w, h - ht));
    }
    if (points.length > 7998) throw new Error("Coverage probe data budget exceeded");
    return {alpha: null, points: points};
}
var tree = scan(0, 0, W, H);
var points = [[-1234, -5678], [W, H]].concat(block(tree, 0, 0, W, H));
createPath(points, [], [], false);
