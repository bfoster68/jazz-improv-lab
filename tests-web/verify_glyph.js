const { chromium } = require('playwright');
(async()=>{
  const b=await chromium.launch({executablePath:'/opt/pw-browsers/chromium'});
  const p=await b.newPage();
  const errs=[]; p.on('pageerror',e=>errs.push('PAGEERR: '+e.message));
  p.on('console',m=>{if(m.type()==='error'&&!/net::|ERR_/.test(m.text()))errs.push('CONSOLE: '+m.text());});
  await p.goto('file:///root/jazz/jazz-improv-app.html'); await p.waitForTimeout(500);
  const R={};

  // 1. unit-test the formatter directly
  R.unit = await p.evaluate(()=>{
    const strip=h=>h.replace(/<[^>]+>/g,'');
    const cases=['Bb','F#','Dm7b5','Bbmaj7','Ebm7','Db7','b3','#11','b13','bb7',
                 'the b7 of G7','1 b3 5 b7','C E G B','Cmaj7','G7alt','Am7b5',
                 'About','Feb','absolutely','number 5','sub 5','climb','bebop','blues','barline','b'];
    const o={}; cases.forEach(c=>o[c]=strip(fmtMusic(c))); return o;
  });

  // 2. rendered glyphs actually present in the UI
  await p.click('#nav button[data-tab="theory"]');
  await p.selectOption('#th-root','10'); await p.selectOption('#th-type','S:Blues scale');
  await p.waitForTimeout(60);
  R.theory = await p.$$eval('#th-notes .notebtn', xs=>xs.map(x=>x.textContent.trim()).join(' '));
  R.theoryAccSpans = await p.$$eval('#th-notes .acc', xs=>xs.length);

  await p.click('#nav button[data-tab="play"]');
  await p.selectOption('#pa-prog','autumn'); await p.waitForTimeout(80);
  R.paBars = await p.$$eval('#pa-bars .bar', xs=>xs.map(x=>x.childNodes[0].textContent.trim()).join(' | '));
  R.paTones = await p.$$eval('#pa-tones .ct', xs=>xs.map(x=>x.textContent.trim()).join(' '));

  await p.click('#nav button[data-tab="practice"]');
  await p.selectOption('#dr-type','altered'); await p.selectOption('#dr-key','0');
  await p.click('#dr-new'); await p.waitForTimeout(60);
  R.drill = await p.$$eval('#dr-notes .notebtn', xs=>xs.map(x=>x.textContent.trim()).join(' '));

  // 3. prose was converted without mangling ordinary words
  R.proseSample = await p.evaluate(()=>{
    const el=[...document.querySelectorAll('#tab-learn p')].find(x=>/9th, 11th/.test(x.textContent));
    return el? el.textContent.slice(0,180):'(not found)';
  });
  R.proseWordsIntact = await p.evaluate(()=>{
    const t=document.getElementById('tab-learn').textContent;
    return {bebop:/bebop/.test(t), blues:/blues/.test(t), barline:/barline/.test(t),
            climbs:!/cli♭s/.test(t), about:!/A♭out/.test(t)};
  });

  // 4. ear training still scores correctly with pretty labels
  await p.click('#nav button[data-tab="ear"]');
  await p.selectOption('#ear-mode','extended'); await p.click('#ear-new'); await p.waitForTimeout(60);
  R.earLabel = await p.$eval('#ear-choices button', e=>e.innerHTML);
  const correct = await p.evaluate(()=>earState.answer);
  await p.evaluate(a=>{ [...document.querySelectorAll('#ear-choices button')].find(b=>b.dataset.label===a).click(); }, correct);
  R.earScored = await p.evaluate(()=>({score:earState.score, streak:earState.streak,
    fb:document.getElementById('ear-feedback').textContent}));

  console.log(JSON.stringify(R,null,1));
  console.log('ERRORS',errs.length); errs.forEach(e=>console.log(' -',e));
  await b.close(); process.exit(errs.length?1:0);
})();
