const fs = require('fs');
const {
  Document, Packer, Paragraph, TextRun, HeadingLevel, AlignmentType,
  Table, TableRow, TableCell, WidthType, BorderStyle, ShadingType,
  PageBreak, LevelFormat, TableOfContents, PositionalTab, PositionalTabAlignment, PositionalTabLeader
} = require('docx');

const ACCENT = "B8860B", INK = "222222", MUTE = "666666", TEAL = "2C7A75";

const H1 = (t)=>new Paragraph({heading:HeadingLevel.HEADING_1, spacing:{before:280,after:120}, children:[new TextRun({text:t,color:INK,bold:true})]});
const H2 = (t)=>new Paragraph({heading:HeadingLevel.HEADING_2, spacing:{before:200,after:80}, children:[new TextRun({text:t,color:ACCENT,bold:true})]});
const P  = (runs, opts={})=>new Paragraph(Object.assign({spacing:{after:120,line:276}}, opts, {children: Array.isArray(runs)?runs:[new TextRun(runs)]}));
const T  = (text,opts={})=>new TextRun(Object.assign({text,color:INK,size:21},opts));
const B  = (text,opts={})=>new TextRun(Object.assign({text,bold:true,color:INK,size:21},opts));
const bullet = (runs)=>new Paragraph({numbering:{reference:"b",level:0}, spacing:{after:60,line:270}, children:Array.isArray(runs)?runs:[new TextRun(runs)]});

function hr(){ return new Paragraph({border:{bottom:{style:BorderStyle.SINGLE,size:6,color:"CCCCCC",space:8}}, spacing:{after:120}}); }

function callout(text){
  return new Table({
    width:{size:9360,type:WidthType.DXA}, columnWidths:[9360],
    borders:{top:{style:BorderStyle.NONE},bottom:{style:BorderStyle.NONE},right:{style:BorderStyle.NONE},insideHorizontal:{style:BorderStyle.NONE},insideVertical:{style:BorderStyle.NONE},
      left:{style:BorderStyle.SINGLE,size:18,color:TEAL}},
    rows:[new TableRow({children:[new TableCell({
      width:{size:9360,type:WidthType.DXA},
      shading:{type:ShadingType.CLEAR,fill:"F0F6F5"},
      margins:{top:100,bottom:100,left:160,right:160},
      children:[new Paragraph({children:[new TextRun({text:"🔑  ",size:21}),new TextRun({text:text,italics:true,color:INK,size:21})]})]
    })]})]
  });
}

// ---- chord/scale reference table ----
function refTable(){
  const head=["Chord quality","Formula","Example","Go-to scale"];
  const rows=[
    ["Major 7 (Imaj7)","1 · 3 · 5 · 7","C E G B","Major (Ionian); Lydian for openness"],
    ["Minor 7 (iim7)","1 · ♭3 · 5 · ♭7","D F A C","Dorian"],
    ["Dominant 7 (V7)","1 · 3 · 5 · ♭7","G B D F","Mixolydian; Altered / Diminished for tension"],
    ["Half-diminished (m7♭5)","1 · ♭3 · ♭5 · ♭7","B D F A","Locrian"],
    ["Minor 6 / minor tonic","1 · ♭3 · 5 · 6","Cm6: C E♭ G A","Melodic minor / Dorian"],
  ];
  const widths=[2600,1900,1860,3000];
  const mkCell=(txt,i,hdr)=>new TableCell({
    width:{size:widths[i],type:WidthType.DXA},
    shading:{type:ShadingType.CLEAR,fill:hdr?ACCENT:"FBF6EC"},
    margins:{top:70,bottom:70,left:110,right:110},
    children:[new Paragraph({children:[new TextRun({text:txt,bold:hdr,color:hdr?"FFFFFF":INK,size:19})]})]
  });
  return new Table({
    width:{size:9360,type:WidthType.DXA}, columnWidths:widths,
    rows:[
      new TableRow({tableHeader:true,children:head.map((h,i)=>mkCell(h,i,true))}),
      ...rows.map(r=>new TableRow({children:r.map((c,i)=>mkCell(c,i,false))}))
    ]
  });
}

// ---- level roadmap table ----
function levelTable(){
  const head=["Level","Harmony & scales","Vocabulary & devices","Practice focus"];
  const rows=[
    ["Beginner","Chord tones, ii–V–I, guide tones; major, Dorian, Mixolydian, pentatonic, blues","Guide-tone lines, enclosures, target notes","Sing first; small loops; leave space; land 3rds & 7ths on strong beats"],
    ["Intermediate","Extensions (9/11/13) & avoid notes; bebop scales; tritone subs","Digital patterns (1-2-3-5), 3-to-9 arpeggios, bebop lines","Play through the changes; outline every chord; all 12 keys"],
    ["Advanced","Altered dominants, upper-structure triads, reharmonization, Coltrane cycle","Side-slipping, superimposition, motivic development, metric displacement","Controlled tension that resolves; storytelling; rhythmic independence"],
  ];
  const widths=[1500,2820,2540,2500];
  const mkCell=(txt,i,hdr)=>new TableCell({
    width:{size:widths[i],type:WidthType.DXA},
    shading:{type:ShadingType.CLEAR,fill:hdr?ACCENT:"FBF6EC"},
    margins:{top:70,bottom:70,left:110,right:110},
    children:[new Paragraph({children:[new TextRun({text:txt,bold:hdr,color:hdr?"FFFFFF":INK,size:18})]})]
  });
  return new Table({
    width:{size:9360,type:WidthType.DXA}, columnWidths:widths,
    rows:[
      new TableRow({tableHeader:true,cantSplit:true,children:head.map((h,i)=>mkCell(h,i,true))}),
      ...rows.map(r=>new TableRow({cantSplit:true,children:r.map((c,i)=>mkCell(c,i,false))}))
    ]
  });
}

// ---- masters listening table ----
function mkTableGeneric(head, rows, widths, fontSize){
  const mkCell=(txt,i,hdr)=>new TableCell({
    width:{size:widths[i],type:WidthType.DXA},
    shading:{type:ShadingType.CLEAR,fill:hdr?ACCENT:"FBF6EC"},
    margins:{top:60,bottom:60,left:100,right:100},
    children:[new Paragraph({children:[new TextRun({text:txt,bold:hdr,color:hdr?"FFFFFF":INK,size:fontSize||18})]})]
  });
  return new Table({
    width:{size:9360,type:WidthType.DXA}, columnWidths:widths,
    rows:[
      new TableRow({tableHeader:true,cantSplit:true,children:head.map((h,i)=>mkCell(h,i,true))}),
      ...rows.map(r=>new TableRow({cantSplit:true,children:r.map((c,i)=>mkCell(c,i,false))}))
    ]
  });
}
function mastersTable(){
  return mkTableGeneric(
    ["Player","Instrument · era","What to listen for"],
    [
      ["Louis Armstrong","Trumpet / voice · 1920s–70s","The first great soloist — joyous swing, singing phrases, pure storytelling."],
      ["Big Joe Turner","Blues vocals · 1930s–80s","The blues-shouting roots under all of jazz — shuffle, call-and-response, feeling."],
      ["Charlie Parker","Alto sax · 1940s–50s (bebop)","Blazing eighth-note lines, chromatic enclosures, the blues in everything."],
      ["Dizzy Gillespie","Trumpet · 1930s–90s (bebop / Afro-Cuban)","Harmonic daring, high-register fire, and the birth of Afro-Cuban jazz."],
      ["Thelonious Monk","Piano · 1940s–70s","Angular lines, space, and “wrong” notes that are exactly right; rhythm as melody."],
      ["Charles Mingus","Bass / composer · 1940s–70s","Gospel-and-blues fire, collective improvisation, huge dynamics."],
      ["Miles Davis","Trumpet · 1940s–90s","Space and reinvention — few notes, deep lyricism; pioneered modal jazz."],
      ["John Coltrane","Tenor / soprano sax · 1950s–60s","“Sheets of sound,” modal jazz, the Giant Steps matrix, spiritual intensity."],
      ["Stan Getz","Tenor sax · 1940s–90s","“The Sound” — a gorgeous tone and effortless, lyrical phrasing; bossa nova."],
      ["Doc Severinsen","Trumpet · 1950s–2010s (big band)","Big-band virtuosity, brilliant tone, dazzling range and command."],
      ["Maynard Ferguson","Trumpet · 1940s–2000s (big band)","Stratospheric high-note power and screaming lead lines."],
      ["Chick Corea","Piano · 1960s–2020s (fusion / Latin / post-bop)","Crystalline lines, Spanish colors, inventive rhythm."],
      ["Arturo Sandoval","Trumpet · 1970s–2020s (Afro-Cuban)","Virtuosity married to bebop and Latin fire; astonishing range."],
    ],
    [2050,2350,4960], 17);
}
function standardsTable(){
  return mkTableGeneric(
    ["Standard","Composer","Key / form","What it teaches"],
    [
      ["So What","Miles Davis","D Dorian · 32-bar AABA","Modal soloing and the power of space."],
      ["Blue Bossa","Kenny Dorham","C minor · 16 bars","Minor ii–V–i, a key change, and Latin feel."],
      ["Take the A Train","Billy Strayhorn","C major · 32-bar AABA","Ellington's theme; the famous D7(♯11)."],
      ["Rhythm Changes (from “I Got Rhythm”)","George Gershwin","played in B♭ · 32-bar AABA","Fast ii–Vs + a dominant-cycle bridge — the bebop proving ground."],
      ["Autumn Leaves","Joseph Kosma","G minor / B♭ · 32 bars","Major and minor ii–V–I side by side."],
      ["Blues (Now's the Time)","Charlie Parker","F · 12-bar blues","The foundation — feel plus chord tones."],
      ["Giant Steps","John Coltrane","B major · 16 bars","The major-third matrix; ultimate changes-running."],
      ["All the Things You Are","Jerome Kern","A♭ major · 36 bars","Chord roots cycling in fourths through shifting key centers."],
    ],
    [2250,1950,1960,3200], 17);
}

const doc = new Document({
  numbering:{config:[{reference:"b",levels:[{level:0,format:LevelFormat.BULLET,text:"•",alignment:AlignmentType.LEFT,style:{run:{color:ACCENT},paragraph:{indent:{left:420,hanging:240}}}}]}]},
  styles:{default:{document:{run:{font:"Calibri",size:21,color:INK}}}},
  sections:[{
    properties:{page:{size:{width:12240,height:15840}, margin:{top:1200,bottom:1200,left:1440,right:1440}}},
    children:[
      // ---------- Title ----------
      new Paragraph({spacing:{before:1600,after:0},alignment:AlignmentType.CENTER,children:[new TextRun({text:"THE RULES OF",color:MUTE,size:28,bold:true,characterSpacing:60})]}),
      new Paragraph({alignment:AlignmentType.CENTER,spacing:{after:60},children:[new TextRun({text:"Jazz Improvisation",color:ACCENT,size:64,bold:true})]}),
      new Paragraph({alignment:AlignmentType.CENTER,spacing:{after:40},children:[new TextRun({text:"A Codified Rule Set and Beginner's Guide for Every Instrument",color:INK,size:26,italics:true})]}),
      new Paragraph({alignment:AlignmentType.CENTER,spacing:{before:200,after:0},children:[new TextRun({text:"Written entirely in note names, scale degrees, and intervals —",color:MUTE,size:20})]}),
      new Paragraph({alignment:AlignmentType.CENTER,spacing:{after:400},children:[new TextRun({text:"no fingerings, no fretboards, so it applies to any instrument or voice.",color:MUTE,size:20})]}),
      hr(),
      P([new TextRun({text:"This document has five parts. ",bold:true,color:INK,size:21}),T("Part I codifies the underlying rules of jazz improvisation as a compact set of axioms — the same rule set used as a generative prompt to build the companion teaching application. Part II is a gentle, beginner-friendly walk-through. Parts III and IV carry the same path upward into intermediate techniques and advanced concepts. Part V is a listening guide — the masters to study and the standards to learn — because the language is ultimately passed down by ear. Together they match the full scope of the app (beginner → intermediate → advanced)."),],{spacing:{before:120,after:60}}),
      new Paragraph({children:[new PageBreak()]}),

      // ---------- Contents (built explicitly so it is never an empty field) ----------
      new Paragraph({heading:HeadingLevel.HEADING_2, spacing:{before:200,after:140},
        children:[new TextRun({text:"Contents",color:INK,bold:true,size:32})]}),
      ...[
        ["Part I","The Codified Rules — eight axioms and a quick-reference table"],
        ["Part II","A Beginner's Guide, Step by Step"],
        ["Part III","Intermediate Techniques"],
        ["Part IV","Advanced Concepts"],
        ["Part V","Listening: The Masters & The Standards"],
        ["","The path at a glance — the three levels side by side"],
        ["","The whole method in one paragraph"],
      ].map(([p,t])=> new Paragraph({spacing:{after:90}, children:[
        new TextRun({text:(p?p+" — ":"• "),bold:!!p,color:p?ACCENT:MUTE,size:21}),
        new TextRun({text:t,color:INK,size:21})
      ]})),
      new Paragraph({children:[new PageBreak()]}),

      // ================= PART I =================
      new Paragraph({spacing:{after:40},children:[new TextRun({text:"PART I",color:MUTE,bold:true,size:22,characterSpacing:80})]}),
      H1("The Codified Rules"),
      P([T("Improvisation is "),B("melody that obeys harmony while sounding like speech."),T(" Every rule below serves one of three masters, in priority order: "),B("(1) sound good"),T(" — the ear is the final judge; "),B("(2) serve the harmony"),T(" — target the notes that define each chord; and "),B("(3) tell a story"),T(" — tension and release, question and answer. When rules conflict, honor the higher-priority master.")]),

      H2("Axiom 1 — Music is intervals, not instruments"),
      P([T("Encode every idea as "),B("scale degrees relative to a root"),T(" (1, ♭2, 2, ♭3, 3, 4, ♯4/♭5, 5, ♭6, 6, ♭7, 7) and as chord tones versus tensions. A lick is a sequence of scale degrees plus a rhythm. This makes any idea transposable to any key and playable on any instrument.")]),

      H2("Axiom 2 — Chord tones are home; the rest is color"),
      P([T("For any chord the chord tones (1, 3, 5, 7) are consonant resting points, and the "),B("3rd and 7th are the guide tones"),T(" that define the chord's quality and function. Non-chord tones (9, 11, 13 and alterations) are color and tension that want to resolve. Rule of thumb: "),B("land a chord tone on strong beats"),T(" — especially the 3rd or 7th — and pass through tensions on weak beats.")]),

      H2("Axiom 3 — The ii–V–I is the sentence"),
      P([T("Most of tonal jazz is built from the "),B("ii–V–I"),T(" progression (in C: Dm7 – G7 – Cmaj7): setup, tension, resolution. Its engine is the "),B("guide-tone line"),T(" — the 7th of each chord falls by a half or whole step to the 3rd of the next (C→B over Dm7→G7; F→E over G7→Cmaj7). Following that line alone already sounds like jazz. The minor ii–V–i (Dm7♭5 – G7alt – Cm) is the same sentence in a darker mood.")]),

      H2("Axiom 4 — Chord–scale relationships (the raw material)"),
      P([T("Choose scales by chord quality. The full map is below; as a beginner, learn "),B("major, Dorian, Mixolydian, minor pentatonic and the blues scale"),T(" first (see Part II), and add Locrian and the altered sounds later:")]),
      bullet([B("Major 7 / 6 → "),T("Major (Ionian); Lydian for openness.")]),
      bullet([B("Minor 7 → "),T("Dorian — your default minor sound.")]),
      bullet([B("Dominant 7 → "),T("Mixolydian when relaxed; altered or half-whole diminished when it resolves and you want tension; Lydian dominant when it does not resolve.")]),
      bullet([B("Half-diminished → "),T("Locrian.")]),
      bullet([B("Blues over everything → "),T("minor pentatonic + the ♭5 blue note.")]),

      H2("Axiom 5 — Rhythm and phrasing outrank note choice"),
      P([B("Swing the eighths"),T(" (long–short) at medium and slow tempos; they naturally even out as the tempo climbs. "),B("Phrase like speech"),T(" and leave space — silence is a note. Start and end phrases "),B("off the barline"),T(". Aim each phrase at a "),B("target note"),T(" on a strong beat, reached by "),B("enclosure"),T(" — surrounding the target from above and below before landing. The enclosure is the single most characteristic bebop device.")]),

      H2("Axiom 6 — Tension and release is the whole game"),
      P([T("Create tension (upper extensions, alterations, chromaticism, register, rhythmic displacement, playing “outside”), then "),B("resolve it"),T(" to consonance. A solo with no tension is bland; one with no release is noise. Shape phrases and whole solos as arcs: begin simple, raise intensity to a peak, then resolve.")]),

      H2("Axiom 7 — Vocabulary is learned by ear, then transposed"),
      P([T("Fluency comes from "),B("transcribing"),T(" phrases from recordings, internalizing them, and moving them through all twelve keys. Store licks as degree + rhythm so they are key- and instrument-independent. "),B("Sing every idea before you play it"),T(" — if you can't hear it, you can't really play it.")]),

      H2("Axiom 8 — Know the form, always"),
      P([T("Keep your place in the tune: song form (AABA, ABAC, 12-bar blues), the current chord, and the target of the current phrase. Losing the form is the cardinal sin; landing chord tones on downbeats keeps you anchored even when ideas run dry.")]),

      hr(),
      H2("Quick reference — chords, formulas, and go-to scales"),
      refTable(),
      new Paragraph({spacing:{after:120}}),
      callout("If you learn one thing, learn the ii–V–I guide-tone line: the 7th of each chord falls to the 3rd of the next. Play only those two guide-tone voices and you already sound like a jazz musician."),

      new Paragraph({children:[new PageBreak()]}),

      // ================= PART II =================
      new Paragraph({spacing:{after:40},children:[new TextRun({text:"PART II",color:MUTE,bold:true,size:22,characterSpacing:80})]}),
      H1("A Beginner's Guide, Step by Step"),

      H2("1. What improvisation really is"),
      P([T("Improvising is composing a melody in real time that fits the chords going by and sounds like a human speaking — with phrases, pauses, questions and answers. It is "),B("not"),T(" running scales up and down. Scales are the alphabet; music is the sentences. Three things decide whether a note sounds good: the note itself, the beat it lands on, and the rhythm and space around it. Beginners obsess over the first and ignore the other two — reverse that.")]),
      callout("The single most useful habit: land a chord tone — ideally the 3rd or 7th — on a strong beat, and leave space after your phrases."),

      H2("2. The building blocks"),
      P([T("Number the major scale 1–7. In C: C(1) D(2) E(3) F(4) G(5) A(6) B(7). A seventh chord stacks 1·3·5·7 — the chord tones, your safe resting places. The 3rd and 7th are the guide tones that give the chord its character. Notes above the 7th (9, 11, 13 and their alterations) are color and tension that want to resolve.")]),

      H2("3. The ii–V–I: the sentence that unlocks jazz"),
      P([T("In C major: "),B("Dm7 → G7 → Cmaj7"),T(" — setup, tension, resolution, the backbone of hundreds of standards. The trick that makes it sound like jazz for almost no effort is the "),B("guide-tone line"),T(". Each chord's 3rd and 7th form two voices that barely move:")]),
      P([B("Voice 1: C → B → B."),T("  C is the ♭7 of Dm7; it falls a half step to B, the 3rd of G7; B is also the 7th of Cmaj7, so it simply stays put.")],{spacing:{after:60}}),
      P([B("Voice 2: F → F → E."),T("  F is the ♭3 of Dm7; it stays put to become the ♭7 of G7; then it falls a half step to E, the 3rd of Cmaj7.")],{spacing:{after:120}}),
      P([T("Notice the pattern: each chord's 7th slides down a half step to become the next chord's 3rd, and the two voices trade roles as they go. Just holding and resolving those notes sounds like a pro. First exercise: loop the progression and play only those guide tones as long notes, then start decorating them.")]),

      H2("4. Aim before you shoot: target notes and enclosures"),
      P([T("Great players aim each phrase at a specific note that lands on a strong beat — usually the 3rd or 7th. The notes before it are just the approach. The classic approach is the "),B("enclosure"),T(": to hit E (the 3rd of Cmaj7) on beat 1, play F (a half step above) and D♯/E♭ (a half step below) just before, then resolve to E. That squeeze-and-arrival is the most bebop sound you can learn.")]),

      H2("5. Scales, chosen by chord quality"),
      P([T("Learn five first, always connected to a chord: "),B("Major (Ionian)"),T(" over major 7; "),B("Dorian"),T(" over minor 7; "),B("Mixolydian"),T(" over relaxed dominants; "),B("minor pentatonic"),T(" (five safe notes); and the "),B("blues scale"),T(" (pentatonic + ♭5). Don't ask “what scale?” first — ask “what chord tone am I aiming at, on what beat?” and use the scale to connect your targets.")]),

      H2("6. Rhythm, swing, and space — the part that matters most"),
      P([B("Swing the eighth notes"),T(" (long–short, triplet lilt) at medium and slow tempos; they even out as the tempo climbs. "),B("Leave space"),T(" — play a phrase, then rest; it is the number-one fix for beginners. "),B("Phrase off the barline"),T(" — not every phrase starts on beat 1. And "),B("converse"),T(": make a statement, leave a gap, answer it.")]),

      H2("7. Tension and release, and the blues"),
      P([T("Every good phrase and whole solo is an arc of tension and release: build intensity to a peak, then resolve. The "),B("12-bar blues"),T(" is the best playground — a blues in F over three dominant chords lets you play one blues scale the whole way and sound good instantly, then add each chord's guide tones for the jazz layer. That blend of feel and harmony is jazz improvisation in miniature.")]),

      H2("8. A practice routine that works"),
      bullet([B("Sing first — "),T("if you can't hear it, you can't play it.")]),
      bullet([B("Small loops, slow, with a metronome — "),T("automate one tiny thing, then speed up.")]),
      bullet([B("All twelve keys — "),T("everything is stored as intervals, so move it around.")]),
      bullet([B("Transcribe by ear — "),T("learn short phrases off records you love.")]),
      bullet([B("Play with tracks and people — "),T("improvising is a conversation; have it.")]),
      bullet([B("Keep your place in the form — "),T("if lost, land a chord tone on a downbeat and you're anchored.")]),

      new Paragraph({children:[new PageBreak()]}),

      // ================= PART III =================
      new Paragraph({spacing:{after:40},children:[new TextRun({text:"PART III",color:MUTE,bold:true,size:22,characterSpacing:80})]}),
      H1("Intermediate Techniques"),
      P([T("Once the beginner path feels natural — guide tones, the ii–V–I, a handful of scales, and phrasing with space — these four techniques are the bridge to sounding like you are playing "),B("through"),T(" the changes rather than merely over them.")]),

      H2("Extensions and the “avoid” notes"),
      P([T("Go above the 7th. The "),B("9th, 11th, and 13th"),T(" are the sweet upper colors, but not every extension fits every chord. On a major or dominant chord the "),B("natural 11 clashes with the 3rd"),T(" (it sits a half step above it), so it is an “avoid note” — use it only as a quick passing tone, or raise it to a "),B("♯11"),T(" for the bright Lydian sound. On minor 7 chords, by contrast, all of 9, 11 and 13 are fair game — that is the Dorian sound. The working rule stays the same: chord tones on strong beats, extensions on weak beats, and let any tension slide by step into the nearest chord tone.")]),

      H2("Bebop scales — keeping chord tones on the beat"),
      P([T("A plain seven-note scale has a rhythmic problem at speed: run straight eighth notes and your chord tones drift onto the weak beats. The fix is the "),B("bebop scale"),T(" — add one chromatic passing tone to make an eight-note scale so that, starting on a chord tone, every downbeat lands on a chord tone. The "),B("dominant bebop scale"),T(" adds a passing tone between the ♭7 and the root (G Mixolydian becomes G A B C D E F F♯ G); the "),B("major bebop scale"),T(" adds one between the 5 and 6. This is the rhythmic secret behind classic bebop lines.")]),

      H2("Tritone substitution"),
      P([T("Any dominant 7 chord can be replaced by the dominant a "),B("tritone away"),T(", because the two contain the "),B("same tritone"),T(" — the 3rd/7th tension pair — with the roles swapped. The 3rd of G7 (B) is the 7th of D♭7, and the 7th of G7 (F) is the 3rd of D♭7: the same two notes, the same pull, a new bass note under them. Substituting "),B("D♭7 for G7"),T(" in a ii–V–I therefore resolves just as strongly but gives a smooth chromatic bass line (D → D♭ → C) and a hipper cadence; over the substitute, play its Lydian-dominant scale — which contains exactly the same seven notes as the altered scale of the chord it replaces, which is why it still resolves.")]),

      H2("Digital patterns and 3-to-9 arpeggios"),
      P([T("Two workhorses for outlining chords melodically. The "),B("1-2-3-5 digital pattern"),T(" spells each chord (over Dm7: D E F A). The "),B("3-to-9 arpeggio"),T(" starts on the 3rd and stacks 3-5-7-9, automatically catching both guide tones plus one sweet extension (over Dm7: F A C E). Run either through a full ii–V–I and you will sound like you are naming every chord as it passes.")]),
      callout("Intermediate mantra: play through the changes, not over them. Outline each chord with its own guide tones and a digital pattern, then decorate with the right extensions."),

      new Paragraph({children:[new PageBreak()]}),

      // ================= PART IV =================
      new Paragraph({spacing:{after:40},children:[new TextRun({text:"PART IV",color:MUTE,bold:true,size:22,characterSpacing:80})]}),
      H1("Advanced Concepts"),
      P([T("The advanced language is about "),B("controlled tension"),T(" and "),B("intention"),T(": richer dominant colors, reharmonization, deliberate dissonance that resolves, and — above all — rhythm and storytelling.")]),

      H2("Altered dominants and upper-structure triads"),
      P([T("When a V7 resolves down a fifth, you can pile on tension with the "),B("altered scale"),T(" (the seventh mode of melodic minor), which supplies ♭9, ♯9, ♯11 and ♭13 — every note that pulls into the I. "),B("Upper-structure triads"),T(" are a clean way to grab a whole set of tensions in one familiar shape: a simple major triad played over the dominant's root. For "),B("altered"),T(" colors over G7, play a "),B("D♭ major triad"),T(" (D♭–F–A♭ = ♯11, ♭7, ♭9) — every note of it lives inside the G altered scale. An "),B("E♭ major triad"),T(" (E♭–G–B♭ = ♭13, 1, ♯9) works the same way."),],{spacing:{after:120}}),
      P([T("The same trick has a softer, "),B("un"),T("altered version. Still over G7, an "),B("A major triad"),T(" (A–C♯–E) gives the 9, ♯11 and 13 — the bright "),B("Lydian-dominant"),T(" sound, which is what you want when a dominant is "),B("not"),T(" resolving, as on the D7 in “Take the A Train.” Its top note E is also the 3rd of Cmaj7, so if the chord does move to I you can hold that note straight through as a common tone.")]),

      H2("Reharmonization and Coltrane changes"),
      P([T("Once you hear the changes, you can bend them. The "),B("backdoor ii–V"),T(" (a bVII7 resolving up to I), "),B("tritone subs"),T(", and inserted secondary dominants all recolor a progression. The most famous advanced device is "),B("Coltrane's major-third cycle"),T(" (the “Giant Steps” matrix): key centers moving by major thirds (for example C → A♭ → E), each approached by its own V7. Practicing clean 1-2-3-5 cells through those fast-moving keys is a rite of passage.")]),

      H2("Playing outside: side-slipping and superimposition"),
      P([T("Deliberate “wrong” notes, resolved on purpose. "),B("Side-slipping"),T(" plays a phrase a half step away from the key and then snaps it back — pure tension and release. "),B("Superimposition"),T(" lays a familiar shape from a different root over the chord: a "),B("D major pentatonic over Cmaj7"),T(", for instance, yields 9, 3, ♯11, 13 and 7 — a shimmering Lydian color — or an outside pentatonic over a V7 for altered tension. The rule that keeps it musical: always resolve the outside idea back inside, on a strong beat.")]),

      H2("Motivic development and telling a story"),
      P([T("The difference between running vocabulary and actually saying something. Take a short "),B("motif"),T(" — three or four notes with a distinctive rhythm — and develop it: transpose it to fit each chord, invert it, augment or shrink its rhythm, sequence it, displace it across the barline. A whole solo built from one evolving idea has a shape and memorability that scale-running never achieves. This is where technique becomes music.")]),

      H2("Rhythmic mastery: displacement and playing over the barline"),
      P([T("At the highest level, rhythm is the frontier. "),B("Metric displacement"),T(" repeats a phrase starting on a different beat so it floats against the pulse; "),B("polyrhythmic groupings"),T(" (playing in threes over 4/4) create rolling tension; and phrasing across the barline in long, asymmetric lines makes time feel elastic. Combine these with the harmonic tools above and you have the full language.")]),
      callout("Advanced mantra: every tension is a promise to resolve. Make the “outside” and the displacement deliberate, then land it — on purpose, on a strong beat."),

      new Paragraph({children:[new PageBreak()]}),

      // ================= PART V =================
      new Paragraph({spacing:{after:40},children:[new TextRun({text:"PART V",color:MUTE,bold:true,size:22,characterSpacing:80})]}),
      H1("Listening: The Masters & The Standards"),
      P([T("Everything in Parts I–IV is only a map. The territory is the "),B("sound"),T(", and you reach it the way every jazz musician has: by listening deeply and transcribing by ear. Live with the players below, sing along, then learn one phrase at a time. Then learn the standards — the shared repertoire every jazz musician is expected to know — so you can sit in anywhere.")]),

      H2("Masters to study"),
      P([T("A starting pantheon. Pick one player a week; sing their lines before you try to name them.")]),
      mastersTable(),
      new Paragraph({spacing:{after:80}}),
      P([new TextRun({text:"And so many more to explore: Ella Fitzgerald, Duke Ellington, Count Basie, Sarah Vaughan, Bill Evans, Sonny Rollins, Wes Montgomery, Herbie Hancock, Clifford Brown, Freddie Hubbard, Oscar Peterson, and Wynton Marsalis.",italics:true,color:MUTE,size:19})]),

      new Paragraph({children:[new PageBreak()]}),
      H2("Essential standards"),
      P([T("Learn the melody and the changes, then improvise. In the companion app, several of these load straight into the play-along engine so you can solo over them immediately (So What and Rhythm Changes run their full 32-bar AABA forms).")]),
      standardsTable(),
      new Paragraph({spacing:{after:120}}),
      callout("Learning by ear is the whole tradition. A transcribed phrase you can sing is worth more than a page of scales you can only read."),

      new Paragraph({children:[new PageBreak()]}),
      H1("The path at a glance"),
      P([T("The whole app, and this document, follow one continuous road. Start where you are and climb.")]),
      levelTable(),

      new Paragraph({spacing:{before:280}}),
      hr(),
      H1("The whole method in one paragraph"),
      P([new TextRun({text:"Improvise a melody that fits the chords and sounds like speech. Aim each phrase at a chord tone — especially the 3rd or 7th — landing on a strong beat, and use the 9th, 11th, 13th and alterations as color that resolves. Navigate tunes by the ii–V–I and its guide-tone line, letting the 7th of each chord fall to the 3rd of the next. Choose scales by chord quality (major→Ionian, minor7→Dorian, dominant→Mixolydian, plus the pentatonic and blues scales). Swing your eighths, leave space, phrase off the barline, and reach targets with enclosures. Shape tension and release on purpose, keep your place in the form, and learn the language by ear — singing every idea before you play it.",italics:true,color:INK,size:21})]),
      new Paragraph({spacing:{before:200},alignment:AlignmentType.CENTER,children:[new TextRun({text:"— Companion app: The Jazz Improvisation Lab (interactive theory, drills, ear training, and a play-along engine) —",color:MUTE,size:18,italics:true})]}),
    ]
  }]
});

Packer.toBuffer(doc).then(buf=>{ fs.writeFileSync('/root/jazz/Jazz-Improvisation-Rules-and-Guide.docx',buf); console.log('WROTE docx',buf.length,'bytes'); });
