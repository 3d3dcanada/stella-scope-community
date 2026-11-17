// What moves on the demo machines the rig ships as static or incomplete, measured off their own model files (fix-0.6.2 WO-08, 2026-10-30; the owner: every demo machine moves).
//   node tools/printer-motion-measured.mjs MODELS_DIR > web/printer-motion-measured.json
// MODELS_DIR is a folder of <id>.glb.gz (the installer's models folder: /usr/lib/Stella Scope/models). src/printer_models.rs merges the result field by field over the other tables.
// Nothing here is eyeballed: every number is a part's box or a nozzle's lowest vertices, and the way it was found is written beside it. A model whose hash is not the catalogue's pin is refused.
import fs from 'node:fs';import zlib from 'node:zlib';import crypto from 'node:crypto';
import { parseGlb, readParts } from '../web/model-glb.js';
const dir = process.argv[2];
const root = new URL('..', import.meta.url).pathname;
const catalogue = JSON.parse(fs.readFileSync(root + 'web/printer-models.json')).machines;
const load = (id) => {
 const buf = zlib.gunzipSync(fs.readFileSync(`${dir}/${id}.glb.gz`));
 const pin = catalogue.find((m) => m.id === id).pin.model_sha256;
 if (crypto.createHash('sha256').update(buf).digest('hex') !== pin) throw Error(`${id} is not the model the catalogue pins`);
 const parts = readParts(parseGlb(buf.buffer.slice(buf.byteOffset, buf.byteOffset + buf.length)));
 for (const p of parts) { const t = p.t || [0, 0, 0]; p.lo = (p.wmin || p.min).map((v, k) => (v + t[k]) * 1000); p.hi = (p.wmax || p.max).map((v, k) => (v + t[k]) * 1000); }
 return { parts, pin };
};
const r3 = (v) => Math.round(v * 1000) / 1000;
const r3s = (a) => a.map(r3);
/** The tip of a nozzle: the centre of the vertices within a quarter millimetre of the part's lowest point (display frame, millimetres). */
const tipOf = (p) => {
 const P = p.pos, t = p.t || [0, 0, 0]; let lo = Infinity;
 for (let i = 1; i < P.length; i += 3) lo = Math.min(lo, P[i]);
 let n = 0, x = 0, z = 0;
 for (let i = 0; i < P.length; i += 3) if (P[i + 1] <= lo + 0.00025) { n++; x += P[i]; z += P[i + 2]; }
 return [(x / n + t[0]) * 1000, (lo + t[1]) * 1000, (z / n + t[2]) * 1000];
};
const byName = (parts, re) => { const hit = parts.filter((p) => re.test(p.name)); if (hit.length !== 1) throw Error(`${re}: ${hit.length} parts`); return hit[0]; };
const notice = 'Measured off the model file by tools/printer-motion-measured.mjs (2026-10-30). Firmware, belt tension and collisions are not simulated.';

// ---- SIBOOR S-BOOM (CoreXZ, the bed slides in Y): its part records already say head, beam, bed, belt and chain; only the registration was missing. Display = (CAD x, CAD z, -CAD y).
function sboom() {
 const id = 'siboor_sboom_220', { parts, pin } = load(id), size = 220;
 const sheet = parts.find((p) => p.key === '00368'), nozzle = tipOf(parts.find((p) => p.key === '00481'));
 const cx = (sheet.lo[0] + sheet.hi[0]) / 2, cz = (sheet.lo[2] + sheet.hi[2]) / 2, top = sheet.hi[1];
 const origin = [cx - size / 2, -(cz + size / 2), top], n = [nozzle[0], -nozzle[2], nozzle[1]];   // CAD frame: x, -display z, display y
 const off = [origin[0] - n[0], -(n[1] - origin[1]), origin[2] - n[2]];   // head x = axis x; bed CAD y = -axis y, and the plate's point y has to come under the nozzle; beam z = axis z
 const keys = (g) => parts.filter((p) => p.group === g).map((p) => p.key);   // ("reference" is the one object standing on the sheet: it rides with the bed)
 return {
  id,
  profile: { model_sha256: pin, axes: { x: r3s([off[0], off[0] + size]), y: r3s([off[1], off[1] + size]), z: r3s([off[2], off[2] + size]) },
   groups: { head: keys('head'), beam: keys('beam'), bed: [...keys('bed'), ...keys('reference')] }, motions: { head: ['x', 0, 'z'], beam: [0, 0, 'z'], bed: [0, { y: -1 }, 0], fixed: [0, 0, 0] },
   notices: ['Axis values are displacements from the model\'s pose: the bed slides in Y, the beam climbs in Z, the head runs along the beam.', notice],
   scope: 'Movement preview from the live position, from the groups in the model\'s own part records.',
   calibration: { offset_mm: r3s(off), sign: [1, 1, 1], source: [0, 1, 2] },
   calibration_basis: `Measured from the model: the PEI surface (part 00368) is centred at X ${r3(cx)}, Z ${r3(cz)}, top ${r3(top)}; the nozzle tip (part 00481, its lowest vertices) is at X ${r3(nozzle[0])}, height ${r3(nozzle[1])}, Z ${r3(nozzle[2])}. X0 and Y0 are the corner of the ${size} mm print area centred on that surface, Y0 its front edge; Z0 the surface touching the nozzle. The Z travel is the catalogue's ${size} mm.`,
   flex: { belts: ['00409', '00578', '00579'], tubes: [{ part: '00365', follows: 'bed' }] } },
  frame: { model_sha256: pin, origin_mm: r3s(origin), axes: [[1, 0, 0], [0, 1, 0], [0, 0, 1]], group: 'bed', plate_mm: [size, size], nozzle_mm: r3s(n),
   basis: `Measured from the model (mm, display frame, Y up): the PEI surface is centred at X ${r3(cx)}, Z ${r3(cz)}, top ${r3(top)}; the ${size} mm print area is centred on it and Y0 is its front edge (+Z). The nozzle tip is ${r3(nozzle[1] - top)} mm over the plate in the model's pose.` },
 };
}

// ---- DoomCube 2 350 (a flying gantry on four Z rails, like the V2.4): its file has no part records at all, so the moving assemblies are told apart by where each body stands. Display = (-CAD x, CAD z, CAD y).
// Its front is -Z in the display (the door, and the head's face, look that way; the rear panel with the mains inlet is +Z), so printer X is CAD +x and printer Y is CAD +y.
function doomcube() {
 const id = 'doomcube2_350_reference', { parts, pin } = load(id), size = 350;
 const plate = byName(parts, /^part_1176__body$/), nozzle = tipOf(byName(parts, /^part_592__source_0_1_1_547_24$/));
 const cx = (plate.lo[0] + plate.hi[0]) / 2, cz = (plate.lo[2] + plate.hi[2]) / 2, top = plate.hi[1];
 // CAD frame. Printer 0,0 is the plate's front left corner seen from the front: the display's +X side (X runs toward -X in the display) and its -Z side.
 const origin = [-(cx + size / 2), cz - size / 2, top], n = [-nozzle[0], nozzle[2], nozzle[1]];
 const off = [origin[0] - n[0], origin[1] - n[1], origin[2] - n[2]];
 // The gantry is the band of bodies between its Z carriages and the top of the extruder, inside the frame (the band is otherwise empty: 1426 of the 2435 bodies stand in it, 12 to 22 per 25 mm above and below it).
 const inBox = (p, x0, x1, y0, y1, z0, z1) => p.lo[0] >= x0 && p.hi[0] <= x1 && p.lo[1] >= y0 && p.hi[1] <= y1 && p.lo[2] >= z0 && p.hi[2] <= z1;
 // (the four Z rails, parts 324 to 327, run through the band with their bolts and stay on the frame)
 const gantry = parts.filter((p) => inBox(p, -256, 256, 280, 440, -254, 254) && !/^part_(324|325|326|327)__/.test(p.name));
 // The X beam's own cross-section: the extrusion, its two rails and the bolts in them; everything else over the middle of the beam rides with the head.
 const inBeam = (p) => p.lo[1] >= 335 && p.hi[1] <= 362 && p.lo[2] >= 109 && p.hi[2] <= 138 && p.hi[0] - p.lo[0] < 40;
 const head = gantry.filter((p) => inBox(p, -45, 45, 294, 440, 30, 140) && !inBeam(p));
 const onHead = new Set(head);
 const beam = gantry.filter((p) => !onHead.has(p) && inBox(p, -256, 256, 300, 425, 84, 140));
 const onBeam = new Set(beam);
 const z = gantry.filter((p) => !onHead.has(p) && !onBeam.has(p));
 return {
  id,
  profile: { model_sha256: pin, axes: { x: r3s([off[0], off[0] + size]), y: r3s([off[1], off[1] + size]), z: r3s([off[2], off[2] + size]) },
   groups: { head: head.map((p) => p.name), x_beam: beam.map((p) => p.name), z_gantry: z.map((p) => p.name) }, motions: { head: ['x', 'y', 'z'], x_beam: [0, 'y', 'z'], z_gantry: [0, 0, 'z'], fixed: [0, 0, 0] },
   notices: ['Axis values are displacements from the model\'s pose: the gantry climbs in Z, the X beam runs in Y on it, the head in X on the beam.', 'This CAD has no belts and no drag chains, so none are drawn.', notice],
   scope: 'Movement preview from the live position. The file has no part records: the assemblies are told apart by where each body stands.',
   see_through: { match: '^part_(283|311|317|320|323)__source' },
   calibration: { offset_mm: r3s(off), sign: [1, 1, 1], source: [0, 1, 2] },
   calibration_basis: `Measured from the model: the plate (part 1176) is centred at X ${r3(cx)}, Z ${r3(cz)}, top ${r3(top)}; the nozzle tip (body part_592..._24, its lowest vertices) is at X ${r3(nozzle[0])}, height ${r3(nozzle[1])}, Z ${r3(nozzle[2])}, ${r3(nozzle[1] - top)} mm over the plate. The print area is the ${size} mm centred on the plate, with its front on the door's side (-Z). The Z travel is ${size} mm: the Z rails are 400 mm with 30 mm carriages.` },
  frame: { model_sha256: pin, origin_mm: r3s(origin), axes: [[1, 0, 0], [0, 1, 0], [0, 0, 1]], group: null, plate_mm: [size, size], nozzle_mm: r3s(n),
   basis: `Measured from the model (mm, display frame, Y up): the plate is centred at X ${r3(cx)}, Z ${r3(cz)}, top ${r3(top)}; the ${size} mm print area is centred on it. The machine's front is -Z (its door and the head's face), so Y0 is the -Z edge and printer X runs toward -X in the display, as the CAD's own axes do.` },
 };
}

// ---- Voron Trident 300: the Stealthburner is drawn now (models/trident-r2-stealthburner.glb.gz, the toolhead of voron_trident_500_custom, whose carriage frames stand exactly where the 300's R2 carriage does), so its nozzle says where Z0 is.
function trident() {
 const id = 'voron_trident_300', { parts } = load('voron_trident_500_custom');
 const frames = parts.filter((p) => p.group === 'Toolhead_Carriage' && p.lo[1] < 320), head = parts.filter((p) => p.group === 'Toolhead_SB_Revo');
 const b300 = zlib.gunzipSync(fs.readFileSync(`${dir}/parts/trident_r2_sb_carriage.glb.gz`)), t300 = readParts(parseGlb(b300.buffer.slice(b300.byteOffset, b300.byteOffset + b300.length)));
 const low300 = Math.min(...t300.map((p) => (p.wmin || p.min)[1] * 1000)), low500 = Math.min(...frames.map((p) => p.lo[1]));
 if (Math.abs(low300 - low500) > 0.01) throw Error(`the carriage frames are not at one height: ${low300} and ${low500}`);
 const nozzle = tipOf(head.reduce((a, p) => (p.lo[1] < a.lo[1] ? p : a)));   // display frame
 const n = [nozzle[0], -nozzle[2], nozzle[1]], plateTop = 303.25, cal = JSON.parse(fs.readFileSync(root + 'web/printer-motion-parts.json')).profiles[id].calibration.offset_mm;
 // The maker's calibration stands (it is what a person's saved corner and the page's "use the bed corner" go by). It was written for no head at all, Z 0 with the plate at the pose: so the bed starts at the nozzle instead (a rest offset, as the Satsuma's does), and the print's corner is where that nozzle reaches at printer 0,0.
 const origin = [n[0] + cal[0], n[1] + cal[1], plateTop], rise = n[2] - plateTop;
 return { id,
  profile: { rest_offsets_mm: { bed: [0, 0, r3(rise)] },
   notices: ["Axis values are displacements from the model's pose: the bed drops as Z rises.", "With the options 'Trident R2 gantry and X carriage' and 'Stealthburner on the Trident R2 carriage' chosen, the X beam moves with Y and the head with X and Y, as the parts of those files say.", `The bed stands ${r3(rise)} mm over its CAD pose at Z 0: the Stealthburner on the R2 carriage (the toolhead of voron_trident_500_custom, whose carriage frames stand at the same height, ${r3(low300)}, as the 300's module) has its nozzle tip at X ${r3(nozzle[0])}, height ${r3(nozzle[1])}, Z ${r3(nozzle[2])}, that far over the plate in the model's pose.`, notice],
   flex: { belts: ['trident_r2_gantry_300_535', 'trident_r2_gantry_300_536'], tubes: [{ part: 'voron_trident_300_base_1409' }] } },
  frame: { nozzle_mm: r3s(n), origin_mm: r3s(origin) } };
}

const out = { about: 'Measured off the model files by tools/printer-motion-measured.mjs (fix-0.6.2 WO-08, 2026-10-30). Merged field by field over the other motion and print-frame tables. `flex` names the parts that are moved vertex by vertex: belts and tubes (web/model-follow.js).', profiles: {}, frames: {} };
for (const f of [sboom(), doomcube(), trident()]) { out.profiles[f.id] = f.profile; out.frames[f.id] = f.frame; }
// The belts and tubes of the machines whose motion the rig registers: the keys are the models' own (or, for the Ender-3's X belt, which has none, its names).
const flex = {
 voron_v24_350_printed: { belts: ['v24_00377', 'v24_00378'] }, voron_v24_300_printed: { belts: ['v24_00377', 'v24_00378'] },
 voron_v02_120: { belts: ['v02_00153', 'v02_00154'] },
 micron_plus_r1_180: { belts: ['m180_01615', 'm180_01616'], tubes: [{ part: 'm180_00411' }, { part: 'm180_01768' }] },
 sovol_sv08_350: { belts: ['446', '447'], tubes: [{ part: '578' }, { part: '625' }] },
 satsuma180_v10: { belts: ['375', '376'] },
 ender3_stock_220: { belts: ['original-x-belt-fixed-return-pulleys', 'original-x-belt-left-cut-tail', 'original-x-belt-left-straight-extension', 'original-x-belt-left-native-bend', 'original-x-belt-left-front-span', 'original-x-belt-right-cut-tail', 'original-x-belt-right-straight-extension', 'original-x-belt-right-native-bend', 'original-x-belt-right-front-span'] },
};
for (const [id, f] of Object.entries(flex)) out.profiles[id] = { flex: f };
process.stdout.write(JSON.stringify(out, null, 1) + '\n');
