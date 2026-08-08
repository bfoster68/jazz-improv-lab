const { chromium } = require('playwright');
(async()=>{
  const b=await chromium.launch({executablePath:'/opt/pw-browsers/chromium'});
  const p=await b.newPage({viewport:{width:1000,height:1150}});
  const errs=[]; p.on('pageerror',e=>errs.push(e.message)); p.on('console',m=>{if(m.type()==='error')errs.push(m.text());});
  await p.goto('file:///root/jazz/jazz-improv-app.html'); await p.waitForTimeout(300);
  // intermediate filter shows cards
  await p.click('#lvl-filter button[data-lvl="intermediate"]'); await p.waitForTimeout(60);
  const interVis = await p.$$eval('#tab-learn .card[data-level="intermediate"]', xs=>xs.filter(x=>x.style.display!=='none').length);
  const begHidden = await p.$$eval('#tab-learn .card[data-level="beginner"]', xs=>xs.filter(x=>x.style.display==='none').length);
  // autumn chords
  await p.click('#nav button[data-tab="play"]');
  await p.selectOption('#pa-prog','autumn'); await p.waitForTimeout(80);
  const autumnBars = await p.$$eval('#pa-bars .bar', xs=>xs.map(x=>x.childNodes[0].textContent.trim()));
  await p.selectOption('#pa-prog','coltrane'); await p.waitForTimeout(80);
  const coltraneBars = await p.$$eval('#pa-bars .bar', xs=>xs.map(x=>x.childNodes[0].textContent.trim()));
  await p.screenshot({path:'app-coltrane.png'});
  console.log('interVisible',interVis,'beginnerHidden',begHidden);
  console.log('AUTUMN',autumnBars.join(' | '));
  console.log('COLTRANE',coltraneBars.join(' | '));
  console.log('ERRORS',errs.length); errs.forEach(e=>console.log(' -',e));
  await b.close();
})();
