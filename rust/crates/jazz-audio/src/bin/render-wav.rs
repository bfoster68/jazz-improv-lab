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

use jazz_audio::engine::{plan_bar, Density, Rng, Settings};
use jazz_audio::Mixer;
use jazz_theory::progression;

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
    let bar_len = 4.0 * 60.0 / settings.tempo;
    let mut mixer = Mixer::new(SR);
    let mut rng = Rng::new(seed);

    let total_bars = prog.bars.len() * choruses;
    for i in 0..total_bars {
        let plan = plan_bar(&prog.bars, i, &settings, &mut rng);
        mixer.schedule(&plan, i as f32 * bar_len);
    }
    let samples = mixer.render(total_bars as f32 * bar_len + 2.0);

    write_wav(&out, &samples, SR as u32)?;

    let peak = samples.iter().fold(0.0f32, |a, b| a.max(b.abs()));
    println!(
        "{}  |  {} bars x{} chorus  |  {:.0} bpm{}  |  {:.1}s  |  peak {:.3}\nwrote {}",
        prog.name,
        prog.bars.len(),
        choruses,
        settings.tempo,
        if settings.swing { " swung" } else { " straight" },
        samples.len() as f32 / SR,
        peak,
        out
    );
    Ok(())
}

/// Minimal 16-bit mono WAV writer.
fn write_wav(path: &str, samples: &[f32], sample_rate: u32) -> std::io::Result<()> {
    let mut f = std::io::BufWriter::new(std::fs::File::create(path)?);
    let data_len = (samples.len() * 2) as u32;
    f.write_all(b"RIFF")?;
    f.write_all(&(36 + data_len).to_le_bytes())?;
    f.write_all(b"WAVEfmt ")?;
    f.write_all(&16u32.to_le_bytes())?;
    f.write_all(&1u16.to_le_bytes())?; // PCM
    f.write_all(&1u16.to_le_bytes())?; // mono
    f.write_all(&sample_rate.to_le_bytes())?;
    f.write_all(&(sample_rate * 2).to_le_bytes())?;
    f.write_all(&2u16.to_le_bytes())?;
    f.write_all(&16u16.to_le_bytes())?;
    f.write_all(b"data")?;
    f.write_all(&data_len.to_le_bytes())?;
    for s in samples {
        let v = (s.clamp(-1.0, 1.0) * i16::MAX as f32) as i16;
        f.write_all(&v.to_le_bytes())?;
    }
    f.flush()
}
