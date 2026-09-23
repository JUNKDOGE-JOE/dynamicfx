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
        if (a === 0 || a === 1 || (w === 1 && h === 1)) return {alpha: a};
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
