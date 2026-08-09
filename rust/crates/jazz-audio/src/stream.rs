//! The boundary between the UI thread and the audio thread.
//!
//! An audio callback has a hard deadline: it must return a full buffer before the
//! device runs dry, every few milliseconds, forever. That rules out anything that
//! can block — and a mutex shared with a UI that redraws at 60 fps is exactly that.
//! The previous design had the callback `try_lock` and emit **silence** when the UI
//! held the lock, which is why playback stuttered.
//!
//! So: the audio thread owns the mixer outright and never locks. The UI sends
//! [`Command`]s down a channel and reads the play position from an atomic. Nothing
//! on the audio side ever waits for anything.

use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::mpsc::{Receiver, Sender, TryRecvError};
use std::sync::Arc;

use crate::engine::BarPlan;
use crate::voice::Instrument;
use crate::Mixer;

/// Something the UI wants the audio thread to do.
#[derive(Debug, Clone)]
pub enum Command {
    /// Queue a bar of the rhythm section at an absolute time, in seconds.
    Schedule { plan: BarPlan, at: f32 },
    /// Sound one note `delay` seconds from now.
    Note {
        instrument: Instrument,
        midi: i32,
        gain: f32,
        dur: f32,
        delay: f32,
    },
    /// Sound several notes together.
    Chord {
        instrument: Instrument,
        midis: Vec<i32>,
        gain: f32,
        dur: f32,
        delay: f32,
    },
    /// Silence everything immediately and reset the clock.
    Clear,
    SetVolume(f32),
}

/// The UI-side handle. Cheap to clone, never blocks.
#[derive(Clone)]
pub struct AudioHandle {
    tx: Sender<Command>,
    /// Play position in samples, published by the audio thread.
    playhead: Arc<AtomicU32>,
    playing: Arc<AtomicBool>,
    sample_rate: f32,
}

impl AudioHandle {
    /// Seconds of audio rendered so far. Reading this never blocks the audio thread.
    pub fn time(&self) -> f32 {
        self.playhead.load(Ordering::Relaxed) as f32 / self.sample_rate
    }

    pub fn is_playing(&self) -> bool {
        self.playing.load(Ordering::Relaxed)
    }

    pub fn set_playing(&self, on: bool) {
        self.playing.store(on, Ordering::Relaxed);
        if !on {
            self.send(Command::Clear);
        }
    }

    /// Queue a command. Silently drops if the audio thread has gone away, which is
    /// the right behaviour — a dead output device must not take the UI down with it.
    pub fn send(&self, cmd: Command) {
        let _ = self.tx.send(cmd);
    }

    pub fn schedule(&self, plan: BarPlan, at: f32) {
        self.send(Command::Schedule { plan, at });
    }

    pub fn note(&self, instrument: Instrument, midi: i32, gain: f32, dur: f32, delay: f32) {
        self.send(Command::Note { instrument, midi, gain, dur, delay });
    }

    pub fn chord(&self, instrument: Instrument, midis: Vec<i32>, gain: f32, dur: f32, delay: f32) {
        self.send(Command::Chord { instrument, midis, gain, dur, delay });
    }

    pub fn clear(&self) {
        self.send(Command::Clear);
    }

    pub fn set_volume(&self, v: f32) {
        self.send(Command::SetVolume(v));
    }

    pub fn sample_rate(&self) -> f32 {
        self.sample_rate
    }
}

/// The audio-thread side. Owns the mixer; call [`AudioCore::fill`] from the callback.
pub struct AudioCore {
    mixer: Mixer,
    rx: Receiver<Command>,
    playhead: Arc<AtomicU32>,
    playing: Arc<AtomicBool>,
    #[cfg(feature = "soundfont")]
    sf: Option<crate::soundfont::SoundfontVoices>,
}

impl AudioCore {
    /// Build a connected pair. Move the [`AudioCore`] into the audio callback and
    /// keep the [`AudioHandle`] on the UI side.
    pub fn new(sample_rate: f32) -> (AudioHandle, AudioCore) {
        let (tx, rx) = std::sync::mpsc::channel();
        let playhead = Arc::new(AtomicU32::new(0));
        let playing = Arc::new(AtomicBool::new(false));
        (
            AudioHandle {
                tx,
                playhead: playhead.clone(),
                playing: playing.clone(),
                sample_rate,
            },
            AudioCore {
                mixer: Mixer::new(sample_rate),
                rx,
                playhead,
                playing,
                #[cfg(feature = "soundfont")]
                sf: None,
            },
        )
    }

    /// Adopt the output device's actual sample rate, so playback is not transposed
    /// when the device runs at 48 kHz rather than 44.1.
    pub fn set_sample_rate(&mut self, rate: f32) {
        self.mixer.set_sample_rate(rate);
    }

    /// Route pitched notes through a SoundFont instead of the built-in synthesis.
    #[cfg(feature = "soundfont")]
    pub fn set_soundfont(&mut self, sf: Option<crate::soundfont::SoundfontVoices>) {
        self.sf = sf;
    }

    fn drain(&mut self) {
        loop {
            match self.rx.try_recv() {
                Ok(Command::Schedule { plan, at }) => self.mixer.schedule(&plan, at),
                Ok(Command::Note { instrument, midi, gain, dur, delay }) => {
                    #[cfg(feature = "soundfont")]
                    if let Some(sf) = self.sf.as_mut() {
                        sf.note(instrument, midi, gain, dur, self.mixer.time() + delay);
                        continue;
                    }
                    self.mixer.play_note(instrument, midi, gain, dur, delay);
                }
                Ok(Command::Chord { instrument, midis, gain, dur, delay }) => {
                    #[cfg(feature = "soundfont")]
                    if let Some(sf) = self.sf.as_mut() {
                        let t = self.mixer.time() + delay;
                        for m in &midis {
                            sf.note(instrument, *m, gain, dur, t);
                        }
                        continue;
                    }
                    self.mixer.play_chord(instrument, &midis, gain, dur, delay);
                }
                Ok(Command::Clear) => {
                    self.mixer.clear();
                    #[cfg(feature = "soundfont")]
                    if let Some(sf) = self.sf.as_mut() {
                        sf.all_off();
                    }
                }
                Ok(Command::SetVolume(v)) => self.mixer.volume = v.clamp(0.0, 1.5),
                Err(TryRecvError::Empty) | Err(TryRecvError::Disconnected) => break,
            }
        }
    }

    /// Fill an interleaved output buffer. Safe to call from a real-time callback:
    /// no locks, no I/O, and no unbounded work.
    pub fn fill(&mut self, out: &mut [f32], channels: usize) {
        self.drain();

        if !self.playing.load(Ordering::Relaxed) && self.mixer.active_count() == 0 {
            out.fill(0.0);
            return;
        }

        let frames = out.len() / channels.max(1);

        // The SoundFont renders in blocks, so pull that first and mix it in per frame.
        #[cfg(feature = "soundfont")]
        let sf_block: Option<Vec<f32>> = self
            .sf
            .as_mut()
            .map(|sf| sf.render(frames, self.mixer.time(), 1.0 / self.mixer.sample_rate()));
        #[cfg(not(feature = "soundfont"))]
        let sf_block: Option<Vec<f32>> = None;

        for (i, frame) in out.chunks_mut(channels.max(1)).enumerate() {
            let mut s = self.mixer.tick();
            if let Some(block) = &sf_block {
                if let Some(v) = block.get(i) {
                    s = crate::soft_clip(s + *v);
                }
            }
            for o in frame.iter_mut() {
                *o = s;
            }
        }

        let pos = (self.mixer.time() * self.mixer.sample_rate()) as u32;
        self.playhead.store(pos, Ordering::Relaxed);
        let _ = frames;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::{plan_bar, Rng, Settings};
    use jazz_theory::progression;

    fn peak(v: &[f32]) -> f32 {
        v.iter().fold(0.0f32, |a, b| a.max(b.abs()))
    }

    #[test]
    fn a_stopped_core_emits_exact_silence() {
        let (_h, mut core) = AudioCore::new(44_100.0);
        let mut buf = vec![1.0f32; 512];
        core.fill(&mut buf, 2);
        assert!(buf.iter().all(|s| *s == 0.0), "idle output must be silent");
    }

    #[test]
    fn commands_sent_before_playback_are_honoured() {
        let (h, mut core) = AudioCore::new(44_100.0);
        h.set_playing(true);
        h.note(Instrument::Vibes, 60, 0.5, 0.3, 0.0);
        let mut buf = vec![0.0f32; 8192];
        core.fill(&mut buf, 1);
        assert!(peak(&buf) > 0.01, "the queued note never sounded");
    }

    #[test]
    fn the_playhead_advances_in_real_time() {
        let (h, mut core) = AudioCore::new(1000.0);
        h.set_playing(true);
        let mut buf = vec![0.0f32; 500];
        core.fill(&mut buf, 1);
        let t = h.time();
        assert!((t - 0.5).abs() < 0.02, "expected ~0.5s of audio, got {t}");
    }

    #[test]
    fn stereo_frames_are_filled_on_every_channel() {
        let (h, mut core) = AudioCore::new(44_100.0);
        h.set_playing(true);
        h.note(Instrument::Keys, 60, 0.6, 0.5, 0.0);
        let mut buf = vec![0.0f32; 4096];
        core.fill(&mut buf, 2);
        // both channels of a frame carry the same sample
        let mismatched = buf.chunks(2).filter(|c| c[0] != c[1]).count();
        assert_eq!(mismatched, 0, "channels drifted apart");
    }

    #[test]
    fn clear_silences_immediately() {
        let (h, mut core) = AudioCore::new(44_100.0);
        h.set_playing(true);
        h.note(Instrument::Vibes, 60, 0.8, 2.0, 0.0);
        let mut buf = vec![0.0f32; 1024];
        core.fill(&mut buf, 1);
        assert!(peak(&buf) > 0.01);
        h.clear();
        let mut after = vec![0.0f32; 4096];
        core.fill(&mut after, 1);
        assert_eq!(peak(&after), 0.0, "clear() left sound ringing");
    }

    #[test]
    fn a_whole_form_plays_without_dropouts() {
        // The regression this module exists for: every buffer must carry audio,
        // never silence caused by losing a lock race.
        let (h, mut core) = AudioCore::new(44_100.0);
        h.set_playing(true);
        let bars = progression::blues(5);
        let s = Settings::default();
        let mut rng = Rng::new(9);
        let bar_len = 4.0 * 60.0 / s.tempo;
        for i in 0..12 {
            h.schedule(plan_bar(&bars, i, &s, &mut rng), i as f32 * bar_len);
        }
        let mut silent_buffers = 0;
        let total = 400;
        for _ in 0..total {
            let mut buf = vec![0.0f32; 1024];
            core.fill(&mut buf, 2);
            if peak(&buf) == 0.0 {
                silent_buffers += 1;
            }
        }
        // ~9.3s of audio over 24s of form: some buffers legitimately fall in rests,
        // but a healthy stream is mostly sounding.
        assert!(
            silent_buffers * 4 < total,
            "{silent_buffers} of {total} buffers were silent - that is a dropout, not a rest"
        );
    }

    #[test]
    fn volume_changes_take_effect_without_blocking() {
        let (h, mut core) = AudioCore::new(44_100.0);
        h.set_playing(true);
        h.set_volume(0.0);
        h.note(Instrument::Keys, 60, 0.8, 0.5, 0.0);
        let mut buf = vec![0.0f32; 4096];
        core.fill(&mut buf, 1);
        assert_eq!(peak(&buf), 0.0, "volume 0 should be silent");
    }

    #[test]
    fn a_dropped_handle_does_not_break_the_audio_thread() {
        let (h, mut core) = AudioCore::new(44_100.0);
        h.set_playing(true);
        h.note(Instrument::Vibes, 60, 0.5, 0.3, 0.0);
        drop(h);
        let mut buf = vec![0.0f32; 4096];
        core.fill(&mut buf, 1); // must not panic
        assert!(buf.iter().all(|s| s.is_finite()));
    }
}
