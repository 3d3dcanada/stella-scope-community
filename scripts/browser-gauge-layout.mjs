// release-2026-11-14 WO-1 (the owner 2026-11-14): the dials at every window size. Temperatures one column on the left, speed, flow and the fans one column on the
// right, every dial one size, nothing over the rail, the header, the job card, the bar, another dial, the machine's stage or the window's edge.
// Runs against a loopback instance with the demo farm added (POST /api/printers/demo-farm). Fails on the first window size that breaks a rule.
// usage: HUD_URL=http://127.0.0.1:7421 [PRINTERS=demo-voron-24,demo-ender-3] [PLAYWRIGHT_MODULE=...] node scripts/browser-gauge-layout.mjs
import {createRequire} from 'node:module';
import assert from 'node:assert/strict';
const require=createRequire(import.meta.url),{chromium}=require(process.env.PLAYWRIGHT_MODULE||'playwright');
const base=process.env.HUD_URL;
if(!/^http:\/\/127\.0\.0\.1:\d+$/.test(base||''))throw Error('HUD_URL must be a loopback instance');
const printers=(process.env.PRINTERS||'demo-voron-24,demo-ender-3,demo-v0').split(',');
// PC windows first (full screens, laptops, half screens, short and tall windows), then tablets and phones
const SIZES=[[2560,1440],[1920,1080],[1920,1200],[1680,1050],[1600,900],[1536,864],[1440,900],[1366,768],[1280,1024],[1280,800],[1280,720],[1200,900],[1100,600],
 [1024,768],[1000,430],[960,1040],[960,540],[900,1000],[800,600],[768,1024],[700,900],[820,1180],[600,800],[412,915],[390,844],[360,740]];
// ONLY=412x915,390x844 runs just those sizes (for a quick look at one failure)
const ONLY=(process.env.ONLY||'').split(',').filter(Boolean);
const over=(a,b)=>a.x<b.x+b.w-1&&b.x<a.x+a.w-1&&a.y<b.y+b.h-1&&b.y<a.y+a.h-1;
const browser=await chromium.launch({executablePath:process.env.CHROME_BIN||'/usr/bin/google-chrome',headless:true,args:['--no-sandbox','--use-gl=angle','--use-angle=swiftshader','--enable-unsafe-swiftshader']});
let checked=0;
try{
 const page=await (await browser.newContext({viewport:{width:1920,height:1080}})).newPage();
 for(const id of printers){
  await page.goto(`${base}/p/${id}`,{timeout:90000});await page.waitForTimeout(8000);
  for(const [w,h] of SIZES.filter(([w,h])=>!ONLY.length||ONLY.includes(`${w}x${h}`))){
   await page.setViewportSize({width:w,height:h});
   // wait until the layout is still (the dials ease to a new size; the bar redraws its keys on the next reading): four readings 300 ms apart that agree
   let m=null,prev='',same=0;for(let i=0;i<40&&same<3;i++){await page.waitForTimeout(300);m=await page.evaluate(()=>{const r=e=>{const b=e.getBoundingClientRect();return {x:b.x,y:b.y,w:b.width,h:b.height};},v=e=>e&&e.getClientRects().length&&getComputedStyle(e).visibility!=='hidden';
    const col=id=>[...document.querySelectorAll(`#${id} .nxctl`)].filter(v).map(e=>({name:e.getAttribute('aria-label')||'',...r(e)}));
    const o={};for(const id of ['nx-top','nx-job','nx-rail','nx-dock'])if(v(document.getElementById(id)))o[id]=r(document.getElementById(id));
    const st=document.querySelector('.nx-stagecard .nx-md-stage'),tl=document.querySelector('.nx-stagecard .nx-md-tools');if(v(tl))o['view tools']=r(tl);
    return {left:col('nx-left'),right:col('nx-right'),others:o,stage:st&&v(st)?r(st):null,W:innerWidth,H:innerHeight,L:document.body.dataset.layout+' railr='+document.body.style.getPropertyValue('--railr')+' rail='+JSON.stringify(document.getElementById('nx-rail')?.getBoundingClientRect())};});const k=JSON.stringify(m);same=k===prev?same+1:0;prev=k;}
   const at=`${id} at ${w}x${h}`,all=[...m.left,...m.right];
   assert.ok(m.left.length===2,`${at}: the left column holds nozzle and bed (found ${m.left.length})`);
   assert.ok(m.right.length>=3,`${at}: the right column holds speed, flow and at least the part fan (found ${m.right.length})`);
   const size=all[0].w;
   for(const d of all){
    assert.ok(Math.abs(d.w-size)<=1.5,`${at}: ${d.name} is ${d.w.toFixed(0)} px, the others ${size.toFixed(0)}: every dial is one size`);
    assert.ok(d.x>=0&&d.y>=0&&d.x+d.w<=m.W+0.5&&d.y+d.h<=m.H+0.5,`${at}: ${d.name} runs past the window's edge`);
    for(const [k,b] of Object.entries(m.others))assert.ok(!over(d,b),`${at}: ${d.name} lies over ${k} ${JSON.stringify([d,b,m.W,m.H,m.L])}`);
    if(m.stage)assert.ok(!over(d,m.stage),`${at}: ${d.name} lies over the machine's stage ${JSON.stringify([d,m.stage])}`);}
   for(const c of [m.left,m.right]){
    for(const d of c)assert.ok(Math.abs((d.x+d.w/2)-(c[0].x+c[0].w/2))<=1.5,`${at}: ${d.name} is out of line with its column`);
    for(let i=1;i<c.length;i++)assert.ok(c[i].y>=c[i-1].y+c[i-1].h-1,`${at}: ${c[i].name} overlaps ${c[i-1].name}: a column is vertical, one under another`);}
   assert.ok(m.left[0].x+m.left[0].w<=m.right[0].x,`${at}: the temperature column is left of the right column`);
   assert.ok(m.left[0].x+m.left[0].w/2<m.W/2&&m.right[0].x+m.right[0].w/2>m.W/2,`${at}: temperatures on the left half, speed and fans on the right half`);
   checked++;}
  console.log(`PASS ${id}: ${SIZES.length} window sizes`);}
}finally{await browser.close();}
console.log(`gauge layout: ${checked} checks passed`);
