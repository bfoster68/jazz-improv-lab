//! Generated practice drills.
//!
//! Every drill is stored as **absolute semitones above the key root**, never as a
//! bare pitch class. That is deliberate: a drill that says "a half step below the
//! target" has to actually sound a half step below it, in all twelve keys. Storing
//! pitch classes alone silently destroys the contour — an enclosure turns into a
//! major seventh leap — which is exactly the bug this design prevents.


use crate::note::{note_name, pc, pc_name, use_sharp_for_root, Note};
use crate::progression::Level;

/// One note of a drill: what to play, how it is spelled, and what it means.
#[derive(Debug, Clone, PartialEq)]
pub struct DrillNote {
    pub name: String,
    pub degree: String,
    /// Semitones above the key root — preserves register and contour.
    pub semitones: i32,
    pub midi: i32,
}

#[derive(Debug, Clone)]
pub struct Drill {
    pub id: &'static str,
    pub title: String,
    pub level: Level,
    /// What the drill teaches, in one or two sentences.
    pub concept: String,
    /// The bars/labels shown above the notes.
    pub bars: Vec<String>,
    pub notes: Vec<DrillNote>,
    /// How to practise it.
    pub how: String,
}

/// Builder that keeps the key, spelling preference and register in one place.
struct Ctx {
    key: i32,
    use_sharp: bool,
    root_midi: i32,
    notes: Vec<DrillNote>,
}

impl Ctx {
    fn new(key: i32) -> Self {
        let key = pc(key);
        Self {
            key,
            use_sharp: use_sharp_for_root(key),
            root_midi: crate::note::midi_for(key, 4),
            notes: Vec::new(),
        }
    }
    fn name_of(&self, p: i32) -> &'static str {
        pc_name(p, self.use_sharp)
    }
    /// Push a note `semi` semitones above the key root, labelled as `deg` relative
    /// to the *local chord root* `local_root` (so spelling follows the harmony).
    fn push(&mut self, semi: i32, deg: &str, local_root: i32) {
        self.push_named(semi, deg, self.name_of(local_root));
    }
    /// As `push`, but with an explicit spelling for the local root — used where the
    /// root is derived (a bII, a distant key centre) and needs its own convention.
    fn push_named(&mut self, semi: i32, deg: &str, root_name: &str) {
        let p = pc(self.key + semi);
        let root = Note::parse(root_name);
        self.notes.push(DrillNote {
            name: note_name(p, deg, root, self.use_sharp),
            degree: deg.to_string(),
            semitones: semi,
            midi: self.root_midi + semi,
        });
    }
    fn ii(&self) -> i32 { pc(self.key + 2) }
    fn v(&self) -> i32 { pc(self.key + 7) }
    fn i(&self) -> i32 { self.key }
}

/// Every drill the app can generate, in teaching order.
pub const DRILL_IDS: &[&str] = &[
    "guidetone", "enclosure", "251lick", "scaleseq", "pentpattern",
    "digital", "39arp", "bebop", "tritonesub",
    "upperstructure", "altered", "pentsuper", "coltrane", "motif",
];

/// Build a drill by id, in the given key.
pub fn build(id: &str, key: i32) -> Option<Drill> {
    let mut c = Ctx::new(key);
    let (ii, v, i) = (c.ii(), c.v(), c.i());
    let kn = c.name_of(i).to_string();
    let iin = c.name_of(ii).to_string();
    let vn = c.name_of(v).to_string();

    let (level, title, concept, bars, how) = match id {
        "guidetone" => {
            // Voice 1: b7 of ii -> 3 of V -> 7 of I  (in C: C B B)
            c.push(12, "b7", ii); c.push(11, "3", v); c.push(11, "7", i);
            // Voice 2: b3 of ii -> b7 of V -> 3 of I (in C: F F E)
            c.push(5, "b3", ii);  c.push(5, "b7", v); c.push(4, "3", i);
            (Level::Beginner,
             "Guide-tone line over ii-V-I".to_string(),
             "Two voices, each barely moving: every chord's 7th slides down a half step to become the next chord's 3rd. This alone sounds like jazz.".to_string(),
             vec![format!("{iin}m7"), format!("{vn}7"), format!("{kn}maj7")],
             "The first three notes are voice 1, the last three voice 2. Each moves by a half step or not at all. Sing each voice, then play it in all 12 keys.".to_string())
        }
        "enclosure" => {
            c.push(5, "11", i);  // a half step above the 3rd
            c.push(3, "#9", i);  // a half step below it
            c.push(4, "3", i);   // the target
            (Level::Beginner,
             format!("Enclosure into the 3rd of {kn}maj7"),
             "Surround the target from a half step above and a half step below, then resolve onto it.".to_string(),
             vec!["half above".into(), "half below".into(), "TARGET".into()],
             "Play it slowly: the two neighbours squeeze in from either side and the arrival releases the tension. Try enclosing the 7th, 5th and root too.".to_string())
        }
        "251lick" => {
            c.push(5,"b3",ii); c.push(9,"5",ii); c.push(12,"b7",ii); c.push(14,"1",ii);
            c.push(17,"b7",v); c.push(16,"13",v); c.push(15,"b13",v); c.push(11,"3",v);
            c.push(14,"9",i);  c.push(17,"11",i); c.push(15,"#9",i);  c.push(16,"3",i);
            (Level::Beginner,
             format!("A ii-V-I bebop lick in {kn}"),
             "Chord tones on the strong beats, chromatic approach notes squeezed between them.".to_string(),
             vec![format!("{iin}m7"), format!("{vn}7"), format!("{kn}maj7")],
             format!("Play it slowly and legato, then swing it. It ends by enclosing the 3rd of {kn} from above and below."))
        }
        "scaleseq" => {
            const MAJ: [i32; 7] = [0, 2, 4, 5, 7, 9, 11];
            for idx in 0..6 {
                let a = MAJ[idx];
                let b = MAJ[(idx + 2) % 7] + if idx + 2 >= 7 { 12 } else { 0 };
                let da = crate::degree_of_major(MAJ[idx]);
                let db = crate::degree_of_major(MAJ[(idx + 2) % 7]);
                c.push(a, da, i);
                c.push(b, db, i);
            }
            (Level::Beginner,
             format!("Scale in 3rds - {kn} major"),
             "A core technical pattern that turns a scale into melodic shapes.".to_string(),
             vec!["1-3".into(), "2-4".into(), "3-5 ...".into()],
             "Every pair is a true ascending 3rd. Practise it ascending and descending, then in 4ths and 6ths.".to_string())
        }
        "pentpattern" => {
            const PENT: [i32; 5] = [0, 3, 5, 7, 10];
            for cell in 0..3 {
                for step in [0usize, 1, 2, 1] {
                    let idx = cell + step;
                    let oct = (idx / 5) as i32 * 12;
                    let s = PENT[idx % 5];
                    c.push(s + oct, crate::degree_of_minor_pent(s), i);
                }
            }
            (Level::Beginner,
             format!("Pentatonic 4-note pattern in {kn} minor pentatonic"),
             "A gateway to fluent, bluesy lines.".to_string(),
             vec!["cell 1".into(), "cell 2".into(), "cell 3".into()],
             "Each cell starts one scale step higher, so the pattern really climbs. Then displace it rhythmically.".to_string())
        }
        "digital" => {
            c.push(2,"1",ii); c.push(4,"2",ii); c.push(5,"b3",ii); c.push(9,"5",ii);
            c.push(7,"1",v);  c.push(9,"2",v);  c.push(11,"3",v);  c.push(14,"5",v);
            c.push(12,"1",i); c.push(14,"2",i); c.push(16,"3",i);  c.push(19,"5",i);
            (Level::Intermediate,
             format!("The 1-2-3-5 digital pattern through ii-V-I in {kn}"),
             "Outline each chord melodically - you sound like you are spelling every change.".to_string(),
             vec![format!("{iin}m7"), format!("{vn}7"), format!("{kn}maj7")],
             "Each chord uses its own scale (note the b3 on the minor chord). Run it through all twelve keys.".to_string())
        }
        "39arp" => {
            c.push(5,"b3",ii); c.push(9,"5",ii);  c.push(12,"b7",ii); c.push(16,"9",ii);
            c.push(11,"3",v);  c.push(14,"5",v);  c.push(17,"b7",v);  c.push(21,"9",v);
            c.push(16,"3",i);  c.push(19,"5",i);  c.push(23,"7",i);   c.push(26,"9",i);
            (Level::Intermediate,
             format!("3-to-9 arpeggios over ii-V-I in {kn}"),
             "Start on the 3rd and stack 3-5-7-9: you catch both guide tones plus a sweet 9th.".to_string(),
             vec![format!("{iin}m7"), format!("{vn}7"), format!("{kn}maj7")],
             "The 3rds and 7ths glue the chords together; the 9ths add colour.".to_string())
        }
        "bebop" => {
            for (s, d) in [(0,"1"),(2,"9"),(4,"3"),(5,"11"),(7,"5"),(9,"13"),(10,"b7"),(11,"7")] {
                c.push(7 + s, d, v);
            }
            c.push(16, "3", i);
            let passing = note_name(pc(v + 11), "7", Note::parse(&vn), c.use_sharp);
            (Level::Intermediate,
             format!("The dominant bebop scale over {vn}7"),
             "Add a chromatic passing tone between the b7 and the root, so that starting on a chord tone every downbeat lands on a chord tone.".to_string(),
             vec![format!("{vn}7"), "8-note scale".into(), format!("-> {kn}maj7")],
             format!("The chord tones (1 3 5 b7) all fall on the beat; the passing {passing} fills the gap. This is the rhythmic secret of bebop lines."))
        }
        "tritonesub" => {
            // The bII is a flat-side function (its root falls chromatically into the I),
            // so it is always spelled flat: Gb7 in F, never F#7.
            let sub_name = pc_name(i + 1, false).to_string();
            c.push(2,"1",ii); c.push(5,"b3",ii); c.push(9,"5",ii); c.push(12,"b7",ii);
            c.push_named(11,"b7",&sub_name); c.push_named(8,"5",&sub_name);
            c.push_named(5,"3",&sub_name);   c.push_named(1,"1",&sub_name);
            c.push(0,"1",i); c.push(4,"3",i);
            (Level::Intermediate,
             format!("Tritone substitution: {iin}m7 -> {sub_name}7 -> {kn}maj7"),
             format!("Replace the {vn}7 with the dominant a tritone away - the two share the same tritone, with the 3rd and 7th swapping roles."),
             vec![format!("{iin}m7"), format!("{sub_name}7 (sub)"), format!("{kn}maj7")],
             format!("{sub_name}7 carries the same tension pair as {vn}7, so it resolves the same way - but the root slides down a half step for a smoother cadence."))
        }
        "upperstructure" => {
            c.push(9,"9",v); c.push(13,"#11",v); c.push(16,"13",v);
            c.push(16,"3",i);
            let triad = note_name(pc(v + 2), "9", Note::parse(&vn), c.use_sharp);
            (Level::Advanced,
             format!("Upper-structure triad over {vn}7"),
             "Play a major triad a whole step above the root: it spells the 9, #11 and 13 - the bright, *un*altered Lydian-dominant sound.".to_string(),
             vec![format!("{vn}7"), format!("{triad} major triad"), format!("{kn}maj7")],
             format!("Its top note is also the 3rd of {kn}maj7, so it resolves by common tone - the same note, reheard."))
        }
        "altered" => {
            for (s, d) in [(10,"b7"),(8,"b13"),(6,"#11"),(4,"3"),(3,"#9"),(1,"b9"),(0,"1")] {
                c.push(7 + s, d, v);
            }
            c.push(4, "3", i);
            (Level::Advanced,
             format!("Altered scale line over {vn}7alt"),
             format!("The 7th mode of melodic minor supplies b9, #9, #11 and b13 - every note that pulls into the {kn}."),
             vec![format!("{vn}7alt"), "descending".into(), format!("-> {kn}maj7")],
             format!("Every altered tension resolves by a half step into a chord tone of {kn}maj7. Play it slowly and hear the pull."))
        }
        "pentsuper" => {
            let pent_root = note_name(pc(i + 2), "9", Note::parse(&kn), c.use_sharp);
            for (s, d) in [(2,"9"),(4,"3"),(6,"#11"),(9,"13"),(11,"7"),(14,"9")] {
                c.push(s, d, i);
            }
            (Level::Advanced,
             format!("Pentatonic superimposition over {kn}maj7"),
             format!("Lay a {pent_root} major pentatonic (a whole step up) over the chord to get 9, 3, #11, 13 and 7 - a shimmering Lydian colour."),
             vec![format!("{kn}maj7"), format!("{pent_root} maj pentatonic"), "Lydian colour".into()],
             "A major pentatonic a whole step above any major 7 chord gives its brightest colours. Resolve back to a chord tone to land it.".to_string())
        }
        "coltrane" => {
            let centres = crate::progression::coltrane_centre_names(i);
            let mut reference = 0i32;
            for (n, &offset) in [0i32, 8, 4].iter().enumerate() {
                // keep each cell near the last instead of leaping an octave
                let mut base = offset;
                while base - reference > 6 { base -= 12; }
                while base - reference < -6 { base += 12; }
                for s in [0, 2, 4, 7] {
                    c.push_named(base + s, crate::degree_of_major(s), centres[n]);
                }
                reference = base;
            }
            (Level::Advanced,
             format!("Coltrane major-3rd cycle in {}", centres[0]),
             "Key centres move by major thirds (the \"Giant Steps\" matrix). Run a clean 1-2-3-5 cell through each.".to_string(),
             vec![format!("{}maj7", centres[0]), format!("{}maj7", centres[1]), format!("{}maj7", centres[2])],
             format!("Practise the 1-2-3-5 cell cleanly through {}, {} and {} until the changes feel natural.", centres[0], centres[1], centres[2]))
        }
        "motif" => {
            c.push(2,"1",ii); c.push(4,"2",ii); c.push(5,"b3",ii);
            c.push(7,"1",v);  c.push(9,"2",v);  c.push(11,"3",v);
            c.push(12,"1",i); c.push(14,"2",i); c.push(16,"3",i);
            (Level::Advanced,
             format!("Motivic development through ii-V-I in {kn}"),
             "Take one three-note shape and refit it to each chord - the seed of telling a story.".to_string(),
             vec![format!("{iin}m7"), format!("{vn}7"), format!("{kn}maj7")],
             "Same contour, refitted to each chord. Now invert it, stretch its rhythm, or start it on a different beat - that is development.".to_string())
        }
        _ => return None,
    };

    Some(Drill {
        id: DRILL_IDS.iter().find(|&&x| x == id).copied()?,
        title,
        level,
        concept,
        bars,
        notes: c.notes,
        how,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const KEYS: [i32; 6] = [0, 3, 5, 7, 9, 11];

    fn d(id: &str, key: i32) -> Drill {
        build(id, key).unwrap_or_else(|| panic!("{id} failed to build"))
    }

    #[test]
    fn every_drill_builds_in_every_key() {
        for id in DRILL_IDS {
            for key in 0..12 {
                let dr = d(id, key);
                assert!(!dr.notes.is_empty(), "{id} produced no notes in key {key}");
                assert!(!dr.bars.is_empty());
                assert!(!dr.concept.is_empty() && !dr.how.is_empty());
            }
        }
    }

    #[test]
    fn no_drill_ever_prints_a_double_accidental() {
        for id in DRILL_IDS {
            for key in 0..12 {
                for n in d(id, key).notes {
                    assert!(
                        !n.name.contains("##") && !n.name.contains("bb"),
                        "{id} in key {key} produced {}",
                        n.name
                    );
                }
            }
        }
    }

    #[test]
    fn spelling_always_matches_the_sounding_pitch() {
        for id in DRILL_IDS {
            for key in 0..12 {
                for n in d(id, key).notes {
                    if let Some(parsed) = Note::parse(&n.name) {
                        assert_eq!(
                            parsed.pitch_class(),
                            pc(n.midi),
                            "{id} key {key}: {} does not sound as spelled",
                            n.name
                        );
                    }
                }
            }
        }
    }

    /// The contour guarantee: no drill should contain a leap wider than an octave.
    #[test]
    fn drills_have_singable_contour() {
        for id in DRILL_IDS {
            for key in KEYS {
                let notes = d(id, key).notes;
                for w in notes.windows(2) {
                    let leap = (w[1].midi - w[0].midi).abs();
                    assert!(
                        leap <= 12,
                        "{id} in key {key} leaps {leap} semitones ({} -> {})",
                        w[0].name, w[1].name
                    );
                }
            }
        }
    }

    #[test]
    fn guide_tone_line_really_moves_by_half_steps() {
        let n = d("guidetone", 0).notes;
        let m: Vec<i32> = n.iter().map(|x| x.midi).collect();
        // voice 1: C -> B -> B, voice 2: F -> F -> E
        assert_eq!(m[1] - m[0], -1, "the 7th must fall a half step to the 3rd");
        assert_eq!(m[2] - m[1], 0);
        assert_eq!(m[4] - m[3], 0);
        assert_eq!(m[5] - m[4], -1);
        let names: Vec<&str> = n.iter().map(|x| x.name.as_str()).collect();
        assert_eq!(names, ["C", "B", "B", "F", "F", "E"]);
    }

    #[test]
    fn guide_tone_labels_the_minor_third_correctly() {
        // F over Dm7 is the b3, not the 3
        assert_eq!(d("guidetone", 0).notes[3].degree, "b3");
    }

    /// An enclosure is only an enclosure if the neighbours really surround the target.
    #[test]
    fn enclosure_surrounds_its_target_in_every_key() {
        for key in 0..12 {
            let n = d("enclosure", key).notes;
            let (above, below, target) = (n[0].midi, n[1].midi, n[2].midi);
            assert_eq!(above - target, 1, "key {key}: upper neighbour must be a half step above");
            assert_eq!(target - below, 1, "key {key}: lower neighbour must be a half step below");
        }
    }

    #[test]
    fn scale_sequence_is_made_of_real_ascending_thirds() {
        for key in KEYS {
            let n = d("scaleseq", key).notes;
            for pair in n.chunks(2) {
                let iv = pair[1].midi - pair[0].midi;
                assert!(iv == 3 || iv == 4, "key {key}: {iv} semitones is not a 3rd");
            }
        }
    }

    #[test]
    fn pentatonic_pattern_actually_climbs() {
        for key in KEYS {
            let n = d("pentpattern", key).notes;
            let first = n[0].midi;
            let last_cell_start = n[8].midi;
            assert!(last_cell_start > first, "key {key}: the pattern should rise");
        }
    }

    #[test]
    fn upper_structure_spells_a_real_major_triad() {
        // over G7 in C: A C# E, never A Db E
        let n = d("upperstructure", 0).notes;
        let names: Vec<&str> = n.iter().map(|x| x.name.as_str()).collect();
        assert_eq!(&names[..3], ["A", "C#", "E"]);
        // the triad is a whole step above the V and its top note is the 3rd of I
        assert_eq!(n[2].midi, n[3].midi);
    }

    #[test]
    fn altered_line_spells_alterations_on_the_right_letters() {
        let n = d("altered", 0).notes; // over G7 in C
        let names: Vec<&str> = n.iter().map(|x| x.name.as_str()).collect();
        assert_eq!(names, ["F", "Eb", "C#", "B", "A#", "Ab", "G", "E"]);
    }

    #[test]
    fn pentatonic_superimposition_gives_the_lydian_colour() {
        let n = d("pentsuper", 0).notes; // D major pentatonic over Cmaj7
        let names: Vec<&str> = n.iter().map(|x| x.name.as_str()).collect();
        assert_eq!(names, ["D", "E", "F#", "A", "B", "D"]);
    }

    #[test]
    fn tritone_sub_uses_a_flat_two_in_every_key() {
        for key in 0..12 {
            let dr = d("tritonesub", key);
            let sub = &dr.bars[1];
            assert!(
                !sub.contains('#'),
                "key {key}: the bII should be spelled flat, got {sub}"
            );
        }
    }

    #[test]
    fn coltrane_cells_are_spelled_in_their_own_key_centre() {
        // In B the centres are B / G / Eb, so the third cell reads Eb F G Bb
        let dr = d("coltrane", 11);
        assert_eq!(dr.bars, ["Bmaj7", "Gmaj7", "Ebmaj7"]);
        let names: Vec<&str> = dr.notes.iter().map(|x| x.name.as_str()).collect();
        assert_eq!(&names[8..12], ["Eb", "F", "G", "Bb"]);
    }

    #[test]
    fn bebop_scale_puts_chord_tones_on_the_beat() {
        let n = d("bebop", 0).notes;
        // eight notes of the scale, then the resolution
        assert_eq!(n.len(), 9);
        // starting on the root, the even positions are chord tones of G7
        let chord_tones = [0, 4, 7, 10];
        for (idx, note) in n.iter().take(8).enumerate() {
            if idx % 2 == 0 {
                let rel = pc(note.midi - crate::note::midi_for(7, 4));
                assert!(
                    chord_tones.contains(&rel),
                    "position {idx} ({}) is not a chord tone",
                    note.name
                );
            }
        }
    }

    #[test]
    fn degree_labels_are_never_empty() {
        for id in DRILL_IDS {
            for key in KEYS {
                for n in d(id, key).notes {
                    assert!(!n.degree.is_empty(), "{id} has an unlabelled note");
                    assert!(!n.name.is_empty());
                }
            }
        }
    }
}
