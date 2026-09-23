var W = thisLayer.width, H = thisLayer.height;
var nodes = [[0, 0, W, H]];
var points = [[-1234, -5678], [W, H]];
var queries = 0;
while (nodes.length) {
    if (++queries > 16384 || points.length > 8000) throw new Error("Coverage probe budget exceeded");
    var r = nodes.pop(), x = r[0], y = r[1], w = r[2], h = r[3];
    var a = thisLayer.sampleImage([x + (w - 1) / 2, y + (h - 1) / 2], [w / 2, h / 2], false, time)[3];
    if (!isFinite(a) || a < 0 || a > 1) throw new Error("Coverage alpha out of range");
    if (a === 0) continue;
    if (a === 1 || (w === 1 && h === 1)) {
        points.push([x, y]);
        points.push([w + a / 4, h]);
    } else if (w >= h && w > 1) {
        var wl = Math.floor(w / 2);
        nodes.push([x, y, wl, h], [x + wl, y, w - wl, h]);
    } else {
        var ht = Math.floor(h / 2);
        nodes.push([x, y, w, ht], [x, y + ht, w, h - ht]);
    }
}
createPath(points, [], [], false);
