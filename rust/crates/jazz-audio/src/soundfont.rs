//! Optional SoundFont playback — real recorded instruments instead of synthesis.
//!
//! Enabled with the `soundfont` feature. Point it at any General MIDI `.sf2` file
//! and the pitched instruments (piano, bass, vibes) are rendered from samples;
//! the drums stay synthesised, since the ride cymbal here is better than most GM
//! drum kits and needs no download.
//!
//! This mirrors the browser build's "High-quality sounds" toggle, and behaves the
//! same way: if the file is missing or unreadable, you keep the built-in synth
//! rather than losing audio.

use std::path::Path;
use std::sync::Arc;

use rustysynth::{SoundFont, Synthesizer, SynthesizerSettings};

use crate::voice::Instrument;

/// General MIDI program numbers that suit each instrument.
const PROGRAM_KEYS: i32 = 4; // Electric Piano 1 — the Rhodes the synth voice imitates
const PROGRAM_PIANO: i32 = 0; // Acoustic Grand Piano
const PROGRAM_GUITAR: i32 = 26; // Electric Guitar (jazz)
const PROGRAM_BASS: i32 = 32; // Acoustic Bass
const PROGRAM_SAX: i32 = 66; // Tenor Sax
const PROGRAM_TROMBONE: i32 = 57; // Trombone
const PROGRAM_TRUMPET: i32 = 56; // Trumpet
const PROGRAM_VIBES: i32 = 11; // Vibraphone

/// One MIDI channel per instrument keeps their programs independent. Channel 9
/// is General MIDI's percussion channel, so it is deliberately skipped.
fn channel_of(i: Instrument) -> i32 {
    match i {
        Instrument::Keys => 0,
        Instrument::Bass => 1,
        Instrument::Vibes => 2,
        Instrument::Piano => 3,
        Instrument::Guitar => 4,
        Instrument::Sax => 5,
        Instrument::Trombone => 6,
        Instrument::Trumpet => 7,
    }
}

fn program_of(i: Instrument) -> i32 {
    match i {
        Instrument::Keys => PROGRAM_KEYS,
        Instrument::Piano => PROGRAM_PIANO,
        Instrument::Guitar => PROGRAM_GUITAR,
        Instrument::Bass => PROGRAM_BASS,
        Instrument::Sax => PROGRAM_SAX,
        Instrument::Trombone => PROGRAM_TROMBONE,
        Instrument::Trumpet => PROGRAM_TRUMPET,
        Instrument::Vibes => PROGRAM_VIBES,
    }
}

/// A note waiting to start. Its note-off is derived from `dur` when it fires,
/// so dropping a queued note can never leave an orphaned note-off behind.
#[derive(Debug, Clone, Copy)]
struct PendingNote {
    at: f32,
    channel: i32,
    key: i32,
    velocity: i32,
    dur: f32,
}

/// A note-off owed to a note that has already started.
#[derive(Debug, Clone, Copy)]
struct PendingOff {
    at: f32,
    channel: i32,
    key: i32,
}

/// Map an engine gain to a MIDI velocity.
///
/// The synthesis voices scale linearly with gain, but a sampled instrument's
/// velocity maps to loudness roughly quadratically — so the old linear
/// `gain * 420` sent the comping (gain ~0.05) out at velocity ~20, a whisper
/// under a bass at ~70. The square root undoes that curve: the comping lands
/// near velocity 60 and the band balances the way the synth mix does.
fn velocity_for(gain: f32) -> i32 {
    (((gain / 0.18).max(0.0).sqrt() * 105.0) as i32).clamp(24, 127)
}

/// How long after the last note-off the synth is assumed to still be releasing.
/// rustysynth exposes no active-voice count, so the render window is padded by
/// this much rather than cutting the decay dead.
const RELEASE_TAIL: f32 = 2.0;

/// Sample-based voices driven by a SoundFont.
pub struct SoundfontVoices {
    synth: Synthesizer,
    notes: Vec<PendingNote>,
    offs: Vec<PendingOff>,
    /// Keep rendering until this absolute time, so release tails are not chopped.
    quiet_after: f32,
    /// The end of the last rendered window.
    rendered_to: f32,
    left: Vec<f32>,
    right: Vec<f32>,
    mono: Vec<f32>,
}

impl SoundfontVoices {
    /// Load a `.sf2` file. Returns an error rather than panicking, so a bad path
    /// simply leaves you on the synth.
    pub fn load(path: impl AsRef<Path>, sample_rate: f32) -> Result<Self, String> {
        let file = std::fs::File::open(path.as_ref())
            .map_err(|e| format!("could not open soundfont: {e}"))?;
        // The parser reads the preset tables a couple of bytes at a time; on an
        // unbuffered File every one of those is a syscall, which turns a large
        // soundfont into seconds of pure read overhead.
        let mut file = std::io::BufReader::with_capacity(1 << 16, file);
        let sf = SoundFont::new(&mut file).map_err(|e| format!("not a usable soundfont: {e}"))?;
        Self::from_soundfont(Arc::new(sf), sample_rate)
    }

    /// Build from an already-parsed SoundFont — used on the web, where the file
    /// arrives as bytes rather than a path.
    pub fn from_soundfont(sf: Arc<SoundFont>, sample_rate: f32) -> Result<Self, String> {
        let settings = SynthesizerSettings::new(sample_rate as i32);
        let mut synth = Synthesizer::new(&sf, &settings)
            .map_err(|e| format!("could not start the synthesizer: {e}"))?;
        for inst in Instrument::ALL {
            synth.process_midi_message(channel_of(inst), 0xC0, program_of(inst), 0);
        }
        Ok(Self {
            synth,
            notes: Vec::with_capacity(64),
            offs: Vec::with_capacity(64),
            quiet_after: 0.0,
            rendered_to: 0.0,
            left: Vec::new(),
            right: Vec::new(),
            mono: Vec::new(),
        })
    }

    /// Queue a note at absolute time `at`, lasting `dur` seconds.
    pub fn note(&mut self, instrument: Instrument, midi: i32, gain: f32, dur: f32, at: f32) {
        let channel = channel_of(instrument);
        self.notes.push(PendingNote {
            at,
            channel,
            key: midi,
            velocity: velocity_for(gain),
            dur: dur.max(0.05),
        });
    }

    /// Stop everything at once.
    pub fn all_off(&mut self) {
        self.notes.clear();
        self.offs.clear();
        self.synth.note_off_all(false);
        self.quiet_after = self.rendered_to + RELEASE_TAIL;
    }

    /// Whether the synth still needs to be rendered: an event is queued, or a
    /// released voice may still be inside its decay tail.
    pub fn is_active(&self) -> bool {
        !self.notes.is_empty() || !self.offs.is_empty() || self.rendered_to < self.quiet_after
    }

    /// Drop queued notes due at or after `from`, keeping earlier ones. Note-offs
    /// are derived when a note fires, so removing a queued note can never leave a
    /// stale note-off behind to cut a re-queued note short.
    pub fn clear_pending_from(&mut self, from: f32) {
        self.notes.retain(|n| n.at < from);
    }

    /// Render `frames` mono samples covering the window starting at `start_time`,
    /// returning a slice of an internal buffer — no allocation, safe for the
    /// real-time callback.
    ///
    /// Events due inside the window fire before rendering. Note-offs fire before
    /// note-ons so a re-attack of the same key in one window is not killed by the
    /// previous note's release.
    pub fn render_mono(&mut self, frames: usize, start_time: f32, dt: f32) -> &[f32] {
        let end = start_time + frames as f32 * dt;

        let mut i = 0;
        while i < self.offs.len() {
            if self.offs[i].at <= end {
                let o = self.offs.swap_remove(i);
                self.synth.note_off(o.channel, o.key);
                self.quiet_after = self.quiet_after.max(o.at + RELEASE_TAIL);
            } else {
                i += 1;
            }
        }
        let mut i = 0;
        while i < self.notes.len() {
            if self.notes[i].at <= end {
                let n = self.notes.swap_remove(i);
                self.synth.note_on(n.channel, n.key, n.velocity);
                self.offs.push(PendingOff { at: n.at + n.dur, channel: n.channel, key: n.key });
            } else {
                i += 1;
            }
        }

        if self.left.len() != frames {
            self.left.resize(frames, 0.0);
            self.right.resize(frames, 0.0);
            self.mono.resize(frames, 0.0);
        }
        self.synth.render(&mut self.left, &mut self.right);
        // The rest of the engine is mono, so fold the stereo render down.
        for ((m, l), r) in self.mono.iter_mut().zip(&self.left).zip(&self.right) {
            *m = (l + r) * 0.5;
        }
        self.rendered_to = end;
        &self.mono
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn instruments_get_distinct_channels_and_sensible_programs() {
        let mut seen = std::collections::HashSet::new();
        for i in Instrument::ALL {
            assert!(seen.insert(channel_of(i)), "{i:?} shares a channel");
            assert_ne!(channel_of(i), 9, "{i:?} landed on the GM drum channel");
            assert!((0..128).contains(&program_of(i)));
        }
        // General MIDI program numbers for the pitched voices.
        assert_eq!(program_of(Instrument::Bass), 32); // Acoustic Bass
        assert_eq!(program_of(Instrument::Vibes), 11); // Vibraphone
        assert_eq!(program_of(Instrument::Keys), 4); // Electric Piano 1
        assert_eq!(program_of(Instrument::Piano), 0); // Acoustic Grand
        assert_eq!(program_of(Instrument::Guitar), 26); // Jazz Guitar
        assert_eq!(program_of(Instrument::Sax), 66); // Tenor Sax
        assert_eq!(program_of(Instrument::Trombone), 57); // Trombone
        assert_eq!(program_of(Instrument::Trumpet), 56); // Trumpet
    }

    #[test]
    fn velocities_keep_the_band_in_balance() {
        // The comping (gain ~0.05) must not whisper under the bass (~0.17):
        // sampled loudness is roughly quadratic in velocity, so the mapping
        // has to lift the quiet parts.
        let comp = velocity_for(0.055);
        let bass = velocity_for(0.17);
        assert!(comp >= 55, "comping still whispers at velocity {comp}");
        assert!((95..=127).contains(&bass), "bass velocity {bass} out of range");
        assert!(velocity_for(0.01) >= 24, "floor must keep ghost notes audible");
        assert!(velocity_for(0.5) <= 127);
        assert!(velocity_for(0.1) > velocity_for(0.05), "mapping must stay monotonic");
    }

    #[test]
    fn a_missing_file_reports_an_error_instead_of_panicking() {
        let r = SoundfontVoices::load("/definitely/not/here.sf2", 44_100.0);
        assert!(r.is_err(), "a missing soundfont must not panic");
    }

    #[test]
    fn a_file_that_is_not_a_soundfont_is_rejected() {
        let path = std::env::temp_dir().join("jazz-not-a-soundfont.sf2");
        std::fs::write(&path, b"this is plainly not a soundfont").unwrap();
        let r = SoundfontVoices::load(&path, 44_100.0);
        let _ = std::fs::remove_file(&path);
        assert!(r.is_err(), "garbage must be rejected, not loaded");
    }
}
