// node scripts/next-knob-tests.mjs: where the held knob stands (0.6.2 the owner 2026-10-30: it "must not cover half of its own gauge": it lifts clear of the gauge or replaces it in place).
import assert from 'node:assert/strict';
import { heldPlace } from '../web/next-control.js';
let checks = 0;
const is = (a, b, label) => { checks++; assert.deepEqual(a, b, label); };
const clear = (p, g, S) => p.cx + S / 2 <= g.gx - g.w / 2 || p.cx - S / 2 >= g.gx + g.w / 2 || p.cy + S / 2 <= g.gy - g.h / 2 || p.cy - S / 2 >= g.gy + g.h / 2;
const inside = (p, S, W, H) => p.cx - S / 2 >= 0 && p.cx + S / 2 <= W && p.cy - S / 2 >= 0 && p.cy + S / 2 <= H;
// A mouse: on the gauge's own centre, and it says it lies on it (the gauge is then hidden whole).
const g = { gx: 214, gy: 419, w: 156, h: 156 };
is(heldPlace({ ...g, S: 200, touch: false, W: 1920, H: 1080 }), { cx: 214, cy: 419, over: true }, 'in place at 1920x1080');
// A gauge by the window's edge: the window pushes the knob off centre, and it still says it lies on the gauge, so no half of it shows.
is(heldPlace({ gx: 1296, gy: 160, w: 92, h: 92, S: 200, touch: false, W: 1366, H: 768 }), { cx: 1254, cy: 160, over: true }, 'pushed in by the edge');
// A page at 4/3 (the film's captures): the sizes come in window pixels, so the same rule holds.
is(heldPlace({ gx: 285, gy: 559, w: 208, h: 208, S: 267, touch: false, W: 2560, H: 1440 }).over, true, 'zoomed');
// A finger: above the gauge, clear of it, inside the window, at both phone sizes, for each gauge in the row and each fan in the stack.
for (const [W, H, gauges] of [[390, 844, [[55, 604, 92], [148, 604, 92], [242, 604, 92], [335, 604, 92], [347, 362, 70], [347, 439, 70], [347, 515, 70]]], [360, 780, [[51, 544, 84], [137, 544, 84], [223, 544, 84], [309, 544, 84], [319, 320, 65], [319, 391, 65], [319, 461, 65]]]]) {
 const S = Math.round(Math.min(W * 0.72, 250));
 for (const [gx, gy, w] of gauges) {
  const b = { gx, gy, w, h: w }, p = heldPlace({ ...b, S, touch: true, W, H });
  is(p.over, false, `${W}: the knob is clear of the gauge at ${gx},${gy}`); is(clear(p, b, S), true, 'and the rule says so'); is(inside(p, S, W, H), true, 'inside the window');
 }
}
// A gauge with no room above it (a phone on its side): the knob goes beside it, still clear.
{ const b = { gx: 120, gy: 300, w: 86, h: 86 }, S = 250, p = heldPlace({ ...b, S, touch: true, W: 844, H: 390 }); is(p.over, false, 'beside it'); is(p.cx > b.gx, true, 'on the side with room'); is(inside(p, S, 844, 390), true, 'inside the window'); }
console.log(`next-knob: ${checks} assertions passed.`);
