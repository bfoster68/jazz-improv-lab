const { chromium } = require('playwright');
(async()=>{
  const b=await chromium.launch({executablePath:'/opt/pw-browsers/chromium'});
  const p=await b.newPage();
  const errs=[]; p.on('pageerror',e=>errs.push('PAGEERR: '+e.message));
  p.on('console',m=>{if(m.type()==='error')errs.push('CONSOLE: '+m.text());});
  await p.goto('file:///root/jazz/jazz-improv-app.html'); await p.waitForTimeout(300);

  // Render each instrument offline and measure the actual audio produced.
  const res = await p.evaluate(async ()=>{
    async function render(fn, secs){
      AC=null; MASTER=null; COMP=null; NOISE=null; REV=null; REVSEND=null;   // reset engine
      const off=new OfflineAudioContext(2, 44100*secs, 44100);
      // make ac() adopt the offline context
      AC=off;
      MASTER=off.createGain(); MASTER.gain.value=1;
      COMP=off.createDynamicsCompressor();
      const og=off.createGain(); og.gain.value=0.85;
      MASTER.connect(COMP); COMP.connect(og); og.connect(off.destination);
      REV=off.createConvolver(); REV.buffer=makeIR(off,1.7,2.6);
      REVSEND=off.createGain(); REVSEND.gain.value=1;
      const rr=off.createGain(); rr.gain.value=0.30;
      const rt=off.createBiquadFilter(); rt.type='highpass'; rt.frequency.value=380;
      REVSEND.connect(REV); REV.connect(rt); rt.connect(rr); rr.connect(COMP);
      fn(off);
      const buf=await off.startRendering();
      const d=buf.getChannelData(0);
      let peak=0,sum=0,nz=0;
      for(let i=0;i<d.length;i++){ const a=Math.abs(d[i]); if(a>peak)peak=a; sum+=d[i]*d[i]; if(a>1e-4)nz++; }
      return {peak:+peak.toFixed(4), rms:+Math.sqrt(sum/d.length).toFixed(5),
              voiced:+(nz/d.length).toFixed(3), clipped:peak>0.999};
    }
    const out={};
    out.vibes = await render(()=>playTone(72,0.05,0.5,{voice:'vibes',gain:0.2}), 2);
    out.keys  = await render(()=>playChordTones([64,67,71,74],0.05,1.2,{}), 2.5);
    out.bass  = await render(()=>playTone(41,0.05,0.5,{voice:'bass',gain:0.19}), 1.5);
    out.ride  = await render(()=>rideHit(0.05,{gain:0.03}), 1.5);
    out.kick  = await render(()=>kick(0.05), 1);
    out.snare = await render(()=>noiseBurst(0.05,0.1,{type:'bandpass',freq:2300,Q:0.8,gain:0.03}), 1);
    // a dense full-band bar: worst case for clipping
    out.fullBand = await render(()=>{
      for(let i=0;i<4;i++){ const t=0.05+i*0.5;
        rideHit(t,{gain:0.034}); playTone(41+i,t,0.4,{voice:'bass',gain:0.19});
        if(i%2===0) playChordTones([64,67,71,74],t,0.45,{gain:0.06});
        if(i===0) kick(t);
      }
    }, 3);
    return out;
  });

  console.log(JSON.stringify(res,null,1));
  console.log('ERRORS',errs.length); errs.forEach(e=>console.log(' -',e));
  await b.close(); process.exit(errs.length?1:0);
})();
