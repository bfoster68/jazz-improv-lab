//! # jazz-theory
//!
//! The music-theory engine behind the Jazz Improvisation Lab.
//!
//! Everything here is expressed in **note names, scale degrees and intervals** —
//! never in fingerings, frets or key positions — so it applies equally to a
//! saxophonist, pianist, guitarist, bassist, trumpeter or singer.
//!
//! Three design rules carry most of the weight:
//!
//! 1. **Spell by letter arithmetic.** The `#11` of G is `C#`, never `Db`. See
//!    [`note::Note::spell`].
//! 2. **Store degree labels as data.** The same pitch is the `b5` of a
//!    half-diminished chord and the `#11` of a dominant; only context decides.
//! 3. **Keep register, not just pitch class.** A drill that claims to enclose a
//!    target has to actually sound a half step either side of it, so drills carry
//!    absolute semitone offsets. See [`drill`].
//!
//! ```
//! use jazz_theory::{drill, note::Note};
//! let d = drill::build("guidetone", 0).unwrap();
//! let names: Vec<&str> = d.notes.iter().map(|n| n.name.as_str()).collect();
//! assert_eq!(names, ["C", "B", "B", "F", "F", "E"]);
//! ```

#![forbid(unsafe_code)]

pub mod drill;
pub mod ear;
pub mod harmony;
pub mod listening;
pub mod note;
pub mod progression;

/// Degree label for a semitone offset within a major scale.
pub(crate) fn degree_of_major(semi: i32) -> &'static str {
    match semi.rem_euclid(12) {
        0 => "1", 2 => "2", 4 => "3", 5 => "4",
        7 => "5", 9 => "6", 11 => "7",
        1 => "b2", 3 => "b3", 6 => "b5", 8 => "b6", 10 => "b7",
        _ => "?",
    }
}

/// Degree label for a semitone offset within a minor pentatonic.
pub(crate) fn degree_of_minor_pent(semi: i32) -> &'static str {
    match semi.rem_euclid(12) {
        0 => "1", 3 => "b3", 5 => "4", 7 => "5", 10 => "b7",
        _ => degree_of_major(semi),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn major_degrees_cover_the_scale() {
        assert_eq!(degree_of_major(0), "1");
        assert_eq!(degree_of_major(4), "3");
        assert_eq!(degree_of_major(11), "7");
        assert_eq!(degree_of_major(10), "b7");
    }

    #[test]
    fn pentatonic_degrees_are_minor_flavoured() {
        assert_eq!(degree_of_minor_pent(3), "b3");
        assert_eq!(degree_of_minor_pent(10), "b7");
    }
}
