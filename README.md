# The Jazz Improvisation Lab

Learn to improvise on **any** instrument. Everything here is expressed in note
names, scale degrees and intervals — never fingerings, frets or key positions — so
it works equally for a saxophonist, pianist, guitarist, bassist, trumpeter or
singer.

Three pieces, one shared body of musical knowledge:

- **`web/index.html`** — the complete app in a single self-contained file. Open it
  in any browser; it works offline.
- **`rust/`** — a native port. A tested music-theory engine, a synthesised rhythm
  section, and a Bevy front end.
- **`docs/`** — a five-part reference document, plus the reusable prompt that
  codifies the rules it was generated from.

---

## The web app

Open `web/index.html`. No build step, no server, no dependencies.

Six sections, each running beginner → intermediate → advanced:

**Learn** — the path from "what improvisation actually is" through guide tones and
target notes to altered dominants, reharmonization, superimposition and motivic
development, filtered by level.

**Theory Lab** — every scale and chord at every root, with correct spelling and
honest degree labels. Click any note to hear it.

**Practice Drills** — fourteen generated exercises in all twelve keys: guide-tone
lines, enclosures, bebop scales, 1-2-3-5 digital patterns, 3-to-9 arpeggios,
tritone subs, upper-structure triads, altered lines, pentatonic superimposition,
the Coltrane cycle and motivic development.

**Ear Training** — intervals, chord qualities, extended and altered chords, and
progressions including tritone-sub and backdoor ii-Vs.

**Play-Along** — a live rhythm section: walking bass, comping piano and brushed
drums, with the comping rhythm, bass line and drum accents changing every bar so it
never loops mechanically. Fourteen progressions including *So What*, *Blue Bossa*,
*Take the A Train* and full *Rhythm Changes*.

**Masters & Standards** — who to listen to and what to listen for, plus a starter
repertoire that loads straight into the play-along.

Optional: tick **High-quality sounds** to stream real recorded piano, bass and
vibraphone; it falls back to the built-in synth instantly when offline.

## The Rust workspace

```bash
cd rust
cargo test --workspace        # 92 tests, all offline
```

| Crate | What it does |
|---|---|
| `jazz-theory` | Note spelling by letter arithmetic, scales, chords, progressions, generated drills, ear-training material. No dependencies. |
| `jazz-audio` | FM electric piano, upright bass, vibraphone, an inharmonic ride cymbal, and a swing scheduler. No audio backend — it renders buffers, so every test runs without a sound card. |
| `jazz-app` | Bevy + egui front end with live `cpal` output. |

Hear the engine without a UI:

```bash
cargo run -p jazz-audio --bin render-wav -- --list
cargo run -p jazz-audio --bin render-wav -- bluebossa --tempo 150
cargo run -p jazz-audio --bin render-wav -- rhythmfull --tempo 220 --busy --choruses 2
```

Run the native app:

```bash
cargo run -p jazz-app --release
```

On Linux the app needs ALSA headers (`libasound2-dev`); the `render-wav` binary and
the test suite need nothing at all.

## The document

`docs/Jazz-Improvisation-Rules-and-Guide.docx` — five parts: the rules codified as
eight axioms, a beginner's walk-through, intermediate techniques, advanced
concepts, and a listening guide to the masters and the standards.

`docs/codification-prompt.md` is the reusable spec those rules came from. Feed it to
a model to generate new lessons, licks or drills in the same voice.

```bash
cd docs && npm install && npm run build
```

## Testing

The tests encode *musical* correctness, not just code behaviour — that an enclosure
really surrounds its target, that a walking bass never repeats a note within a bar,
that no drill leaps more than an octave, that no dominant resolving to a minor
chord is left suggesting plain Mixolydian.

```bash
cd rust && cargo test --workspace     # Rust
node tests-web/verify7.js             # browser theory + spelling + contour
node tests-web/verify_audio.js        # offline audio render and level checks
```

The browser suites need Playwright and drive a real Chromium instance.

## Licence

MIT OR Apache-2.0. Standards are included as chord changes only — the harmony a
fake book teaches — not as melodies.

## Verified state

- `jazz-theory` — 60 tests
- `jazz-audio` — 30 tests + 2 doc tests
- `jazz-app` — compiles clean against Bevy 0.19 / bevy_egui 0.41 / egui 0.35

Note that `cargo test --workspace` will link the whole Bevy binary and take a
while; `cargo test -p jazz-theory -p jazz-audio` runs the musical test suite in
about a second.
