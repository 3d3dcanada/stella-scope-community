// A part file cut out of a machine's model (fix-0.6.2 WO-08, 2026-10-30): the parts of the groups named, byte for byte, as a GLB of their own in the same frame.
//   node tools/printer-part-extract.mjs MODEL.glb.gz 'GROUP_REGEX' OUT.glb.gz
// Nothing is reshaped: positions, normals, indices, materials and part records are the source's. Prints the pins the catalogue row needs.
// Made for models/trident-r2-stealthburner.glb.gz: the rig ships the Trident 300's gantry and X carriage as modules, and the toolhead that hangs from that carriage only
// inside its larger Tridents (voron_trident_500_custom, GPL-3.0, VoronDesign), whose carriage frames stand at the very same place (x 21.6, y 319 to 387.7, z -8.9 to 23.5 in both).
import fs from 'node:fs';import zlib from 'node:zlib';import crypto from 'node:crypto';
const [src, re, out] = process.argv.slice(2);
const raw = fs.readFileSync(src), buf = raw[0] === 0x1f ? zlib.gunzipSync(raw) : raw;
const jlen = buf.readUInt32LE(12), json = JSON.parse(buf.subarray(20, 20 + jlen).toString()), binAt = 20 + jlen + 8;
const R = new RegExp(re), o = { asset: { version: '2.0', generator: 'stella-scope tools/printer-part-extract.mjs' }, scene: 0, scenes: [{ nodes: [] }], nodes: [], meshes: [], materials: [], accessors: [], bufferViews: [], buffers: [{ byteLength: 0 }] };
const chunks = []; let at = 0; const mats = new Map();
const copy = (i) => {
 const a = json.accessors[i], v = json.bufferViews[a.bufferView], comps = { SCALAR: 1, VEC2: 2, VEC3: 3, VEC4: 4 }[a.type], size = { 5126: 4, 5125: 4, 5123: 2, 5121: 1 }[a.componentType];
 const start = binAt + (v.byteOffset || 0) + (a.byteOffset || 0), len = a.count * comps * size, pad = (4 - (len % 4)) % 4;
 chunks.push(buf.subarray(start, start + len), Buffer.alloc(pad));
 o.bufferViews.push({ buffer: 0, byteOffset: at, byteLength: len }); at += len + pad;
 const { bufferView, byteOffset, ...rest } = a; o.accessors.push({ ...rest, bufferView: o.bufferViews.length - 1 });
 return o.accessors.length - 1;
};
for (const n of json.nodes) {
 if (n.mesh === undefined) continue;
 const m = json.meshes[n.mesh], g = m.extras?.group ?? n.extras?.group ?? '';
 if (!R.test(g)) continue;
 const prims = m.primitives.map((p) => {
  if (!mats.has(p.material)) { mats.set(p.material, o.materials.length); o.materials.push(json.materials[p.material]); }
  const attributes = { POSITION: copy(p.attributes.POSITION) }; if (p.attributes.NORMAL !== undefined) attributes.NORMAL = copy(p.attributes.NORMAL);
  return { attributes, ...(p.indices !== undefined ? { indices: copy(p.indices) } : {}), material: mats.get(p.material) };
 });
 o.meshes.push({ name: m.name, primitives: prims, extras: m.extras });
 o.nodes.push({ name: n.name, mesh: o.meshes.length - 1, ...(n.translation ? { translation: n.translation } : {}), ...(n.extras ? { extras: n.extras } : {}) });
 o.scenes[0].nodes.push(o.nodes.length - 1);
}
o.buffers[0].byteLength = at;
let js = Buffer.from(JSON.stringify(o)); if (js.length % 4) js = Buffer.concat([js, Buffer.alloc(4 - (js.length % 4), 0x20)]);
const head = Buffer.alloc(12), jh = Buffer.alloc(8), bh = Buffer.alloc(8);
head.write('glTF', 0); head.writeUInt32LE(2, 4); head.writeUInt32LE(12 + 8 + js.length + 8 + at, 8); jh.writeUInt32LE(js.length, 0); jh.write('JSON', 4); bh.writeUInt32LE(at, 0); bh.write('BIN\0', 4);
const glb = Buffer.concat([head, jh, js, bh, ...chunks]), gz = zlib.gzipSync(glb, { level: 9 });
fs.writeFileSync(out, gz);
const sha = (b) => crypto.createHash('sha256').update(b).digest('hex');
console.log(JSON.stringify({ out, parts: o.nodes.length, decoded_bytes: glb.length, bytes: gz.length, model_sha256: sha(glb), stored_model_sha256: sha(gz) }));
