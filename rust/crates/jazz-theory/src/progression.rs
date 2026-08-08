//! Chord progressions: ii-V-Is, blues, and real standards.
//!
//! A [`Bar`] can hold two chords, because rhythm changes and countless standards
//! move twice a bar. Each half may override the suggested scale, since the right
//! scale depends on where a chord is going, not only on what it is: the V7 of a
//! minor key wants altered tensions, while a non-resolving dominant wants Lydian
//! dominant. Quality alone cannot express that.

use crate::harmony::Quality;
use crate::note::{key_centre_name, pc, pc_name, use_sharp_for_root};

/// The scale a bar suggests, overriding the chord quality's default.
pub const ALTERED: &str = "Altered (dominant)";
pub const LYDIAN_DOM: &str = "Lydian dominant";

/// Half a bar: a chord and, optionally, the scale to think in.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Slot {
    pub root: i32,
    pub quality: Quality,
    pub scale: Option<&'static str>,
}

impl Slot {
    pub fn new(root: i32, quality: Quality) -> Self {
        Self { root: pc(root), quality, scale: None }
    }
    pub fn with_scale(root: i32, quality: Quality, scale: &'static str) -> Self {
        Self { root: pc(root), quality, scale: Some(scale) }
    }
    /// Chord symbol, e.g. `Bbmaj7`, `Dm7b5`.
    pub fn symbol(&self) -> String {
        format!(
            "{}{}",
            pc_name(self.root, use_sharp_for_root(self.root)),
            self.quality.symbol()
        )
    }
    /// The scale to display: the override if present, else the quality default.
    pub fn scale_name(&self) -> &'static str {
        self.scale.unwrap_or_else(|| self.quality.default_scale())
    }
}

/// One bar of music: one chord, or two split at beat 3.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Bar {
    pub first: Slot,
    pub second: Option<Slot>,
}

impl Bar {
    pub fn one(s: Slot) -> Self {
        Self { first: s, second: None }
    }
    pub fn two(a: Slot, b: Slot) -> Self {
        Self { first: a, second: Some(b) }
    }
    pub fn is_split(&self) -> bool {
        self.second.is_some()
    }
    /// Which chord is sounding on a given beat (0-3).
    pub fn chord_at(&self, beat: usize) -> Slot {
        match self.second {
            Some(s) if beat >= 2 => s,
            _ => self.first,
        }
    }
    pub fn symbol(&self) -> String {
        match self.second {
            Some(s) => format!("{} {}", self.first.symbol(), s.symbol()),
            None => self.first.symbol(),
        }
    }
}

/// A named, playable progression.
#[derive(Debug, Clone)]
pub struct Progression {
    pub id: &'static str,
    pub name: &'static str,
    pub level: Level,
    pub bars: Vec<Bar>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Level {
    Beginner,
    Intermediate,
    Advanced,
    Standard,
}

use Quality::*;

fn one(root: i32, q: Quality) -> Bar {
    Bar::one(Slot::new(root, q))
}
fn one_s(root: i32, q: Quality, sc: &'static str) -> Bar {
    Bar::one(Slot::with_scale(root, q, sc))
}

/// ii-V-I in a major key.
pub fn two_five_one(key: i32) -> Vec<Bar> {
    vec![
        one(key + 2, Min7),
        one(key + 7, Dom7),
        one(key, Maj7),
        one(key, Maj7),
    ]
}

/// ii-V-i in a minor key. The V takes altered tensions.
pub fn two_five_one_minor(key: i32) -> Vec<Bar> {
    vec![
        one(key + 2, Min7b5),
        one_s(key + 7, Dom7, ALTERED),
        one(key, MinTonic),
        one(key, MinTonic),
    ]
}

/// The 12-bar blues.
pub fn blues(key: i32) -> Vec<Bar> {
    let (i, iv, v) = (key, key + 5, key + 7);
    vec![
        one(i, Dom7), one(iv, Dom7), one(i, Dom7), one(i, Dom7),
        one(iv, Dom7), one(iv, Dom7), one(i, Dom7), one(i, Dom7),
        one(v, Dom7), one(iv, Dom7), one(i, Dom7), one(v, Dom7),
    ]
}

/// A modal vamp — one chord, all the room in the world.
pub fn modal_vamp(key: i32) -> Vec<Bar> {
    vec![one(key, MinTonic); 4]
}

/// ii-V-I moving down in whole steps.
pub fn cycle(start: i32) -> Vec<Bar> {
    let mut bars = Vec::new();
    let mut k = start;
    for _ in 0..3 {
        bars.extend(two_five_one(k));
        k = pc(k + 10);
    }
    bars
}

/// Tritone-sub turnaround: the bII7 stands in for the V7.
pub fn tritone_sub(key: i32) -> Vec<Bar> {
    vec![
        one(key + 2, Min7),
        one_s(key + 1, Dom7, LYDIAN_DOM),
        one(key, Maj7),
        one(key, Maj7),
    ]
}

/// Minor blues.
pub fn minor_blues(key: i32) -> Vec<Bar> {
    let (i, iv, bvi, v) = (key, key + 5, key + 8, key + 7);
    vec![
        one(i, MinTonic), one(i, MinTonic), one(i, MinTonic), one(i, MinTonic),
        one(iv, MinTonic), one(iv, MinTonic), one(i, MinTonic), one(i, MinTonic),
        one_s(bvi, Dom7, LYDIAN_DOM), one_s(v, Dom7, ALTERED),
        one(i, MinTonic), one_s(v, Dom7, ALTERED),
    ]
}

/// Coltrane's major-third cycle: each tonic approached by its own V7.
pub fn coltrane_cycle(key: i32) -> Vec<Bar> {
    let centres = [pc(key), pc(key + 8), pc(key + 4)];
    let mut bars = Vec::new();
    for (i, &c) in centres.iter().enumerate() {
        bars.push(one(c, Maj7));
        bars.push(one(centres[(i + 1) % 3] + 7, Dom7));
    }
    bars
}

// ---------- standards ----------

/// Autumn Leaves, A section (in Bb / G minor).
pub fn autumn_leaves(key: i32) -> Vec<Bar> {
    vec![
        one(key + 2, Min7), one(key + 7, Dom7), one(key, Maj7), one(key + 5, Maj7),
        one(key + 11, Min7b5), one_s(key + 4, Dom7, ALTERED), one(key + 9, MinTonic), one(key + 9, MinTonic),
    ]
}

/// So What — full 32-bar AABA.
pub fn so_what() -> Vec<Bar> {
    let mut b = vec![one(2, MinTonic); 16];
    b.extend(vec![one(3, MinTonic); 8]);
    b.extend(vec![one(2, MinTonic); 8]);
    b
}

/// Blue Bossa in C minor.
pub fn blue_bossa() -> Vec<Bar> {
    vec![
        one(0, MinTonic), one(0, MinTonic), one(5, Min7), one(5, Min7),
        one(2, Min7b5), one_s(7, Dom7, ALTERED), one(0, MinTonic), one(0, MinTonic),
        one(3, Min7), one(8, Dom7), one(1, Maj7), one(1, Maj7),
        one(2, Min7b5), one_s(7, Dom7, ALTERED), one(0, MinTonic), one_s(7, Dom7, ALTERED),
    ]
}

/// Take the A Train, A section. The II7 is the tune's signature D7(#11).
pub fn a_train() -> Vec<Bar> {
    vec![
        one(0, Maj7), one(0, Maj7),
        one_s(2, Dom7, LYDIAN_DOM), one_s(2, Dom7, LYDIAN_DOM),
        one(2, Min7), one(7, Dom7), one(0, Maj7), one(0, Maj7),
    ]
}

/// Rhythm changes A section in Bb. The VI7 resolves to a minor chord, so it is altered.
pub fn rhythm_a() -> Vec<Bar> {
    vec![
        Bar::two(Slot::new(10, Maj7), Slot::with_scale(7, Dom7, ALTERED)),
        Bar::two(Slot::new(0, Min7), Slot::new(5, Dom7)),
        Bar::two(Slot::new(2, Min7), Slot::with_scale(7, Dom7, ALTERED)),
        Bar::two(Slot::new(0, Min7), Slot::new(5, Dom7)),
        Bar::two(Slot::new(5, Min7), Slot::new(10, Dom7)),
        Bar::two(Slot::new(3, Maj7), Slot::new(8, Dom7)),
        Bar::two(Slot::new(2, Min7), Slot::with_scale(7, Dom7, ALTERED)),
        Bar::two(Slot::new(0, Min7), Slot::new(5, Dom7)),
    ]
}

/// Rhythm changes, full 32-bar AABA with the III7-VI7-II7-V7 bridge.
pub fn rhythm_full() -> Vec<Bar> {
    let a = rhythm_a();
    let bridge = vec![
        one(2, Dom7), one(2, Dom7), one(7, Dom7), one(7, Dom7),
        one(0, Dom7), one(0, Dom7), one(5, Dom7), one(5, Dom7),
    ];
    let mut out = a.clone();
    out.extend(a.clone());
    out.extend(bridge);
    out.extend(a);
    out
}

/// Everything the play-along engine can load.
pub fn all() -> Vec<Progression> {
    use Level::*;
    vec![
        Progression { id: "251",        name: "ii-V-I in C major",            level: Beginner,     bars: two_five_one(0) },
        Progression { id: "251min",     name: "ii-V-i in C minor",            level: Beginner,     bars: two_five_one_minor(0) },
        Progression { id: "blues",      name: "12-bar Blues in F",            level: Beginner,     bars: blues(5) },
        Progression { id: "modal",      name: "Modal vamp - D Dorian",        level: Beginner,     bars: modal_vamp(2) },
        Progression { id: "autumn",     name: "Autumn Leaves (A)",            level: Intermediate, bars: autumn_leaves(10) },
        Progression { id: "ttsub",      name: "Tritone-sub turnaround",       level: Intermediate, bars: tritone_sub(0) },
        Progression { id: "cycle",      name: "ii-V-I through keys",          level: Intermediate, bars: cycle(0) },
        Progression { id: "minorblues", name: "Minor blues in C",             level: Advanced,     bars: minor_blues(0) },
        Progression { id: "coltrane",   name: "Coltrane cycle",               level: Advanced,     bars: coltrane_cycle(0) },
        Progression { id: "sowhat",     name: "So What - full AABA",          level: Standard,     bars: so_what() },
        Progression { id: "bluebossa",  name: "Blue Bossa",                   level: Standard,     bars: blue_bossa() },
        Progression { id: "atrain",     name: "Take the A Train (A)",         level: Standard,     bars: a_train() },
        Progression { id: "rhythma",    name: "Rhythm Changes (A) - Bb",      level: Standard,     bars: rhythm_a() },
        Progression { id: "rhythmfull", name: "Rhythm Changes - full AABA",   level: Standard,     bars: rhythm_full() },
    ]
}

/// The key centres of a Coltrane cycle, spelled conventionally (B / G / Eb).
pub fn coltrane_centre_names(key: i32) -> [&'static str; 3] {
    [
        key_centre_name(key),
        key_centre_name(key + 8),
        key_centre_name(key + 4),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn syms(bars: &[Bar]) -> Vec<String> {
        bars.iter().map(|b| b.symbol()).collect()
    }

    #[test]
    fn two_five_one_is_right_in_c() {
        assert_eq!(syms(&two_five_one(0)), ["Dm7", "G7", "Cmaj7", "Cmaj7"]);
    }

    #[test]
    fn minor_two_five_uses_half_diminished_and_altered_v() {
        let b = two_five_one_minor(0);
        assert_eq!(syms(&b)[0], "Dm7b5");
        assert_eq!(b[1].first.scale_name(), ALTERED);
    }

    #[test]
    fn blues_in_f_matches_the_standard_form() {
        assert_eq!(
            syms(&blues(5)),
            ["F7","Bb7","F7","F7","Bb7","Bb7","F7","F7","C7","Bb7","F7","C7"]
        );
    }

    #[test]
    fn autumn_leaves_a_section() {
        assert_eq!(
            syms(&autumn_leaves(10)),
            ["Cm7","F7","Bbmaj7","Ebmaj7","Am7b5","D7","Gm7","Gm7"]
        );
    }

    #[test]
    fn autumn_leaves_secondary_dominant_is_altered_not_mixolydian() {
        // D7 resolves to Gm7; plain D Mixolydian would supply a B natural against Bb.
        let b = autumn_leaves(10);
        assert_eq!(b[5].first.scale_name(), ALTERED);
    }

    #[test]
    fn blue_bossa_has_the_db_major_bridge_and_g7_turnaround() {
        let s = syms(&blue_bossa());
        assert_eq!(s.len(), 16);
        assert_eq!(&s[8..12], ["Ebm7", "Ab7", "Dbmaj7", "Dbmaj7"]);
        assert_eq!(s[15], "G7");
    }

    #[test]
    fn a_train_two_seven_suggests_lydian_dominant() {
        let b = a_train();
        assert_eq!(b[2].first.symbol(), "D7");
        assert_eq!(b[2].first.scale_name(), LYDIAN_DOM);
    }

    #[test]
    fn so_what_is_a_real_32_bar_aaba() {
        let b = so_what();
        assert_eq!(b.len(), 32);
        assert_eq!(b[0].symbol(), "Dm7");
        assert_eq!(b[16].symbol(), "Ebm7"); // bridge up a half step
        assert_eq!(b[24].symbol(), "Dm7");  // last A
    }

    #[test]
    fn rhythm_changes_bridge_is_a_cycle_of_dominants() {
        let b = rhythm_full();
        assert_eq!(b.len(), 32);
        let bridge: Vec<String> = b[16..24].iter().map(|x| x.symbol()).collect();
        assert_eq!(bridge, ["D7","D7","G7","G7","C7","C7","F7","F7"]);
    }

    #[test]
    fn rhythm_changes_a_section_splits_every_bar() {
        for bar in rhythm_a() {
            assert!(bar.is_split(), "rhythm changes move twice a bar");
        }
        assert_eq!(rhythm_a()[0].symbol(), "Bbmaj7 G7");
    }

    #[test]
    fn split_bars_switch_chord_at_beat_three() {
        let bar = rhythm_a()[0];
        assert_eq!(bar.chord_at(0).symbol(), "Bbmaj7");
        assert_eq!(bar.chord_at(1).symbol(), "Bbmaj7");
        assert_eq!(bar.chord_at(2).symbol(), "G7");
        assert_eq!(bar.chord_at(3).symbol(), "G7");
    }

    #[test]
    fn coltrane_cycle_is_the_giant_steps_pattern() {
        assert_eq!(
            syms(&coltrane_cycle(0)),
            ["Cmaj7","Eb7","Abmaj7","B7","Emaj7","G7"]
        );
    }

    #[test]
    fn coltrane_centres_are_spelled_conventionally() {
        assert_eq!(coltrane_centre_names(11), ["B", "G", "Eb"]);
        assert_eq!(coltrane_centre_names(0), ["C", "Ab", "E"]);
    }

    #[test]
    fn tritone_sub_is_a_flat_two_and_uses_lydian_dominant() {
        let b = tritone_sub(9); // key of A -> Bb7, never A#7
        assert_eq!(b[1].first.symbol(), "Bb7");
        assert_eq!(b[1].first.scale_name(), LYDIAN_DOM);
    }

    #[test]
    fn every_progression_is_non_empty_and_symbol_renders() {
        for p in all() {
            assert!(!p.bars.is_empty(), "{} is empty", p.id);
            for b in &p.bars {
                assert!(!b.symbol().is_empty());
                assert!(!b.first.scale_name().is_empty());
            }
        }
    }

    #[test]
    fn no_dominant_resolving_to_minor_is_left_on_plain_mixolydian() {
        // Walk every progression; when a Dom7 is followed by a minor chord a fifth
        // below, it must not still be suggesting plain Mixolydian.
        for p in all() {
            let n = p.bars.len();
            for i in 0..n {
                let cur = p.bars[i].chord_at(3);
                let next = p.bars[(i + 1) % n].chord_at(0);
                let resolves_to_minor = matches!(next.quality, Quality::Min7 | Quality::MinTonic)
                    && pc(cur.root - next.root) == 7;
                if cur.quality == Quality::Dom7 && resolves_to_minor {
                    assert_ne!(
                        cur.scale_name(),
                        "Mixolydian (dominant)",
                        "{} bar {} ({}) resolves to {} but suggests Mixolydian",
                        p.id, i + 1, cur.symbol(), next.symbol()
                    );
                }
            }
        }
    }
}
