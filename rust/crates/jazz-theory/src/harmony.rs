//! Scales and chords, each carrying the degree labels a student should actually read.
//!
//! Degree labels are data, not derived, because the honest label depends on
//! context: the same pitch is the `b5` of a half-diminished chord and the `#11`
//! of a dominant. Storing them per-scale removes that ambiguity.

use crate::note::{note_name, pc, pc_name, Note};

/// A named scale: intervals in semitones plus the degree label for each.
#[derive(Debug, Clone, Copy)]
pub struct Scale {
    pub name: &'static str,
    pub ints: &'static [i32],
    pub degs: &'static [&'static str],
    pub tip: &'static str,
}

/// A named chord quality.
#[derive(Debug, Clone, Copy)]
pub struct ChordType {
    pub name: &'static str,
    /// Symbol suffix, e.g. `"maj7"`, `"m7"`, `"7"`, `"m7b5"`.
    pub symbol: &'static str,
    pub ints: &'static [i32],
    pub degs: &'static [&'static str],
    /// The scale most players reach for over this chord by default.
    pub scale: &'static str,
}

macro_rules! scale {
    ($name:literal, $ints:expr, $degs:expr, $tip:literal) => {
        Scale {
            name: $name,
            ints: &$ints,
            degs: &$degs,
            tip: $tip,
        }
    };
}

pub const SCALES: &[Scale] = &[
    scale!("Major (Ionian)", [0, 2, 4, 5, 7, 9, 11], ["1","2","3","4","5","6","7"],
        "Use over major 7 / the I chord. Bright and at home."),
    scale!("Dorian (minor)", [0, 2, 3, 5, 7, 9, 10], ["1","2","b3","4","5","6","b7"],
        "Your default minor-7 sound (the ii chord). Warm, not sad."),
    scale!("Mixolydian (dominant)", [0, 2, 4, 5, 7, 9, 10], ["1","2","3","4","5","6","b7"],
        "Over a relaxed dominant 7. The V chord when it is not tense."),
    scale!("Minor pentatonic", [0, 3, 5, 7, 10], ["1","b3","4","5","b7"],
        "Five safe notes - hard to play a wrong one. Great for blues and minor."),
    scale!("Major pentatonic", [0, 2, 4, 7, 9], ["1","2","3","5","6"],
        "Sweet and open. Try it over major and dominant chords."),
    scale!("Blues scale", [0, 3, 5, 6, 7, 10], ["1","b3","4","b5","5","b7"],
        "Minor pentatonic + the b5 blue note. The crying, vocal sound."),
    scale!("Melodic minor", [0, 2, 3, 5, 7, 9, 11], ["1","2","b3","4","5","6","7"],
        "The minor-tonic (i) sound and the parent of the altered scale."),
    scale!("Lydian dominant", [0, 2, 4, 6, 7, 9, 10], ["1","9","3","#11","5","13","b7"],
        "A dominant 7 with a #11 - for dominants that do not resolve."),
    scale!("Altered (dominant)", [0, 1, 3, 4, 6, 8, 10], ["1","b9","#9","3","#11","b13","b7"],
        "Maximum tension over a V that resolves down a 5th: b9 #9 #11 b13."),
    scale!("Phrygian dominant", [0, 1, 4, 5, 7, 8, 10], ["1","b9","3","11","5","b13","b7"],
        "The V7 sound in a minor key (5th mode of harmonic minor)."),
    scale!("Half-whole diminished", [0, 1, 3, 4, 6, 7, 9, 10], ["1","b9","#9","3","#11","5","13","b7"],
        "Spicy dominant sound: b9, #9, #11 and a natural 13."),
    scale!("Whole-half diminished", [0, 2, 3, 5, 6, 8, 9, 11], ["1","9","b3","11","b5","b13","bb7","7"],
        "The scale for a diminished 7 chord itself (not for dominants)."),
    scale!("Locrian (m7b5)", [0, 1, 3, 5, 6, 8, 10], ["1","b9","b3","11","b5","b13","b7"],
        "Over half-diminished chords - the ii of a minor key."),
];

pub const CHORD_TYPES: &[ChordType] = &[
    ChordType { name: "Major 7",        symbol: "maj7", ints: &[0,4,7,11], degs: &["1","3","5","7"],     scale: "Major (Ionian)" },
    ChordType { name: "Minor 7",        symbol: "m7",   ints: &[0,3,7,10], degs: &["1","b3","5","b7"],   scale: "Dorian (minor)" },
    ChordType { name: "Dominant 7",     symbol: "7",    ints: &[0,4,7,10], degs: &["1","3","5","b7"],    scale: "Mixolydian (dominant)" },
    ChordType { name: "Half-diminished",symbol: "m7b5", ints: &[0,3,6,10], degs: &["1","b3","b5","b7"],  scale: "Locrian (m7b5)" },
    ChordType { name: "Diminished 7",   symbol: "dim7", ints: &[0,3,6,9],  degs: &["1","b3","b5","bb7"], scale: "Whole-half diminished" },
    ChordType { name: "Major 6",        symbol: "6",    ints: &[0,4,7,9],  degs: &["1","3","5","6"],     scale: "Major (Ionian)" },
    ChordType { name: "Minor 6",        symbol: "m6",   ints: &[0,3,7,9],  degs: &["1","b3","5","6"],    scale: "Melodic minor" },
];

/// The chord qualities the play-along engine uses.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Quality {
    Maj7,
    Min7,
    Dom7,
    Min7b5,
    /// Minor tonic — voiced as m7 but functions as home.
    MinTonic,
}

impl Quality {
    pub fn chord_type(self) -> &'static ChordType {
        let name = match self {
            Quality::Maj7 => "Major 7",
            Quality::Min7 | Quality::MinTonic => "Minor 7",
            Quality::Dom7 => "Dominant 7",
            Quality::Min7b5 => "Half-diminished",
        };
        find_chord(name).expect("built-in chord type")
    }
    pub fn ints(self) -> &'static [i32] {
        self.chord_type().ints
    }
    pub fn degs(self) -> &'static [&'static str] {
        self.chord_type().degs
    }
    pub fn symbol(self) -> &'static str {
        self.chord_type().symbol
    }
    /// Default scale when the progression does not override it.
    pub fn default_scale(self) -> &'static str {
        self.chord_type().scale
    }
}

pub fn find_scale(name: &str) -> Option<&'static Scale> {
    SCALES.iter().find(|s| s.name == name)
}
pub fn find_chord(name: &str) -> Option<&'static ChordType> {
    CHORD_TYPES.iter().find(|c| c.name == name)
}

/// One rendered note: how it is spelled, what degree it is, and what it sounds.
#[derive(Debug, Clone, PartialEq)]
pub struct RenderedNote {
    pub name: String,
    pub degree: String,
    pub pitch_class: i32,
    pub semitones: i32,
}

/// Spell every note of a scale from a given root.
pub fn render_scale(scale: &Scale, root: i32, use_sharp: bool) -> Vec<RenderedNote> {
    let r = Note::parse(pc_name(root, use_sharp));
    scale
        .ints
        .iter()
        .zip(scale.degs.iter())
        .map(|(&i, &d)| RenderedNote {
            name: note_name(pc(root + i), d, r, use_sharp),
            degree: d.to_string(),
            pitch_class: pc(root + i),
            semitones: i,
        })
        .collect()
}

/// Spell every chord tone from a given root.
pub fn render_chord(ct: &ChordType, root: i32, use_sharp: bool) -> Vec<RenderedNote> {
    let r = Note::parse(pc_name(root, use_sharp));
    ct.ints
        .iter()
        .zip(ct.degs.iter())
        .map(|(&i, &d)| RenderedNote {
            name: note_name(pc(root + i), d, r, use_sharp),
            degree: d.to_string(),
            pitch_class: pc(root + i),
            semitones: i,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::note::use_sharp_for_root;

    fn names(v: &[RenderedNote]) -> Vec<String> {
        v.iter().map(|n| n.name.clone()).collect()
    }
    fn degs(v: &[RenderedNote]) -> Vec<String> {
        v.iter().map(|n| n.degree.clone()).collect()
    }

    #[test]
    fn every_scale_has_one_degree_label_per_interval() {
        for s in SCALES {
            assert_eq!(s.ints.len(), s.degs.len(), "{} is mislabelled", s.name);
        }
    }

    #[test]
    fn every_chord_has_one_degree_label_per_interval() {
        for c in CHORD_TYPES {
            assert_eq!(c.ints.len(), c.degs.len(), "{} is mislabelled", c.name);
        }
    }

    #[test]
    fn scale_intervals_are_strictly_ascending_and_in_range() {
        for s in SCALES {
            for w in s.ints.windows(2) {
                assert!(w[0] < w[1], "{} is not ascending", s.name);
            }
            assert!(s.ints.iter().all(|&i| (0..12).contains(&i)), "{}", s.name);
        }
    }

    #[test]
    fn half_diminished_shows_b5_not_sharp11() {
        let ct = find_chord("Half-diminished").unwrap();
        let v = render_chord(ct, 11, use_sharp_for_root(11)); // Bm7b5
        assert_eq!(names(&v), ["B", "D", "F", "A"]);
        assert_eq!(degs(&v), ["1", "b3", "b5", "b7"]);
    }

    #[test]
    fn diminished_seventh_shows_bb7() {
        let ct = find_chord("Diminished 7").unwrap();
        let v = render_chord(ct, 0, false);
        assert_eq!(degs(&v), ["1", "b3", "b5", "bb7"]);
        // bb7 of C would be Bbb; we fall back to a readable plain name
        assert_eq!(v[3].name, "A");
    }

    #[test]
    fn altered_scale_reads_as_alterations_not_plain_flats() {
        let s = find_scale("Altered (dominant)").unwrap();
        let v = render_scale(s, 7, false); // G altered
        assert_eq!(degs(&v), ["1", "b9", "#9", "3", "#11", "b13", "b7"]);
        assert_eq!(names(&v), ["G", "Ab", "A#", "B", "C#", "Eb", "F"]);
    }

    #[test]
    fn lydian_dominant_has_sharp_eleven_and_natural_thirteen() {
        let s = find_scale("Lydian dominant").unwrap();
        let v = render_scale(s, 2, true); // D Lydian dominant
        assert_eq!(names(&v), ["D", "E", "F#", "G#", "A", "B", "C"]);
    }

    #[test]
    fn diminished_scales_are_not_swapped() {
        let hw = find_scale("Half-whole diminished").unwrap();
        let wh = find_scale("Whole-half diminished").unwrap();
        assert_eq!(hw.ints[1], 1, "half-whole must start with a half step");
        assert_eq!(wh.ints[1], 2, "whole-half must start with a whole step");
        // a diminished 7 chord takes the whole-half scale
        assert_eq!(find_chord("Diminished 7").unwrap().scale, wh.name);
    }

    #[test]
    fn no_scale_produces_a_double_accidental_in_any_key() {
        for s in SCALES {
            for root in 0..12 {
                for n in render_scale(s, root, use_sharp_for_root(root)) {
                    assert!(
                        !n.name.contains("##") && !n.name.contains("bb"),
                        "{} in {} produced {}",
                        s.name,
                        root,
                        n.name
                    );
                }
            }
        }
    }

    #[test]
    fn rendered_pitches_match_the_declared_intervals() {
        for s in SCALES {
            for root in 0..12 {
                for n in render_scale(s, root, false) {
                    assert_eq!(n.pitch_class, pc(root + n.semitones));
                    if let Some(parsed) = Note::parse(&n.name) {
                        assert_eq!(
                            parsed.pitch_class(),
                            n.pitch_class,
                            "{} spelled {} does not sound its pitch",
                            s.name,
                            n.name
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn quality_lookup_is_consistent() {
        assert_eq!(Quality::Dom7.symbol(), "7");
        assert_eq!(Quality::Min7b5.symbol(), "m7b5");
        assert_eq!(Quality::MinTonic.ints(), &[0, 3, 7, 10]);
        assert_eq!(Quality::Maj7.default_scale(), "Major (Ionian)");
    }
}
