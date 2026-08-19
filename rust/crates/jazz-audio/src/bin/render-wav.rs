//! Render a chorus of any progression to a WAV file.
//!
//! No audio device and no dependencies — this is the fastest way to hear the
//! Rust engine, and it doubles as a regression check you can actually listen to.
//!
//! ```text
//! cargo run -p jazz-audio --bin render-wav -- --list
//! cargo run -p jazz-audio --bin render-wav -- bluebossa --tempo 150 --out bossa.wav
//! ```

use std::io::Write;

use jazz_audio::engine::{plan_bar, BassStyle, CompStyle, Density, HornStyle, Rng, Settings};

/// Match a CLI name against a set of style labels: exact, with punctuation
/// collapsed ("fourtothebar", "riffpush"), or by first or last word ("pedal"
/// is enough for "Pedal point", "push" for "Riff: push"). Unknown names are an
/// error, never a silent fallback.
fn match_style<T: Copy>(name: &str, all: &[T], label: fn(T) -> &'static str, kind: &str) -> T {
    let wanted = name.to_ascii_lowercase();
    let found = all.iter().copied().find(|&s| {
        let l = label(s).to_ascii_lowercase();
        l == wanted
            || l.replace(['-', ' ', ':'], "") == wanted
            || l.split(' ').next() == Some(wanted.as_str())
            || l.rsplit(' ').next() == Some(wanted.as_str())
    });
    match found {
        Some(s) => s,
        None => {
            let names: Vec<&str> = all.iter().map(|&s| label(s)).collect();
            eprintln!("Unknown {kind} '{name}'. Choices: {}", names.join(", "));
            std::process::exit(2);
        }
    }
}
use jazz_audio::stream;
use jazz_audio::voice::Instrument;
use jazz_theory::progression;

/// Map a name on the command line to an instrument. The canonical names come
/// from the same list the app's menus use, so a newly added instrument is
/// accepted here without touching this file; an unknown name is an error, not a
/// silent fallback to the wrong sound.
fn parse_instrument(name: &str) -> Instrument {
    let found = match name.to_ascii_lowercase().as_str() {
        "keys" | "ep" | "rhodes" => Some(Instrument::Keys),
        "bone" => Some(Instrument::Trombone),
        other => Instrument::ALL.iter().copied().find(|i| i.label().eq_ignore_ascii_case(other)),
    };
    match found {
        Some(i) => i,
        None => {
            let names: Vec<&str> = Instrument::ALL.iter().map(|i| i.label()).collect();
            eprintln!("Unknown instrument '{name}'. Choices: {}", names.join(", "));
            std::process::exit(2);
        }
    }
}

fn main() -> std::io::Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let progs = progression::all();

    if args.iter().any(|a| a == "--list" || a == "-l") {
        println!("Available progressions:\n");
        for p in &progs {
            println!("  {:<12} {:<34} {:>2} bars  [{:?}]", p.id, p.name, p.bars.len(), p.level);
        }
        return Ok(());
    }

    let mut id = "blues".to_string();
    let mut out = String::new();
    let mut settings = Settings::default();
    let mut choruses = 1usize;
    let mut seed = 20_260_808u64;
    let mut sf_path: Option<String> = None;

    let mut i = 0;
    while i < args.len() {
        let a = args[i].as_str();
        let val = |i: &mut usize| -> String {
            *i += 1;
            args.get(*i).cloned().unwrap_or_default()
        };
        match a {
            "--tempo" => settings.tempo = val(&mut i).parse().unwrap_or(120.0),
            "--out" | "-o" => out = val(&mut i),
            "--choruses" => choruses = val(&mut i).parse().unwrap_or(1),
            "--seed" => seed = val(&mut i).parse().unwrap_or(seed),
            "--straight" => settings.swing = false,
            "--sparse" => settings.density = Density::Sparse,
            "--busy" => settings.density = Density::Busy,
            "--no-bass" => settings.bass = false,
            "--no-chords" => settings.chords = false,
            "--no-drums" => settings.drums = false,
            "--horns" => settings.horns = true,
            "--solo" => settings.solo = true,
            "--solo-inst" => settings.solo_inst = parse_instrument(&val(&mut i)),
            "--soundfont" => sf_path = Some(val(&mut i)),
            // `-inst` suffixes so none of these collide with the `--no-*` part
            // toggles: a bare `--bass <name>` would swallow the progression id.
            "--horn-style" => {
                settings.horn_style =
                    match_style(&val(&mut i), &HornStyle::ALL, HornStyle::label, "horn style")
            }
            "--comp-inst" => settings.comp_inst = parse_instrument(&val(&mut i)),
            "--bass-inst" => settings.bass_inst = parse_instrument(&val(&mut i)),
            "--bass-style" => {
                settings.bass_style =
                    match_style(&val(&mut i), &BassStyle::ALL, BassStyle::label, "bass style")
            }
            "--comp-style" => {
                settings.comp_style =
                    match_style(&val(&mut i), &CompStyle::ALL, CompStyle::label, "comp style")
            }
            other if !other.starts_with('-') => id = other.to_string(),
            other => eprintln!("(ignoring unknown flag {other})"),
        }
        i += 1;
    }

    let prog = match progs.iter().find(|p| p.id == id) {
        Some(p) => p,
        None => {
            eprintln!("Unknown progression '{id}'. Try --list.");
            std::process::exit(2);
        }
    };
    if out.is_empty() {
        out = format!("{id}.wav");
    }

    const SR: f32 = 44_100.0;
    // A tune's meter travels with its bars, exactly as the app loads it.
    settings.meter = prog.meter;
    let bar_len = settings.bar_len();
    let total_bars = prog.bars.len() * choruses;
    let tail = total_bars as f32 * bar_len + 2.0;

    // Every render drives the real playback path — the same AudioCore the
    // app's callback owns — so the WAV carries exactly what the app plays:
    // the SoundFont's stereo image and the master room reverb included.
    let samples = {
        let (handle, mut core) = stream::AudioCore::new(SR);
        #[cfg(feature = "soundfont")]
        if let Some(path) = &sf_path {
            match jazz_audio::soundfont::SoundfontVoices::load(path, SR) {
                Ok(sf) => core.set_soundfont(Some(sf)),
                Err(e) => {
                    eprintln!("could not load soundfont '{path}': {e}");
                    std::process::exit(2);
                }
            }
        }
        #[cfg(not(feature = "soundfont"))]
        if sf_path.is_some() {
            eprintln!("--soundfont requires building with --features soundfont");
            std::process::exit(2);
        }
        handle.set_playing(true);
        let mut rng = Rng::new(seed);
        for i in 0..total_bars {
            handle.schedule(plan_bar(&prog.bars, i, &settings, &mut rng), i as f32 * bar_len);
        }
        let mut out = vec![0.0f32; (tail * SR) as usize * 2];
        for chunk in out.chunks_mut(1024) {
            core.fill(chunk, 2);
        }
        out
    };

    write_wav(&out, &samples, SR as u32, 2)?;

    let peak = samples.iter().fold(0.0f32, |a, b| a.max(b.abs()));
    println!(
        "{}  |  {} bars x{} chorus  |  {:.0} bpm{}  |  {:.1}s  |  peak {:.3}\nwrote {}",
        prog.name,
        prog.bars.len(),
        choruses,
        settings.tempo,
        if settings.swing { " swung" } else { " straight" },
        samples.len() as f32 / SR / 2.0,
        peak,
        out
    );
    Ok(())
}

/// Minimal 16-bit mono WAV writer.
fn write_wav(path: &str, samples: &[f32], sample_rate: u32, channels: u16) -> std::io::Result<()> {
    let mut f = std::io::BufWriter::new(std::fs::File::create(path)?);
    let data_len = (samples.len() * 2) as u32;
    let block_align = channels as u32 * 2;
    f.write_all(b"RIFF")?;
    f.write_all(&(36 + data_len).to_le_bytes())?;
    f.write_all(b"WAVEfmt ")?;
    f.write_all(&16u32.to_le_bytes())?;
    f.write_all(&1u16.to_le_bytes())?; // PCM
    f.write_all(&channels.to_le_bytes())?;
    f.write_all(&sample_rate.to_le_bytes())?;
    f.write_all(&(sample_rate * block_align).to_le_bytes())?;
    f.write_all(&(block_align as u16).to_le_bytes())?;
    f.write_all(&16u16.to_le_bytes())?;
    f.write_all(b"data")?;
    f.write_all(&data_len.to_le_bytes())?;
    for s in samples {
        let v = (s.clamp(-1.0, 1.0) * i16::MAX as f32) as i16;
        f.write_all(&v.to_le_bytes())?;
    }
    f.flush()
}
