const { chromium } = require('playwright');
(async()=>{
  const b=await chromium.launch({executablePath:'/opt/pw-browsers/chromium'});
  const p=await b.newPage();
  const errs=[]; p.on('pageerror',e=>errs.push('PAGEERR: '+e.message));
  p.on('console',m=>{if(m.type()==='error'&&!/ERR_|net::/.test(m.text()))errs.push('CONSOLE: '+m.text());});
  await p.goto('file:///root/jazz/jazz-improv-app.html'); await p.waitForTimeout(300);
  const R={};

  // ---------- 1. FAILURE PATH: CDN unreachable (real condition in this sandbox) ----------
  await p.click('#sf-toggle');
  await p.waitForFunction(()=>{ const s=document.getElementById('sf-status');
    return s && /Couldn|built-in|High-quality/.test(s.textContent); }, {timeout:70000});
  R.afterFailure = {
    status: await p.$eval('#sf-status',e=>e.textContent),
    statusClass: await p.$eval('#sf-status',e=>e.className),
    checkboxReset: !(await p.$eval('#sf-toggle',e=>e.checked)),
    sfOn: await p.evaluate(()=>SF.on),
    toggleClass: await p.$eval('.sftoggle',e=>e.className)
  };
  // audio must still work after a failed download
  R.synthStillWorks = await p.evaluate(()=>{ try{ playTone(60,null,0.3,{voice:'keys',gain:0.2}); return true; }catch(e){ return 'ERR '+e.message; } });

  // ---------- 2. SUCCESS PATH: inject a fake soundfont table, no network ----------
  R.sampleTest = await p.evaluate(async ()=>{
    // build a tiny valid WAV (sine) as a data URI, one per note
    function wavURI(freq){
      const sr=8000, n=sr*0.2, buf=new ArrayBuffer(44+n*2), dv=new DataView(buf);
      const W=(o,s)=>{ for(let i=0;i<s.length;i++) dv.setUint8(o+i,s.charCodeAt(i)); };
      W(0,'RIFF'); dv.setUint32(4,36+n*2,true); W(8,'WAVEfmt '); dv.setUint32(16,16,true);
      dv.setUint16(20,1,true); dv.setUint16(22,1,true); dv.setUint32(24,sr,true);
      dv.setUint32(28,sr*2,true); dv.setUint16(32,2,true); dv.setUint16(34,16,true);
      W(36,'data'); dv.setUint32(40,n*2,true);
      for(let i=0;i<n;i++) dv.setInt16(44+i*2, Math.sin(2*Math.PI*freq*i/sr)*12000*(1-i/n), true);
      let bin=''; const u8=new Uint8Array(buf);
      for(let i=0;i<u8.length;i++) bin+=String.fromCharCode(u8[i]);
      return 'data:audio/wav;base64,'+btoa(bin);
    }
    window.MIDI={Soundfont:{}};
    const names=['C3','C4','E4','G4','C5'];
    ['acoustic_grand_piano','acoustic_bass','vibraphone'].forEach(inst=>{
      const t={}; names.forEach(n=>{ t[n]=wavURI(440); }); MIDI.Soundfont[inst]=t;
    });
    // widen ranges so our sparse test notes are in-band
    SF_DEF.keys.lo=40; SF_DEF.bass.lo=40; SF_DEF.bass.hi=90; SF_DEF.vibes.lo=40;
    await sfEnable();
    return { ready:SF.ready, on:SF.on,
             loaded:{keys:Object.keys(SF.buf.keys).length, bass:Object.keys(SF.buf.bass).length, vibes:Object.keys(SF.buf.vibes).length},
             status:document.getElementById('sf-status').textContent };
  });

  // ---------- 3. Samples actually produce audio, and nearest-pitch transposition works ----------
  R.render = await p.evaluate(async ()=>{
    async function render(fn,secs){
      const off=new OfflineAudioContext(2,44100*secs,44100);
      const savedAC=AC, savedM=MASTER, savedR=REVSEND;
      AC=off; MASTER=off.createGain(); COMP=off.createDynamicsCompressor();
      MASTER.connect(COMP); COMP.connect(off.destination); REVSEND=null; REV=null;
      // re-decode our sample into the offline context is unnecessary: AudioBuffers are portable
      fn();
      const buf=await off.startRendering();
      AC=savedAC; MASTER=savedM; REVSEND=savedR;
      const d=buf.getChannelData(0); let peak=0;
      for(let i=0;i<d.length;i++) peak=Math.max(peak,Math.abs(d[i]));
      return +peak.toFixed(4);
    }
    return {
      exactPitch: await render(()=>playTone(60,0.02,0.3,{voice:'keys',gain:0.06}),1),
      transposed: await render(()=>playTone(61,0.02,0.3,{voice:'keys',gain:0.06}),1), // no C#4 sample -> nearest
      farOutOfRange: await render(()=>playTone(20,0.02,0.3,{voice:'bass',gain:0.06}),1) // >6 semis -> synth fallback
    };
  });

  // ---------- 4. Toggling off returns to synth; band still plays ----------
  await p.click('#nav button[data-tab="play"]');
  await p.click('#pa-toggle'); await p.waitForTimeout(500);
  R.playingWithSamples = await p.evaluate(()=>PA.playing);
  await p.evaluate(()=>{ const t=document.getElementById('sf-toggle'); t.checked=false; t.dispatchEvent(new Event('change')); });
  await p.waitForTimeout(400);
  R.afterToggleOff = await p.evaluate(()=>({sfOn:SF.on, ready:SF.ready, playing:PA.playing,
    status:document.getElementById('sf-status').textContent}));
  await p.evaluate(()=>{ if(PA.playing) document.getElementById('pa-toggle').click(); });

  console.log(JSON.stringify(R,null,1));
  console.log('ERRORS',errs.length); errs.forEach(e=>console.log(' -',e));
  await b.close(); process.exit(errs.length?1:0);
})();
