//! # jazz-audio
//!
//! The synthesised rhythm section: instrument models ([`voice`]) and the swing
//! scheduler that decides what they play ([`engine`]).
//!
//! The crate is deliberately free of any audio-backend dependency. It produces
//! samples and events; wiring those to a device (cpal, Bevy, an offline WAV
//! render) is the caller's job. That keeps the musical logic testable without a
//! sound card — every test in here runs offline.
//!
//! ```
//! use jazz_audio::{Mixer, engine::{Settings, Rng, plan_bar}};
//! use jazz_theory::progression;
//!
//! let bars = progression::two_five_one(0);
//! let mut rng = Rng::new(1);
//! let plan = plan_bar(&bars, 0, &Settings::default(), &mut rng);
//!
//! let mut mixer = Mixer::new(44_100.0);
//! mixer.schedule(&plan, 0.0);
//! let buf = mixer.render(2.0);
//! assert!(buf.iter().any(|s| s.abs() > 0.001));
//! ```

#![forbid(unsafe_code)]

pub mod engine;
#[cfg(feature = "soundfont")]
pub mod soundfont;
pub mod stream;
pub mod voice;

use engine::{BarPlan, PercRole};
use jazz_theory::note::midi_to_freq;
use voice::{Instrument, Perc, PercKind, Ride, Voice};

/// A scheduled sound waiting for its moment.
enum Pending {
    Note(f32, Voice),
    Ride(f32, Ride),
    Perc(f32, Perc),
}

/// Mixes scheduled voices into a buffer.
///
/// Holds a soft limiter on the output so a dense bar of bass, comping and drums
/// cannot clip — the same job the compressor does in the browser version.
pub struct Mixer {
    sample_rate: f32,
    time: f32,
    pending: Vec<Pending>,
    active_notes: Vec<Voice>,
    active_rides: Vec<Ride>,
    active_perc: Vec<Perc>,
    seed: u32,
    /// Master gain, 0.0 – 1.5.
    pub volume: f32,
}

impl Mixer {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            sample_rate,
            time: 0.0,
            pending: Vec::new(),
            active_notes: Vec::new(),
            active_rides: Vec::new(),
            active_perc: Vec::new(),
            seed: 0x9E37_79B9,
            volume: 1.0,
        }
    }

    fn next_seed(&mut self) -> u32 {
        self.seed = self.seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        self.seed
    }

    /// Queue everything in a bar plan, offset from `bar_start` seconds.
    pub fn schedule(&mut self, plan: &BarPlan, bar_start: f32) {
        for e in &plan.notes {
            let v = Voice::new(
                e.instrument,
                midi_to_freq(e.midi as f32),
                e.gain,
                e.dur,
                self.next_seed(),
            );
            self.pending.push(Pending::Note(bar_start + e.offset, v));
        }
        for p in &plan.perc {
            let at = bar_start + p.offset;
            match p.kind {
                PercRole::Ride => self.pending.push(Pending::Ride(at, Ride::new(p.gain, 1.0))),
                PercRole::RideSkip => self.pending.push(Pending::Ride(at, Ride::new(p.gain, 0.5))),
                PercRole::HiHat => {
                    let s = self.next_seed();
                    self.pending.push(Pending::Perc(at, Perc::new(PercKind::HiHat, p.gain, s)));
                }
                PercRole::Brush => {
                    let s = self.next_seed();
                    self.pending.push(Pending::Perc(at, Perc::new(PercKind::Brush, p.gain, s)));
                }
                PercRole::Kick => {
                    let s = self.next_seed();
                    self.pending.push(Pending::Perc(at, Perc::new(PercKind::Kick, p.gain, s)));
                }
            }
        }
    }

    /// Sound a single note `delay` seconds from the play head.
    ///
    /// Ear training and note previews need one-off notes rather than a whole bar,
    /// and going through `BarPlan` for that would be clumsy.
    pub fn play_note(
        &mut self,
        instrument: Instrument,
        midi: i32,
        gain: f32,
        dur: f32,
        delay: f32,
    ) {
        let seed = self.next_seed();
        let v = Voice::new(instrument, midi_to_freq(midi as f32), gain, dur, seed);
        self.pending.push(Pending::Note(self.time + delay.max(0.0), v));
    }

    /// Sound several notes together — a chord for ear training.
    pub fn play_chord(
        &mut self,
        instrument: Instrument,
        midis: &[i32],
        gain: f32,
        dur: f32,
        delay: f32,
    ) {
        for &m in midis {
            self.play_note(instrument, m, gain, dur, delay);
        }
    }

    /// Silence everything immediately.
    pub fn clear(&mut self) {
        self.pending.clear();
        self.active_notes.clear();
        self.active_rides.clear();
        self.active_perc.clear();
    }

    pub fn time(&self) -> f32 {
        self.time
    }

    pub fn sample_rate(&self) -> f32 {
        self.sample_rate
    }

    /// Adopt the output device's real rate. Anything already scheduled keeps its
    /// wall-clock timing because events are stored in seconds, not samples.
    pub fn set_sample_rate(&mut self, rate: f32) {
        if rate > 0.0 {
            self.sample_rate = rate;
        }
    }

    /// How many sounds are currently audible — handy for leak tests.
    pub fn active_count(&self) -> usize {
        self.active_notes.len() + self.active_rides.len() + self.active_perc.len()
    }

    /// Render one sample and advance the clock.
    pub fn tick(&mut self) -> f32 {
        let dt = 1.0 / self.sample_rate;
        let now = self.time;

        // promote anything whose time has come
        let mut i = 0;
        while i < self.pending.len() {
            let due = match &self.pending[i] {
                Pending::Note(t, _) | Pending::Ride(t, _) | Pending::Perc(t, _) => *t <= now,
            };
            if due {
                match self.pending.swap_remove(i) {
                    Pending::Note(_, v) => self.active_notes.push(v),
                    Pending::Ride(_, r) => self.active_rides.push(r),
                    Pending::Perc(_, p) => self.active_perc.push(p),
                }
            } else {
                i += 1;
            }
        }

        let mut sum = 0.0;
        for v in &mut self.active_notes {
            sum += v.tick(dt);
        }
        for r in &mut self.active_rides {
            sum += r.tick(dt);
        }
        for p in &mut self.active_perc {
            sum += p.tick(dt);
        }
        self.active_notes.retain(|v| !v.is_done());
        self.active_rides.retain(|r| !r.is_done());
        self.active_perc.retain(|p| !p.is_done());

        self.time += dt;
        soft_clip(sum * self.volume)
    }

    /// Render `seconds` of audio into a fresh buffer.
    pub fn render(&mut self, seconds: f32) -> Vec<f32> {
        let n = (seconds * self.sample_rate) as usize;
        (0..n).map(|_| self.tick()).collect()
    }

    /// Fill an existing buffer, e.g. a cpal callback.
    pub fn fill(&mut self, out: &mut [f32]) {
        for s in out.iter_mut() {
            *s = self.tick();
        }
    }
}

/// Gentle saturation instead of hard clipping, so a dense bar stays musical.
#[inline]
pub fn soft_clip(x: f32) -> f32 {
    // tanh-like curve without the transcendental cost
    let t = x.clamp(-3.0, 3.0);
    t * (27.0 + t * t) / (27.0 + 9.0 * t * t)
}

#[cfg(test)]
mod tests {
    use super::*;
    use engine::{plan_bar, Density, Rng, Settings};
    use voice::Instrument;
    use jazz_theory::progression;

    fn peak(v: &[f32]) -> f32 {
        v.iter().fold(0.0f32, |a, b| a.max(b.abs()))
    }
    fn rms(v: &[f32]) -> f32 {
        (v.iter().map(|x| x * x).sum::<f32>() / v.len().max(1) as f32).sqrt()
    }

    #[test]
    fn soft_clip_is_bounded_and_linear_near_zero() {
        assert!((soft_clip(0.0)).abs() < 1e-6);
        assert!((soft_clip(0.1) - 0.1).abs() < 0.005, "should be near-linear when quiet");
        for x in [-100.0f32, -3.0, 3.0, 100.0] {
            assert!(soft_clip(x).abs() <= 1.0, "soft clip let {x} through");
        }
    }

    #[test]
    fn a_scheduled_bar_actually_makes_sound() {
        let bars = progression::two_five_one(0);
        let mut rng = Rng::new(1);
        let plan = plan_bar(&bars, 0, &Settings::default(), &mut rng);
        let mut m = Mixer::new(44_100.0);
        m.schedule(&plan, 0.0);
        let buf = m.render(2.5);
        assert!(peak(&buf) > 0.01, "bar was silent");
        assert!(buf.iter().all(|s| s.is_finite()));
    }

    #[test]
    fn a_dense_full_band_never_clips() {
        let bars = progression::rhythm_full();
        let mut rng = Rng::new(77);
        let s = Settings { density: Density::Busy, tempo: 240.0, ..Settings::default() };
        let mut m = Mixer::new(44_100.0);
        let spb = 60.0 / s.tempo;
        for i in 0..8 {
            let plan = plan_bar(&bars, i, &s, &mut rng);
            m.schedule(&plan, i as f32 * spb * 4.0);
        }
        let buf = m.render(9.0);
        assert!(peak(&buf) <= 1.0, "clipped at {}", peak(&buf));
        assert!(rms(&buf) > 0.002, "band was too quiet: {}", rms(&buf));
    }

    #[test]
    fn voices_are_reclaimed_so_a_long_session_does_not_leak() {
        let bars = progression::blues(5);
        let mut rng = Rng::new(5);
        let s = Settings::default();
        let mut m = Mixer::new(22_050.0);
        let bar_len = 4.0 * 60.0 / s.tempo;
        for i in 0..24 {
            let plan = plan_bar(&bars, i, &s, &mut rng);
            m.schedule(&plan, i as f32 * bar_len);
        }
        let _ = m.render(24.0 * bar_len + 3.0);
        assert_eq!(m.active_count(), 0, "voices were not reclaimed");
    }

    #[test]
    fn clear_silences_everything_at_once() {
        let bars = progression::blues(5);
        let mut rng = Rng::new(9);
        let plan = plan_bar(&bars, 0, &Settings::default(), &mut rng);
        let mut m = Mixer::new(44_100.0);
        m.schedule(&plan, 0.0);
        let _ = m.render(0.3);
        m.clear();
        let after = m.render(0.5);
        assert_eq!(peak(&after), 0.0, "clear() left sound ringing");
    }

    #[test]
    fn volume_scales_the_mix() {
        let bars = progression::two_five_one(0);
        let render_at = |vol: f32| {
            let mut rng = Rng::new(3);
            let plan = plan_bar(&bars, 0, &Settings::default(), &mut rng);
            let mut m = Mixer::new(44_100.0);
            m.volume = vol;
            m.schedule(&plan, 0.0);
            rms(&m.render(2.0))
        };
        let quiet = render_at(0.25);
        let loud = render_at(1.0);
        assert!(loud > quiet * 2.0, "volume had little effect: {quiet} vs {loud}");
        // and silence really is silent
        let mut rng = Rng::new(3);
        let plan = plan_bar(&bars, 0, &Settings::default(), &mut rng);
        let mut m = Mixer::new(44_100.0);
        m.volume = 0.0;
        m.schedule(&plan, 0.0);
        assert_eq!(peak(&m.render(1.0)), 0.0);
    }

    #[test]
    fn scheduling_respects_the_requested_time() {
        let mut m = Mixer::new(44_100.0);
        let plan = BarPlan {
            notes: vec![engine::Event {
                offset: 0.5,
                midi: 60,
                gain: 0.5,
                dur: 0.2,
                instrument: Instrument::Vibes,
            }],
            perc: vec![],
            two_feel: false,
        };
        m.schedule(&plan, 0.0);
        let before = m.render(0.45);
        assert_eq!(peak(&before), 0.0, "the note sounded early");
        let after = m.render(0.4);
        assert!(peak(&after) > 0.01, "the note never sounded");
    }

    #[test]
    fn a_single_note_sounds_after_its_delay() {
        let mut m = Mixer::new(44_100.0);
        m.play_note(Instrument::Vibes, 60, 0.5, 0.3, 0.5);
        assert_eq!(peak(&m.render(0.45)), 0.0, "sounded early");
        assert!(peak(&m.render(0.4)) > 0.01, "never sounded");
    }

    #[test]
    fn a_chord_sounds_all_of_its_notes() {
        let mut solo = Mixer::new(44_100.0);
        solo.play_note(Instrument::Keys, 60, 0.2, 0.5, 0.0);
        let one = rms(&solo.render(1.0));

        let mut chord = Mixer::new(44_100.0);
        chord.play_chord(Instrument::Keys, &[60, 64, 67, 71], 0.2, 0.5, 0.0);
        let four = rms(&chord.render(1.0));

        assert!(four > one * 1.5, "a four-note chord should be fuller: {one} vs {four}");
    }

    #[test]
    fn fill_matches_render() {
        let mk = || {
            let bars = progression::two_five_one(0);
            let mut rng = Rng::new(17);
            let plan = plan_bar(&bars, 0, &Settings::default(), &mut rng);
            let mut m = Mixer::new(8_000.0);
            m.schedule(&plan, 0.0);
            m
        };
        let a = mk().render(1.0);
        let mut b = vec![0.0; 8_000];
        mk().fill(&mut b);
        assert_eq!(a, b);
    }
}
