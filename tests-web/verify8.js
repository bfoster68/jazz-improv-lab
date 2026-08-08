const { chromium } = require('playwright');
(async()=>{
  const b=await chromium.launch({executablePath:'/opt/pw-browsers/chromium'});
  const p=await b.newPage();
  const errs=[]; p.on('pageerror',e=>errs.push(e.message));
  p.on('console',m=>{if(m.type()==='error')errs.push(m.text());});
  await p.goto('file:///root/jazz/jazz-improv-app.html'); await p.waitForTimeout(300);
  const R={};
  await p.click('#nav button[data-tab="practice"]');
  const drill=async(t,k)=>{ await p.selectOption('#dr-type',t); await p.selectOption('#dr-key',String(k)); await p.click('#dr-new'); await p.waitForTimeout(25);
    return p.evaluate(()=>({notes:drSequence.map(n=>n.name+'/'+n.deg),
      bars:[...document.querySelectorAll('#dr-bars .bar')].map(x=>x.textContent)})); };
  R.guidetoneC = (await drill('guidetone',0)).notes;
  R.coltraneB  = await drill('coltrane',11);
  R.coltraneC  = (await drill('coltrane',0)).bars;
  R.ttsubA     = await drill('tritonesub',9);
  R.ttsubG     = (await drill('tritonesub',7)).bars;
  R.lickC      = (await drill('251lick',0)).notes.slice(-4);
  // keyboard access on drill chips
  R.chipRole   = await p.$eval('#dr-notes .notebtn', e=>e.getAttribute('role')+'/'+e.getAttribute('tabindex'));

  // whole-half dim degrees
  await p.click('#nav button[data-tab="theory"]');
  await p.selectOption('#th-root','0'); await p.selectOption('#th-type','S:Whole-half diminished');
  await p.waitForTimeout(30);
  R.wholeHalf = await p.$$eval('#th-notes .notebtn', xs=>xs.map(x=>x.querySelector('.nm').textContent+'('+x.querySelector('.dg').textContent+')').join(' '));

  // altered comping voicings avoid natural 5 / natural 9
  R.altVoicing = await p.evaluate(()=>{
    const bad=[];
    for(let t=0;t<300;t++){
      const v=compVoicing(7,'dom7','Altered (dominant)');
      const pcs=v.map(m=>((m-rootMidiFor(7,4))%12+12)%12);
      if(pcs.includes(7)) bad.push('nat5');
      if(pcs.includes(2)) bad.push('nat9');
    }
    return bad.length? bad.slice(0,3) : 'clean (no natural 5 or 9 on altered bars)';
  });
  // rhythm changes VI7 now suggests altered
  R.rhythmScales = await p.evaluate(()=>rhythmAsection().map(b=>
    chordSymOne(b.root,b.q)+'/'+chordSymOne(b.root2,b.q2)+(b.scale2?' ['+b.scale2.split(' ')[0]+']':'')).join(' | '));

  // aria-current only on the active tab
  for(const t of ['theory','practice','ear','play','masters','learn']) await p.click(`#nav button[data-tab="${t}"]`);
  R.ariaCurrent = await p.$$eval('#nav button', xs=>xs.map(x=>x.getAttribute('aria-current')).filter(Boolean).length);

  // mobile: no horizontal overflow
  await p.setViewportSize({width:360,height:740});
  await p.waitForTimeout(200);
  R.overflow = await p.evaluate(()=>{ const o={};
    ['learn','theory','practice','ear','play','masters'].forEach(t=>{ o[t]=document.documentElement.scrollWidth; });
    return document.documentElement.scrollWidth+'px vs 360 viewport'; });

  console.log(JSON.stringify(R,null,1));
  console.log('ERRORS',errs.length); errs.forEach(e=>console.log(' -',e));
  await b.close(); process.exit(errs.length?1:0);
})();
