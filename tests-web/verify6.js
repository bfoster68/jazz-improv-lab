const { chromium } = require('playwright');
(async()=>{
  const b=await chromium.launch({executablePath:'/opt/pw-browsers/chromium'});
  const p=await b.newPage();
  const errs=[]; p.on('pageerror',e=>errs.push(e.message)); p.on('console',m=>{if(m.type()==='error')errs.push(m.text());});
  await p.goto('file:///root/jazz/jazz-improv-app.html'); await p.waitForTimeout(300);
  await p.click('#nav button[data-tab="play"]');
  const grab=async v=>{ await p.selectOption('#pa-prog',v); await p.waitForTimeout(50);
    return p.$$eval('#pa-bars .bar', xs=>xs.map(x=>x.childNodes[0].textContent.trim())); };
  const sw=await grab('sowhat');
  const rf=await grab('rhythmFull');
  // run rhythmFull briefly through the bridge
  await p.evaluate(()=>{ const t=document.getElementById('pa-tempo'); t.value=220; t.dispatchEvent(new Event('input')); });
  await p.click('#pa-toggle'); await p.waitForTimeout(1500); await p.click('#pa-toggle');
  console.log('SoWhat bars:',sw.length,'| first',sw[0],'| bar17',sw[16],'| bar25',sw[24]);
  console.log('RhythmFull bars:',rf.length);
  console.log('  A end/bridge:',rf.slice(14,24).join('  '));
  console.log('ERRORS',errs.length); errs.forEach(e=>console.log(' -',e));
  await b.close(); process.exit(errs.length?1:0);
})();
