//! EXPERIMENTAL: an idea generator for the new jazz musician.
//!
//! Short, singable phrases over the changes — material to read off the sheet,
//! sing along with, and steal — built from what the curriculum teaches:
//! chord tones on the downbeats, scale-wise motion between them
//! (using each bar's *suggested* scale, so an altered V7 gets altered lines),
//! chromatic approaches into the next chord, enclosure pickups at phrase
//! starts, and four-bar phrasing with real breaths.
//!
//! The generator is stateless and deterministic: every bar's line is a pure
//! function of (form, bar index, settings), seeded per bar. That keeps it
//! consistent with the rest of the engine — a resync or a re-listen plays the
//! same solo, toggling it never re-rolls the band, and continuity across
//! barlines needs no memory because each bar can compute its neighbour's
//! downbeat target for itself.
//!
//! This feature may or may not become permanent; everything specific to it
//! lives in this module plus one `if settings.solo` hook per meter.

use crate::engine::{nearest_octave, Event, Rng, Settings};
use crate::voice::Instrument;
use jazz_theory::harmony::find_scale;
use jazz_theory::note::pc;
use jazz_theory::progression::Bar;

/// A comfortable solo register for each instrument, MIDI note numbers.
fn range_of(i: Instrument) -> (i32, i32) {
    match i {
        Instrument::Bass => (36, 60),
        Instrument::Trombone => (46, 70),
        Instrument::Guitar => (55, 81),
        Instrument::Sax => (55, 80),
        Instrument::Keys => (60, 86),
        Instrument::Piano => (60, 88),
        Instrument::Trumpet => (60, 84),
        Instrument::Vibes => (60, 88),
    }
}

/// The note the line aims to land on at the top of `index`: a chord tone of
/// that bar — guide tones (3rd and 7th) most often — placed in the
/// instrument's register with a little seeded drift so the lines sit in
/// different places on different passes through the form.
fn target_note(bars: &[Bar], index: usize, inst: Instrument) -> i32 {
    let slot = bars[index % bars.len()].chord_at(0);
    let ints = slot.quality.ints();
    let mut rng = Rng::new(0x7A26_0001 ^ (index as u64).wrapping_mul(0x9E37_79B9));
    let choice = match rng.below(6) {
        0 | 1 => ints[1], // the 3rd
        2 | 3 => ints[3], // the 7th
        4 => ints[2],     // the 5th
        _ => 0,           // the root
    };
    let (lo, hi) = range_of(inst);
    let centre = (lo + hi) / 2 + rng.below(5) as i32 - 2;
    let mut m = nearest_octave(pc(slot.root + choice), centre);
    while m < lo {
        m += 12;
    }
    while m > hi {
        m -= 12;
    }
    m
}

/// Generate one bar of the solo. Empty when the phrasing calls for a breath.
pub fn solo_bar(bars: &[Bar], index: usize, settings: &Settings) -> Vec<Event> {
    if bars.is_empty() || !settings.solo {
        return Vec::new();
    }
    let beats = settings.meter.beats();
    let slots = beats * 2; // eighth-note grid
    let spb = 60.0 / settings.tempo.max(20.0);
    let swing_off = if settings.swing { spb * 2.0 / 3.0 } else { spb * 0.5 };
    let mut rng = Rng::new(0x50F0_CAFE ^ (index as u64).wrapping_mul(0x9E37_79B9));
    let inst = settings.solo_inst;
    let (lo, hi) = range_of(inst);
    let chord = bars[index % bars.len()].chord_at(0);
    let goal = target_note(bars, index + 1, inst);

    let offset_of =
        |s: usize| (s / 2) as f32 * spb + if s % 2 == 1 { swing_off } else { 0.0 };

    // (eighth slot, midi) picks, in slot order.
    let mut picks: Vec<(usize, i32)> = Vec::new();

    if index % 4 == 3 {
        // The breath bar of the four-bar phrase: silence, except sometimes an
        // enclosure pickup — above, then chromatically below — that resolves
        // onto the next phrase's opening chord tone.
        if rng.chance(0.7) {
            picks.push((slots - 2, (goal + 2).min(hi)));
            picks.push((slots - 1, (goal - 1).max(lo)));
        }
    } else {
        // The pitch classes of this bar's *suggested* scale (altered dominants
        // get altered lines), falling back to plain chord tones.
        let scale_pcs: Vec<i32> = find_scale(chord.scale_name())
            .map(|s| s.ints.iter().map(|i| pc(chord.root + i)).collect())
            .unwrap_or_else(|| chord.quality.ints().iter().map(|i| pc(chord.root + i)).collect());

        let mut cur = target_note(bars, index, inst);
        picks.push((0, cur));
        for s in 1..slots {
            if s == slots - 1 {
                // Lean into the next bar: a chromatic approach one half step
                // from its downbeat target.
                if rng.chance(0.8) {
                    let ap = if cur >= goal { goal + 1 } else { goal - 1 };
                    picks.push((s, ap.clamp(lo, hi)));
                }
                break;
            }
            if !rng.chance(0.78) {
                continue; // space inside the line
            }
            let toward = (goal - cur).signum();
            let dir = if toward == 0 || !rng.chance(0.75) {
                if toward >= 0 { -1 } else { 1 } // contour: sometimes turn away
            } else {
                toward
            };
            if rng.chance(0.15) {
                // A small leap onto a chord tone.
                let ints = chord.quality.ints();
                let t = ints[rng.below(ints.len())];
                let mut leap = nearest_octave(pc(chord.root + t), cur + dir * 5);
                if leap == cur {
                    leap += dir * 12;
                }
                cur = leap.clamp(lo, hi);
            } else {
                // Step to the next scale tone; where the scale has a gap the
                // half step becomes a chromatic passing tone.
                let mut next = cur;
                for k in 1..=3 {
                    let cand = cur + dir * k;
                    if scale_pcs.contains(&pc(cand)) {
                        next = cand;
                        break;
                    }
                }
                if next == cur {
                    next = cur + dir;
                }
                cur = next.clamp(lo, hi);
            }
            picks.push((s, cur));
        }
    }

    // Durations come from the gaps, so held notes ring to the next attack and
    // swung eighth pairs keep the grid's lilt.
    let bar_len = beats as f32 * spb;
    let mut out = Vec::with_capacity(picks.len());
    for (n, &(s, m)) in picks.iter().enumerate() {
        let start = offset_of(s);
        let end = picks.get(n + 1).map(|&(s2, _)| offset_of(s2)).unwrap_or(bar_len);
        let dur = ((end - start) * 0.92).max(0.12);
        let strong = s % 2 == 0;
        let gain = if strong { 0.105 } else { 0.09 } + rng.next_f32() * 0.01;
        out.push(Event { offset: start, midi: m, gain, dur, instrument: inst });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::Settings;
    use jazz_theory::progression::{self, Meter};

    fn solo_settings() -> Settings {
        Settings { solo: true, ..Settings::default() }
    }

    #[test]
    fn the_solo_is_deterministic_and_stateless() {
        let bars = progression::blues(5);
        let s = solo_settings();
        for i in 0..12 {
            assert_eq!(solo_bar(&bars, i, &s), solo_bar(&bars, i, &s), "bar {i} changed its mind");
        }
    }

    #[test]
    fn downbeats_land_on_chord_tones() {
        let bars = progression::two_five_one(0);
        let s = solo_settings();
        for i in 0..32 {
            if i % 4 == 3 {
                continue; // breath bar
            }
            let line = solo_bar(&bars, i, &s);
            let first = line.first().expect("a playing bar must open with a note");
            assert_eq!(first.offset, 0.0);
            let chord = bars[i % bars.len()].chord_at(0);
            let tones: Vec<i32> =
                chord.quality.ints().iter().map(|t| pc(chord.root + t)).collect();
            assert!(
                tones.contains(&pc(first.midi)),
                "bar {i}: downbeat {} is not a chord tone of {}",
                first.midi,
                chord.symbol()
            );
        }
    }

    #[test]
    fn the_line_stays_in_register_and_inside_the_bar_in_both_meters() {
        let bars = progression::blues(5);
        for meter in [Meter::Four, Meter::Three] {
            let s = Settings { meter, ..solo_settings() };
            let (lo, hi) = range_of(s.solo_inst);
            let bar_len = s.bar_len();
            for i in 0..24 {
                for e in solo_bar(&bars, i, &s) {
                    assert!(e.offset >= 0.0 && e.offset < bar_len, "{meter:?}: outside the bar");
                    assert!((lo..=hi).contains(&e.midi), "{meter:?}: {} out of register", e.midi);
                }
            }
        }
    }

    #[test]
    fn the_phrasing_breathes() {
        let bars = progression::blues(5);
        let s = solo_settings();
        let counts: Vec<usize> = (0..16).map(|i| solo_bar(&bars, i, &s).len()).collect();
        assert!(
            counts.iter().any(|&c| c <= 2),
            "no breath anywhere in four phrases: {counts:?}"
        );
        assert!(
            counts.iter().any(|&c| c >= 4),
            "the line never actually plays: {counts:?}"
        );
    }

    #[test]
    fn approaches_resolve_by_half_step_onto_the_next_downbeat() {
        let bars = progression::two_five_one(0);
        let s = solo_settings();
        let slots = s.meter.beats() * 2;
        let spb = 60.0 / s.tempo;
        let swing_off = spb * 2.0 / 3.0;
        let approach_offset = ((slots - 1) / 2) as f32 * spb + swing_off;
        let mut resolutions = 0;
        for i in 0..32 {
            if i % 4 == 3 || (i + 1) % 4 == 3 {
                continue;
            }
            let this = solo_bar(&bars, i, &s);
            let next = solo_bar(&bars, i + 1, &s);
            let (Some(last), Some(first)) = (this.last(), next.first()) else { continue };
            if (last.offset - approach_offset).abs() < 1e-4 {
                assert!(
                    (last.midi - first.midi).abs() <= 1,
                    "bar {i}: approach {} left the next downbeat {} hanging",
                    last.midi,
                    first.midi
                );
                resolutions += 1;
            }
        }
        assert!(resolutions > 4, "the approach figure almost never appeared");
    }
}
