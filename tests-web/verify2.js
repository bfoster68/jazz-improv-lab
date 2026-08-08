const { chromium } = require('playwright');
(async () => {
  const b = await chromium.launch({ executablePath:'/opt/pw-browsers/chromium' });
  const p = await b.newPage();
  const errs=[];
  p.on('console',m=>{ if(m.type()==='error') errs.push('CONSOLE: '+m.text()); });
  p.on('pageerror',e=>errs.push('PAGEERR: '+e.message));
  await p.goto('file:///root/jazz/jazz-improv-app.html');
  await p.waitForTimeout(300);

  // Learn level filter
  for(const l of ['beginner','intermediate','advanced','all','beginner']){
    await p.click(`#lvl-filter button[data-lvl="${l}"]`); await p.waitForTimeout(40);
  }
  const visIntermediate = await p.evaluate(()=>{ 
    let vis=0; document.querySelectorAll('#tab-learn .card[data-level]').forEach(c=>{ if(c.getAttribute('data-level')==='intermediate' && c.style.display!=='none') vis++; }); return vis; });

  // Every drill in a couple keys
  await p.click('#nav button[data-tab="practice"]');
  const drills = await p.$$eval('#dr-type option', os=>os.map(o=>o.value));
  let drillMin=99;
  for(const d of drills){ for(const k of ['0','5','9']){
    await p.selectOption('#dr-type',d); await p.selectOption('#dr-key',k); await p.click('#dr-new');
    const n = await p.$$eval('#dr-notes .notebtn', x=>x.length); if(n<drillMin) drillMin=n;
    // ensure every note has a name and degree
    const bad = await p.$$eval('#dr-notes .notebtn', xs=>xs.filter(x=>!x.querySelector('.nm').textContent||!x.querySelector('.dg').textContent).length);
    if(bad) errs.push('drill '+d+' key '+k+' has '+bad+' empty notes');
  }}
  await p.click('#dr-play'); await p.waitForTimeout(200);

  // Every ear mode
  await p.click('#nav button[data-tab="ear"]');
  const modes = await p.$$eval('#ear-mode option', os=>os.map(o=>o.value));
  for(const m of modes){ await p.selectOption('#ear-mode',m); await p.click('#ear-new'); await p.waitForTimeout(50);
    const c = await p.$$eval('#ear-choices button', x=>x.length); if(c<2) errs.push('ear mode '+m+' has <2 choices');
    const btn=await p.$('#ear-choices button'); if(btn) await btn.click();
  }

  // Every progression: build + play briefly
  await p.click('#nav button[data-tab="play"]');
  const progs = await p.$$eval('#pa-prog option', os=>os.map(o=>o.value));
  const progInfo={};
  for(const pr of progs){ await p.selectOption('#pa-prog',pr); await p.waitForTimeout(60);
    const bars = await p.$$eval('#pa-bars .bar', x=>x.length);
    const now = await p.$eval('#pa-chordnow', e=>e.textContent);
    const tones = await p.$$eval('#pa-tones .ct', x=>x.length);
    progInfo[pr]=bars+'bars/'+now+'/'+tones+'tones';
    if(bars<1||tones<3) errs.push('prog '+pr+' bad: '+progInfo[pr]);
  }
  await p.click('#pa-toggle'); await p.waitForTimeout(600); await p.click('#pa-toggle');

  console.log('visIntermediateCards', visIntermediate);
  console.log('drillMinNotes', drillMin);
  console.log('progInfo', JSON.stringify(progInfo));
  console.log('ERRORS', errs.length);
  errs.forEach(e=>console.log(' -',e));
  await b.close();
  process.exit(errs.length?1:0);
})();
