//! The listening curriculum: who to study and what repertoire to learn.
//!
//! The language is passed down by ear, so this is not decoration — it is the part
//! of the method that says *stop reading and go listen*. Facts here were checked
//! against sources; the tests below guard the ones that are easy to get wrong
//! (composer credits especially, since "Take the A Train" is routinely misattributed
//! to Ellington when Strayhorn wrote it).

/// A player worth living with.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Master {
    pub name: &'static str,
    /// Primary instrument.
    pub instrument: &'static str,
    /// Active era, and style where it clarifies.
    pub era: &'static str,
    /// What to listen *for* — the point is to train the ear, not to memorise trivia.
    pub listen_for: &'static str,
}

pub const MASTERS: &[Master] = &[
    Master { name: "Louis Armstrong", instrument: "Trumpet / voice", era: "1920s-70s",
        listen_for: "The first great soloist - joyous swing, singing phrases, pure storytelling." },
    Master { name: "Big Joe Turner", instrument: "Blues vocals", era: "1930s-80s",
        listen_for: "The blues-shouting roots under all of jazz - shuffle, call-and-response, feeling." },
    Master { name: "Charlie Parker", instrument: "Alto sax", era: "1940s-50s (bebop)",
        listen_for: "Blazing eighth-note lines, chromatic enclosures, the blues inside everything." },
    Master { name: "Dizzy Gillespie", instrument: "Trumpet", era: "1930s-90s (bebop / Afro-Cuban)",
        listen_for: "Harmonic daring, high-register fire, and the birth of Afro-Cuban jazz." },
    Master { name: "Thelonious Monk", instrument: "Piano", era: "1940s-70s",
        listen_for: "Angular lines, space, and \"wrong\" notes that are exactly right; rhythm as melody." },
    Master { name: "Charles Mingus", instrument: "Bass / composer", era: "1940s-70s",
        listen_for: "Gospel-and-blues fire, collective improvisation, huge dynamics." },
    Master { name: "Miles Davis", instrument: "Trumpet", era: "1940s-90s",
        listen_for: "Space and reinvention - few notes, deep lyricism; pioneered modal jazz." },
    Master { name: "John Coltrane", instrument: "Tenor / soprano sax", era: "1950s-60s",
        listen_for: "\"Sheets of sound,\" modal jazz, the Giant Steps matrix, spiritual intensity." },
    Master { name: "Stan Getz", instrument: "Tenor sax", era: "1940s-90s",
        listen_for: "\"The Sound\" - a gorgeous tone and effortless, lyrical phrasing; bossa nova." },
    Master { name: "Doc Severinsen", instrument: "Trumpet", era: "1950s-2010s (big band)",
        listen_for: "Big-band virtuosity, brilliant tone, dazzling range and command." },
    Master { name: "Maynard Ferguson", instrument: "Trumpet", era: "1940s-2000s (big band)",
        listen_for: "Stratospheric high-note power and screaming lead lines." },
    Master { name: "Chick Corea", instrument: "Piano", era: "1960s-2020s (fusion / Latin / post-bop)",
        listen_for: "Crystalline lines, Spanish colours, endlessly inventive rhythm." },
    Master { name: "Arturo Sandoval", instrument: "Trumpet", era: "1970s-2020s (Afro-Cuban)",
        listen_for: "Virtuosity married to bebop and Latin fire; astonishing range." },
];

/// Further listening, named rather than described.
pub const ALSO_EXPLORE: &str =
    "Ella Fitzgerald, Duke Ellington, Count Basie, Sarah Vaughan, Bill Evans, Sonny Rollins, \
     Wes Montgomery, Herbie Hancock, Clifford Brown, Freddie Hubbard, Oscar Peterson, \
     Wynton Marsalis.";

/// A tune worth knowing, and why.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Standard {
    pub title: &'static str,
    pub composer: &'static str,
    pub key_and_form: &'static str,
    pub teaches: &'static str,
    /// Id of the matching progression in [`crate::progression::all`], when the
    /// changes are playable here.
    pub progression_id: Option<&'static str>,
}

pub const STANDARDS: &[Standard] = &[
    Standard { title: "So What", composer: "Miles Davis",
        key_and_form: "D Dorian - 32-bar AABA",
        teaches: "Modal soloing and the power of space.",
        progression_id: Some("sowhat") },
    Standard { title: "Blue Bossa", composer: "Kenny Dorham",
        key_and_form: "C minor - 16 bars",
        teaches: "Minor ii-V-i, a key change, and Latin feel.",
        progression_id: Some("bluebossa") },
    Standard { title: "Take the A Train", composer: "Billy Strayhorn",
        key_and_form: "C major - 32-bar AABA",
        teaches: "Ellington's theme, written by Strayhorn; the famous D7(#11).",
        progression_id: Some("atrain") },
    Standard { title: "Rhythm Changes (from \"I Got Rhythm\")", composer: "George Gershwin",
        key_and_form: "played in Bb - 32-bar AABA",
        teaches: "Fast ii-Vs plus a dominant-cycle bridge - the bebop proving ground.",
        progression_id: Some("rhythmfull") },
    Standard { title: "Autumn Leaves", composer: "Joseph Kosma",
        key_and_form: "G minor / Bb - 32 bars",
        teaches: "Major and minor ii-V-I side by side.",
        progression_id: Some("autumn") },
    Standard { title: "Blues (Now's the Time)", composer: "Charlie Parker",
        key_and_form: "F - 12-bar blues",
        teaches: "The foundation - feel plus chord tones.",
        progression_id: Some("blues") },
    Standard { title: "Giant Steps", composer: "John Coltrane",
        key_and_form: "B major - 16 bars",
        teaches: "The major-third matrix; the ultimate changes-running challenge. (Plays here as the same cycle in C.)",
        progression_id: Some("coltrane") },
    Standard { title: "All the Things You Are", composer: "Jerome Kern",
        key_and_form: "Ab major - 36 bars",
        teaches: "Chord roots cycling in fourths through shifting key centres.",
        progression_id: None },
];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::progression;

    #[test]
    fn every_master_is_fully_described() {
        for m in MASTERS {
            assert!(!m.name.is_empty() && !m.instrument.is_empty());
            assert!(!m.era.is_empty(), "{} has no era", m.name);
            assert!(
                m.listen_for.len() > 30,
                "{} needs a real listening note, not a label",
                m.name
            );
        }
    }

    #[test]
    fn every_era_gives_a_decade_range() {
        // A bare genre ("big band") tells a student nothing about when to look.
        for m in MASTERS {
            assert!(
                m.era.contains("s-"),
                "{} has era '{}' with no decade range",
                m.name,
                m.era
            );
        }
    }

    /// Strayhorn wrote it. Crediting Ellington repeats exactly the erasure he is
    /// famous for suffering.
    #[test]
    fn take_the_a_train_is_credited_to_strayhorn() {
        let s = STANDARDS.iter().find(|s| s.title.contains("A Train")).unwrap();
        assert_eq!(s.composer, "Billy Strayhorn");
        assert!(!s.composer.contains("Ellington"));
    }

    #[test]
    fn rhythm_changes_distinguishes_the_jazz_key_from_the_original() {
        let s = STANDARDS.iter().find(|s| s.title.starts_with("Rhythm Changes")).unwrap();
        assert_eq!(s.composer, "George Gershwin");
        // Bb / 32 bars is the jazz vehicle; the song itself is Db and 34 bars.
        assert!(s.key_and_form.contains("Bb"));
        assert!(s.key_and_form.contains("played in"));
    }

    #[test]
    fn composer_credits_are_the_verified_ones() {
        let by = |t: &str| STANDARDS.iter().find(|s| s.title.starts_with(t)).unwrap();
        assert_eq!(by("So What").composer, "Miles Davis");
        assert_eq!(by("Blue Bossa").composer, "Kenny Dorham");
        assert_eq!(by("Autumn Leaves").composer, "Joseph Kosma");
        assert_eq!(by("Giant Steps").composer, "John Coltrane");
        assert_eq!(by("All the Things").composer, "Jerome Kern");
    }

    /// Every playable standard must point at a progression that really exists,
    /// or the button does nothing.
    #[test]
    fn playable_standards_resolve_to_a_real_progression() {
        let progs = progression::all();
        for s in STANDARDS {
            if let Some(id) = s.progression_id {
                assert!(
                    progs.iter().any(|p| p.id == id),
                    "{} points at unknown progression '{}'",
                    s.title,
                    id
                );
            }
        }
    }

    #[test]
    fn a_standard_without_changes_says_so_rather_than_pointing_nowhere() {
        let atya = STANDARDS.iter().find(|s| s.title.starts_with("All the Things")).unwrap();
        assert!(atya.progression_id.is_none());
    }

    #[test]
    fn titles_are_unique() {
        let mut t: Vec<&str> = STANDARDS.iter().map(|s| s.title).collect();
        t.sort_unstable();
        let n = t.len();
        t.dedup();
        assert_eq!(n, t.len(), "duplicate standard");
        let mut m: Vec<&str> = MASTERS.iter().map(|x| x.name).collect();
        m.sort_unstable();
        let n = m.len();
        m.dedup();
        assert_eq!(n, m.len(), "duplicate master");
    }
}
