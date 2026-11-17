// The first-run walkthrough (web/next-tour.js; the owner 2026-11-14: "maximum 10 frames ... at the very end, have fun and please hit the feedback
// button ... and happy printing"). Fails if it grows past ten steps, loses the last step's feedback line, a page's walk does not end on it,
// or a card lands outside the window or over the control it describes at the window sizes the board names.
//   node scripts/tour-tests.mjs
import assert from 'node:assert/strict';
globalThis.document ??= {addEventListener(){},documentElement:{dataset:{}},getElementById:()=>null};
globalThis.window ??= globalThis;globalThis.matchMedia ??= ()=>({matches:false});
const {STEPS,MAX_STEPS,MIN_CARD,stepsFor,placeCard}=await import('../web/next-tour.js');
let n=0;const ok=(v,m)=>{n++;assert.ok(v,m);},is=(a,b,m)=>{n++;assert.deepEqual(a,b,m);};

is(MAX_STEPS,10,'the owner: at most ten');
ok(STEPS.length<=10,`the walkthrough has ${STEPS.length} steps; the owner asked for at most 10`);
const last=STEPS.at(-1);
is(last.page,'both','the last word is said on every page');
ok(/have fun/i.test(last.title+' '+last.text),'the last step says have fun');
ok(/Feedback/.test(last.text)&&/More, Feedback/.test(last.text)&&/All pages, Feedback/.test(last.text),'the last step says where Feedback is on both pages');
ok(/happy printing/i.test(last.text),'the last step ends on happy printing');
for(const page of ['farm','printer']){
 const s=stepsFor(page);
 ok(s.length>=2&&s.length<=10,`${page}: ${s.length} steps`);
 is(s.at(-1),last,`${page}: ends on the last word`);
 ok(s.slice(0,-1).every(x=>x.page===page),`${page}: only its own steps before it`);}
for(const s of STEPS){
 ok(s.title&&s.title.length<=40,`a short title: ${s.title}`);
 ok(s.text.length>=40&&s.text.length<=280,`one short card (${s.text.length} characters): ${s.title}`);
 ok(Array.isArray(s.at)&&s.at.length>0&&s.at.every(a=>typeof a==='string'&&a),`points at a control: ${s.title}`);}
// what the owner listed is all there
const all=STEPS.map(s=>s.title+' '+s.text).join(' ');
for(const word of ['farm wall','Add a printer','five printers are free','demo farm','Get the 3D model','Temperatures are on the left','fans are on the right','press and hold','Calibrate','Print history','Maintenance','Pair phone','Spools'])
 ok(all.includes(word),`the walkthrough mentions: ${word}`);

// the card: inside the window, and never over the control (PC first, then phone)
const sizes=[[1920,1080],[1366,768],[1280,720],[1024,768],[390,844],[360,740]];
const hit=(p,cw,ch,h)=>p.left<h.r&&p.left+cw>h.l&&p.top<h.b&&p.top+ch>h.t;
for(const [W,H] of sizes){
 const cw=Math.min(380,W-24),ch=W<500?260:210,m=12;
 const cases={
  'a key in the top right corner':[{l:W-70,t:8,r:W-10,b:60}],
  'the page rail on the left edge':[{l:8,t:90,r:64,b:H-90}],
  'a tile in the middle':[{l:W/2-90,t:H/2-70,r:W/2+90,b:H/2+70}],
  // the dials as placeGauges puts them: one column at each edge (beside the rail on a computer), a fifth of the width at most
  'the two columns of dials':(d=>[{l:(W<500?2:70),t:90,r:(W<500?2:70)+d+12,b:H-80},{l:W-18-d,t:90,r:W-6,b:H-80}])(Math.min(120,(W-20)/5)),
  'a key in the bottom bar':[{l:W/2-40,t:H-70,r:W/2+40,b:H-14}],
  'no control on screen':[]};
 for(const [name,holes] of Object.entries(cases)){
  let w=cw,h=ch,p=placeCard({W,H,cw:w,ch:h,holes});
  // as the page does: no free place as wide as the card, so it takes the widest free column (and grows taller) and is placed again
  if(p.over>0&&p.room>=MIN_CARD&&p.room<w){h=Math.ceil(ch*w/p.room)+60;w=p.room;p=placeCard({W,H,cw:w,ch:h,holes});}
  ok(w>=MIN_CARD,`${W}x${H}, ${name}: the card is at least ${MIN_CARD} wide`);
  ok(p.left>=m-1&&p.top>=m-1&&p.left+w<=W-m+1&&p.top+h<=H-m+1,`${W}x${H}, ${name}: the card is inside the window (${JSON.stringify(p)}, ${w}x${h})`);
  ok(p.over===0&&!holes.some(o=>hit(p,w,h,o)),`${W}x${H}, ${name}: the card is not over the control (${JSON.stringify(p)}, ${w}x${h})`);}}
// the wall itself fills the window: the card covers as little of it as it can, and stays inside
{const p=placeCard({W:1366,H:768,cw:380,ch:210,holes:[{l:10,t:120,r:1356,b:768}]});ok(p.top>=11&&p.top+210<=757&&p.left>=11,'the whole wall: still inside the window');}
// a printer page scopes /api/ to its printer (web/printer-scope.js); /api/tour belongs to the installation, so it stays unscoped,
// and the hidden radial host must not hold the walk back (both found on the 0.6.3 build: the printer walk never showed)
{const {readFileSync}=await import('node:fs');
 const s=readFileSync(new URL('../web/printer-scope.js',import.meta.url),'utf8'),t=readFileSync(new URL('../web/next-tour.js',import.meta.url),'utf8');
 ok(/\|tour\|/.test(s),'/api/tour is a farm-wide route on a printer page');
 ok(t.includes('.nx-radial:not([hidden])'),'only an open radial menu holds the walkthrough back');}
console.log(`tour: ${n} checks pass (${STEPS.length} steps)`);
