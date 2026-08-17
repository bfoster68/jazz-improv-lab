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

/// A rootless voicing: 3rd, 5th and 7th, with an optional colour tone on top.
///
/// On an altered bar the natural 5 and 9 are replaced by the b13 and b9, so the
/// piano never fights the tensions the app is telling the student to play.
pub fn comp_voicing(slot: &Slot, rng: &mut Rng) -> Vec<i32> {
    let ints = slot.quality.ints();
    let mut base = midi_for(slot.root, 4);
    while base < 58 {
        base += 12;
    }
    while base > 69 {
        base -= 12;
    }
    let altered = slot.scale == Some(ALTERED);
    let fifth = if altered { 8 } else { ints[2] };
    let mut v = vec![base + ints[1], base + fifth, base + ints[3]];
    if slot.quality != Quality::Min7b5 && rng.chance(0.55) {
        v.push(base + if altered { 13 } else { 14 });
    }
    v
}

/// Everything that should sound during one bar.
#[derive(Debug, Clone, Default)]
pub struct BarPlan {
    pub notes: Vec<Event>,
    pub perc: Vec<PercEvent>,
    pub two_feel: bool,
}

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
    pub density: Density,
    pub bass: bool,
    pub bass_inst: Instrument,
    pub chords: bool,
    pub comp_inst: Instrument,
    /// A sustained horn-section pad over the changes.
    pub horns: bool,
    pub horn_inst: Instrument,
    pub drums: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            tempo: 120.0,
            meter: Meter::Four,
            swing: true,
            density: Density::Medium,
            bass: true,
            bass_inst: Instrument::Bass,
            chords: true,
            comp_inst: Instrument::Keys,
            horns: false,
            horn_inst: Instrument::Sax,
            drums: true,
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
    let swing_off = if settings.swing { spb * 2.0 / 3.0 } else { spb * 0.5 };
    let last_of_form = index % bars.len() == bars.len() - 1;

    plan.two_feel = !bar.is_split() && rng.chance(0.16);

    // ---- bass ----
    let walk = walking_bar(&bar, next_root, rng);
    if settings.bass {
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

    // ---- comping ----
    if settings.chords {
        let first = bar.chord_at(0);
        let second = bar.chord_at(2);
        let v1 = comp_voicing(&first, rng);
        let v2 = if bar.is_split() { comp_voicing(&second, rng) } else { v1.clone() };
        let vnext = comp_voicing(&next.chord_at(0), rng);
        if plan.two_feel {
            for &m in &v1 {
                plan.notes.push(Event { offset: 0.0, midi: m, gain: 0.055, dur: spb * 1.5, instrument: settings.comp_inst });
            }
        } else {
            let p = pool(settings.density);
            let hits = p[rng.below(p.len())];
            for &slot in hits {
                let beat = slot / 2;
                let on_and = slot % 2 == 1;
                let offset = beat as f32 * spb + if on_and { swing_off } else { 0.0 };
                // the "and" of 4 anticipates the next chord
                let voicing = if slot == 7 { &vnext } else if beat >= 2 { &v2 } else { &v1 };
                let (gain, dur) = if on_and { (0.053, spb * 0.65) } else { (0.060, spb * 0.85) };
                for &m in voicing {
                    plan.notes.push(Event { offset, midi: m, gain, dur, instrument: settings.comp_inst });
                }
            }
        }
    }

    // ---- horns (a sustained section pad over the changes) ----
    if settings.horns {
        let bar_dur = 4.0 * spb;
        let octave = settings.horn_inst.section_octave();
        // The horns roll their own dice, seeded per bar. Sharing the bar RNG
        // would shift every later draw, so merely toggling the section would
        // re-roll the walking bass and comping of the rest of the form — and an
        // A/B render with and without `--horns` could never isolate the horns.
        let mut horn_rng = Rng::new(0x484F_524E ^ (index as u64).wrapping_mul(0x9E37_79B9));
        let mut pad = |slot: &Slot, offset: f32, dur: f32, rng: &mut Rng| {
            for &m in &comp_voicing(slot, rng) {
                plan.notes.push(Event {
                    offset,
                    midi: m + octave,
                    gain: 0.06,
                    dur,
                    instrument: settings.horn_inst,
                });
            }
        };
        if bar.is_split() {
            pad(&bar.chord_at(0), 0.0, 2.0 * spb, &mut horn_rng);
            pad(&bar.chord_at(2), 2.0 * spb, 2.0 * spb, &mut horn_rng);
        } else {
            pad(&bar.chord_at(0), 0.0, bar_dur, &mut horn_rng);
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
    let swing_off = if settings.swing { spb * 2.0 / 3.0 } else { spb * 0.5 };
    let last_of_form = index % bars.len() == bars.len() - 1;
    let slot = bar.chord_at(0);

    // ---- bass: root on one, a lead note on three more often than not ----
    if settings.bass {
        let root = clamp_bass(nearest_octave(slot.root, 44));
        plan.notes.push(Event {
            offset: 0.0,
            midi: root,
            gain: 0.175,
            dur: spb * 1.9,
            instrument: settings.bass_inst,
        });
        if rng.chance(0.65) {
            let ints = slot.quality.ints();
            // Either a chord tone, or a half-step approach into the next root.
            let mut tone = if rng.chance(0.5) {
                clamp_bass(nearest_octave(slot.root + ints[2], root))
            } else {
                let dir = if rng.chance(0.5) { 1 } else { -1 };
                clamp_bass(nearest_octave(next_root + dir, root))
            };
            if tone == root {
                tone = clamp_bass(nearest_octave(slot.root + ints[1], root));
            }
            plan.notes.push(Event {
                offset: 2.0 * spb,
                midi: tone,
                gain: 0.15,
                dur: spb * 0.8,
                instrument: settings.bass_inst,
            });
        }
    }

    // ---- comping: six eighth-note slots; beat two carries the waltz ----
    if settings.chords {
        let v = comp_voicing(&slot, rng);
        let vnext = comp_voicing(&next.chord_at(0), rng);
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
            for &m in voicing {
                plan.notes.push(Event { offset, midi: m, gain, dur, instrument: settings.comp_inst });
            }
        }
    }

    // ---- horns: a dotted-half pad ----
    if settings.horns {
        let octave = settings.horn_inst.section_octave();
        let mut horn_rng = Rng::new(0x57_414C_545A ^ (index as u64).wrapping_mul(0x9E37_79B9));
        for &m in &comp_voicing(&slot, &mut horn_rng) {
            plan.notes.push(Event {
                offset: 0.0,
                midi: m + octave,
                gain: 0.06,
                dur: 3.0 * spb,
                instrument: settings.horn_inst,
            });
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
                .filter(|e| e.instrument == Instrument::Keys)
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
                .filter(|e| e.instrument != Settings::default().horn_inst)
                .map(|e| (e.midi, (e.offset * 1000.0) as i32))
                .collect::<Vec<_>>()
        };
        assert_eq!(band(false), band(true), "enabling horns changed the bass/comping");
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
                plan.notes.iter().all(|e| e.instrument != Instrument::Sax),
                "horns sounded while disabled"
            );
        }
        // Enabled: the chosen horn instrument appears.
        let on = Settings { horns: true, horn_inst: Instrument::Trumpet, ..Settings::default() };
        let plan = plan_bar(&bars, 0, &on, &mut rng);
        assert!(
            plan.notes.iter().any(|e| e.instrument == Instrument::Trumpet),
            "horn section produced no notes"
        );
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
