//! Instrument models.
//!
//! Each voice renders itself sample by sample, so the whole engine is a pure
//! function of time with no allocation in the audio path. The models mirror the
//! browser version: an FM electric piano, an upright bass with a finger
//! transient, a struck vibraphone bar, and a genuinely inharmonic ride cymbal.

use core::f32::consts::TAU;

/// Deterministic noise source — a real cymbal needs noise, but an audio thread
/// must never allocate or lock, so this is a tiny xorshift.
#[derive(Debug, Clone, Copy)]
pub struct Noise(u32);

impl Noise {
    pub fn new(seed: u32) -> Self {
        Self(seed | 1)
    }
    #[inline]
    pub fn next(&mut self) -> f32 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.0 = x;
        (x as f32 / u32::MAX as f32) * 2.0 - 1.0
    }
}

/// Exponential decay from 1.0 to ~0 over `time_constant` seconds.
#[inline]
fn decay(t: f32, time_constant: f32) -> f32 {
    (-t / time_constant.max(1e-4)).exp()
}

/// A short raised-cosine fade-in, so notes start without a click.
#[inline]
fn attack(t: f32, len: f32) -> f32 {
    if t >= len {
        1.0
    } else if t <= 0.0 {
        0.0
    } else {
        0.5 - 0.5 * (TAU * 0.5 * t / len).cos()
    }
}

/// Which instrument a note should be played on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Instrument {
    /// FM electric piano — comping.
    Keys,
    /// Upright bass — walking lines.
    Bass,
    /// Vibraphone — drills, theory lab, ear training.
    Vibes,
}

/// A sounding note.
#[derive(Debug, Clone, Copy)]
pub struct Voice {
    pub instrument: Instrument,
    pub freq: f32,
    pub gain: f32,
    /// Seconds since the note started.
    pub t: f32,
    /// How long the note is held before release.
    pub dur: f32,
    noise: Noise,
    done: bool,
}

impl Voice {
    pub fn new(instrument: Instrument, freq: f32, gain: f32, dur: f32, seed: u32) -> Self {
        Self {
            instrument,
            freq,
            gain,
            t: 0.0,
            dur: dur.max(0.03),
            noise: Noise::new(seed),
            done: false,
        }
    }

    pub fn is_done(&self) -> bool {
        self.done
    }

    /// Total lifetime including release tail.
    fn lifetime(&self) -> f32 {
        self.dur
            + match self.instrument {
                Instrument::Keys => 0.6,
                Instrument::Bass => 0.25,
                Instrument::Vibes => 1.0,
            }
    }

    /// Render one sample and advance by `dt` seconds.
    pub fn tick(&mut self, dt: f32) -> f32 {
        if self.done {
            return 0.0;
        }
        let t = self.t;
        if t > self.lifetime() {
            self.done = true;
            return 0.0;
        }
        let out = match self.instrument {
            Instrument::Keys => self.keys(t),
            Instrument::Bass => self.bass(t),
            Instrument::Vibes => self.vibes(t),
        };
        self.t += dt;
        out * self.gain
    }

    /// FM electric piano: a bell-like modulation index that decays into a sine
    /// body — the thing that makes a Rhodes sound warm rather than buzzy.
    fn keys(&self, t: f32) -> f32 {
        let index = 2.4 * decay(t, 0.09);
        let modulator = (TAU * self.freq * 2.0 * t).sin() * index;
        let body = (TAU * self.freq * t + modulator).sin();
        let env = attack(t, 0.006)
            * if t < self.dur {
                0.35 + 0.65 * decay(t, 0.45)
            } else {
                (0.35 + 0.65 * decay(self.dur, 0.45)) * decay(t - self.dur, 0.18)
            };
        body * env * 0.5
    }

    /// Upright bass: pitch drop, fast filter-like harmonic decay, finger noise.
    fn bass(&mut self, t: f32) -> f32 {
        let f = self.freq * (1.0 + 0.045 * decay(t, 0.02));
        let phase = TAU * f * t;
        // a few harmonics whose upper partials die quickly = a closing filter
        let bright = decay(t, 0.10);
        let tone = phase.sin()
            + 0.5 * bright * (2.0 * phase).sin()
            + 0.25 * bright * bright * (3.0 * phase).sin();
        let env = attack(t, 0.008)
            * if t < self.dur {
                0.55 + 0.45 * decay(t, 0.30)
            } else {
                (0.55 + 0.45 * decay(self.dur, 0.30)) * decay(t - self.dur, 0.06)
            };
        let finger = if t < 0.05 {
            self.noise.next() * decay(t, 0.012) * 0.25
        } else {
            0.0
        };
        (tone * 0.42 + finger) * env
    }

    /// Struck metal bar: fundamental plus the characteristic 4th and ~10th
    /// partials, with a slow tremolo.
    fn vibes(&self, t: f32) -> f32 {
        let p = TAU * self.freq * t;
        let body =
            p.sin() + 0.30 * (4.0 * p).sin() * decay(t, 0.5) + 0.12 * (9.2 * p).sin() * decay(t, 0.25);
        let trem = 1.0 - 0.16 * (1.0 - (TAU * 4.4 * t).sin()) * 0.5;
        body * attack(t, 0.004) * decay(t, 0.9) * trem * 0.45
    }
}

/// A ride cymbal: six inharmonic partials, split into a bright stick "ting"
/// and a longer wash. Harmonic oscillators cannot sound like a cymbal.
#[derive(Debug, Clone, Copy)]
pub struct Ride {
    pub t: f32,
    pub gain: f32,
    pub decay_s: f32,
    base: f32,
    done: bool,
}

const RIDE_RATIOS: [f32; 6] = [1.0, 1.4471, 1.6170, 1.9265, 2.5028, 2.6637];

impl Ride {
    pub fn new(gain: f32, decay_s: f32) -> Self {
        Self { t: 0.0, gain, decay_s, base: 286.0, done: false }
    }
    pub fn is_done(&self) -> bool {
        self.done
    }
    pub fn tick(&mut self, dt: f32) -> f32 {
        if self.done {
            return 0.0;
        }
        if self.t > self.decay_s + 0.05 {
            self.done = true;
            return 0.0;
        }
        let t = self.t;
        let mut sum = 0.0;
        for r in RIDE_RATIOS {
            // square-ish partial, cheap: sign of a sine
            let s = (TAU * self.base * r * t).sin();
            sum += if s >= 0.0 { 1.0 } else { -1.0 };
        }
        sum /= RIDE_RATIOS.len() as f32;
        let ting = sum * decay(t, 0.035) * 0.9;
        let wash = sum * decay(t, self.decay_s * 0.35) * 0.35;
        self.t += dt;
        (ting + wash) * self.gain * 0.5
    }
}

/// Brushes, hi-hat and the kick: short filtered noise or a pitched thump.
#[derive(Debug, Clone, Copy)]
pub enum PercKind {
    Brush,
    HiHat,
    Kick,
}

#[derive(Debug, Clone, Copy)]
pub struct Perc {
    kind: PercKind,
    t: f32,
    gain: f32,
    len: f32,
    noise: Noise,
    lp: f32,
    done: bool,
}

impl Perc {
    pub fn new(kind: PercKind, gain: f32, seed: u32) -> Self {
        let len = match kind {
            PercKind::Brush => 0.12,
            PercKind::HiHat => 0.05,
            PercKind::Kick => 0.22,
        };
        Self { kind, t: 0.0, gain, len, noise: Noise::new(seed), lp: 0.0, done: false }
    }
    pub fn is_done(&self) -> bool {
        self.done
    }
    pub fn tick(&mut self, dt: f32) -> f32 {
        if self.done {
            return 0.0;
        }
        if self.t > self.len {
            self.done = true;
            return 0.0;
        }
        let t = self.t;
        let v = match self.kind {
            PercKind::Kick => {
                let f = 115.0 * decay(t, 0.045) + 42.0;
                (TAU * f * t).sin() * decay(t, 0.06)
            }
            PercKind::HiHat => self.noise.next() * decay(t, 0.012),
            PercKind::Brush => {
                // one-pole low-pass on noise gives the soft brushed hiss
                let n = self.noise.next();
                self.lp += (n - self.lp) * 0.35;
                self.lp * decay(t, 0.045) * attack(t, 0.006)
            }
        };
        self.t += dt;
        v * self.gain
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn render<F: FnMut(f32) -> f32>(secs: f32, sr: f32, mut f: F) -> Vec<f32> {
        let n = (secs * sr) as usize;
        (0..n).map(|_| f(1.0 / sr)).collect()
    }
    fn peak(v: &[f32]) -> f32 {
        v.iter().fold(0.0f32, |a, b| a.max(b.abs()))
    }
    fn rms(v: &[f32]) -> f32 {
        (v.iter().map(|x| x * x).sum::<f32>() / v.len() as f32).sqrt()
    }

    #[test]
    fn noise_is_bounded_and_not_constant() {
        let mut n = Noise::new(12345);
        let v: Vec<f32> = (0..2000).map(|_| n.next()).collect();
        assert!(v.iter().all(|x| (-1.0..=1.0).contains(x)));
        assert!(rms(&v) > 0.3, "noise should actually be noisy");
    }

    #[test]
    fn every_instrument_produces_sound_and_then_stops() {
        for inst in [Instrument::Keys, Instrument::Bass, Instrument::Vibes] {
            let mut v = Voice::new(inst, 220.0, 0.5, 0.4, 7);
            let out = render(3.0, 44100.0, |dt| v.tick(dt));
            assert!(peak(&out) > 0.01, "{inst:?} produced silence");
            assert!(peak(&out) <= 1.0, "{inst:?} clipped at {}", peak(&out));
            assert!(v.is_done(), "{inst:?} never finished");
        }
    }

    #[test]
    fn notes_start_from_silence_so_there_is_no_click() {
        for inst in [Instrument::Keys, Instrument::Bass, Instrument::Vibes] {
            let mut v = Voice::new(inst, 220.0, 0.8, 0.4, 3);
            let first = v.tick(1.0 / 44100.0);
            assert!(first.abs() < 0.02, "{inst:?} starts at {first}, that will click");
        }
    }

    #[test]
    fn a_finished_voice_stays_silent() {
        let mut v = Voice::new(Instrument::Vibes, 440.0, 0.5, 0.1, 1);
        let _ = render(5.0, 22050.0, |dt| v.tick(dt));
        assert!(v.is_done());
        assert_eq!(v.tick(1.0 / 22050.0), 0.0);
    }

    #[test]
    fn gain_scales_output_linearly() {
        let mk = |g: f32| {
            let mut v = Voice::new(Instrument::Vibes, 330.0, g, 0.3, 5);
            peak(&render(1.0, 44100.0, |dt| v.tick(dt)))
        };
        let (a, b) = (mk(0.25), mk(0.5));
        assert!((b / a - 2.0).abs() < 0.05, "expected linear gain, got {a} and {b}");
    }

    #[test]
    fn ride_is_inharmonic_and_decays() {
        let mut r = Ride::new(0.6, 0.9);
        let out = render(1.5, 44100.0, |dt| r.tick(dt));
        assert!(peak(&out) > 0.01);
        assert!(r.is_done());
        // energy in the first 50 ms should exceed the tail — that is the "ting"
        let head = rms(&out[..2205]);
        let tail = rms(&out[out.len() - 2205..]);
        assert!(head > tail * 5.0, "ride should decay: head {head}, tail {tail}");
    }

    #[test]
    fn percussion_voices_all_sound_and_terminate() {
        for k in [PercKind::Brush, PercKind::HiHat, PercKind::Kick] {
            let mut p = Perc::new(k, 0.8, 11);
            let out = render(1.0, 44100.0, |dt| p.tick(dt));
            assert!(peak(&out) > 0.005, "{k:?} was silent");
            assert!(p.is_done(), "{k:?} never finished");
        }
    }

    #[test]
    fn output_is_always_finite() {
        for inst in [Instrument::Keys, Instrument::Bass, Instrument::Vibes] {
            for f in [55.0, 110.0, 440.0, 1760.0] {
                let mut v = Voice::new(inst, f, 1.0, 0.5, 9);
                let out = render(2.0, 44100.0, |dt| v.tick(dt));
                assert!(out.iter().all(|x| x.is_finite()), "{inst:?} at {f}Hz went non-finite");
            }
        }
    }
}
