//! Pitch classes and *correct* note spelling.
//!
//! The whole library is built on one idea: a note is never just a pitch, it is a
//! pitch **plus a letter**. The #11 of G is C♯ and never D♭, even though the two
//! sound identical. Getting that right means spelling by letter arithmetic rather
//! than by lookup table, which is what [`Note::spell`] does.

use core::fmt;

/// The seven letter names, in order.
pub const LETTERS: [char; 7] = ['C', 'D', 'E', 'F', 'G', 'A', 'B'];
/// Semitone offset of each natural letter above C.
pub const LETTER_PC: [i32; 7] = [0, 2, 4, 5, 7, 9, 11];

/// Default flat-side spelling for each pitch class.
pub const FLAT_NAMES: [&str; 12] = [
    "C", "Db", "D", "Eb", "E", "F", "Gb", "G", "Ab", "A", "Bb", "B",
];
/// Default sharp-side spelling for each pitch class.
pub const SHARP_NAMES: [&str; 12] = [
    "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
];

/// Normalise any integer to `0..12`.
#[inline]
pub fn pc(n: i32) -> i32 {
    n.rem_euclid(12)
}

/// Keys conventionally written with sharps.
const SHARP_KEYS: [i32; 5] = [7, 2, 9, 4, 11]; // G D A E B

/// Should this root be spelled with sharps?
pub fn use_sharp_for_root(root: i32) -> bool {
    SHARP_KEYS.contains(&pc(root))
}

/// The conventional spelling of a *key centre* — the one with fewest accidentals.
///
/// Used for roots we derive rather than receive, so the Coltrane cycle in B reads
/// B / G / E♭ instead of B / G / D♯.
pub fn key_centre_name(p: i32) -> &'static str {
    let p = pc(p);
    if p == 6 {
        "F#"
    } else {
        FLAT_NAMES[p as usize]
    }
}

/// Plain pitch-class name using the key's sharp/flat preference.
pub fn pc_name(p: i32, use_sharp: bool) -> &'static str {
    let i = pc(p) as usize;
    if use_sharp {
        SHARP_NAMES[i]
    } else {
        FLAT_NAMES[i]
    }
}

/// A spelled note: a letter, an accidental offset, and the pitch class that results.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Note {
    /// Index into [`LETTERS`].
    pub letter: u8,
    /// Accidental in semitones: -2 = 𝄫, -1 = ♭, 0 = ♮, 1 = ♯, 2 = 𝄪.
    pub alter: i8,
}

impl Note {
    pub fn new(letter: u8, alter: i8) -> Self {
        Self {
            letter: letter % 7,
            alter,
        }
    }

    /// Parse a name such as `"Bb"`, `"F#"`, `"C"`.
    pub fn parse(s: &str) -> Option<Self> {
        let mut it = s.chars();
        let l = it.next()?.to_ascii_uppercase();
        let letter = LETTERS.iter().position(|&c| c == l)? as u8;
        let mut alter: i8 = 0;
        for c in it {
            match c {
                'b' | '\u{266D}' => alter -= 1,
                '#' | '\u{266F}' => alter += 1,
                'n' | '\u{266E}' => {}
                _ => return None,
            }
        }
        Some(Self { letter, alter })
    }

    /// The pitch class this note sounds.
    pub fn pitch_class(&self) -> i32 {
        pc(LETTER_PC[self.letter as usize] + self.alter as i32)
    }

    /// Spell `target_pc` as the given diatonic degree above `root`.
    ///
    /// `degree` is the diatonic number (1 = unison/root, 3 = a third, 5 = a fifth …);
    /// compound degrees are reduced, so 9 behaves as 2 and 11 as 4.
    ///
    /// Returns `None` when the result would need a double accidental — the caller
    /// then falls back to a plain enharmonic name, because a beginner reading
    /// "B𝄫" learns nothing.
    pub fn spell(root: Note, degree: u32, target_pc: i32) -> Option<Note> {
        if degree == 0 {
            return None;
        }
        let dn = ((degree - 1) % 7) as u8;
        let letter = (root.letter + dn) % 7;
        let natural = LETTER_PC[letter as usize];
        let mut alter = pc(target_pc - natural);
        if alter > 6 {
            alter -= 12;
        }
        if alter.abs() > 1 {
            return None;
        }
        Some(Note {
            letter,
            alter: alter as i8,
        })
    }
}

impl fmt::Display for Note {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", LETTERS[self.letter as usize])?;
        let (c, n) = if self.alter < 0 {
            ('b', -self.alter)
        } else {
            ('#', self.alter)
        };
        for _ in 0..n {
            write!(f, "{c}")?;
        }
        Ok(())
    }
}

/// Extract the diatonic number from a degree label such as `"b3"`, `"#11"`, `"bb7"`.
pub fn diatonic_of(label: &str) -> Option<u32> {
    let digits: String = label.chars().filter(|c| c.is_ascii_digit()).collect();
    if digits.is_empty() {
        return None;
    }
    let n: u32 = digits.parse().ok()?;
    if n == 0 {
        None
    } else if n > 7 {
        Some(((n - 1) % 7) + 1)
    } else {
        Some(n)
    }
}

/// Best-effort display name: spell relative to `root` when the degree is known,
/// otherwise fall back to the key's preferred plain name.
pub fn note_name(target_pc: i32, degree_label: &str, root: Option<Note>, use_sharp: bool) -> String {
    if let (Some(r), Some(d)) = (root, diatonic_of(degree_label)) {
        if let Some(n) = Note::spell(r, d, target_pc) {
            return n.to_string();
        }
    }
    pc_name(target_pc, use_sharp).to_string()
}

/// MIDI number for a pitch class in a given octave (C4 = 60 when `octave` is 4).
pub fn midi_for(pitch_class: i32, octave: i32) -> i32 {
    12 * (octave + 1) + pc(pitch_class)
}

/// Equal-tempered frequency of a MIDI note.
pub fn midi_to_freq(m: f32) -> f32 {
    440.0 * 2f32.powf((m - 69.0) / 12.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn n(s: &str) -> Note {
        Note::parse(s).unwrap()
    }

    #[test]
    fn parses_and_renders_round_trip() {
        for s in ["C", "Bb", "F#", "Eb", "A#", "Cb"] {
            assert_eq!(n(s).to_string(), s, "round trip failed for {s}");
        }
    }

    #[test]
    fn pitch_classes_are_right() {
        assert_eq!(n("C").pitch_class(), 0);
        assert_eq!(n("Bb").pitch_class(), 10);
        assert_eq!(n("F#").pitch_class(), 6);
        assert_eq!(n("Cb").pitch_class(), 11);
        assert_eq!(n("B#").pitch_class(), 0);
    }

    /// The heart of the library: altered tensions must spell on the correct letter.
    #[test]
    fn altered_tensions_spell_correctly_over_g7() {
        let g = n("G");
        // #9 of G is A#, never Bb
        assert_eq!(Note::spell(g, 9, pc(7 + 3)).unwrap().to_string(), "A#");
        // #11 of G is C#, never Db
        assert_eq!(Note::spell(g, 11, pc(7 + 6)).unwrap().to_string(), "C#");
        // b13 of G is Eb
        assert_eq!(Note::spell(g, 13, pc(7 + 8)).unwrap().to_string(), "Eb");
        // b9 of G is Ab
        assert_eq!(Note::spell(g, 9, pc(7 + 1)).unwrap().to_string(), "Ab");
    }

    #[test]
    fn upper_structure_triad_spells_as_a_real_triad() {
        // A major triad over G7 must read A C# E, not A Db E
        let g = n("G");
        assert_eq!(Note::spell(g, 9, 9).unwrap().to_string(), "A");
        assert_eq!(Note::spell(g, 11, 1).unwrap().to_string(), "C#");
        assert_eq!(Note::spell(g, 13, 4).unwrap().to_string(), "E");
    }

    #[test]
    fn double_accidentals_fall_back_rather_than_print() {
        // The 3rd of D# would be F##; we refuse it so the caller uses a plain name.
        let dsharp = n("D#");
        assert!(Note::spell(dsharp, 3, 7).is_none());
    }

    #[test]
    fn key_centres_use_the_conventional_spelling() {
        // Giant Steps in B: B / G / Eb (not D#)
        assert_eq!(key_centre_name(11), "B");
        assert_eq!(key_centre_name(pc(11 + 8)), "G");
        assert_eq!(key_centre_name(pc(11 + 4)), "Eb");
        // and in C: C / Ab / E
        assert_eq!(key_centre_name(0), "C");
        assert_eq!(key_centre_name(8), "Ab");
        assert_eq!(key_centre_name(4), "E");
    }

    #[test]
    fn compound_degrees_reduce() {
        assert_eq!(diatonic_of("b9"), Some(2));
        assert_eq!(diatonic_of("#11"), Some(4));
        assert_eq!(diatonic_of("13"), Some(6));
        assert_eq!(diatonic_of("bb7"), Some(7));
        assert_eq!(diatonic_of("1"), Some(1));
        assert_eq!(diatonic_of("x"), None);
    }

    #[test]
    fn midi_and_frequency() {
        assert_eq!(midi_for(0, 4), 60);
        assert_eq!(midi_for(9, 4), 69);
        assert!((midi_to_freq(69.0) - 440.0).abs() < 1e-3);
    }
}
