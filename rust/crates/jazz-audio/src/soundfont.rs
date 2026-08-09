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

/// General MIDI programs that suit each instrument.
const PROGRAM_KEYS: i32 = 4; // Electric Piano 1 — the Rhodes the synth voice imitates
const PROGRAM_BASS: i32 = 32; // Acoustic Bass
const PROGRAM_VIBES: i32 = 11; // Vibraphone

/// One MIDI channel per instrument keeps their programs independent.
fn channel_of(i: Instrument) -> i32 {
    match i {
        Instrument::Keys => 0,
        Instrument::Bass => 1,
        Instrument::Vibes => 2,
    }
}

fn program_of(i: Instrument) -> i32 {
    match i {
        Instrument::Keys => PROGRAM_KEYS,
        Instrument::Bass => PROGRAM_BASS,
        Instrument::Vibes => PROGRAM_VIBES,
    }
}

/// A note waiting to start or stop.
#[derive(Debug, Clone, Copy)]
struct Pending {
    at: f32,
    channel: i32,
    key: i32,
    /// Velocity for note-on; `None` means note-off.
    velocity: Option<i32>,
}

/// Sample-based voices driven by a SoundFont.
pub struct SoundfontVoices {
    synth: Synthesizer,
    pending: Vec<Pending>,
    left: Vec<f32>,
    right: Vec<f32>,
}

impl SoundfontVoices {
    /// Load a `.sf2` file. Returns an error rather than panicking, so a bad path
    /// simply leaves you on the synth.
    pub fn load(path: impl AsRef<Path>, sample_rate: f32) -> Result<Self, String> {
        let mut file = std::fs::File::open(path.as_ref())
            .map_err(|e| format!("could not open soundfont: {e}"))?;
        let sf = SoundFont::new(&mut file).map_err(|e| format!("not a usable soundfont: {e}"))?;
        Self::from_soundfont(Arc::new(sf), sample_rate)
    }

    /// Build from an already-parsed SoundFont — used on the web, where the file
    /// arrives as bytes rather than a path.
    pub fn from_soundfont(sf: Arc<SoundFont>, sample_rate: f32) -> Result<Self, String> {
        let settings = SynthesizerSettings::new(sample_rate as i32);
        let mut synth = Synthesizer::new(&sf, &settings)
            .map_err(|e| format!("could not start the synthesizer: {e}"))?;
        for inst in [Instrument::Keys, Instrument::Bass, Instrument::Vibes] {
            synth.process_midi_message(channel_of(inst), 0xC0, program_of(inst), 0);
        }
        Ok(Self {
            synth,
            pending: Vec::with_capacity(64),
            left: Vec::new(),
            right: Vec::new(),
        })
    }

    /// Queue a note at absolute time `at`, lasting `dur` seconds.
    pub fn note(&mut self, instrument: Instrument, midi: i32, gain: f32, dur: f32, at: f32) {
        let channel = channel_of(instrument);
        // The synth voices are trimmed well below unity, so scale up to a sensible
        // MIDI velocity rather than mapping gain straight onto 0-127.
        let velocity = ((gain * 420.0) as i32).clamp(20, 127);
        self.pending.push(Pending { at, channel, key: midi, velocity: Some(velocity) });
        self.pending.push(Pending { at: at + dur.max(0.05), channel, key: midi, velocity: None });
    }

    /// Stop everything at once.
    pub fn all_off(&mut self) {
        self.pending.clear();
        self.synth.note_off_all(false);
    }

    /// Render `frames` mono samples covering the window starting at `start_time`.
    ///
    /// Notes whose moment falls inside the window are triggered before rendering,
    /// which keeps timing accurate to the buffer rather than the frame.
    pub fn render(&mut self, frames: usize, start_time: f32, dt: f32) -> Vec<f32> {
        let end = start_time + frames as f32 * dt;
        // fire anything due in this window, in time order
        self.pending.sort_by(|a, b| a.at.partial_cmp(&b.at).unwrap_or(std::cmp::Ordering::Equal));
        let mut i = 0;
        while i < self.pending.len() {
            if self.pending[i].at <= end {
                let p = self.pending.remove(i);
                match p.velocity {
                    Some(v) => self.synth.note_on(p.channel, p.key, v),
                    None => self.synth.note_off(p.channel, p.key),
                }
            } else {
                i += 1;
            }
        }

        if self.left.len() != frames {
            self.left.resize(frames, 0.0);
            self.right.resize(frames, 0.0);
        }
        self.synth.render(&mut self.left, &mut self.right);
        // The rest of the engine is mono, so fold the stereo render down.
        self.left
            .iter()
            .zip(self.right.iter())
            .map(|(l, r)| (l + r) * 0.5)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn instruments_get_distinct_channels_and_sensible_programs() {
        let mut seen = std::collections::HashSet::new();
        for i in [Instrument::Keys, Instrument::Bass, Instrument::Vibes] {
            assert!(seen.insert(channel_of(i)), "{i:?} shares a channel");
            assert!((0..128).contains(&program_of(i)));
        }
        // General MIDI: 32 is Acoustic Bass, 11 Vibraphone, 4 Electric Piano 1
        assert_eq!(program_of(Instrument::Bass), 32);
        assert_eq!(program_of(Instrument::Vibes), 11);
        assert_eq!(program_of(Instrument::Keys), 4);
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
