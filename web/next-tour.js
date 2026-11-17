// The first-run walkthrough.
// Each step rings the real control on the page and dims the rest, with one short card beside it: what it is and why it helps. The farm wall
// walks its own steps and a printer's page walks its own, each once per installation (the service remembers, GET and POST /api/tour, so a second
// window or browser does not show it again), and both end on the same last word. Skip is on every step; Escape closes; the arrow keys step.
// The emergency stop is never covered. More, Walkthrough (farm) and All pages, Walkthrough (a printer) show it again.
// scripts/tour-tests.mjs fails if this grows past ten steps, loses the feedback line, or a card lands over its control or off the window.
import {el,button} from './next-ui.js';

export const MAX_STEPS=10;
// page: the page whose control the step points at ('both' is the last word). at: the controls, tried in order; the first on screen is ringed
// (every: ring all of them that are on screen). A step whose control is not on screen right now shows its card in the middle, with no ring.
export const STEPS=[
 {page:'farm',title:'Your farm wall',at:['#fm-grid'],
  text:'Every printer you add gets a tile here: what it is doing, how far along it is, the time left and its camera. One look tells you which printer needs you.'},
 {page:'farm',title:'Add a printer',at:['[data-fm-act=add]','[data-act=add-first]','[data-fm-act=more]'],
  text:'Add a printer finds Klipper, Prusa and FlashForge printers on your network, or takes an address you type. There is no printer limit.'},
 {page:'farm',title:'Try the demo farm',at:['#fm-democard','[data-fm-act=more]'],
  text:'No printer nearby? The demo farm adds nine simulated printers running sample prints, so you can try everything first. Nothing real is touched. It is offered here until you add a printer; after that it is under More, Printers.'},
 {page:'farm',title:'A printer\'s tile, and its 3D model',at:['.fm-tile:not(.fm-ghost) .fm-mdl:not([hidden])','.fm-tile:not(.fm-ghost)'],
  text:'Tap a tile to open that printer. Stella Scope can draw your real machine in 3D: press Get the 3D model on a tile and only that printer\'s model is downloaded. Nothing downloads by itself.'},
 {page:'farm',title:'Spools and the rest',at:['[data-fm-act=more]'],
  text:'More holds the rest. Under Spools, give each spool its colour and put it on a printer, so you always know what is loaded and how much is left. The queue, backup and alert settings are here too.'},
 {page:'printer',title:'The dials',every:true,at:['#nx-left','#nx-right'],
  text:'Temperatures are on the left; speed, flow and the fans are on the right. They stay at the edges, so the machine in the middle is never covered.'},
 {page:'printer',title:'Hold a dial, then turn it',at:['#nx-left .nxctl','#nx-right .nxctl'],
  text:'Tap a dial for its usual values. To set any value, press and hold the dial, then turn it. The hold comes first so a stray touch never changes a running print.'},
 {page:'printer',title:'The other pages',at:['#nx-rail'],
  text:'This rail opens the other pages of this printer: Calibrate sets it up step by step, Print history shows how every print went, and Maintenance reminds you by hours printed. The last key lists every page.'},
 {page:'printer',title:'Your phone',at:['#nx-phonebtn','#nx-rail .nx-btn:last-of-type'],
  text:'Pair phone shows a code to scan with the Stella app. After that you can watch and control this printer from anywhere, over an encrypted link. It is also under All pages, Phone & WhatsApp.'},
 {page:'both',title:'Have fun',at:['[data-fm-act=more]','#nx-rail .nx-btn:last-of-type'],
  text:'That is the tour. Tell us anything, good or bad, with Feedback: More, Feedback on the farm wall, or All pages, Feedback on a printer. Nothing is sent until you press Send. Happy printing.'},
];
export const stepsFor=page=>STEPS.filter(s=>s.page===page||s.page==='both');

/** Where the card goes (window pixels): beside the ringed controls, inside the window, never over one of them. `holes` are {l,t,r,b}.
 *  Tried in order: below, above, right, left of all of them, a free column between or beside them, then the middle, the bottom and the top of
 *  the window. The answer carries `over` (how much of a control the card still covers, 0 when none) and `room` (the widest free column, less its
 *  margins): on a phone a card too wide for any free place is made that narrow and placed again (`layout`, below). */
export function placeCard({W,H,cw,ch,holes,gap=14,m=12}){
 const mid={left:Math.max(m,(W-cw)/2),top:Math.max(m,(H-ch)/2),over:0,room:W-2*m};
 if(!holes.length)return mid;
 const u={l:Math.min(...holes.map(h=>h.l)),t:Math.min(...holes.map(h=>h.t)),r:Math.max(...holes.map(h=>h.r)),b:Math.max(...holes.map(h=>h.b))};
 const cx=Math.max(m,Math.min(W-cw-m,(u.l+u.r)/2-cw/2)),cy=Math.max(m,Math.min(H-ch-m,(u.t+u.b)/2-ch/2));
 // the free columns: the window's width with every control's own width taken out
 const cols=[];let x=0;for(const h of [...holes].sort((a,b)=>a.l-b.l)){if(h.l>x)cols.push([x,h.l]);x=Math.max(x,h.r);}if(x<W)cols.push([x,W]);
 const room=Math.max(0,...cols.map(c=>c[1]-c[0]))-2*m;
 const tries=[{left:cx,top:u.b+gap},{left:cx,top:u.t-gap-ch},{left:u.r+gap,top:cy},{left:u.l-gap-cw,top:cy},
  ...cols.filter(c=>c[1]-c[0]>=cw+2*m-1).map(c=>({left:(c[0]+c[1])/2-cw/2,top:mid.top})),{left:mid.left,top:mid.top},{left:mid.left,top:H-ch-m},{left:mid.left,top:m}];
 const inside=p=>p.left>=m-.5&&p.top>=m-.5&&p.left+cw<=W-m+.5&&p.top+ch<=H-m+.5;
 const over=p=>holes.reduce((n,h)=>n+Math.max(0,Math.min(p.left+cw,h.r)-Math.max(p.left,h.l))*Math.max(0,Math.min(p.top+ch,h.b)-Math.max(p.top,h.t)),0);
 const fits=tries.filter(inside).map(p=>({...p,over:over(p),room}));
 // a control bigger than the room around it (the wall itself): the card covers as little of it as it can
 return fits.find(p=>p.over===0)||fits.sort((a,b)=>a.over-b.over)[0]||{...mid,over:over(mid),room};}

/** The narrowest a card is made to stand clear of a control. */
export const MIN_CARD=150;

const KEY=p=>'stella-tour-'+p;
const localSeen=p=>{try{return localStorage.getItem(KEY(p))==='1';}catch{return false;}};
function remember(page){
 try{localStorage.setItem(KEY(page),'1');}catch{}
 fetch('/api/tour',{method:'POST',headers:{'Content-Type':'application/json'},body:JSON.stringify({seen:page})}).catch(()=>{});}

const SVG='http://www.w3.org/2000/svg';
const svg=(tag,cls)=>{const e=document.createElementNS(SVG,tag);if(cls)e.setAttribute('class',cls);return e;};
const shown=e=>{if(!e||e.hidden||!e.getClientRects().length)return false;const r=e.getBoundingClientRect(),s=getComputedStyle(e);return r.width>4&&r.height>4&&s.visibility!=='hidden';};
const zoomOf=()=>parseFloat(getComputedStyle(document.documentElement).zoom)||1;
let open=null;

/** Show the walkthrough of `page` ('farm' or 'printer'). Returns its close function. Closing it by any way counts as seen. */
export function startTour(page){
 if(open)return open;
 const steps=stepsFor(page),was=document.activeElement;let i=0,targets=[],timer=0;
 const root=el('div','nx-tour'),dim=svg('svg','nx-tour-dim'),card=el('div','nx-tour-card nx-glass nx-glass--raised');
 dim.setAttribute('aria-hidden','true');dim.setAttribute('preserveAspectRatio','none');
 const count=el('div','nx-tour-n'),title=el('h2','nx-tour-title'),text=el('p','nx-tour-text'),acts=el('div','nx-tour-acts');
 title.id='nx-tour-title';text.id='nx-tour-text';
 card.setAttribute('role','dialog');card.setAttribute('aria-modal','true');card.setAttribute('aria-labelledby','nx-tour-title');card.setAttribute('aria-describedby','nx-tour-text');card.tabIndex=-1;
 const close=()=>{if(!open)return;open=null;clearInterval(timer);removeEventListener('resize',layout);document.removeEventListener('keydown',key,true);root.remove();remember(page);if(was&&was.isConnected)was.focus?.({preventScroll:true});};
 const go=n=>{i=Math.max(0,Math.min(steps.length-1,n));draw();};
 const skip=button({icon:'i-close',label:'Skip',size:'row',kind:'ghost',title:'Skip the walkthrough',does:'Closes the walkthrough. You can see it again any time.',onPress:close});skip.classList.add('nx-tour-skip');
 const back=button({icon:'i-back',label:'Back',size:'row',title:'Back',does:'The step before this one.',onPress:()=>go(i-1)});
 const next=button({icon:'i-arrow-right',label:'Next',size:'row',kind:'primary',title:'Next',does:'The next step.',onPress:()=>{if(i>=steps.length-1)close();else go(i+1);}});
  // the words of a step are read out when they change; the keyboard starts on Next, so Enter walks the steps
 const say=el('div','nx-tour-say');say.setAttribute('aria-live','polite');say.setAttribute('aria-atomic','true');say.append(count,title,text);
 acts.append(skip,back,next);card.append(say,acts);root.append(dim,card);

 // the controls of this step that are on screen now
 function find(step){
  const hit=[];for(const sel of step.at){const e=[...document.querySelectorAll(sel)].find(shown);if(e){hit.push(e);if(!step.every)break;}}
  return hit;}
 function layout(){
  const W=innerWidth,H=innerHeight,pad=6;
  const box=e=>{const r=e.getBoundingClientRect();return {l:Math.max(0,r.left-pad),t:Math.max(0,r.top-pad),r:Math.min(W,r.right+pad),b:Math.min(H,r.bottom+pad)};};
  const holes=targets.filter(shown).map(box).filter(h=>h.r-h.l>4&&h.b-h.t>4);
  // the emergency stop stays bright and pressable through every step
  const stop=document.getElementById('nx-estop'),free=shown(stop)?[box(stop)]:[];
  const rect=h=>`M${h.l} ${h.t}H${h.r}V${h.b}H${h.l}Z`;
  dim.setAttribute('viewBox',`0 0 ${W} ${H}`);dim.replaceChildren();
  const shade=svg('path','nx-tour-shade');shade.setAttribute('fill-rule','evenodd');shade.setAttribute('d',`M0 0H${W}V${H}H0Z`+[...holes,...free].map(rect).join(''));dim.append(shade);
  for(const h of holes){
   // the ringed control is shown, not pressed: the walkthrough moves only by its own keys
   const stopper=svg('rect','nx-tour-stopper'),ring=svg('rect','nx-tour-ring');
   for(const r of [stopper,ring]){r.setAttribute('x',h.l);r.setAttribute('y',h.t);r.setAttribute('width',h.r-h.l);r.setAttribute('height',h.b-h.t);r.setAttribute('rx',10);}
   dim.append(stopper,ring);}
  const z=zoomOf();card.style.width='';let c=card.getBoundingClientRect(),at=placeCard({W,H,cw:c.width,ch:c.height,holes});
  // no free place as wide as the card (a phone, the two columns of dials): the card takes the width of the widest free column and is placed again
  if(at.over>0&&at.room>=MIN_CARD&&at.room<c.width){card.style.width=at.room/z+'px';c=card.getBoundingClientRect();at=placeCard({W,H,cw:c.width,ch:c.height,holes});}
  card.style.left=at.left/z+'px';card.style.top=at.top/z+'px';}
 function draw(){
  const s=steps[i],last=i===steps.length-1;
  count.textContent=`Step ${i+1} of ${steps.length}`;title.textContent=s.title;text.textContent=s.text;
  back.disabled=i===0;const lab=next.querySelector('.nx-lbl');if(lab)lab.textContent=last?'Done':'Next';next.setAttribute('aria-label',last?'Done':'Next');
  targets=find(s);
  // a tile further down the wall is brought into view before it is ringed
  const far=targets.find(e=>{const r=e.getBoundingClientRect();return r.bottom>innerHeight||r.top<0;});far?.scrollIntoView({block:'center'});
  layout();if(!card.contains(document.activeElement)||document.activeElement.disabled)next.focus({preventScroll:true});}
 function key(e){
  if(e.key==='Escape'){e.preventDefault();e.stopPropagation();close();return;}
  if(e.key==='ArrowRight'){e.preventDefault();e.stopPropagation();if(i<steps.length-1)go(i+1);return;}
  if(e.key==='ArrowLeft'){e.preventDefault();e.stopPropagation();go(i-1);return;}
  if(e.key==='Tab'){   // the keyboard stays on the card while it is open
   const f=[skip,back,next].filter(b=>!b.disabled),at=f.indexOf(document.activeElement);e.preventDefault();e.stopPropagation();
   f[(at+(e.shiftKey?-1:1)+f.length)%f.length].focus();}}
 document.addEventListener('keydown',key,true);addEventListener('resize',layout);
 document.body.append(root);open=close;draw();
 // the page keeps moving under it (tiles arrive, dials are placed): the ring and the card follow
 timer=setInterval(()=>{if(!document.hidden){if(!targets.length||!targets.every(shown))targets=find(steps[i]);layout();}},600);
 return close;}

/** The first start: show `page`'s walkthrough once per installation. Never over an open sheet or dialog, and never for a test's browser. */
export async function maybeTour(page){
 if(open||localSeen(page)||navigator.webdriver)return;
 let seen=null;try{const r=await fetch('/api/tour',{cache:'no-store'});if(r.ok)seen=(await r.json()).seen;}catch{}
 if(!Array.isArray(seen))return;   // the service could not say: better no walkthrough than one on every start
 if(seen.includes(page)){try{localStorage.setItem(KEY(page),'1');}catch{}return;}
 if(document.querySelector('.nx-modal,#nx-sheet:not([hidden]),.nx-radial:not([hidden]),.nx-tour'))return;
 startTour(page);}
