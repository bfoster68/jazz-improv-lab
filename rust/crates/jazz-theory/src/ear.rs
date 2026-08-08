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
}
