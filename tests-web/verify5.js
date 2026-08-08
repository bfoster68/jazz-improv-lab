const { chromium } = require('playwright');
(async()=>{
  const b=await chromium.launch({executablePath:'/opt/pw-browsers/chromium'});
  const p=await b.newPage();
  const errs=[]; p.on('pageerror',e=>errs.push(e.message)); p.on('console',m=>{if(m.type()==='error')errs.push(m.text());});
  await p.goto('file:///root/jazz/jazz-improv-app.html'); await p.waitForTimeout(300);

  // Masters tab renders
  await p.click('#nav button[data-tab="masters"]');
  const masters = await p.$$eval('#masters-list .mname', xs=>xs.map(x=>x.textContent));
  const stds = await p.$$eval('#standards-list b', xs=>xs.map(x=>x.textContent));
  // click a master name (sound) and a couple standard-heads
  await p.click('#masters-list .mname');

  // Check standard chord layouts
  await p.click('#nav button[data-tab="play"]');
  const grab = async(v)=>{ await p.selectOption('#pa-prog',v); await p.waitForTimeout(50);
    return p.$$eval('#pa-bars .bar', xs=>xs.map(x=>x.childNodes[0].textContent.trim())); };
  const sowhat=await grab('sowhat');
  const bossa=await grab('bluebossa');
  const atrain=await grab('atrain');
  const rhythm=await grab('rhythmA');

  // Run rhythm changes (split bars) and confirm now-playing changes within a bar
  await p.selectOption('#pa-prog','rhythmA');
  await p.evaluate(()=>{ const t=document.getElementById('pa-tempo'); t.value=160; t.dispatchEvent(new Event('input')); });
  const seen=new Set();
  await p.click('#pa-toggle');
  for(let i=0;i<14;i++){ await p.waitForTimeout(180); seen.add(await p.$eval('#pa-chordnow',e=>e.textContent)); }
  await p.click('#pa-toggle');

  // Test loadStandard from Masters tab (▶ Play)
  await p.click('#nav button[data-tab="masters"]');
  const btn = await p.$('#standards-list button');
  await btn.click(); await p.waitForTimeout(400);
  const onPlayTab = await p.$eval('#tab-play', e=>e.classList.contains('active'));
  const playing = await p.evaluate(()=>PA.playing);
  await p.evaluate(()=>{ if(PA.playing) document.getElementById('pa-toggle').click(); });

  console.log('mastersCount', masters.length, '| standardsCount', stds.length);
  console.log('SOWHAT', sowhat.length, sowhat.slice(0,2).join(','), '...', sowhat[8]);
  console.log('BOSSA', bossa.join(' '));
  console.log('ATRAIN', atrain.join(' '));
  console.log('RHYTHM', rhythm.join(' | '));
  console.log('rhythm now-playing distinct chords seen while running:', [...seen].join(' '));
  console.log('loadStandard -> onPlayTab', onPlayTab, 'playing', playing);
  console.log('ERRORS', errs.length); errs.forEach(e=>console.log(' -',e));
  await b.close();
  process.exit(errs.length?1:0);
})();
