const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const vm = require('node:vm');

const expression = fs.readFileSync(path.join(__dirname, 'coverage-exact-expression.js'), 'utf8');

function evaluate(width, height, pixels) {
  let calls = 0;
  const points = vm.runInNewContext(expression, {
    thisLayer: {
      width, height,
      sampleImage([x, y], radius, postEffect, time) {
        assert.equal(postEffect, false);
        assert.equal(time, 0.25);
        calls++;
        if (calls === 1) {
          assert.deepEqual([x, y], [(width - 1) / 2, (height - 1) / 2]);
          assert.deepEqual([...radius], [width / 2, height / 2]);
          return [0, 0, 0, 0];
        }
        assert.deepEqual([...radius], [0.5, 0.5]);
        return [0, 0, 0, pixels[y * width + x]];
      },
    },
    time: 0.25,
    createPath: p => p.map(v => v.map(c => Math.round(c * 65536) / 65536 + 1 / 512)),
  });
  return { points, calls };
}

function decode(points) {
  assert.deepEqual(Array.from(points[0], Math.round), [-1234, -5680]);
  const [width, height] = points[1].map(Math.round);
  const pixels = new Float32Array(width * height);
  const seen = new Set();
  const bits = new Uint32Array(1);
  const value = new Float32Array(bits.buffer);
  for (let i = 2; i < points.length; i += 3) {
    const [x, y] = points[i].map(Math.round), [w, h] = points[i + 1].map(Math.round);
    const [hi, lowHalf] = points[i + 2];
    bits[0] = Math.round(hi) * 65536 + Math.round(lowHalf * 2);
    for (let row = y; row < y + h; row++) {
      for (let col = x; col < x + w; col++) {
        const at = row * width + col;
        assert.ok(!seen.has(at));
        seen.add(at);
        pixels[at] = value[0];
      }
    }
  }
  return pixels;
}

const cases = [
  { name: 'isolated low-opacity edge', width: 8, height: 8,
    pixels: Array.from({ length: 64 }, (_, i) => i === 23 ? 16 / 255 : 0) },
  { name: 'partial interior and disconnected edge', width: 4, height: 3,
    pixels: [0, 0.5, 0.5, 0, 0, 0.5, 0.5, 0, 1 / 255, 0, 0, 1] },
  { name: 'float32 precision through fixed-point coordinates', width: 6, height: 1,
    pixels: [1e-10, 1e-7, 1 / 32768, 0.123456789, 1 - 2 ** -24, 1] },
];
for (const test of cases) {
  const { points, calls } = evaluate(test.width, test.height, test.pixels);
  assert.equal(calls, test.width * test.height + 1);
  assert.deepEqual(decode(points), Float32Array.from(test.pixels));
  console.log(`PASS ${test.name}`);
}
assert.equal(evaluate(8, 8, Array(64).fill(0.5)).points.length, 5);
assert.equal(evaluate(8, 8, Array(64).fill(0)).points.length, 2);
assert.throws(() => evaluate(1, 1, [NaN]), /alpha out of range/);
assert.throws(() => evaluate(1, 1, [1.1]), /alpha out of range/);
assert.throws(() => evaluate(1920, 1080, []), /pixel budget/);
console.log('PASS exact merging, empty alpha, invalid samples and bounded work');
