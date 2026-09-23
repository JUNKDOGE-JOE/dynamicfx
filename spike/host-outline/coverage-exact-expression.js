var W = thisLayer.width, H = thisLayer.height;
if (W < 1 || H < 1 || W !== Math.floor(W) || H !== Math.floor(H) ||
    W > 32767 || H > 32767 || W * H > 524288) {
    throw new Error("Exact coverage diagnostic pixel budget exceeded");
}
// A full-frame request lets subsequent pixel reads reuse the same source raster.
var warmAlpha = thisLayer.sampleImage([(W - 1) / 2, (H - 1) / 2], [W / 2, H / 2], false, time)[3];
if (!isFinite(warmAlpha) || warmAlpha < 0 || warmAlpha > 1) {
    throw new Error("Coverage source alpha out of range");
}
var bytes = new ArrayBuffer(4), f32 = new Float32Array(bytes), u32 = new Uint32Array(bytes);
var rectangles = [], active = Object.create(null);
function sample(x, y) {
    var alpha = thisLayer.sampleImage([x, y], [0.5, 0.5], false, time)[3];
    if (!isFinite(alpha) || alpha < 0 || alpha > 1) throw new Error("Coverage alpha out of range");
    f32[0] = alpha === 0 ? 0 : alpha;
    return u32[0];
}
function emit(next, x, y, width, bits) {
    if (bits === 0) return;
    var key = x + ":" + width + ":" + bits;
    var rectangle = active[key];
    if (rectangle) {
        rectangle.height++;
    } else {
        if (rectangles.length >= 2666) throw new Error("Exact coverage diagnostic data budget exceeded");
        rectangle = {x: x, y: y, width: width, height: 1, bits: bits};
        rectangles.push(rectangle);
    }
    next[key] = rectangle;
}
for (var y = 0; y < H; y++) {
    var next = Object.create(null), start = 0, previous = sample(0, y);
    for (var x = 1; x < W; x++) {
        var current = sample(x, y);
        if (current !== previous) {
            emit(next, start, y, x - start, previous);
            start = x;
            previous = current;
        }
    }
    emit(next, start, y, W - start, previous);
    active = next;
}
var points = [[-1234, -5680], [W, H]];
for (var i = 0; i < rectangles.length; i++) {
    var r = rectangles[i];
    // Decode these symbols by rounding after the host's path-coordinate quantization.
    points.push([r.x, r.y], [r.width, r.height], [r.bits >>> 16, (r.bits & 65535) / 2]);
}
createPath(points, [], [], false);
