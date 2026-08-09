//! The written curriculum: the Learn tab's content, as data.
//!
//! This is the prose half of the method — the part that explains *why* before the
//! drills ask you to do it. Keeping it here rather than in the UI means both front
//! ends teach the same thing, and the tests can enforce that the path is coherent:
//! every level populated, in teaching order, with cross-references that resolve.

use crate::progression::Level;

/// One lesson card.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Lesson {
    pub level: Level,
    pub title: &'static str,
    /// The teaching text. Paragraphs are separated by a blank line.
    pub body: &'static str,
    /// An optional pointer at something in the app that makes the idea audible.
    pub try_it: Option<TryIt>,
}

/// A concrete "go and hear this" link into the rest of the app.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TryIt {
    /// Open a drill by its id in [`crate::drill::DRILL_IDS`].
    Drill(&'static str),
    /// Load a progression by its id in [`crate::progression::all`].
    Progression(&'static str),
    /// Open a scale in the theory browser.
    Scale(&'static str),
}

use Level::{Advanced, Beginner, Intermediate};

pub const LESSONS: &[Lesson] = &[
    // ------------------------------------------------------------- beginner
    Lesson {
        level: Beginner,
        title: "What improvisation really is",
        body: "Improvising is composing a melody in real time that fits the chords going by and \
sounds like a human speaking - with phrases, pauses, questions and answers. It is not running \
scales up and down. Scales are the alphabet; music is the sentences you build from them.\n\n\
Three things decide whether a note sounds good: the note itself, the beat it lands on, and the \
rhythm and space around it. Beginners obsess over the first and ignore the other two. Reverse \
that. The single most useful habit is to land a chord tone - ideally the 3rd or 7th - on a \
strong beat, and leave space after your phrases.",
        try_it: None,
    },
    Lesson {
        level: Beginner,
        title: "The building blocks",
        body: "Number the notes of a major scale 1 to 7. In C that is C(1) D(2) E(3) F(4) G(5) \
A(6) B(7). Describing ideas as numbers makes them instantly transposable to any key and any \
instrument.\n\nA seventh chord stacks 1-3-5-7: the chord tones, your safe resting places. The \
3rd and 7th are the guide tones, the two notes that tell your ear what the chord is and where \
it wants to go. Notes above the 7th - the 9th, 11th and 13th and their alterations - are colour \
and tension that want to resolve.",
        try_it: Some(TryIt::Scale("Major (Ionian)")),
    },
    Lesson {
        level: Beginner,
        title: "The ii-V-I: the sentence that unlocks jazz",
        body: "In C major: Dm7 to G7 to Cmaj7 - setup, tension, resolution, and the backbone of \
hundreds of standards.\n\nThe trick that makes it sound like jazz for almost no effort is the \
guide-tone line. Each chord's 3rd and 7th form two voices that barely move. Voice 1: C is the \
b7 of Dm7, falls a half step to B (the 3rd of G7), and B is also the 7th of Cmaj7 so it simply \
stays. Voice 2: F is the b3 of Dm7, stays put to become the b7 of G7, then falls a half step to \
E, the 3rd of Cmaj7.\n\nNotice the pattern: each chord's 7th slides down a half step to become \
the next chord's 3rd. Loop the progression and play only those notes as long tones before you \
decorate anything.",
        try_it: Some(TryIt::Drill("guidetone")),
    },
    Lesson {
        level: Beginner,
        title: "Aim before you shoot: target notes and enclosures",
        body: "Great players do not play a scale over a chord. They aim each phrase at a specific \
note that lands on a strong beat - usually the 3rd or 7th - and the notes before it are just the \
approach.\n\nThe classic approach is the enclosure: circle the target from above and below before \
landing. To hit E (the 3rd of Cmaj7) on beat 1, play F just above and D#/Eb just below, then \
resolve to E. That squeeze and arrival is the most characteristic bebop sound there is.",
        try_it: Some(TryIt::Drill("enclosure")),
    },
    Lesson {
        level: Beginner,
        title: "Scales, chosen by chord quality",
        body: "Learn five first, always connected to a chord: major (Ionian) over major 7, Dorian \
over minor 7, Mixolydian over a relaxed dominant, the minor pentatonic for five safe notes, and \
the blues scale - the pentatonic plus the b5 blue note.\n\nDo not ask \"what scale?\" first. Ask \
\"what chord tone am I aiming at, on what beat?\" and use the scale to connect your targets.",
        try_it: Some(TryIt::Scale("Blues scale")),
    },
    Lesson {
        level: Beginner,
        title: "Rhythm, swing and space",
        body: "You can play all the right notes and still sound wrong if the rhythm is dead, and \
you can play very simple notes and sound wonderful if it is alive.\n\nSwing the eighth notes - \
long then short, with a triplet lilt - at medium and slow tempos; they even out as the tempo \
climbs. Leave space: play a phrase, then rest. Silence lets an idea land and gives you time to \
hear the next one. Phrase off the barline rather than starting everything on beat 1. And \
converse: make a statement, leave a gap, answer it.",
        try_it: Some(TryIt::Progression("blues")),
    },
    Lesson {
        level: Beginner,
        title: "Tension and release, and the blues",
        body: "Every good phrase, and every good solo, is an arc of tension and release. Tension \
comes from higher register, denser rhythm, chromatic notes and upper extensions; release comes \
from chord tones, downbeats and thinner rhythm. Start simple, build to a peak, then resolve.\n\n\
The 12-bar blues is the best playground. A blues in F over three dominant chords lets you play \
one blues scale the whole way and sound good immediately - then start targeting each chord's \
guide tones for the jazz layer on top. That combination of feel and harmony is jazz improvisation \
in miniature.",
        try_it: Some(TryIt::Progression("blues")),
    },
    Lesson {
        level: Beginner,
        title: "A practice routine that works",
        body: "Learning to improvise is learning a language: vocabulary, grammar, conversation.\n\n\
Sing first - if you cannot hear it, you cannot really play it. Work in small loops, slowly, with \
a metronome. Take everything through all twelve keys, since it is all stored as intervals. \
Transcribe short phrases by ear from records you love. Play with tracks and with people. And \
always keep your place in the form: if you get lost, land a chord tone on a downbeat and you are \
anchored again.",
        try_it: None,
    },
    // --------------------------------------------------------- intermediate
    Lesson {
        level: Intermediate,
        title: "Extensions and the avoid notes",
        body: "Go above the 7th. The 9th, 11th and 13th are the sweet upper colours, but not every \
extension fits every chord. On a major or dominant chord the natural 11 clashes with the 3rd, a \
half step above it, so it is an avoid note: use it as a quick passing tone, or raise it to a #11 \
for the bright Lydian sound. On minor 7 chords all of 9, 11 and 13 are fair game - that is the \
Dorian sound.\n\nThe working rule stays the same: chord tones on strong beats, extensions on weak \
beats, and let any tension slide by step into the nearest chord tone.",
        try_it: Some(TryIt::Drill("39arp")),
    },
    Lesson {
        level: Intermediate,
        title: "Bebop scales: keeping chord tones on the beat",
        body: "A plain seven-note scale has a rhythmic problem at speed. Run straight eighth notes \
and your chord tones drift onto the weak beats. The fix is the bebop scale: add one chromatic \
passing tone to make an eight-note scale, so that starting on a chord tone, every downbeat lands \
on a chord tone.\n\nThe dominant bebop scale adds a passing tone between the b7 and the root; the \
major bebop scale adds one between the 5 and 6. This is the rhythmic secret behind classic bebop \
lines.",
        try_it: Some(TryIt::Drill("bebop")),
    },
    Lesson {
        level: Intermediate,
        title: "Tritone substitution",
        body: "Any dominant 7 chord can be replaced by the dominant a tritone away, because the two \
contain the same tritone - the 3rd/7th tension pair - with the roles swapped. The 3rd of G7 (B) is \
the 7th of Db7, and the 7th of G7 (F) is the 3rd of Db7: the same two notes, the same pull, a new \
bass note underneath.\n\nSubstituting Db7 for G7 therefore resolves just as strongly, but gives a \
smooth chromatic bass line (D to Db to C) and a hipper cadence. Over the substitute, play its \
Lydian dominant scale.",
        try_it: Some(TryIt::Drill("tritonesub")),
    },
    Lesson {
        level: Intermediate,
        title: "Digital patterns and 3-to-9 arpeggios",
        body: "Two workhorses for outlining chords melodically. The 1-2-3-5 digital pattern spells \
each chord - over Dm7 that is D E F A. The 3-to-9 arpeggio starts on the 3rd and stacks 3-5-7-9, \
which automatically catches both guide tones plus one sweet extension: over Dm7, F A C E.\n\nRun \
either through a full ii-V-I and you will sound like you are naming every chord as it passes.",
        try_it: Some(TryIt::Drill("digital")),
    },
    // -------------------------------------------------------------- advanced
    Lesson {
        level: Advanced,
        title: "Altered dominants and upper-structure triads",
        body: "When a V7 resolves down a fifth you can pile on tension with the altered scale - the \
seventh mode of melodic minor - which supplies b9, #9, #11 and b13: every note that pulls into the \
I.\n\nUpper-structure triads are a clean way to grab a whole tension set in one familiar shape: a \
plain major triad played over the dominant's root. Over G7, a Db major triad (Db F Ab) gives the \
#11, b7 and b9, every note of it inside the altered scale. The same trick has a softer, unaltered \
version: an A major triad over G7 (A C# E) gives the 9, #11 and 13 - the bright Lydian dominant \
sound, which is what you want when a dominant is not resolving.",
        try_it: Some(TryIt::Drill("altered")),
    },
    Lesson {
        level: Advanced,
        title: "Reharmonization and Coltrane changes",
        body: "Once you hear the changes you can bend them. The backdoor ii-V (a bVII7 resolving up \
to I), tritone subs, and inserted secondary dominants all recolour a progression.\n\nThe most \
famous advanced device is Coltrane's major-third cycle, the Giant Steps matrix: key centres moving \
by major thirds - C, Ab, E - each approached by its own V7. Practising clean 1-2-3-5 cells through \
those fast-moving keys is a rite of passage.",
        try_it: Some(TryIt::Progression("coltrane")),
    },
    Lesson {
        level: Advanced,
        title: "Playing outside: side-slipping and superimposition",
        body: "Deliberate wrong notes, resolved on purpose. Side-slipping plays a phrase a half step \
away from the key and then snaps it back - pure tension and release.\n\nSuperimposition lays a \
familiar shape from a different root over the chord: a D major pentatonic over Cmaj7 yields 9, 3, \
#11, 13 and 7, a shimmering Lydian colour. The rule that keeps it musical is simple: always resolve \
the outside idea back inside, on a strong beat.",
        try_it: Some(TryIt::Drill("pentsuper")),
    },
    Lesson {
        level: Advanced,
        title: "Motivic development and telling a story",
        body: "The difference between running vocabulary and actually saying something. Take a short \
motif - three or four notes with a distinctive rhythm - and develop it: transpose it to fit each \
chord, invert it, augment or shrink its rhythm, sequence it, displace it across the barline.\n\nA \
whole solo built from one evolving idea has a shape and memorability that scale-running never \
achieves. This is where technique becomes music.",
        try_it: Some(TryIt::Drill("motif")),
    },
    Lesson {
        level: Advanced,
        title: "Rhythmic mastery: displacement and playing over the barline",
        body: "At the highest level, rhythm is the frontier. Metric displacement repeats a phrase \
starting on a different beat so it floats against the pulse. Polyrhythmic groupings - playing in \
threes over 4/4 - create rolling tension. Phrasing across the barline in long, asymmetric lines \
makes time feel elastic.\n\nCombine these with the harmonic tools above and you have the full \
language.",
        try_it: Some(TryIt::Progression("rhythmfull")),
    },
];

/// Lessons for one level, in teaching order.
pub fn for_level(level: Level) -> Vec<&'static Lesson> {
    LESSONS.iter().filter(|l| l.level == level).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{drill::DRILL_IDS, harmony::find_scale, progression};

    #[test]
    fn every_level_has_a_real_path_through_it() {
        for level in [Beginner, Intermediate, Advanced] {
            let n = for_level(level).len();
            assert!(n >= 4, "{level:?} has only {n} lessons");
        }
    }

    #[test]
    fn lessons_are_grouped_in_teaching_order() {
        // Beginner material must all come before intermediate, and so on, so the
        // list can be rendered straight through without sorting.
        let rank = |l: Level| match l {
            Beginner => 0,
            Intermediate => 1,
            Advanced => 2,
            Level::Standard => 3,
        };
        let mut seen = 0;
        for l in LESSONS {
            let r = rank(l.level);
            assert!(r >= seen, "{} appears out of level order", l.title);
            seen = r;
        }
    }

    #[test]
    fn every_lesson_is_substantial_and_titled() {
        for l in LESSONS {
            assert!(!l.title.is_empty());
            assert!(
                l.body.len() > 200,
                "{} is too thin to teach anything ({} chars)",
                l.title,
                l.body.len()
            );
            assert!(l.body.contains("\n\n"), "{} should have more than one paragraph", l.title);
        }
    }

    /// A "try it" that points nowhere is worse than none at all.
    #[test]
    fn every_cross_reference_resolves() {
        let progs = progression::all();
        for l in LESSONS {
            match l.try_it {
                Some(TryIt::Drill(id)) => assert!(
                    DRILL_IDS.contains(&id),
                    "{} points at unknown drill '{id}'",
                    l.title
                ),
                Some(TryIt::Progression(id)) => assert!(
                    progs.iter().any(|p| p.id == id),
                    "{} points at unknown progression '{id}'",
                    l.title
                ),
                Some(TryIt::Scale(name)) => assert!(
                    find_scale(name).is_some(),
                    "{} points at unknown scale '{name}'",
                    l.title
                ),
                None => {}
            }
        }
    }

    #[test]
    fn most_lessons_offer_something_to_hear() {
        let with = LESSONS.iter().filter(|l| l.try_it.is_some()).count();
        assert!(
            with * 2 >= LESSONS.len(),
            "only {with} of {} lessons link to something audible",
            LESSONS.len()
        );
    }

    #[test]
    fn titles_are_unique() {
        let mut t: Vec<&str> = LESSONS.iter().map(|l| l.title).collect();
        t.sort_unstable();
        let n = t.len();
        t.dedup();
        assert_eq!(n, t.len(), "duplicate lesson title");
    }

    /// The prose is the teaching voice; notation inside it must use the same ASCII
    /// convention as the engine so a renderer can substitute glyphs uniformly.
    #[test]
    fn prose_uses_ascii_accidentals_not_unicode() {
        for l in LESSONS {
            for bad in ['\u{266D}', '\u{266F}', '\u{266E}'] {
                assert!(
                    !l.body.contains(bad) && !l.title.contains(bad),
                    "{} contains a unicode accidental; use b and # so display can convert",
                    l.title
                );
            }
        }
    }

    #[test]
    fn the_guide_tone_lesson_states_the_rule_correctly() {
        let l = LESSONS.iter().find(|l| l.title.contains("ii-V-I")).unwrap();
        // the central claim of the whole method
        assert!(l.body.contains("7th slides down a half step to become the next chord's 3rd"));
        assert!(l.body.contains("b3 of Dm7"), "F over Dm7 is the b3, not the 3");
    }
}
