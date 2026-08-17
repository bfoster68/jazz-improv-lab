# CLAUDE.md — project context for Claude Code

## What this is

A jazz improvisation teaching system, delivered three ways from one shared body of
musical knowledge:

| Path | What it is | Status |
|---|---|---|
| `web/index.html` | The complete app: Learn, Theory Lab, Drills, Ear Training, Play-Along, Masters & Standards. Single self-contained file, works offline. | Done, audited twice |
| `rust/` | A native Rust port. Tested theory + audio engines, plus a Bevy front end. | Engine + UI done; 110 tests, links clean |
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
cargo test -p jazz-theory -p jazz-audio   # 149 tests, ~2s, no audio device needed
cargo test -p jazz-audio --features soundfont   # includes the SoundFont paths
cargo check --workspace                  # includes the Bevy app
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
- **Never lock across the audio boundary.** The audio callback has a hard deadline,
  so it owns the mixer outright and the UI talks to it through
  `jazz_audio::stream` — a command channel plus an atomic play head. An earlier
  design shared a `Mutex`; the UI held it for a whole frame draw while the callback
  `try_lock`ed and emitted silence on failure, which is what made playback stutter.
  `stream::tests::a_whole_form_plays_without_dropouts` guards the regression.
- **Follow the device's sample rate.** `AudioCore::set_sample_rate` is called with
  the real rate when the stream opens; assuming 44.1 kHz on a 48 kHz device plays
  everything about a tone sharp.
- **Audio is testable without a sound card.** `jazz-audio` renders to buffers, so
  every audio test runs offline. Keep it that way — do not add a backend dependency
  to that crate.
- **egui layout gotcha.** `vertical_centered` claims all remaining width, so a chip
  built with it inside `horizontal_wrapped` swallows the row and hides every item
  after the first. Running the app is what exposed this; use `note_chip`, which
  sets an explicit width.
- **Notation is ASCII internally, glyphs at the edge.** The theory engine always
  uses `b` and `#`; ♭ and ♯ are substituted only at display time (`fmtMusic` in the
  web app). Never feed glyphs back into the theory layer.


## Where things stand

Last verified state: **149 tests pass** (95 theory + 54 audio), `cargo check
--workspace` is clean with no warnings, and `jazz-app` builds, links, and has
been run and heard on real hardware.

The progression catalogue holds 23 forms including seven full standards added
as changes-only charts (All the Things You Are, All of Me, Fly Me to the Moon,
Misty, Tune Up, Solar, Cantaloupe Island — the last two credited to Chuck Wayne
and Eddie Vinson, who wrote them, with the Miles Davis connection in the
teaching note). The Play-Along tab renders a lead-sheet chart: four-bar systems
with barlines, gold section letters from `Progression::sections`, fake-book `%`
repeat marks, a meter label, and a gold highlight that follows the sounding bar.

Meter is a property of the tune (`Progression::meter`, `Meter::Four`/`Three`),
not a user setting; `Settings::bar_len()` is the only place beats-per-bar meets
tempo. 3/4 gets its own arranger in `plan_bar_three` — a waltz bass rooting
beat one with a lead note on three, the waltz ride with the skip on the and of
two, comping over six eighth slots — exercised by "ii-V-I jazz waltz" and
"Jazz-waltz Blues in F". Split bars and the two-feel remain 4/4-only ideas.
Adding 6/4 or 6/8 later means a new `Meter` variant plus its own arranger arm.

The bass has five selectable lines (`BassStyle`, the "Line" dropdown on the
bass row, `--bass-style` in render-wav), in both meters: Walking (keeps its
random drop-into-two bars), Two-beat (roots and fifths on one and three),
Bebop (chromatic passing tones, pushed eighths — its register window is wider
than `clamp_bass` on purpose, so approach notes keep their contour), Pedal
point (the form's opening root held under the changes) and Syncopated (a
deliberately deterministic off-beat pattern anticipating the next chord — a
groove that varies is not a groove). Tests pin each style's defining property.

Masters and Standards are separate tabs. Each `Master` carries a biography —
born/died (died empty for the living: Severinsen, Sandoval), a `bio`
introduction and a chronological `life` timeline of (year, event) pairs mixing
the personal and the musical; clicking a gallery card opens the page. Tests
guard the data the way the composer credits are guarded: timelines must move
forward through plausible years, bios must be substantive, and
`the_living_are_not_declared_dead` means exactly what it says. Biographical
facts were written from standard jazz history — check before "correcting".
The waltz feel is a first pass the user expects to tune by ear; the knobs are
in `plan_bar_three` — the beat-three lead-note chance (0.65), the ride accent
gains, and the comping slot pools.

Since then, in this order: fixed invisible UI text (bevy_egui needs the
`default_fonts` feature); anchored the transport to an `epoch` so restarting or
changing progressions cannot dump every bar at once; added Piano, Guitar, Sax,
Trombone and Trumpet with per-part instrument selection and an optional horn
section; bundled FluidR3 GM (fetched by `crates/jazz-app/assets/fetch-soundfont.ps1`,
gitignored) so all pitched parts play from real samples while drums stay synth;
made the mixer clock f64 (the f32 clock warped tempo after ~4 minutes and froze
near 512 s); routed the SoundFont through the master volume; and made live mix
changes re-queue from the next bar without silencing the current one.

Recently done, in order:
1. Ported the musical core to Rust (`jazz-theory`, `jazz-audio`) with tests that
   check musical truth, not just code behaviour.
2. Brought the Bevy app to five tabs: Play-Along, Theory, Drills, Ear Training,
   Masters & Standards.
3. Ran the app under a virtual display, which exposed an egui layout bug that
   compilation could not have caught (see the layout gotcha above).
4. Rewrote the audio boundary to be lock-free after playback stuttered, and made
   the mixer follow the device sample rate.
5. Added optional SoundFont playback, and trimmed Bevy from 409 crates to 258.

### Next step: wire the Learn tab

The content is written and tested in `jazz_theory::curriculum` but nothing renders
it yet. `LESSONS` is a flat slice already in teaching order, each carrying:

- `level` — `Beginner` / `Intermediate` / `Advanced`
- `title`, `body` (paragraphs separated by a blank line)
- `try_it: Option<TryIt>` — `Drill(id)`, `Progression(id)` or `Scale(name)`

The work is: add a `Tab::Learn`, put a level filter across the top (the browser
build defaults to Beginner), render each lesson as a card, and turn `try_it` into a
button that switches tab and loads the thing it names. `listening_ui` already does
exactly that pattern for standards — copy its `load` approach, which defers the
mutation until after the loop to avoid borrowing `state` twice.

Every `try_it` is guaranteed to resolve; a test enforces it.

### Open questions worth checking with the user

- Does the audio dropout fix actually sound right on real hardware? It is verified
  by test, but nobody has heard it yet.
- The wasm build has never been compiled. `itch/build-wasm.ps1` is written but
  unproven; expect a `wasm-bindgen-cli` version mismatch to be the first hurdle.

## Known gaps

- The Learn curriculum is written and tested in `jazz_theory::curriculum` but not
  yet wired into a Bevy tab. `LESSONS` carries level, title, body and an optional
  `TryIt` cross-reference; rendering it is the next step.
- Both builds can use real samples: the web app streams FluidR3 from a CDN, the
  Rust app loads any GM `.sf2` from `crates/jazz-app/assets/soundfont.sf2` via
  `rustysynth` (feature `soundfont`). Neither bundles one.
- Rust has no reverb yet; the browser build has a convolution reverb.
- Bevy is built with `default-features = false`. Adding a feature back is fine, but
  check the crate count and wasm size first — the defaults pull in glTF, gamepads,
  Bevy's own audio stack and every image codec, none of which this app uses.
