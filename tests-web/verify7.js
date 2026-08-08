const { chromium } = require('playwright');
(async()=>{
  const b=await chromium.launch({executablePath:'/opt/pw-browsers/chromium'});
  const p=await b.newPage();
  const errs=[]; p.on('pageerror',e=>errs.push('PAGEERR: '+e.message));
  p.on('console',m=>{if(m.type()==='error')errs.push('CONSOLE: '+m.text());});
  await p.goto('file:///root/jazz/jazz-improv-app.html'); await p.waitForTimeout(300);

  const R={};
  // --- A. Theory Lab: degree labels + spelling ---
  await p.click('#nav button[data-tab="theory"]');
  const th=async(root,type)=>{ await p.selectOption('#th-root',String(root)); await p.selectOption('#th-type',type); await p.waitForTimeout(30);
    return p.$$eval('#th-notes .notebtn', xs=>xs.map(x=>x.querySelector('.nm').textContent+'('+x.querySelector('.dg').textContent+')').join(' ')); };
  R.halfdim = await th(11,'C:Half-dim (m7b5)');
  R.dim7    = await th(0,'C:Diminished 7');
  R.altered = await th(0,'S:Altered (dominant)');
  R.lydDom  = await th(2,'S:Lydian dominant');

  // --- B. Drills: contour + spelling in multiple keys ---
  await p.click('#nav button[data-tab="practice"]');
  const drill=async(type,key)=>{ await p.selectOption('#dr-type',type); await p.selectOption('#dr-key',String(key)); await p.click('#dr-new'); await p.waitForTimeout(30);
    return p.evaluate(()=>drSequence.map(n=>n.name+':'+n.midi)); };
  R.guidetoneC = await drill('guidetone',0);
  R.enclosureAb= await drill('enclosure',8);
  R.scaleseqF  = await drill('scaleseq',5);
  R.pentF      = await drill('pentpattern',5);
  R.arpC       = await drill('39arp',0);
  R.usC        = await drill('upperstructure',0);
  R.coltraneC  = await drill('coltrane',0);
  R.pentsuperC = await drill('pentsuper',0);
  R.alteredC   = await drill('altered',0);

  // contour sanity: no drill should contain a leap > 12 semitones
  const drills = await p.$$eval('#dr-type option', os=>os.map(o=>o.value));
  const leaps=[];
  for(const d of drills){ for(const k of [0,3,5,8,10]){
    const seq=await drill(d,k);
    const m=seq.map(s=>parseInt(s.split(':')[1]));
    for(let i=1;i<m.length;i++){ const iv=Math.abs(m[i]-m[i-1]); if(iv>12) leaps.push(d+' key'+k+' leap '+iv); }
    // no note should be spelled with a double accidental or be empty
    seq.forEach(s=>{ const nm=s.split(':')[0]; if(!nm||/##|bb/.test(nm)) leaps.push(d+' key'+k+' bad spelling '+nm); });
  }}
  R.badLeaps=leaps;

  // --- C. Ear training: randomness (not deterministic) ---
  await p.click('#nav button[data-tab="ear"]');
  const qs=new Set();
  for(let i=0;i<12;i++){ await p.click('#ear-new'); await p.waitForTimeout(20); qs.add(await p.evaluate(()=>earState.answer)); }
  R.distinctEarQs=qs.size;
  R.earProgs = await p.evaluate(()=>EAR_PROGS.map(x=>x[0]));

  // --- D. Play-along: scales + chord tone degrees ---
  await p.click('#nav button[data-tab="play"]');
  const chk=async(prog,barClicks)=>{ await p.selectOption('#pa-prog',prog); await p.waitForTimeout(60);
    return p.evaluate(()=>({chord:document.getElementById('pa-chordnow').textContent,
      tones:[...document.querySelectorAll('#pa-tones .ct')].map(x=>x.textContent.trim()).join(' '),
      scale:document.getElementById('pa-scale').textContent})); };
  R.minor251 = await chk('251minC');
  R.atrain   = await chk('atrain');
  R.bossa    = await chk('bluebossa');

  // walk through Autumn Leaves to the D7 bar and check the suggested scale
  R.autumnD7 = await p.evaluate(()=>{ const bars=PROGS['autumn'].build(PROGS['autumn'].key);
     return bars.map(b=>chordSymOne(b.root,b.q)+(b.scale?' ['+b.scale+']':'')).join(' | '); });

  // --- E. Voicing/bass sanity over many bars ---
  R.voiceCheck = await p.evaluate(()=>{
    const bad=[];
    for(const q of ['maj7','m7','dom7','m7b5','m7t']){
      for(let r=0;r<12;r++){
        for(let t=0;t<40;t++){
          const v=compVoicing(r,q).slice().sort((a,b)=>a-b);
          for(let i=1;i<v.length;i++) if(v[i]-v[i-1]<2) bad.push(q+' r'+r+' m2 clash '+v.join(','));
          if(v[0]<52||v[v.length-1]>84) bad.push(q+' r'+r+' register '+v.join(','));
        }
      }
    }
    return bad.slice(0,5).concat(['count='+bad.length]);
  });
  R.walkCheck = await p.evaluate(()=>{
    let dup=0,total=0;
    for(let t=0;t<400;t++){
      const bar={root:Math.floor(Math.random()*12),q:'m7'};
      const w=walkingBar(bar, Math.floor(Math.random()*12));
      total++; for(let i=1;i<w.length;i++) if(w[i]===w[i-1]) dup++;
    }
    return 'dupAdjacent='+dup+'/'+total;
  });

  // --- F. progression switch while playing keeps playing ---
  await p.selectOption('#pa-prog','bluesF');
  await p.click('#pa-toggle'); await p.waitForTimeout(400);
  await p.selectOption('#pa-prog','atrain'); await p.waitForTimeout(400);
  R.stillPlayingAfterSwitch = await p.evaluate(()=>PA.playing);
  await p.evaluate(()=>{ if(PA.playing) document.getElementById('pa-toggle').click(); });

  console.log(JSON.stringify(R,null,1));
  console.log('ERRORS', errs.length); errs.forEach(e=>console.log(' -',e));
  await b.close(); process.exit(errs.length?1:0);
})();
