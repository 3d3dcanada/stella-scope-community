// node scripts/wo02-tests.mjs: app-final WO-02 (2026-11-01). The held knob stands above the layer its gauge is in (an opened farm tile is z 60),
// and a list opened on a zoomed page (the film's 2560 recorder zooms 4/3) opens inside the window.
import assert from 'node:assert/strict';
import { topLayer } from '../web/next-control.js';
import { popPlace } from '../web/next-ui.js';
let n = 0;const is = (a, b, l) => { n++; assert.deepEqual(a, b, l); };
is(topLayer(['auto', '60', 'auto', '2']), 60, 'the opened farm tile is the top layer');
is(Math.max(41, topLayer(['60']) + 2), 62, 'the knob stands above the opened tile');
is(Math.max(41, topLayer(['auto', 'auto']) + 2), 41, 'on the printer page it keeps its own 41');
for (const z of [1, 4 / 3]) {
 const W = 2560, H = 1440, p = { width: 463 * z, height: 143 * z };
 for (const r of [{ left: 2470, top: 20, width: 44, height: 44, bottom: 64 }, { left: 700, top: 1320, width: 460, height: 64, bottom: 1384 }]) {
  const at = popPlace({ r, p, W, H, z }), right = at.left * z + p.width, bottom = at.top * z + p.height;
  n++; assert.ok(at.left * z >= 0 && right <= W && at.top * z >= 0 && bottom <= H, `inside the window at zoom ${z}: ${JSON.stringify(at)}`);
 }
}
console.log(`WO-02: ${n} checks passed.`);
