//! Ear-training material: intervals, chord qualities and progressions.
//!
//! The point of this module is that the *labels* and the *sounds* must agree.
//! A "Coltrane fragment" that does not actually contain a dominant resolution is
//! worse than useless, so the tests here check the harmony, not just the strings.

use crate::harmony::Quality;

#[derive(Debug, Clone, Copy)]
pub struct Interval {
    pub name: &'static str,
    pub semitones: i32,
}

pub const INTERVALS: &[Interval] = &[
    Interval { name: "Minor 2nd",   semitones: 1 },
    Interval { name: "Major 2nd",   semitones: 2 },
    Interval { name: "Minor 3rd",   semitones: 3 },
    Interval { name: "Major 3rd",   semitones: 4 },
    Interval { name: "Perfect 4th", semitones: 5 },
    Interval { name: "Tritone",     semitones: 6 },
    Interval { name: "Perfect 5th", semitones: 7 },
    Interval { name: "Minor 6th",   semitones: 8 },
    Interval { name: "Major 6th",   semitones: 9 },
    Interval { name: "Minor 7th",   semitones: 10 },
    Interval { name: "Major 7th",   semitones: 11 },
    Interval { name: "Octave",      semitones: 12 },
];

#[derive(Debug, Clone, Copy)]
pub struct EarChord {
    pub name: &'static str,
    pub ints: &'static [i32],
}

pub const BASIC_CHORDS: &[EarChord] = &[
    EarChord { name: "Major 7",                 ints: &[0, 4, 7, 11] },
    EarChord { name: "Minor 7",                 ints: &[0, 3, 7, 10] },
    EarChord { name: "Dominant 7",              ints: &[0, 4, 7, 10] },
    EarChord { name: "Half-diminished (m7b5)",  ints: &[0, 3, 6, 10] },
    EarChord { name: "Diminished 7",            ints: &[0, 3, 6, 9] },
];

pub const EXTENDED_CHORDS: &[EarChord] = &[
    EarChord { name: "Major 9",                  ints: &[0, 4, 7, 11, 14] },
    EarChord { name: "Dominant 9",               ints: &[0, 4, 7, 10, 14] },
    EarChord { name: "Dominant 7b9",             ints: &[0, 4, 7, 10, 13] },
    EarChord { name: "Dominant 7#11 (Lydian dom)", ints: &[0, 4, 7, 10, 18] },
    EarChord { name: "Dominant 7#9",             ints: &[0, 4, 7, 10, 15] },
    EarChord { name: "Minor 9",                  ints: &[0, 3, 7, 10, 14] },
    EarChord { name: "Minor-major 7",            ints: &[0, 3, 7, 11] },
];

/// A progression to recognise: a label plus (root offset, quality) pairs.
#[derive(Debug, Clone, Copy)]
pub struct EarProgression {
    pub name: &'static str,
    pub chords: &'static [(i32, Quality)],
}

use Quality::*;

pub const PROGRESSIONS: &[EarProgression] = &[
    EarProgression { name: "ii-V-I (major)",             chords: &[(2, Min7), (7, Dom7), (0, Maj7)] },
    EarProgression { name: "ii-V-i (minor)",             chords: &[(2, Min7b5), (7, Dom7), (0, MinTonic)] },
    EarProgression { name: "I-vi-ii-V (turnaround)",     chords: &[(0, Maj7), (9, Min7), (2, Min7), (7, Dom7)] },
    EarProgression { name: "12-bar blues opening",       chords: &[(0, Dom7), (5, Dom7), (0, Dom7)] },
    EarProgression { name: "Tritone-sub ii-V (iim7-bII7-I)", chords: &[(2, Min7), (1, Dom7), (0, Maj7)] },
    EarProgression { name: "Backdoor ii-V (iv-bVII7-I)", chords: &[(5, Min7), (10, Dom7), (0, Maj7)] },
    EarProgression { name: "Coltrane fragment (I-bIII7-bVI)", chords: &[(0, Maj7), (3, Dom7), (8, Maj7)] },
];

/// Which pool a question is drawn from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Intervals,
    Chords,
    Extended,
    Progressions,
}

impl Mode {
    pub const ALL: [Mode; 4] = [Mode::Intervals, Mode::Chords, Mode::Extended, Mode::Progressions];

    pub fn label(self) -> &'static str {
        match self {
            Mode::Intervals => "Intervals",
            Mode::Chords => "Chord qualities",
            Mode::Extended => "Extended & altered",
            Mode::Progressions => "Progressions",
        }
    }

    /// Every possible answer in this pool, in a stable order.
    pub fn answers(self) -> Vec<&'static str> {
        match self {
            Mode::Intervals => INTERVALS.iter().map(|i| i.name).collect(),
            Mode::Chords => BASIC_CHORDS.iter().map(|c| c.name).collect(),
            Mode::Extended => EXTENDED_CHORDS.iter().map(|c| c.name).collect(),
            Mode::Progressions => PROGRESSIONS.iter().map(|p| p.name).collect(),
        }
    }
}

/// A drawn question: what the answer is, what to offer, and the root to voice it from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Question {
    pub mode: Mode,
    pub answer: &'static str,
    pub choices: Vec<&'static str>,
    /// MIDI root, wandered so absolute pitch cannot be used as a shortcut.
    pub root: i32,
}

/// Draw a question.
///
/// Randomness is supplied by the caller so this stays a pure function and can be
/// tested exhaustively. `previous` is the last answer given in this mode; the draw
/// never repeats it back to back, because a quiz that can ask the same thing twice
/// in a row feels broken even when it is technically random.
pub fn draw(mode: Mode, previous: Option<&str>, r_index: u32, r_root: u32) -> Question {
    let choices = mode.answers();
    let n = choices.len();
    let mut idx = (r_index as usize) % n;
    if n > 1 && Some(choices[idx]) == previous {
        idx = (idx + 1 + (r_index as usize) % (n - 1)) % n;
        if Some(choices[idx]) == previous {
            idx = (idx + 1) % n;
        }
    }
    Question {
        mode,
        answer: choices[idx],
        choices,
        root: 55 + (r_root % 10) as i32,
    }
}

/// The MIDI notes a question should sound, as (delay-in-slots, pitches) pairs.
///
/// Slots are sequential: an interval sounds two notes one after another, a
/// progression sounds one chord per slot, a single chord sounds in slot 0.
pub fn voicing(q: &Question) -> Vec<(usize, Vec<i32>)> {
    match q.mode {
        Mode::Intervals => INTERVALS
            .iter()
            .find(|i| i.name == q.answer)
            .map(|i| vec![(0, vec![q.root]), (1, vec![q.root + i.semitones])])
            .unwrap_or_default(),
        Mode::Chords | Mode::Extended => {
            let pool = if q.mode == Mode::Chords { BASIC_CHORDS } else { EXTENDED_CHORDS };
            pool.iter()
                .find(|c| c.name == q.answer)
                .map(|c| vec![(0, c.ints.iter().map(|i| q.root + i).collect())])
                .unwrap_or_default()
        }
        Mode::Progressions => PROGRESSIONS
            .iter()
            .find(|p| p.name == q.answer)
            .map(|p| {
                p.chords
                    .iter()
                    .enumerate()
                    .map(|(n, (off, qual))| {
                        (n, qual.ints().iter().map(|i| q.root + off + i).collect())
                    })
                    .collect()
            })
            .unwrap_or_default(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::note::pc;

    #[test]
    fn intervals_are_ordered_and_complete() {
        for (i, iv) in INTERVALS.iter().enumerate() {
            assert_eq!(iv.semitones, i as i32 + 1, "{} is misnumbered", iv.name);
        }
    }

    #[test]
    fn chord_intervals_start_on_the_root_and_ascend() {
        for set in [BASIC_CHORDS, EXTENDED_CHORDS] {
            for c in set {
                assert_eq!(c.ints[0], 0, "{} must start on the root", c.name);
                for w in c.ints.windows(2) {
                    assert!(w[0] < w[1], "{} is not ascending", c.name);
                }
            }
        }
    }

    #[test]
    fn extended_chord_tensions_are_what_their_names_claim() {
        let by = |n: &str| EXTENDED_CHORDS.iter().find(|c| c.name == n).unwrap();
        assert_eq!(*by("Dominant 9").ints.last().unwrap(), 14);          // natural 9
        assert_eq!(*by("Dominant 7b9").ints.last().unwrap(), 13);        // b9
        assert_eq!(*by("Dominant 7#9").ints.last().unwrap(), 15);        // #9
        assert_eq!(*by("Dominant 7#11 (Lydian dom)").ints.last().unwrap(), 18); // #11
        // minor-major 7 keeps a minor 3rd with a major 7th
        let mm = by("Minor-major 7");
        assert!(mm.ints.contains(&3) && mm.ints.contains(&11));
    }

    #[test]
    fn progression_roman_numerals_match_their_roots() {
        let find = |n: &str| PROGRESSIONS.iter().find(|p| p.name.starts_with(n)).unwrap();
        // bII7 must be one semitone above the tonic
        assert_eq!(find("Tritone-sub").chords[1].0, 1);
        // bVII7 is ten semitones above
        assert_eq!(find("Backdoor").chords[1].0, 10);
        // iv is a fourth above
        assert_eq!(find("Backdoor").chords[0].0, 5);
    }

    /// The Coltrane fragment must contain a genuine V-I, or it teaches nothing.
    #[test]
    fn coltrane_fragment_contains_a_real_dominant_resolution() {
        let p = PROGRESSIONS.iter().find(|p| p.name.starts_with("Coltrane")).unwrap();
        let (dom_root, dom_q) = p.chords[1];
        let (target_root, _) = p.chords[2];
        assert_eq!(dom_q, Dom7);
        assert_eq!(
            pc(dom_root + 5),
            pc(target_root),
            "the dominant must resolve down a fifth into the next chord"
        );
    }

    #[test]
    fn minor_two_five_uses_a_half_diminished_two() {
        let p = PROGRESSIONS.iter().find(|p| p.name.starts_with("ii-V-i")).unwrap();
        assert_eq!(p.chords[0].1, Min7b5);
        assert_eq!(p.chords[2].1, MinTonic);
    }

    #[test]
    fn names_are_unique_within_each_answer_pool() {
        for set in [BASIC_CHORDS, EXTENDED_CHORDS] {
            let mut names: Vec<&str> = set.iter().map(|c| c.name).collect();
            names.sort_unstable();
            let before = names.len();
            names.dedup();
            assert_eq!(before, names.len(), "duplicate answer in a chord pool");
        }
        let mut p: Vec<&str> = PROGRESSIONS.iter().map(|x| x.name).collect();
        p.sort_unstable();
        let before = p.len();
        p.dedup();
        assert_eq!(before, p.len(), "duplicate progression name");
    }

    #[test]
    fn every_mode_offers_a_non_trivial_pool() {
        for m in Mode::ALL {
            let a = m.answers();
            assert!(a.len() >= 4, "{:?} has only {} answers", m, a.len());
            assert!(!m.label().is_empty());
        }
    }

    #[test]
    fn a_drawn_answer_is_always_one_of_the_offered_choices() {
        for m in Mode::ALL {
            for r in 0..200u32 {
                let q = draw(m, None, r, r * 7);
                assert!(q.choices.contains(&q.answer));
                assert!((55..65).contains(&q.root), "root {} out of range", q.root);
            }
        }
    }

    #[test]
    fn a_draw_never_repeats_the_previous_answer() {
        for m in Mode::ALL {
            let mut previous: Option<&'static str> = None;
            for r in 0..500u32 {
                let q = draw(m, previous, r * 13 + 5, r);
                assert_ne!(
                    Some(q.answer),
                    previous,
                    "{:?} asked the same question twice in a row",
                    m
                );
                previous = Some(q.answer);
            }
        }
    }

    #[test]
    fn draws_cover_the_whole_pool_rather_than_favouring_a_few() {
        for m in Mode::ALL {
            let mut seen = std::collections::HashSet::new();
            let mut previous = None;
            for r in 0..600u32 {
                let q = draw(m, previous, r, r);
                seen.insert(q.answer);
                previous = Some(q.answer);
            }
            assert_eq!(seen.len(), m.answers().len(), "{:?} never asked some answers", m);
        }
    }

    #[test]
    fn intervals_sound_two_notes_the_right_distance_apart() {
        for iv in INTERVALS {
            let q = Question { mode: Mode::Intervals, answer: iv.name, choices: vec![], root: 60 };
            let v = voicing(&q);
            assert_eq!(v.len(), 2, "{} should sound two notes", iv.name);
            assert_eq!(v[0].1[0], 60);
            assert_eq!(v[1].1[0] - v[0].1[0], iv.semitones, "{} is mis-voiced", iv.name);
            assert!(v[1].0 > v[0].0, "the second note must come after the first");
        }
    }

    #[test]
    fn chords_sound_all_their_tones_at_once() {
        for c in BASIC_CHORDS.iter().chain(EXTENDED_CHORDS) {
            let mode = if BASIC_CHORDS.iter().any(|b| b.name == c.name) {
                Mode::Chords
            } else {
                Mode::Extended
            };
            let q = Question { mode, answer: c.name, choices: vec![], root: 60 };
            let v = voicing(&q);
            assert_eq!(v.len(), 1, "{} should sound as one simultaneous chord", c.name);
            assert_eq!(v[0].1.len(), c.ints.len());
            assert_eq!(v[0].1[0], 60);
        }
    }

    #[test]
    fn progressions_sound_one_chord_per_slot_in_order() {
        for p in PROGRESSIONS {
            let q = Question { mode: Mode::Progressions, answer: p.name, choices: vec![], root: 60 };
            let v = voicing(&q);
            assert_eq!(v.len(), p.chords.len(), "{} has the wrong chord count", p.name);
            for (n, (slot, notes)) in v.iter().enumerate() {
                assert_eq!(*slot, n, "{} plays out of order", p.name);
                assert!(!notes.is_empty());
                // the chord must be built on its stated root offset
                assert_eq!(notes[0], 60 + p.chords[n].0);
            }
        }
    }

    #[test]
    fn every_answer_in_every_pool_can_actually_be_voiced() {
        for m in Mode::ALL {
            for answer in m.answers() {
                let q = Question { mode: m, answer, choices: vec![], root: 60 };
                assert!(!voicing(&q).is_empty(), "{answer} produces no sound");
            }
        }
    }
}
