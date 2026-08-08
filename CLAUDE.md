# CLAUDE.md — project context for Claude Code

## What this is

A jazz improvisation teaching system, delivered three ways from one shared body of
musical knowledge:

| Path | What it is | Status |
|---|---|---|
| `web/index.html` | The complete app: Learn, Theory Lab, Drills, Ear Training, Play-Along, Masters & Standards. Single self-contained file, works offline. | Done, audited twice |
| `rust/` | A native Rust port. Tested theory + audio engines, plus a Bevy front end. | Engine done & tested; UI is the current frontier |
| `docs/` | A five-part Word document and the reusable "codification prompt" that generated it. | Done |

**Everything is instrument-agnostic.** Notes, degrees and intervals only — never
fingerings, frets or key positions. It has to work for a saxophonist, pianist,
guitarist, bassist, trumpeter or singer alike. Do not add instrument-specific
diagrams without being asked.

## The three rules that hold the whole thing together

These were learned the hard way, from two audit rounds that found real bugs. Break
them and you will reintroduce those bugs.

**1. Spell notes by letter arithmetic, never by lookup table.**
The `#11` of G is `C#`, never `Db`. A lookup table gets this wrong and prints an
"A major triad" as `A Db E`. See `Note::spell` (Rust) / `spellDeg` (JS): take the
root's letter, step up the diatonic degree, then work out the accidental from the
pitch. If the result needs a double accidental, fall back to a plain name — a
beginner reading `B𝄫` learns nothing.

**2. Degree labels are data, not derived.**
The same pitch is the `b5` of a half-diminished chord and the `#11` of a dominant.
Only context decides, so every scale and chord carries its own labels. Deriving
them from semitones produced `Bm7b5 = B D F(#11) A`, which is simply wrong.

**3. Keep register, not just pitch class.**
A drill that says "a half step below the target" must *sound* a half step below it.
Storing pitch classes alone silently destroys contour: the guide-tone drill turned
a descending half step into an ascending major 7th. Drills store absolute semitone
offsets from the key root. `drill::tests::drills_have_singable_contour` guards this.

## Musical decisions worth knowing

- **Suggested scales are context-sensitive.** A V7 resolving to a minor tonic takes
  the altered scale, not plain Mixolydian (Mixolydian's natural 13 fights the minor
  key). A dominant that does *not* resolve — the D7 in *Take the A Train* — takes
  Lydian dominant. This is why `Slot` carries an optional `scale` override; chord
  quality alone cannot express it. `no_dominant_resolving_to_minor_is_left_on_plain_mixolydian`
  enforces it.
- **Comping follows the altered bars.** When a bar is altered, the piano voices the
  b9/b13 instead of the natural 9/5, so it never fights the tensions the app is
  telling the student to play.
- **The ♭II is always spelled flat.** `Gb7` in F, never `F#7` — its root falls
  chromatically into the I.
- **Key centres use the conventional spelling.** The Giant Steps cycle in B reads
  B / G / E♭, not B / G / D♯.
- **Standards ship changes, not melodies.** Chord progressions only, the way a fake
  book teaches. Do not add copyrighted melodies.

## Layout

```
web/index.html            the browser app (single file — inline everything)
rust/
  crates/jazz-theory/     notes, spelling, scales, chords, progressions, drills, ear training
  crates/jazz-audio/      instrument models, swing scheduler, mixer, WAV renderer
  crates/jazz-app/        Bevy + egui front end
docs/                     the Word document, its build script, the codification prompt
tests-web/                Playwright suites for the browser app
```

## Commands

```bash
# Rust
cd rust
cargo test --workspace                 # 92 tests, all offline, no audio device needed
cargo run -p jazz-audio --bin render-wav -- --list
cargo run -p jazz-audio --bin render-wav -- bluebossa --tempo 150   # hear the engine
cargo run -p jazz-app --release        # the native app

# Web (needs playwright installed)
node tests-web/verify7.js              # theory + spelling + contour
node tests-web/verify_audio.js         # offline audio render + level checks
node tests-web/verify_glyph.js         # accidental rendering, prose safety

# Document
cd docs && node build-docx.js
```

## Working on this

- **The tests are the specification.** They encode musical correctness, not just
  code behaviour: that an enclosure really surrounds its target, that a walking
  bass never repeats a note within a bar, that no drill leaps more than an octave.
  If a test looks odd, read the reason before changing it.
- **Verify by computing, not by eye.** Both audit rounds found bugs that looked
  fine in source and were only exposed by extracting actual rendered output. For
  the web app, drive it with Playwright and read `drSequence`; for Rust, assert on
  real note names.
- **Audio is testable without a sound card.** `jazz-audio` renders to buffers, so
  every audio test runs offline. Keep it that way — do not add a backend dependency
  to that crate.
- **Notation is ASCII internally, glyphs at the edge.** The theory engine always
  uses `b` and `#`; ♭ and ♯ are substituted only at display time (`fmtMusic` in the
  web app). Never feed glyphs back into the theory layer.

## Known gaps

- The Bevy app covers Play-Along, Theory and Drills; Ear Training and the Masters
  reference are web-only so far.
- The web app's optional high-quality sounds download FluidR3 samples from a CDN;
  the Rust side is synthesis-only.
- Rust has no soundfont support and no reverb yet (the browser build has a
  convolution reverb).
