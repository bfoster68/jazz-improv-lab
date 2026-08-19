//! The rhythm section: a swing scheduler that turns a progression into events.
//!
//! The musical goal is that it should never loop mechanically. Every bar picks a
//! fresh comping rhythm, re-walks the bass, and re-rolls the drum accents, with
//! occasional two-feel bars and a fill at the top of the form.

use jazz_theory::harmony::Quality;
use jazz_theory::note::{midi_for, pc};
use jazz_theory::progression::{Bar, Meter, Slot, ALTERED};

pub use crate::voice::Instrument;

/// A note the engine wants played at a given moment.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Event {
    /// Seconds from the start of the bar.
    pub offset: f32,
    pub midi: i32,
    pub gain: f32,
    pub dur: f32,
    pub instrument: Instrument,
}

/// A percussion hit.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PercEvent {
    pub offset: f32,
    pub kind: PercRole,
    pub gain: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PercRole {
    Ride,
    RideSkip,
    HiHat,
    Brush,
    Kick,
}

/// How busy the comping should be.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Density {
    Sparse,
    Medium,
    Busy,
}

/// How the bass builds its line.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BassStyle {
    /// Quarter notes through chord tones with approach notes — the backbone.
    Walking,
    /// Traditional two-beat: roots and fifths on one and three.
    TwoBeat,
    /// Chromatic passing tones and pushed eighths, after the bebop melodists.
    Bebop,
    /// One note — the form's opening root — held under the moving changes,
    /// trading harmonic motion for tension and release.
    Pedal,
    /// A locked off-beat pattern that anticipates the next chord. Deliberately
    /// deterministic: a groove that varies is not a groove.
    Syncopated,
}

impl BassStyle {
    pub const ALL: [BassStyle; 5] = [
        BassStyle::Walking,
        BassStyle::TwoBeat,
        BassStyle::Bebop,
        BassStyle::Pedal,
        BassStyle::Syncopated,
    ];

    pub fn label(self) -> &'static str {
        match self {
            BassStyle::Walking => "Walking",
            BassStyle::TwoBeat => "Two-beat",
            BassStyle::Bebop => "Bebop",
            BassStyle::Pedal => "Pedal point",
            BassStyle::Syncopated => "Syncopated",
        }
    }
}

/// How the comping instrument places its chords.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompStyle {
    /// Varied syncopated hits from the density pools — the default pianist.
    /// The Feel (density) setting shapes this style and Guide tones; the
    /// fixed figures below ignore it.
    Rhythmic,
    /// The classic figure: a long chord on one, a push on the and of two.
    Charleston,
    /// Soft quarter-note chords on every beat, rhythm-guitar style.
    FourToBar,
    /// One voicing held through the bar.
    Sustained,
    /// The density-pool rhythms voiced with only the 3rd and 7th — the
    /// curriculum's guide tones, comped.
    GuideTones,
}

impl CompStyle {
    pub const ALL: [CompStyle; 5] = [
        CompStyle::Rhythmic,
        CompStyle::Charleston,
        CompStyle::FourToBar,
        CompStyle::Sustained,
        CompStyle::GuideTones,
    ];

    pub fn label(self) -> &'static str {
        match self {
            CompStyle::Rhythmic => "Rhythmic",
            CompStyle::Charleston => "Charleston",
            CompStyle::FourToBar => "Four to the bar",
            CompStyle::Sustained => "Sustained",
            CompStyle::GuideTones => "Guide tones",
        }
    }
}

/// What the three-piece horn section plays. The section is always trombone on
/// the bottom, tenor sax in the middle and trumpet on top — these styles choose
/// the figure. Backgrounds are *arranged* parts, so every style is
/// deterministic: the same bar of the same form always plays the same figure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HornStyle {
    /// Held section chords that phrase like players with lungs: two bars on,
    /// two bars off, the second bar softer.
    Pads,
    /// A long chord on one, then the next change pushed on the and of four.
    RiffPush,
    /// Short stabs on the backbeats.
    RiffBackbeat,
    /// Silence for the soloist's call, then the section answers on beat three.
    RiffCall,
    /// Offbeat shout-chorus pushes on the ands of one and three.
    RiffShout,
}

impl HornStyle {
    pub const ALL: [HornStyle; 5] = [
        HornStyle::Pads,
        HornStyle::RiffPush,
        HornStyle::RiffBackbeat,
        HornStyle::RiffCall,
        HornStyle::RiffShout,
    ];

    pub fn label(self) -> &'static str {
        match self {
            HornStyle::Pads => "Pads",
            HornStyle::RiffPush => "Riff: push",
            HornStyle::RiffBackbeat => "Riff: backbeat",
            HornStyle::RiffCall => "Riff: call",
            HornStyle::RiffShout => "Riff: shout",
        }
    }
}

/// Spread a voicing across the section: trombone takes the bottom note down an
/// octave into its own register, the tenor sax the middle, the trumpet the top.
fn section_voices(voicing: &[i32]) -> [(Instrument, i32); 3] {
    let mut v = voicing.to_vec();
    v.sort_unstable();
    let bottom = v[0];
    let middle = v[v.len() / 2];
    let top = *v.last().unwrap();
    [
        (Instrument::Trombone, bottom - 12),
        (Instrument::Sax, middle),
        (Instrument::Trumpet, top),
    ]
}

/// Eighth-note slots in a bar: 0 = beat 1, 1 = the "and", 2 = beat 2 …
const SPARSE: &[&[usize]] = &[&[0], &[4], &[3], &[6], &[1], &[0, 4]];
const MEDIUM: &[&[usize]] = &[
    &[0, 3], &[1, 4], &[3, 6], &[2, 5], &[0, 4], &[3, 7], &[1, 5], &[0, 6], &[4, 7],
];
const BUSY: &[&[usize]] = &[
    &[0, 3, 5], &[1, 4, 6], &[0, 3, 6], &[2, 5, 7], &[1, 3, 6], &[0, 2, 5], &[3, 5, 7], &[0, 3, 5, 7],
];

fn pool(d: Density) -> &'static [&'static [usize]] {
    match d {
        Density::Sparse => SPARSE,
        Density::Medium => MEDIUM,
        Density::Busy => BUSY,
    }
}

/// Small deterministic RNG — reproducible bars make the engine testable.
#[derive(Debug, Clone, Copy)]
pub struct Rng(u64);

impl Rng {
    pub fn new(seed: u64) -> Self {
        Self(seed | 1)
    }
    #[inline]
    fn next_u32(&mut self) -> u32 {
        // xorshift64*
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        (x.wrapping_mul(0x2545F4914F6CDD1D) >> 32) as u32
    }
    /// Raw draw, for callers that need to feed randomness into a pure function.
    #[inline]
    pub fn next_u32_pub(&mut self) -> u32 {
        self.next_u32()
    }
    #[inline]
    pub fn next_f32(&mut self) -> f32 {
        self.next_u32() as f32 / u32::MAX as f32
    }
    #[inline]
    pub fn below(&mut self, n: usize) -> usize {
        (self.next_u32() as usize) % n.max(1)
    }
    #[inline]
    pub fn chance(&mut self, p: f32) -> bool {
        self.next_f32() < p
    }
}

/// Move `pitch_class` into the octave nearest `reference`.
pub fn nearest_octave(pitch_class: i32, reference: i32) -> i32 {
    let p = pc(pitch_class);
    let diff = (reference - p) as f32 / 12.0;
    p + 12 * diff.round() as i32
}

/// Keep a bass note in a playable upright register.
pub fn clamp_bass(mut m: i32) -> i32 {
    while m > 54 {
        m -= 12;
    }
    while m < 31 {
        m += 12;
    }
    m
}

/// Four quarter notes that outline the bar and approach the next root.
///
/// Beats 2 and 3 always take *different* chord tones, so the line never repeats
/// the same note twice in a row.
pub fn walking_bar(bar: &Bar, next_root: i32, rng: &mut Rng) -> [i32; 4] {
    let first = bar.chord_at(0);
    let ints = first.quality.ints();
    let b1 = clamp_bass(nearest_octave(first.root, 44));

    if let Some(second) = bar.second {
        let b2 = clamp_bass(nearest_octave(
            second.root + if rng.chance(0.5) { 1 } else { -1 },
            b1,
        ));
        let b3 = clamp_bass(nearest_octave(second.root, b2));
        let b4 = clamp_bass(nearest_octave(
            next_root + if rng.chance(0.5) { 1 } else { -1 },
            b3,
        ));
        return [b1, b2, b3, b4];
    }

    let third_first = rng.chance(0.5);
    let (d2, d3) = if third_first { (ints[1], ints[2]) } else { (ints[2], ints[1]) };
    let b2 = clamp_bass(nearest_octave(first.root + d2, b1));
    let b3 = clamp_bass(nearest_octave(first.root + d3, b2));
    let dir = if rng.chance(0.5) { 1 } else { -1 };
    let mut b4 = clamp_bass(nearest_octave(next_root + dir, b3));
    if b4 == b3 {
        b4 = clamp_bass(nearest_octave(next_root - dir, b3));
    }
    [b1, b2, b3, b4]
}

/// A rootless voicing with real voice-leading, the way pianists actually comp.
///
/// Two classic forms: A stacks 3-5-7-9 with the 3rd on the bottom, B stacks
/// 7-9-3-5 with the 7th on the bottom. Each chord takes whichever form, in
/// whichever octave, puts its bottom note nearest middle C. Neighbouring
/// chords then share and step their voices instead of transposing one frozen
/// shape — the textbook Dm7 - G7 - Cmaj7 hand movement falls straight out of
/// the rule — and the register stays anchored without every chord living on
/// the same shelf.
///
/// On an altered bar the natural 5 and 9 are replaced by the b13 and b9, so
/// the piano never fights the tensions the app is telling the student to play.
pub fn comp_voicing(slot: &Slot, rng: &mut Rng) -> Vec<i32> {
    let ints = slot.quality.ints();
    let altered = slot.scale == Some(ALTERED);
    let third = ints[1];
    let fifth = if altered { 8 } else { ints[2] };
    let seventh = ints[3];
    let ninth = if altered { 13 } else { 14 };
    // Interval stacks above the root for the two forms. No 9th over a
    // half-diminished chord, and on the minor family the B form carries its
    // 9th on top — tucked between b7 and b3 it lands a half step under the
    // third, the clash the voicing rules forbid.
    let minor = matches!(slot.quality, Quality::Min7 | Quality::MinTonic);
    let (form_a, form_b): (Vec<i32>, Vec<i32>) = if slot.quality == Quality::Min7b5 {
        (vec![third, fifth, seventh], vec![seventh - 12, third, fifth])
    } else if minor {
        (
            vec![third, fifth, seventh, ninth],
            vec![seventh - 12, third, fifth, ninth],
        )
    } else {
        (
            vec![third, fifth, seventh, ninth],
            vec![seventh - 12, ninth - 12, third, fifth],
        )
    };
    let root4 = midi_for(slot.root, 4);
    let mut best: (i32, Vec<i32>) = (i32::MAX, Vec::new());
    for stack in [&form_a, &form_b] {
        for oct in [-12, 0, 12] {
            let r = root4 + oct;
            let d = (r + stack[0] - 60).abs();
            if d < best.0 {
                best = (d, stack.iter().map(|i| r + i).collect());
            }
        }
    }
    let mut v = best.1;
    // Breathe: sometimes thin the voicing to three notes.
    if v.len() == 4 && rng.chance(0.35) {
        v.pop();
    }
    v
}

/// Just the guide tones — the 3rd and the 7th — placed by the same
/// nearest-to-middle-C rule as [`comp_voicing`], so even the thinnest texture
/// voice-leads: 3rd under 7th or 7th under 3rd, whichever connects.
pub fn comp_shells(slot: &Slot) -> Vec<i32> {
    let ints = slot.quality.ints();
    let (third, seventh) = (ints[1], ints[3]);
    let root4 = midi_for(slot.root, 4);
    let mut best: (i32, Vec<i32>) = (i32::MAX, Vec::new());
    for stack in [[third, seventh], [seventh - 12, third]] {
        for oct in [-12, 0, 12] {
            let r = root4 + oct;
            let d = (r + stack[0] - 60).abs();
            if d < best.0 {
                best = (d, stack.iter().map(|i| r + i).collect());
            }
        }
    }
    best.1
}

/// Everything that should sound during one bar.
#[derive(Debug, Clone, Default)]
pub struct BarPlan {
    pub notes: Vec<Event>,
    pub perc: Vec<PercEvent>,
    pub two_feel: bool,
}

/// The default swing ratio: a triplet feel. 0.5 is straight eighths.
pub const SWING_RATIO: f32 = 2.0 / 3.0;
/// The default density of the experimental idea generator's lines.
pub const IDEAS_BUSYNESS: f32 = 0.78;

/// Settings for the rhythm section.
///
/// Each pitched part carries its own instrument so the play-along can be mixed
/// like a small band: pick the bass, the comping voice and an optional horn
/// section independently.
#[derive(Debug, Clone, Copy)]
pub struct Settings {
    pub tempo: f32,
    /// Beats per bar — set from the loaded progression, since meter belongs to
    /// the tune the way its key does.
    pub meter: Meter,
    pub swing: bool,
    /// Where the swung eighth lands inside the beat: 0.5 straight, 2/3 triplet.
    pub swing_ratio: f32,
    pub density: Density,
    pub bass: bool,
    pub bass_inst: Instrument,
    pub bass_style: BassStyle,
    pub chords: bool,
    pub comp_inst: Instrument,
    pub comp_style: CompStyle,
    /// The three-piece horn section: trombone, tenor sax, trumpet.
    pub horns: bool,
    pub horn_style: HornStyle,
    pub drums: bool,
    /// EXPERIMENTAL: the computer takes a chorus. See [`crate::solo`].
    pub solo: bool,
    pub solo_inst: Instrument,
    /// How dense the idea generator's lines are (chance a grid slot sounds).
    pub solo_density: f32,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            tempo: 120.0,
            meter: Meter::Four,
            swing: true,
            swing_ratio: SWING_RATIO,
            density: Density::Medium,
            bass: true,
            bass_inst: Instrument::Bass,
            bass_style: BassStyle::Walking,
            chords: true,
            comp_inst: Instrument::Piano,
            comp_style: CompStyle::Rhythmic,
            horns: false,
            horn_style: HornStyle::Pads,
            drums: true,
            solo: false,
            solo_inst: Instrument::Sax,
            solo_density: IDEAS_BUSYNESS,
        }
    }
}

impl Settings {
    /// One bar's length in seconds — the only place beats-per-bar meets tempo,
    /// so 4/4 and 3/4 can never disagree across the scheduler, the chart
    /// highlight and the offline renderer.
    pub fn bar_len(&self) -> f32 {
        self.meter.beats() as f32 * 60.0 / self.tempo.max(20.0)
    }
}

/// Build one bar's worth of events for the active meter.
pub fn plan_bar(
    bars: &[Bar],
    index: usize,
    settings: &Settings,
    rng: &mut Rng,
) -> BarPlan {
    match settings.meter {
        Meter::Four => plan_bar_four(bars, index, settings, rng),
        Meter::Three => plan_bar_three(bars, index, settings, rng),
    }
}

/// The 4/4 rhythm section: walking bass, comping hits over eight eighth-note
/// slots, ride on all four beats.
fn plan_bar_four(
    bars: &[Bar],
    index: usize,
    settings: &Settings,
    rng: &mut Rng,
) -> BarPlan {
    let mut plan = BarPlan::default();
    if bars.is_empty() {
        return plan;
    }
    let bar = bars[index % bars.len()];
    let next = bars[(index + 1) % bars.len()];
    let next_root = next.chord_at(0).root;
    let spb = 60.0 / settings.tempo.max(20.0);
    let swing_off = spb * if settings.swing { settings.swing_ratio } else { 0.5 };
    let last_of_form = index % bars.len() == bars.len() - 1;

    // The occasional drop into two belongs to the walking style; a selected
    // two-beat, pedal or groove line must never surprise the player.
    plan.two_feel =
        settings.bass_style == BassStyle::Walking && !bar.is_split() && rng.chance(0.16);

    // ---- bass ----
    // The walk is always rolled, so switching the part on and off never
    // re-rolls the rest of the bar's dice.
    let walk = walking_bar(&bar, next_root, rng);
    if settings.bass {
        let first = bar.chord_at(0);
        let second = bar.chord_at(2);
        match settings.bass_style {
            BassStyle::Walking => {
                if plan.two_feel {
                    for beat in [0usize, 2] {
                        plan.notes.push(Event {
                            offset: beat as f32 * spb,
                            midi: walk[beat],
                            gain: 0.175,
                            dur: spb * 1.6,
                            instrument: settings.bass_inst,
                        });
                    }
                } else {
                    for (beat, &m) in walk.iter().enumerate() {
                        plan.notes.push(Event {
                            offset: beat as f32 * spb,
                            midi: m,
                            gain: 0.165,
                            dur: spb * 0.8,
                            instrument: settings.bass_inst,
                        });
                    }
                    if rng.chance(0.3) {
                        let target = nearest_octave(next_root, walk[3]);
                        let step = if target > walk[3] { 1 } else { -1 };
                        plan.notes.push(Event {
                            offset: 3.0 * spb + swing_off,
                            midi: clamp_bass(walk[3] + step),
                            gain: 0.13,
                            dur: spb * 0.38,
                            instrument: settings.bass_inst,
                        });
                    }
                }
            }
            BassStyle::TwoBeat => {
                let one = clamp_bass(nearest_octave(first.root, 44));
                // Beat three takes the second chord's root on a split bar,
                // otherwise mostly the fifth, sometimes the root again.
                let three = if bar.is_split() {
                    clamp_bass(nearest_octave(second.root, one))
                } else if rng.chance(0.65) {
                    clamp_bass(nearest_octave(first.root + first.quality.ints()[2], one))
                } else {
                    one
                };
                for (offset, midi) in [(0.0, one), (2.0 * spb, three)] {
                    plan.notes.push(Event {
                        offset,
                        midi,
                        gain: 0.175,
                        dur: spb * 1.8,
                        instrument: settings.bass_inst,
                    });
                }
                if rng.chance(0.22) {
                    let target = nearest_octave(next_root, three);
                    let step = if target > three { 1 } else { -1 };
                    plan.notes.push(Event {
                        offset: 3.0 * spb + swing_off,
                        midi: clamp_bass(target - step),
                        gain: 0.12,
                        dur: spb * 0.35,
                        instrument: settings.bass_inst,
                    });
                }
            }
            BassStyle::Bebop => {
                // Chromatic passing tones between chord-tone landings, and a
                // pushed eighth now and then — the melodists' vocabulary.
                let b1 = clamp_bass(nearest_octave(first.root, 46));
                let (b2, b3);
                if bar.is_split() {
                    let land = clamp_bass(nearest_octave(second.root, b1));
                    b2 = land + if land >= b1 { -1 } else { 1 };
                    b3 = land;
                } else {
                    let land = clamp_bass(nearest_octave(first.root + first.quality.ints()[1], b1));
                    b2 = land + if land >= b1 { -1 } else { 1 };
                    b3 = land;
                }
                // The window is wider than clamp_bass on purpose: folding the
                // target down an octave to satisfy a register cap turns a
                // half-step approach into a sixth leap — contour first.
                let mut target = nearest_octave(next_root, b3);
                if target > 58 {
                    target -= 12;
                }
                if target < 30 {
                    target += 12;
                }
                let mut b4 = target + if target >= b3 { -1 } else { 1 };
                if b4 == b3 {
                    b4 = target + if target >= b3 { 1 } else { -1 };
                }
                for (beat, midi) in [b1, b2, b3, b4].into_iter().enumerate() {
                    plan.notes.push(Event {
                        offset: beat as f32 * spb,
                        midi,
                        gain: 0.165,
                        dur: spb * 0.8,
                        instrument: settings.bass_inst,
                    });
                }
                if rng.chance(0.4) {
                    // The push lands on the (already register-checked) target
                    // itself; clamping it again could fold it an octave away
                    // from the approach note it follows.
                    plan.notes.push(Event {
                        offset: 3.0 * spb + swing_off,
                        midi: target,
                        gain: 0.12,
                        dur: spb * 0.35,
                        instrument: settings.bass_inst,
                    });
                }
            }
            BassStyle::Pedal => {
                // The form's opening root, held under everything.
                let pedal = clamp_bass(nearest_octave(bars[0].chord_at(0).root, 44));
                for (offset, gain) in [(0.0, 0.175), (2.0 * spb, 0.15)] {
                    plan.notes.push(Event {
                        offset,
                        midi: pedal,
                        gain,
                        dur: spb * 1.9,
                        instrument: settings.bass_inst,
                    });
                }
            }
            BassStyle::Syncopated => {
                // Locked: root on one, fifth on the and of two, and the next
                // chord anticipated on the and of four. No dice anywhere.
                let root = clamp_bass(nearest_octave(first.root, 44));
                let fifth = if bar.is_split() {
                    clamp_bass(nearest_octave(second.root, root))
                } else {
                    clamp_bass(nearest_octave(first.root + first.quality.ints()[2], root))
                };
                let push = clamp_bass(nearest_octave(next_root, root));
                for (offset, midi, gain, dur) in [
                    (0.0, root, 0.175, spb * 1.4),
                    (spb + swing_off, fifth, 0.155, spb * 1.1),
                    (3.0 * spb + swing_off, push, 0.15, spb * 0.6),
                ] {
                    plan.notes.push(Event { offset, midi, gain, dur, instrument: settings.bass_inst });
                }
            }
        }
    }

    // ---- comping ----
    if settings.chords {
        let first = bar.chord_at(0);
        let second = bar.chord_at(2);
        // Guide tones swap the voicing, not the rhythm.
        let voice = |slot: &Slot, rng: &mut Rng| -> Vec<i32> {
            match settings.comp_style {
                CompStyle::GuideTones => comp_shells(slot),
                _ => comp_voicing(slot, rng),
            }
        };
        let v1 = voice(&first, rng);
        let v2 = if bar.is_split() { voice(&second, rng) } else { v1.clone() };
        let vnext = voice(&next.chord_at(0), rng);
        let hit = |plan: &mut BarPlan, offset: f32, v: &[i32], gain: f32, dur: f32| {
            for &m in v {
                plan.notes.push(Event { offset, midi: m, gain, dur, instrument: settings.comp_inst });
            }
        };
        match settings.comp_style {
            // The pool styles keep today's behaviour, including the occasional
            // sustained two-feel bar. A chosen figure below is never overridden.
            CompStyle::Rhythmic | CompStyle::GuideTones => {
                if plan.two_feel {
                    hit(&mut plan, 0.0, &v1, 0.055, spb * 1.5);
                } else {
                    let p = pool(settings.density);
                    let hits = p[rng.below(p.len())];
                    for &s in hits {
                        let beat = s / 2;
                        let on_and = s % 2 == 1;
                        let offset = beat as f32 * spb + if on_and { swing_off } else { 0.0 };
                        // the "and" of 4 anticipates the next chord
                        let voicing = if s == 7 { &vnext } else if beat >= 2 { &v2 } else { &v1 };
                        let (gain, dur) = if on_and { (0.053, spb * 0.65) } else { (0.060, spb * 0.85) };
                        hit(&mut plan, offset, voicing, gain, dur);
                    }
                }
            }
            CompStyle::Charleston => {
                // The figure is locked; only the touch breathes.
                hit(&mut plan, 0.0, &v1, 0.058 + rng.next_f32() * 0.006, spb * 1.4);
                hit(&mut plan, spb + swing_off, &v1, 0.050 + rng.next_f32() * 0.006, spb * 0.5);
                if bar.is_split() {
                    hit(&mut plan, 2.0 * spb, &v2, 0.055, spb * 1.2);
                }
            }
            CompStyle::FourToBar => {
                for beat in 0..4 {
                    let v = if beat >= 2 { &v2 } else { &v1 };
                    hit(&mut plan, beat as f32 * spb, v, 0.045, spb * 0.6);
                }
            }
            CompStyle::Sustained => {
                if bar.is_split() {
                    hit(&mut plan, 0.0, &v1, 0.055, spb * 1.9);
                    hit(&mut plan, 2.0 * spb, &v2, 0.055, spb * 1.9);
                } else {
                    hit(&mut plan, 0.0, &v1, 0.055, spb * 3.8);
                }
            }
        }
    }

    // ---- horns: the three-piece section plays its arranged background ----
    if settings.horns {
        // The section rolls its own per-bar dice: toggling it never re-rolls
        // the rest of the band, and the same bar always plays the same figure —
        // backgrounds are arranged parts, not improvisations.
        let mut horn_rng = Rng::new(0x484F_524E ^ (index as u64).wrapping_mul(0x9E37_79B9));
        let v = comp_voicing(&bar.chord_at(0), &mut horn_rng);
        let v2 = if bar.is_split() {
            comp_voicing(&bar.chord_at(2), &mut horn_rng)
        } else {
            v.clone()
        };
        let vnext = comp_voicing(&next.chord_at(0), &mut horn_rng);
        let blow = |plan: &mut BarPlan, voicing: &[i32], offset: f32, dur: f32, gain: f32| {
            for (instrument, midi) in section_voices(voicing) {
                plan.notes.push(Event { offset, midi, gain, dur, instrument });
            }
        };
        match settings.horn_style {
            HornStyle::Pads => match index % 4 {
                // Two bars on, two bars off; the second breath leans back.
                0 => {
                    if bar.is_split() {
                        blow(&mut plan, &v, 0.0, 1.9 * spb, 0.055);
                        blow(&mut plan, &v2, 2.0 * spb, 1.8 * spb, 0.055);
                    } else {
                        blow(&mut plan, &v, 0.0, 3.8 * spb, 0.055);
                    }
                }
                1 => blow(&mut plan, &v, 0.0, 3.8 * spb, 0.045),
                _ => {}
            },
            HornStyle::RiffPush => {
                blow(&mut plan, &v, 0.0, 1.6 * spb, 0.055);
                blow(&mut plan, &vnext, 3.0 * spb + swing_off, 0.5 * spb, 0.05);
            }
            HornStyle::RiffBackbeat => {
                blow(&mut plan, &v, spb, 0.4 * spb, 0.05);
                blow(&mut plan, &v2, 3.0 * spb, 0.4 * spb, 0.05);
            }
            HornStyle::RiffCall => {
                blow(&mut plan, &v2, 2.0 * spb, 1.8 * spb, 0.055);
            }
            HornStyle::RiffShout => {
                blow(&mut plan, &v, swing_off, 0.4 * spb, 0.055);
                blow(&mut plan, &v2, 2.0 * spb + swing_off, 0.4 * spb, 0.05);
            }
        }
    }

    // ---- drums ----
    if settings.drums {
        for beat in 0..4 {
            let t = beat as f32 * spb;
            let accent = beat == 0 || beat == 2;
            plan.perc.push(PercEvent {
                offset: t,
                kind: PercRole::Ride,
                gain: if accent { 0.058 } else { 0.046 } + rng.next_f32() * 0.012,
            });
            if beat == 1 || beat == 3 {
                plan.perc.push(PercEvent { offset: t + swing_off, kind: PercRole::RideSkip, gain: 0.033 });
                plan.perc.push(PercEvent { offset: t, kind: PercRole::HiHat, gain: 0.055 });
            }
            if rng.chance(0.22) {
                let off = if rng.chance(0.5) { swing_off } else { 0.0 };
                plan.perc.push(PercEvent { offset: t + off, kind: PercRole::Kick, gain: 0.17 });
            }
            if rng.chance(0.16) {
                plan.perc.push(PercEvent { offset: t + swing_off, kind: PercRole::Brush, gain: 0.085 });
            }
        }
        if last_of_form {
            for (i, g) in [(0.0, 0.085), (0.5, 0.095), (0.75, 0.085)] {
                plan.perc.push(PercEvent { offset: 3.0 * spb + i * spb, kind: PercRole::Brush, gain: g });
            }
        }
    }


    // ---- solo (experimental): the computer takes a chorus ----
    if settings.solo {
        plan.notes.extend(crate::solo::solo_bar(bars, index, settings));
    }

    plan
}

/// The jazz waltz: not a truncated walk. The bass roots beat one and leads on
/// beat three; the ride marks all three beats with the skip on the and of two;
/// comping floats on beat two and its offbeats. A waltz bar holds one chord —
/// split bars are a 4/4 idea and `bar.second` is ignored here.
fn plan_bar_three(
    bars: &[Bar],
    index: usize,
    settings: &Settings,
    rng: &mut Rng,
) -> BarPlan {
    let mut plan = BarPlan::default();
    if bars.is_empty() {
        return plan;
    }
    let bar = bars[index % bars.len()];
    let next = bars[(index + 1) % bars.len()];
    let next_root = next.chord_at(0).root;
    let spb = 60.0 / settings.tempo.max(20.0);
    let swing_off = spb * if settings.swing { settings.swing_ratio } else { 0.5 };
    let last_of_form = index % bars.len() == bars.len() - 1;
    let slot = bar.chord_at(0);

    // ---- bass ----
    if settings.bass {
        let root = clamp_bass(nearest_octave(slot.root, 44));
        let ints = slot.quality.ints();
        let push = |plan: &mut BarPlan, offset: f32, midi: i32, gain: f32, dur: f32| {
            plan.notes.push(Event { offset, midi, gain, dur, instrument: settings.bass_inst });
        };
        match settings.bass_style {
            // Root on one, a lead note on three more often than not.
            BassStyle::Walking => {
                push(&mut plan, 0.0, root, 0.175, spb * 1.9);
                if rng.chance(0.65) {
                    let mut tone = if rng.chance(0.5) {
                        clamp_bass(nearest_octave(slot.root + ints[2], root))
                    } else {
                        let dir = if rng.chance(0.5) { 1 } else { -1 };
                        clamp_bass(nearest_octave(next_root + dir, root))
                    };
                    if tone == root {
                        tone = clamp_bass(nearest_octave(slot.root + ints[1], root));
                    }
                    push(&mut plan, 2.0 * spb, tone, 0.15, spb * 0.8);
                }
            }
            BassStyle::TwoBeat => {
                let fifth = clamp_bass(nearest_octave(slot.root + ints[2], root));
                push(&mut plan, 0.0, root, 0.175, spb * 1.8);
                push(&mut plan, 2.0 * spb, fifth, 0.16, spb * 0.8);
            }
            BassStyle::Bebop => {
                let land = clamp_bass(nearest_octave(slot.root + ints[1], root));
                let passing = land + if land >= root { -1 } else { 1 };
                // Wider than clamp_bass so the approach keeps its contour.
                let mut target = nearest_octave(next_root, land);
                if target > 58 {
                    target -= 12;
                }
                if target < 30 {
                    target += 12;
                }
                let approach = target + if target >= land { -1 } else { 1 };
                push(&mut plan, 0.0, root, 0.17, spb * 0.85);
                push(&mut plan, spb, passing, 0.155, spb * 0.85);
                push(&mut plan, 2.0 * spb, approach, 0.16, spb * 0.85);
            }
            BassStyle::Pedal => {
                let pedal = clamp_bass(nearest_octave(bars[0].chord_at(0).root, 44));
                push(&mut plan, 0.0, pedal, 0.175, spb * 2.8);
            }
            BassStyle::Syncopated => {
                // Locked, no dice: one, the and of two, the next chord pushed
                // on the and of three.
                let fifth = clamp_bass(nearest_octave(slot.root + ints[2], root));
                let anticipation = clamp_bass(nearest_octave(next_root, root));
                push(&mut plan, 0.0, root, 0.175, spb * 1.3);
                push(&mut plan, spb + swing_off, fifth, 0.15, spb * 0.9);
                push(&mut plan, 2.0 * spb + swing_off, anticipation, 0.15, spb * 0.5);
            }
        }
    }

    // ---- comping ----
    if settings.chords {
        let v = match settings.comp_style {
            CompStyle::GuideTones => comp_shells(&slot),
            _ => comp_voicing(&slot, rng),
        };
        let vnext = match settings.comp_style {
            CompStyle::GuideTones => comp_shells(&next.chord_at(0)),
            _ => comp_voicing(&next.chord_at(0), rng),
        };
        let hit = |plan: &mut BarPlan, offset: f32, v: &[i32], gain: f32, dur: f32| {
            for &m in v {
                plan.notes.push(Event { offset, midi: m, gain, dur, instrument: settings.comp_inst });
            }
        };
        match settings.comp_style {
            // Six eighth-note slots; beat two carries the waltz.
            CompStyle::Rhythmic | CompStyle::GuideTones => {
                let pool: &[&[usize]] = match settings.density {
                    Density::Sparse => &[&[2], &[3], &[0]],
                    Density::Medium => &[&[2], &[3], &[2, 5], &[0, 3]],
                    Density::Busy => &[&[1, 3], &[2, 5], &[0, 3, 5], &[3, 4]],
                };
                let hits = pool[rng.below(pool.len())];
                for &s in hits {
                    let beat = s / 2;
                    let on_and = s % 2 == 1;
                    let offset = beat as f32 * spb + if on_and { swing_off } else { 0.0 };
                    // the and of three anticipates the next chord
                    let voicing = if s == 5 { &vnext } else { &v };
                    let (gain, dur) = if on_and { (0.053, spb * 0.6) } else { (0.060, spb * 0.8) };
                    hit(&mut plan, offset, voicing, gain, dur);
                }
            }
            CompStyle::Charleston => {
                hit(&mut plan, 0.0, &v, 0.058 + rng.next_f32() * 0.006, spb * 1.3);
                hit(&mut plan, spb + swing_off, &v, 0.050 + rng.next_f32() * 0.006, spb * 0.5);
            }
            CompStyle::FourToBar => {
                for beat in 0..3 {
                    hit(&mut plan, beat as f32 * spb, &v, 0.045, spb * 0.6);
                }
            }
            CompStyle::Sustained => {
                hit(&mut plan, 0.0, &v, 0.055, spb * 2.8);
            }
        }
    }

    // ---- horns: the section's figures, sized for three beats ----
    if settings.horns {
        let mut horn_rng = Rng::new(0x57_414C_545A ^ (index as u64).wrapping_mul(0x9E37_79B9));
        let v = comp_voicing(&slot, &mut horn_rng);
        let vnext = comp_voicing(&next.chord_at(0), &mut horn_rng);
        let blow = |plan: &mut BarPlan, voicing: &[i32], offset: f32, dur: f32, gain: f32| {
            for (instrument, midi) in section_voices(voicing) {
                plan.notes.push(Event { offset, midi, gain, dur, instrument });
            }
        };
        match settings.horn_style {
            HornStyle::Pads => match index % 4 {
                0 => blow(&mut plan, &v, 0.0, 2.8 * spb, 0.055),
                1 => blow(&mut plan, &v, 0.0, 2.8 * spb, 0.045),
                _ => {}
            },
            HornStyle::RiffPush => {
                blow(&mut plan, &v, 0.0, 1.4 * spb, 0.055);
                blow(&mut plan, &vnext, 2.0 * spb + swing_off, 0.4 * spb, 0.05);
            }
            HornStyle::RiffBackbeat => {
                blow(&mut plan, &v, spb, 0.4 * spb, 0.05);
                blow(&mut plan, &v, 2.0 * spb, 0.4 * spb, 0.05);
            }
            HornStyle::RiffCall => {
                blow(&mut plan, &v, spb, 1.6 * spb, 0.055);
            }
            HornStyle::RiffShout => {
                blow(&mut plan, &v, swing_off, 0.4 * spb, 0.055);
                blow(&mut plan, &v, spb + swing_off, 0.4 * spb, 0.05);
            }
        }
    }

    // ---- drums: the waltz ride, hats on two ----
    if settings.drums {
        for beat in 0..3 {
            let t = beat as f32 * spb;
            plan.perc.push(PercEvent {
                offset: t,
                kind: PercRole::Ride,
                gain: if beat == 0 { 0.060 } else { 0.046 } + rng.next_f32() * 0.010,
            });
        }
        // the skip on the and of two is what gives the waltz its lilt
        plan.perc.push(PercEvent { offset: spb + swing_off, kind: PercRole::RideSkip, gain: 0.033 });
        plan.perc.push(PercEvent { offset: spb, kind: PercRole::HiHat, gain: 0.055 });
        if rng.chance(0.20) {
            plan.perc.push(PercEvent { offset: 0.0, kind: PercRole::Kick, gain: 0.16 });
        }
        if rng.chance(0.15) {
            plan.perc.push(PercEvent { offset: 2.0 * spb + swing_off, kind: PercRole::Brush, gain: 0.08 });
        }
        if last_of_form {
            for (i, g) in [(0.0, 0.085), (0.5, 0.090)] {
                plan.perc.push(PercEvent { offset: 2.0 * spb + i * spb, kind: PercRole::Brush, gain: g });
            }
        }
    }


    // ---- solo (experimental): the computer takes a chorus ----
    if settings.solo {
        plan.notes.extend(crate::solo::solo_bar(bars, index, settings));
    }

    plan
}

#[cfg(test)]
mod tests {
    use super::*;
    use jazz_theory::progression;

    fn settings() -> Settings {
        Settings::default()
    }

    #[test]
    fn nearest_octave_lands_close_to_the_reference() {
        for pcv in 0..12 {
            for r in [31, 40, 44, 55] {
                let m = nearest_octave(pcv, r);
                assert_eq!(pc(m), pcv);
                assert!((m - r).abs() <= 6, "{m} is not near {r}");
            }
        }
    }

    #[test]
    fn bass_stays_in_a_playable_register() {
        for m in -20..120 {
            let c = clamp_bass(m);
            assert!((31..=54).contains(&c), "{m} clamped to {c}");
        }
    }

    #[test]
    fn walking_bass_never_repeats_a_note_within_a_bar() {
        let bars = progression::blues(5);
        let mut rng = Rng::new(42);
        for _ in 0..500 {
            for i in 0..bars.len() {
                let w = walking_bar(&bars[i], bars[(i + 1) % bars.len()].chord_at(0).root, &mut rng);
                for pair in w.windows(2) {
                    assert_ne!(pair[0], pair[1], "walking line repeated {}", pair[0]);
                }
            }
        }
    }

    #[test]
    fn walking_bass_starts_on_the_chord_root() {
        let bars = progression::two_five_one(0);
        let mut rng = Rng::new(7);
        for i in 0..bars.len() {
            let w = walking_bar(&bars[i], bars[(i + 1) % bars.len()].chord_at(0).root, &mut rng);
            assert_eq!(pc(w[0]), bars[i].chord_at(0).root, "beat 1 should be the root");
        }
    }

    #[test]
    fn split_bars_put_the_second_chord_on_beat_three() {
        let bars = progression::rhythm_a();
        let mut rng = Rng::new(3);
        for i in 0..bars.len() {
            let w = walking_bar(&bars[i], bars[(i + 1) % bars.len()].chord_at(0).root, &mut rng);
            assert_eq!(pc(w[2]), bars[i].chord_at(2).root, "beat 3 should land the second chord");
        }
    }

    #[test]
    fn voicings_have_no_semitone_clashes_and_sit_in_register() {
        let mut rng = Rng::new(99);
        for q in [Quality::Maj7, Quality::Min7, Quality::Dom7, Quality::Min7b5, Quality::MinTonic] {
            for root in 0..12 {
                for _ in 0..40 {
                    let slot = Slot::new(root, q);
                    let mut v = comp_voicing(&slot, &mut rng);
                    v.sort_unstable();
                    for w in v.windows(2) {
                        assert!(w[1] - w[0] >= 2, "{q:?} on {root} clashes: {v:?}");
                    }
                    assert!(v[0] >= 52 && *v.last().unwrap() <= 84, "{q:?} out of register: {v:?}");
                }
            }
        }
    }

    #[test]
    fn altered_bars_avoid_the_natural_five_and_nine() {
        let mut rng = Rng::new(5);
        let slot = Slot::with_scale(7, Quality::Dom7, ALTERED);
        for _ in 0..300 {
            let v = comp_voicing(&slot, &mut rng);
            let root = midi_for(7, 4);
            for m in v {
                let rel = pc(m - root);
                assert_ne!(rel, 7, "natural 5 on an altered bar");
                assert_ne!(rel, 2, "natural 9 on an altered bar");
            }
        }
    }

    #[test]
    fn a_bar_plan_fits_inside_its_bar() {
        let bars = progression::blue_bossa();
        let mut rng = Rng::new(11);
        let s = settings();
        let bar_len = s.bar_len();
        for i in 0..bars.len() {
            let plan = plan_bar(&bars, i, &s, &mut rng);
            for e in &plan.notes {
                assert!(e.offset >= 0.0 && e.offset < bar_len + 1e-3, "note at {}", e.offset);
                assert!(e.gain > 0.0 && e.dur > 0.0);
            }
            for p in &plan.perc {
                assert!(p.offset >= 0.0 && p.offset < bar_len + 1e-3, "hit at {}", p.offset);
            }
        }
    }

    #[test]
    fn the_comping_rhythm_actually_varies() {
        let bars = progression::two_five_one(0);
        let mut rng = Rng::new(2024);
        let s = settings();
        let mut shapes = std::collections::HashSet::new();
        for i in 0..200 {
            let plan = plan_bar(&bars, i, &s, &mut rng);
            let mut offs: Vec<String> = plan
                .notes
                .iter()
                .filter(|e| e.instrument == s.comp_inst)
                .map(|e| format!("{:.3}", e.offset))
                .collect();
            offs.sort();
            offs.dedup();
            shapes.insert(offs.join(","));
        }
        assert!(shapes.len() >= 5, "comping only produced {} shapes", shapes.len());
    }

    #[test]
    fn switching_off_a_part_removes_it() {
        let bars = progression::blues(5);
        let mut rng = Rng::new(1);
        let s = Settings { bass: false, chords: false, drums: false, ..Settings::default() };
        let plan = plan_bar(&bars, 0, &s, &mut rng);
        assert!(plan.notes.is_empty() && plan.perc.is_empty());
    }

    #[test]
    fn each_part_plays_its_chosen_instrument() {
        let bars = progression::two_five_one(0);
        let mut rng = Rng::new(3);
        let s = Settings {
            bass_inst: Instrument::Trombone,
            comp_inst: Instrument::Guitar,
            ..Settings::default()
        };
        // Sweep several bars so two-feel randomness cannot hide a part.
        let mut used = std::collections::HashSet::new();
        for i in 0..64 {
            for e in plan_bar(&bars, i, &s, &mut rng).notes {
                used.insert(e.instrument);
            }
        }
        assert!(used.contains(&Instrument::Trombone), "bass part ignored its instrument");
        assert!(used.contains(&Instrument::Guitar), "comping ignored its instrument");
        // The two-feel branch once kept a hardcoded Rhodes: with every part
        // assigned elsewhere, no event may fall back to the default voice.
        assert!(
            !used.contains(&Instrument::Keys),
            "some bars fell back to the default comp instrument"
        );
    }

    #[test]
    fn every_bass_style_makes_a_line_in_both_meters() {
        let bars = progression::two_five_one(0);
        for meter in [Meter::Four, Meter::Three] {
            for style in BassStyle::ALL {
                let s = Settings {
                    meter,
                    bass_style: style,
                    chords: false,
                    drums: false,
                    ..Settings::default()
                };
                let mut rng = Rng::new(9);
                let bar_len = s.bar_len();
                for i in 0..16 {
                    let plan = plan_bar(&bars, i, &s, &mut rng);
                    assert!(!plan.notes.is_empty(), "{style:?} in {meter:?} was silent");
                    for e in &plan.notes {
                        assert!(e.offset < bar_len, "{style:?} in {meter:?}: note past the barline");
                    }
                }
            }
        }
    }

    #[test]
    fn the_two_beat_line_sits_on_one_and_three() {
        let bars = progression::blues(5);
        let s = Settings {
            bass_style: BassStyle::TwoBeat,
            chords: false,
            drums: false,
            ..Settings::default()
        };
        let mut rng = Rng::new(4);
        let spb = 60.0 / s.tempo;
        for i in 0..48 {
            let plan = plan_bar(&bars, i, &s, &mut rng);
            assert_eq!(plan.notes[0].offset, 0.0);
            assert!((plan.notes[1].offset - 2.0 * spb).abs() < 1e-4, "beat three is the second note");
            // Beat one is always the root of the sounding chord.
            let root = bars[i % bars.len()].chord_at(0).root;
            assert_eq!(pc(plan.notes[0].midi), pc(root), "bar {i}: beat one must be the root");
        }
    }

    #[test]
    fn the_bebop_line_passes_chromatically_and_never_leaps_far() {
        let bars = progression::two_five_one(0);
        let s = Settings {
            bass_style: BassStyle::Bebop,
            chords: false,
            drums: false,
            ..Settings::default()
        };
        let mut rng = Rng::new(12);
        for i in 0..32 {
            let plan = plan_bar(&bars, i, &s, &mut rng);
            let line: Vec<i32> = plan.notes.iter().map(|e| e.midi).collect();
            assert!(
                line.windows(2).any(|w| (w[0] - w[1]).abs() == 1),
                "bar {i}: a bebop line without a chromatic step"
            );
            for w in line.windows(2) {
                assert!((w[0] - w[1]).abs() <= 7, "bar {i}: the line leapt past a fifth");
            }
        }
    }

    #[test]
    fn the_pedal_holds_one_note_under_the_changes() {
        let bars = progression::blues(5);
        let s = Settings {
            bass_style: BassStyle::Pedal,
            chords: false,
            drums: false,
            ..Settings::default()
        };
        let mut rng = Rng::new(3);
        for i in 0..24 {
            let plan = plan_bar(&bars, i, &s, &mut rng);
            for e in &plan.notes {
                assert_eq!(pc(e.midi), pc(5), "the pedal moved off the opening F");
            }
        }
    }

    #[test]
    fn the_syncopated_line_locks_its_pattern() {
        // On a one-chord vamp every bar must be byte-identical: a groove that
        // varies is not a groove.
        let bars = progression::modal_vamp(2);
        let s = Settings {
            bass_style: BassStyle::Syncopated,
            chords: false,
            drums: false,
            ..Settings::default()
        };
        let mut rng = Rng::new(77);
        let first = plan_bar(&bars, 0, &s, &mut rng).notes;
        for i in 1..12 {
            assert_eq!(plan_bar(&bars, i, &s, &mut rng).notes, first, "bar {i} broke the lock");
        }
        // And it must actually syncopate: at least one event off the beat.
        let spb = 60.0 / s.tempo;
        assert!(
            first.iter().any(|e| (e.offset / spb).fract() > 0.01),
            "a syncopated line with nothing off the beat"
        );
    }

    #[test]
    fn comping_voice_leads_instead_of_transposing_one_shape() {
        // The old voicing folded every root into one octave and stacked the
        // same shape on it, so every chord sounded like the same chord moved.
        // The A/B-form rule must connect neighbouring chords by step in the
        // bottom voice while staying anchored near middle C.
        let mut rng = Rng::new(1);
        let d = comp_voicing(&Slot::new(2, Quality::Min7), &mut rng);
        let g = comp_voicing(&Slot::new(7, Quality::Dom7), &mut rng);
        let c = comp_voicing(&Slot::new(0, Quality::Maj7), &mut rng);
        for v in [&d, &g, &c] {
            assert!((54..=66).contains(&v[0]), "bottom {} left the anchor band", v[0]);
        }
        assert!((d[0] - g[0]).abs() <= 2, "Dm7->G7 bottom voice leapt: {} -> {}", d[0], g[0]);
        assert!((g[0] - c[0]).abs() <= 2, "G7->Cmaj7 bottom voice leapt: {} -> {}", g[0], c[0]);
        // Adjacent chords take different inversions — bottom intervals differ —
        // so the colour changes chord to chord instead of repeating.
        let d_bottom_interval = pc(d[0] - 2);
        let g_bottom_interval = pc(g[0] - 7);
        assert_ne!(
            d_bottom_interval, g_bottom_interval,
            "both chords bottomed on the same chord degree - still one shape"
        );
    }

    #[test]
    fn the_shells_voice_lead_too() {
        let d = comp_shells(&Slot::new(2, Quality::Min7));
        let g = comp_shells(&Slot::new(7, Quality::Dom7));
        let c = comp_shells(&Slot::new(0, Quality::Maj7));
        assert!((d[0] - g[0]).abs() <= 2, "guide tones leapt Dm7->G7");
        assert!((g[0] - c[0]).abs() <= 2, "guide tones leapt G7->Cmaj7");
    }

    #[test]
    fn every_comp_style_comps_in_both_meters() {
        let bars = progression::two_five_one(0);
        for meter in [Meter::Four, Meter::Three] {
            for style in CompStyle::ALL {
                let s = Settings {
                    meter,
                    comp_style: style,
                    bass: false,
                    drums: false,
                    ..Settings::default()
                };
                let mut rng = Rng::new(6);
                let bar_len = s.bar_len();
                for i in 0..16 {
                    let plan = plan_bar(&bars, i, &s, &mut rng);
                    assert!(!plan.notes.is_empty(), "{style:?} in {meter:?} was silent");
                    for e in &plan.notes {
                        assert!(e.offset < bar_len, "{style:?} in {meter:?}: chord past the barline");
                    }
                }
            }
        }
    }

    #[test]
    fn the_swing_ratio_is_a_real_dial() {
        // The Charleston's push sits on the and of two, wherever the swing
        // ratio puts it.
        let bars = progression::two_five_one(0);
        for ratio in [0.5_f32, 0.7] {
            let s = Settings {
                comp_style: CompStyle::Charleston,
                swing_ratio: ratio,
                bass: false,
                drums: false,
                ..Settings::default()
            };
            let mut rng = Rng::new(3);
            let spb = 60.0 / s.tempo;
            let plan = plan_bar(&bars, 0, &s, &mut rng);
            let push = plan.notes.iter().map(|e| e.offset).fold(0.0_f32, f32::max);
            assert!(
                (push - (spb + spb * ratio)).abs() < 1e-4,
                "ratio {ratio}: push landed at {push}"
            );
        }
    }

    #[test]
    fn the_charleston_locks_its_figure() {
        let bars = progression::two_five_one(0); // no split bars
        let s = Settings {
            comp_style: CompStyle::Charleston,
            bass: false,
            drums: false,
            ..Settings::default()
        };
        let mut rng = Rng::new(8);
        let spb = 60.0 / s.tempo;
        let push = spb + spb * 2.0 / 3.0; // the swung and of two
        for i in 0..32 {
            let plan = plan_bar(&bars, i, &s, &mut rng);
            let mut onsets: Vec<f32> = plan.notes.iter().map(|e| e.offset).collect();
            onsets.dedup();
            assert_eq!(onsets.len(), 2, "bar {i}: the charleston is exactly two hits");
            assert_eq!(onsets[0], 0.0, "bar {i}: the long chord sits on one");
            assert!((onsets[1] - push).abs() < 1e-4, "bar {i}: the push lives on the and of two");
        }
    }

    #[test]
    fn four_to_the_bar_sits_on_every_beat() {
        let bars = progression::blues(5);
        let s = Settings {
            comp_style: CompStyle::FourToBar,
            bass: false,
            drums: false,
            ..Settings::default()
        };
        let mut rng = Rng::new(2);
        let spb = 60.0 / s.tempo;
        for i in 0..24 {
            let plan = plan_bar(&bars, i, &s, &mut rng);
            let mut onsets: Vec<f32> = plan.notes.iter().map(|e| e.offset).collect();
            onsets.dedup();
            assert_eq!(onsets.len(), 4, "bar {i}: four to the bar means four");
            for (beat, onset) in onsets.iter().enumerate() {
                assert!(
                    (onset - beat as f32 * spb).abs() < 1e-4,
                    "bar {i}: a chord left the beat"
                );
            }
        }
    }

    #[test]
    fn sustained_holds_one_voicing_through_the_bar() {
        let bars = progression::two_five_one(0); // no split bars
        let s = Settings {
            comp_style: CompStyle::Sustained,
            bass: false,
            drums: false,
            ..Settings::default()
        };
        let mut rng = Rng::new(4);
        let spb = 60.0 / s.tempo;
        for i in 0..16 {
            let plan = plan_bar(&bars, i, &s, &mut rng);
            assert!(plan.notes.iter().all(|e| e.offset == 0.0), "bar {i}: sustained means one attack");
            assert!(
                plan.notes.iter().all(|e| e.dur > 3.0 * spb),
                "bar {i}: the voicing must ring the whole bar"
            );
        }
    }

    #[test]
    fn guide_tones_comp_only_the_third_and_seventh() {
        let bars = progression::blues(5); // no split bars
        let s = Settings {
            comp_style: CompStyle::GuideTones,
            bass: false,
            drums: false,
            ..Settings::default()
        };
        let mut rng = Rng::new(10);
        for i in 0..48 {
            let plan = plan_bar(&bars, i, &s, &mut rng);
            let cur = bars[i % bars.len()].chord_at(0);
            let nxt = bars[(i + 1) % bars.len()].chord_at(0);
            let allowed: Vec<i32> = [cur, nxt]
                .iter()
                .flat_map(|c| {
                    let ints = c.quality.ints();
                    [pc(c.root + ints[1]), pc(c.root + ints[3])]
                })
                .collect();
            for e in &plan.notes {
                assert!(
                    allowed.contains(&pc(e.midi)),
                    "bar {i}: {} is not a guide tone of {} or {}",
                    e.midi,
                    cur.symbol(),
                    nxt.symbol()
                );
            }
            // Every hit is exactly the two shells.
            let mut onsets: Vec<f32> = plan.notes.iter().map(|e| e.offset).collect();
            onsets.dedup();
            assert_eq!(plan.notes.len(), onsets.len() * 2, "bar {i}: a shell has two notes");
        }
    }

    #[test]
    fn a_chosen_figure_survives_the_two_feel() {
        // The walking bass still drops into two now and then; an explicitly
        // chosen comping figure must keep playing through those bars.
        let bars = progression::two_five_one(0);
        let s = Settings {
            comp_style: CompStyle::Charleston,
            drums: false,
            ..Settings::default()
        };
        let mut rng = Rng::new(14);
        let spb = 60.0 / s.tempo;
        let push = spb + spb * 2.0 / 3.0;
        let mut two_feel_bars = 0;
        for i in 0..200 {
            let plan = plan_bar(&bars, i, &s, &mut rng);
            if plan.two_feel {
                two_feel_bars += 1;
            }
            let mut comp: Vec<f32> = plan
                .notes
                .iter()
                .filter(|e| e.instrument == s.comp_inst)
                .map(|e| e.offset)
                .collect();
            comp.dedup();
            assert_eq!(comp.len(), 2, "bar {i}: the charleston vanished");
            assert!((comp[1] - push).abs() < 1e-4);
        }
        assert!(two_feel_bars > 0, "the test never exercised a two-feel bar");
    }

    #[test]
    fn a_waltz_bar_lays_out_in_three() {
        let bars = progression::two_five_one(0);
        let s = Settings { meter: Meter::Three, ..Settings::default() };
        let mut rng = Rng::new(5);
        let bar_len = s.bar_len();
        for i in 0..48 {
            let plan = plan_bar(&bars, i, &s, &mut rng);
            for e in &plan.notes {
                assert!(e.offset < bar_len, "a note at {}s is past the 3/4 barline", e.offset);
            }
            for p in &plan.perc {
                assert!(p.offset < bar_len + 0.001, "percussion at {}s is past the barline", p.offset);
            }
            let rides = plan.perc.iter().filter(|p| p.kind == PercRole::Ride).count();
            assert_eq!(rides, 3, "the waltz ride marks all three beats");
            assert!(!plan.two_feel, "the two-feel is a 4/4 idea");
        }
    }

    #[test]
    fn the_waltz_bass_roots_beat_one_and_never_repeats_itself() {
        let bars = progression::blues(5);
        let s = Settings {
            meter: Meter::Three,
            chords: false,
            drums: false,
            ..Settings::default()
        };
        let mut rng = Rng::new(21);
        let spb = 60.0 / s.tempo;
        for i in 0..96 {
            let plan = plan_bar(&bars, i, &s, &mut rng);
            let first = &plan.notes[0];
            assert_eq!(first.offset, 0.0, "the waltz bass must root beat one");
            if let Some(second) = plan.notes.get(1) {
                assert!((second.offset - 2.0 * spb).abs() < 1e-4, "the lead note lives on beat three");
                assert_ne!(second.midi, first.midi, "bar {i}: the bass repeated its note");
            }
        }
    }

    #[test]
    fn a_waltz_bar_is_three_quarters_the_length_of_common_time() {
        let four = Settings::default();
        let three = Settings { meter: Meter::Three, ..Settings::default() };
        assert!((three.bar_len() / four.bar_len() - 0.75).abs() < 1e-6);
    }

    #[test]
    fn toggling_horns_does_not_reroll_the_rhythm_section() {
        // The horn pad draws from its own per-bar RNG. If it shared the bar RNG,
        // enabling horns would shift every later draw and silently rewrite the
        // walking bass and comping of the whole form.
        let bars = progression::two_five_one(0);
        let band = |horns: bool| {
            let mut rng = Rng::new(11);
            let s = Settings { horns, ..Settings::default() };
            (0..16)
                .flat_map(|i| plan_bar(&bars, i, &s, &mut rng).notes)
                .filter(|e| !is_horn(e.instrument))
                .map(|e| (e.midi, (e.offset * 1000.0) as i32))
                .collect::<Vec<_>>()
        };
        assert_eq!(band(false), band(true), "enabling horns changed the bass/comping");
    }

    fn is_horn(i: Instrument) -> bool {
        matches!(i, Instrument::Sax | Instrument::Trumpet | Instrument::Trombone)
    }

    #[test]
    fn the_horn_section_sounds_only_when_enabled() {
        let bars = progression::two_five_one(0);
        let mut rng = Rng::new(7);
        // Off by default: no horn voice anywhere.
        let off = Settings::default();
        for i in 0..16 {
            let plan = plan_bar(&bars, i, &off, &mut rng);
            assert!(
                plan.notes.iter().all(|e| !is_horn(e.instrument)),
                "horns sounded while disabled"
            );
        }
        // Enabled: the section appears.
        let on = Settings { horns: true, ..Settings::default() };
        let plan = plan_bar(&bars, 0, &on, &mut rng);
        assert!(
            plan.notes.iter().any(|e| is_horn(e.instrument)),
            "horn section produced no notes"
        );
    }

    #[test]
    fn the_horn_section_is_three_voices_low_to_high() {
        let bars = progression::two_five_one(0);
        let s = Settings {
            horns: true,
            bass: false,
            chords: false,
            drums: false,
            ..Settings::default()
        };
        let mut rng = Rng::new(3);
        let plan = plan_bar(&bars, 0, &s, &mut rng); // Pads: bar 0 blows
        assert_eq!(plan.notes.len(), 3, "the section is exactly three horns: {:?}", plan.notes);
        let by = |i: Instrument| plan.notes.iter().find(|e| e.instrument == i).map(|e| e.midi);
        let bone = by(Instrument::Trombone).expect("no trombone chair");
        let sax = by(Instrument::Sax).expect("no sax chair");
        let tpt = by(Instrument::Trumpet).expect("no trumpet chair");
        assert!(bone < sax && sax < tpt, "section stacked out of order: {bone} {sax} {tpt}");
    }

    #[test]
    fn the_pads_breathe_two_bars_on_two_off() {
        let bars = progression::blues(5);
        let s = Settings {
            horns: true,
            bass: false,
            chords: false,
            drums: false,
            ..Settings::default()
        };
        let mut rng = Rng::new(9);
        for i in 0..16 {
            let plan = plan_bar(&bars, i, &s, &mut rng);
            assert_eq!(
                !plan.notes.is_empty(),
                i % 4 < 2,
                "bar {i}: the pads must breathe - two bars on, two off"
            );
        }
    }

    #[test]
    fn every_horn_style_stays_inside_the_bar_in_both_meters() {
        let bars = progression::two_five_one(0);
        for meter in [Meter::Four, Meter::Three] {
            for style in HornStyle::ALL {
                let s = Settings {
                    meter,
                    horns: true,
                    horn_style: style,
                    bass: false,
                    chords: false,
                    drums: false,
                    ..Settings::default()
                };
                let mut rng = Rng::new(7);
                let bar_len = s.bar_len();
                let mut heard = false;
                for i in 0..16 {
                    let plan = plan_bar(&bars, i, &s, &mut rng);
                    heard |= !plan.notes.is_empty();
                    for e in &plan.notes {
                        assert!(
                            e.offset >= 0.0 && e.offset < bar_len,
                            "{style:?} in {meter:?}: a horn left the bar"
                        );
                    }
                }
                assert!(heard, "{style:?} in {meter:?} never sounded");
            }
        }
    }

    #[test]
    fn the_riffs_are_arranged_not_improvised() {
        // Backgrounds are parts, not solos: the same bar of the same form must
        // play the identical figure regardless of the band's own dice.
        let bars = progression::modal_vamp(2);
        let s = Settings {
            horns: true,
            horn_style: HornStyle::RiffPush,
            bass: false,
            chords: false,
            drums: false,
            ..Settings::default()
        };
        let mut r1 = Rng::new(1);
        let mut r2 = Rng::new(999);
        let a = plan_bar(&bars, 3, &s, &mut r1).notes;
        let b = plan_bar(&bars, 3, &s, &mut r2).notes;
        assert_eq!(a, b, "the section improvised");
    }

    #[test]
    fn the_ride_marks_every_beat() {
        let bars = progression::blues(5);
        let mut rng = Rng::new(8);
        let s = settings();
        for i in 0..12 {
            let plan = plan_bar(&bars, i, &s, &mut rng);
            let rides = plan.perc.iter().filter(|p| p.kind == PercRole::Ride).count();
            assert_eq!(rides, 4, "the ride should mark all four beats");
        }
    }

    #[test]
    fn the_form_ends_with_a_fill() {
        let bars = progression::two_five_one(0);
        let mut rng = Rng::new(4);
        let s = settings();
        let last = plan_bar(&bars, bars.len() - 1, &s, &mut rng);
        let brushes = last.perc.iter().filter(|p| p.kind == PercRole::Brush).count();
        assert!(brushes >= 3, "expected a turnaround fill, saw {brushes} brush hits");
    }

    #[test]
    fn swing_places_the_and_late_but_straight_does_not() {
        let bars = progression::two_five_one(0);
        let spb = 0.5; // 120bpm
        let mut swung = Settings::default();
        swung.density = Density::Busy;
        let mut straight = swung;
        straight.swing = false;
        // the skip note on beat 2 lands at 2/3 of a beat when swinging, 1/2 when not
        let mut r1 = Rng::new(21);
        let a = plan_bar(&bars, 0, &swung, &mut r1);
        let mut r2 = Rng::new(21);
        let b = plan_bar(&bars, 0, &straight, &mut r2);
        let skip = |p: &BarPlan| {
            p.perc.iter().find(|x| x.kind == PercRole::RideSkip).map(|x| x.offset).unwrap()
        };
        assert!((skip(&a) - (spb + spb * 2.0 / 3.0)).abs() < 1e-4);
        assert!((skip(&b) - (spb + spb * 0.5)).abs() < 1e-4);
    }

    #[test]
    fn planning_is_deterministic_for_a_given_seed() {
        let bars = progression::rhythm_full();
        let s = settings();
        let run = |seed: u64| {
            let mut rng = Rng::new(seed);
            (0..8).map(|i| plan_bar(&bars, i, &s, &mut rng)).map(|p| p.notes.len()).collect::<Vec<_>>()
        };
        assert_eq!(run(1234), run(1234));
    }
}
