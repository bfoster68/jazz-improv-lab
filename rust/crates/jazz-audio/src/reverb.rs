//! A small room for the band: a Freeverb-style send reverb.
//!
//! The classic topology — eight damped comb filters in parallel feeding four
//! allpasses in series, per channel, with the right channel's delay lines
//! detuned by a fixed spread so the tail decorrelates into real stereo. Mono
//! in (the mix), stereo out (the room).
//!
//! Written by hand so `jazz-audio` stays dependency-free, and written for the
//! audio callback: every buffer is allocated up front, `process` never
//! allocates, and `mute` is a plain clear for hard stops.

/// Freeverb's tunings, in samples at 44.1 kHz; scaled to the device rate.
const COMB_TUNINGS: [usize; 8] = [1116, 1188, 1277, 1356, 1422, 1491, 1557, 1617];
const ALLPASS_TUNINGS: [usize; 4] = [556, 441, 341, 225];
/// The right channel runs this many samples longer — the stereo spread.
const SPREAD: usize = 23;

struct Comb {
    buf: Vec<f32>,
    idx: usize,
    feedback: f32,
    damp: f32,
    filter_store: f32,
}

impl Comb {
    fn new(len: usize, feedback: f32, damp: f32) -> Self {
        Self { buf: vec![0.0; len.max(1)], idx: 0, feedback, damp, filter_store: 0.0 }
    }

    #[inline]
    fn process(&mut self, input: f32) -> f32 {
        let out = self.buf[self.idx];
        // One-pole low-pass in the feedback loop: high frequencies die faster,
        // the way air soaks them up in a real room.
        self.filter_store = out * (1.0 - self.damp) + self.filter_store * self.damp;
        self.buf[self.idx] = input + self.filter_store * self.feedback;
        self.idx = (self.idx + 1) % self.buf.len();
        out
    }

    fn mute(&mut self) {
        self.buf.fill(0.0);
        self.filter_store = 0.0;
    }
}

struct Allpass {
    buf: Vec<f32>,
    idx: usize,
}

impl Allpass {
    fn new(len: usize) -> Self {
        Self { buf: vec![0.0; len.max(1)], idx: 0 }
    }

    #[inline]
    fn process(&mut self, input: f32) -> f32 {
        let buffered = self.buf[self.idx];
        let out = buffered - input;
        self.buf[self.idx] = input + buffered * 0.5;
        self.idx = (self.idx + 1) % self.buf.len();
        out
    }

    fn mute(&mut self) {
        self.buf.fill(0.0);
    }
}

/// The room. Feed it the (post-volume) mono mix each sample; it returns the
/// stereo wet signal to add on top of the dry.
pub struct Reverb {
    combs_l: Vec<Comb>,
    combs_r: Vec<Comb>,
    allpasses_l: Vec<Allpass>,
    allpasses_r: Vec<Allpass>,
    /// Wet level, applied on output.
    wet: f32,
}

impl Reverb {
    /// Comb feedback on Freeverb's scale (it maps room size upward: raw 0.84
    /// would decay in a blink). 0.92 gives roughly a 2.5-3 s tail — a club,
    /// not a cathedral — inside the engine's 3 s tail budget.
    pub const DEFAULT_ROOM: f32 = 0.92;
    /// High-frequency absorption in the feedback loop.
    pub const DEFAULT_DAMP: f32 = 0.35;
    /// How much room is mixed on top of the dry band.
    pub const DEFAULT_WET: f32 = 0.28;

    /// A medium jazz-club room. The three numbers worth tuning by ear live in
    /// the `DEFAULT_*` consts and on the Settings tab.
    pub fn new(sample_rate: f32) -> Self {
        let room = Self::DEFAULT_ROOM;
        let damp = Self::DEFAULT_DAMP;
        let wet = Self::DEFAULT_WET;
        let scale = (sample_rate / 44_100.0).max(0.1);
        let sized = |n: usize| ((n as f32 * scale) as usize).max(1);
        Self {
            combs_l: COMB_TUNINGS.iter().map(|&n| Comb::new(sized(n), room, damp)).collect(),
            combs_r: COMB_TUNINGS
                .iter()
                .map(|&n| Comb::new(sized(n + SPREAD), room, damp))
                .collect(),
            allpasses_l: ALLPASS_TUNINGS.iter().map(|&n| Allpass::new(sized(n))).collect(),
            allpasses_r: ALLPASS_TUNINGS
                .iter()
                .map(|&n| Allpass::new(sized(n + SPREAD)))
                .collect(),
            wet,
        }
    }

    /// One sample of mix in, one stereo sample of room out. No allocation.
    #[inline]
    pub fn process(&mut self, input: f32) -> (f32, f32) {
        // Freeverb's fixed input attenuation keeps eight parallel combs from
        // running away.
        let x = input * 0.015;
        let mut l = 0.0;
        for c in &mut self.combs_l {
            l += c.process(x);
        }
        let mut r = 0.0;
        for c in &mut self.combs_r {
            r += c.process(x);
        }
        for a in &mut self.allpasses_l {
            l = a.process(l);
        }
        for a in &mut self.allpasses_r {
            r = a.process(r);
        }
        (l * self.wet, r * self.wet)
    }

    /// Retune the room live: comb feedback (size), damping and wet level are
    /// plain scalars, so this touches no buffers — allocation-free and safe to
    /// call from the callback's command drain. Inputs are clamped hard: comb
    /// feedback at or above 1.0 is a self-oscillating room.
    pub fn set_params(&mut self, room: f32, damp: f32, wet: f32) {
        let room = room.clamp(0.70, 0.97);
        let damp = damp.clamp(0.0, 0.95);
        self.wet = wet.clamp(0.0, 1.0);
        for c in self.combs_l.iter_mut().chain(self.combs_r.iter_mut()) {
            c.feedback = room;
            c.damp = damp;
        }
    }

    /// Hard stop: empty the room. Cheap enough for the callback — it is a
    /// memset over ~100 KB, and only runs on an explicit Clear.
    pub fn mute(&mut self) {
        for c in self.combs_l.iter_mut().chain(self.combs_r.iter_mut()) {
            c.mute();
        }
        for a in self.allpasses_l.iter_mut().chain(self.allpasses_r.iter_mut()) {
            a.mute();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn peak(v: &[f32]) -> f32 {
        v.iter().fold(0.0f32, |a, b| a.max(b.abs()))
    }

    #[test]
    fn an_impulse_grows_a_tail_that_decays() {
        let mut rv = Reverb::new(44_100.0);
        let (mut ls, mut rs) = (Vec::new(), Vec::new());
        for i in 0..(44_100 * 4) {
            let x = if i == 0 { 1.0 } else { 0.0 };
            let (l, r) = rv.process(x);
            ls.push(l);
            rs.push(r);
        }
        // Energy appears after the impulse (it is a tail, not a passthrough)...
        assert!(peak(&ls[2000..8000]) > 1e-4, "no early reflections");
        // ...keeps ringing well past it...
        assert!(peak(&ls[44_100..88_200]) > 1e-5, "the tail died immediately");
        // ...and decays instead of feeding back forever.
        assert!(
            peak(&ls[3 * 44_100..]) < peak(&ls[2000..44_100]) * 0.5,
            "the room does not decay"
        );
        for v in ls.iter().chain(rs.iter()) {
            assert!(v.is_finite());
        }
    }

    #[test]
    fn the_tail_is_actually_stereo() {
        let mut rv = Reverb::new(48_000.0);
        let mut differ = 0;
        for i in 0..48_000 {
            let x = if i == 0 { 1.0 } else { 0.0 };
            let (l, r) = rv.process(x);
            if (l - r).abs() > 1e-6 {
                differ += 1;
            }
        }
        assert!(differ > 10_000, "left and right barely decorrelate: {differ}");
    }

    #[test]
    fn set_params_takes_effect_and_clamps() {
        // Wet zero silences the room entirely.
        let mut dry = Reverb::new(44_100.0);
        dry.set_params(Reverb::DEFAULT_ROOM, Reverb::DEFAULT_DAMP, 0.0);
        let mut heard = 0.0_f32;
        for i in 0..8_192 {
            let (l, r) = dry.process(if i == 0 { 1.0 } else { 0.0 });
            heard = heard.max(l.abs()).max(r.abs());
        }
        assert_eq!(heard, 0.0, "wet zero still leaked room");

        // An absurd room size is clamped: the tail must still decay, never
        // self-oscillate.
        let mut wild = Reverb::new(44_100.0);
        wild.set_params(2.0, 0.0, 1.0);
        let mut early = 0.0_f32;
        let mut late = 0.0_f32;
        for i in 0..(44_100 * 6) {
            let (l, _) = wild.process(if i == 0 { 1.0 } else { 0.0 });
            if i < 44_100 {
                early = early.max(l.abs());
            } else if i >= 44_100 * 5 {
                late = late.max(l.abs());
            }
        }
        assert!(late < early, "the clamp failed: the room is running away");
        assert!(late.is_finite());
    }

    #[test]
    fn mute_empties_the_room() {
        let mut rv = Reverb::new(44_100.0);
        for i in 0..4096 {
            rv.process(if i % 64 == 0 { 0.8 } else { 0.0 });
        }
        rv.mute();
        let mut after = Vec::new();
        for _ in 0..4096 {
            let (l, r) = rv.process(0.0);
            after.push(l);
            after.push(r);
        }
        assert_eq!(peak(&after), 0.0, "the room still rings after mute");
    }
}
