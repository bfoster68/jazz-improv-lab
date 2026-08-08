//! The Jazz Improvisation Lab — native front end.
//!
//! Bevy drives the frame loop and UI; [`jazz_theory`] supplies the musical
//! content and [`jazz_audio`] renders the rhythm section. Audio goes out through
//! `cpal` on a dedicated thread, fed by a lock-free-ish shared mixer, because the
//! one thing an audio callback must never do is block on the render loop.
//!
//! Run with `cargo run -p jazz-app --release`.

use std::sync::{Arc, Mutex};

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPlugin, EguiPrimaryContextPass};

use jazz_audio::engine::{plan_bar, Density, Rng, Settings};
use jazz_audio::Mixer;
use jazz_theory::drill::{self, DRILL_IDS};
use jazz_theory::harmony::{render_scale, find_scale, SCALES};
use jazz_theory::note::{pc_name, use_sharp_for_root};
use jazz_theory::ear::{self, Mode as EarMode, Question};
use jazz_theory::listening::{ALSO_EXPLORE, MASTERS, STANDARDS};
use jazz_theory::progression::{self, Bar, Progression};
use jazz_audio::voice::Instrument;

/// Shared audio state. The Bevy side schedules bars into it; the cpal callback
/// drains samples from it.
struct Shared {
    mixer: Mixer,
    playing: bool,
    /// Which bar of the form has been scheduled so far.
    scheduled_bars: usize,
    bars: Vec<Bar>,
    settings: Settings,
    rng: Rng,
}

#[derive(Resource, Clone)]
struct Audio(Arc<Mutex<Shared>>);

#[derive(Resource)]
struct AppState {
    progs: Vec<Progression>,
    prog_index: usize,
    drill_index: usize,
    drill_key: i32,
    scale_index: usize,
    scale_root: i32,
    tab: Tab,
    volume: f32,
    ear_mode: EarMode,
    ear: EarState,
    rng: Rng,
}

#[derive(PartialEq, Eq, Clone, Copy)]
enum Tab {
    PlayAlong,
    Theory,
    Drills,
    Ear,
    Listening,
}

#[derive(Default)]
struct EarState {
    current: Option<Question>,
    /// What the student chose for the current question, if they have answered.
    picked: Option<String>,
    score: u32,
    streak: u32,
    total: u32,
}

const SAMPLE_RATE: f32 = 44_100.0;

fn main() {
    #[cfg(target_arch = "wasm32")]
    console_error_panic_hook::set_once();

    let progs = progression::all();
    let bars = progs[0].bars.clone();

    let shared = Arc::new(Mutex::new(Shared {
        mixer: Mixer::new(SAMPLE_RATE),
        playing: false,
        scheduled_bars: 0,
        bars,
        settings: Settings::default(),
        rng: Rng::new(0xC0FFEE),
    }));

    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "The Jazz Improvisation Lab".into(),
                resolution: (1040, 780).into(),
                // On the web, Bevy attaches to this canvas and follows its size.
                #[cfg(target_arch = "wasm32")]
                canvas: Some("#jazz-canvas".into()),
                #[cfg(target_arch = "wasm32")]
                fit_canvas_to_parent: true,
                #[cfg(target_arch = "wasm32")]
                prevent_default_event_handling: false,
                ..default()
            }),
            ..default()
        }))
        .add_plugins(EguiPlugin::default())
        .insert_resource(Audio(shared))
        .insert_resource(AppState {
            progs,
            prog_index: 0,
            drill_index: 0,
            drill_key: 0,
            scale_index: 0,
            scale_root: 0,
            tab: Tab::PlayAlong,
            volume: 1.0,
            ear_mode: EarMode::Intervals,
            ear: EarState::default(),
            rng: Rng::new(0x51EED),
        })
        .insert_resource(ClearColor(Color::srgb(0.06, 0.07, 0.09)))
        .init_non_send::<AudioStream>()
        .add_systems(Startup, setup_camera)
        .add_systems(Update, (start_audio_on_first_interaction, keep_audio_fed).chain())
        .add_systems(EguiPrimaryContextPass, ui)
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

/// The live output stream. `cpal::Stream` is `!Send`, and on the web the audio
/// context may not be created before a user gesture, so it is a non-send resource
/// opened lazily rather than at start-up.
#[derive(Default)]
struct AudioStream(Option<cpal::Stream>);

/// Browsers block audio until the user interacts with the page, so the stream is
/// opened on the first click or key press. Native behaves identically, which keeps
/// one code path for both.
fn start_audio_on_first_interaction(
    mut stream: NonSendMut<AudioStream>,
    audio: Res<Audio>,
    buttons: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    touches: Res<Touches>,
) {
    if stream.0.is_some() {
        return;
    }
    let interacted = buttons.get_just_pressed().next().is_some()
        || keys.get_just_pressed().next().is_some()
        || touches.iter_just_pressed().next().is_some();
    if interacted {
        stream.0 = start_audio(audio.0.clone());
    }
}

/// Top up the scheduler so there is always at least a bar of music queued ahead.
fn keep_audio_fed(audio: Res<Audio>) {
    let mut s = audio.0.lock().unwrap();
    if !s.playing {
        return;
    }
    let bar_len = 4.0 * 60.0 / s.settings.tempo;
    let now = s.mixer.time();
    // schedule up to two bars beyond the play head
    while (s.scheduled_bars as f32) * bar_len < now + bar_len * 2.0 {
        let i = s.scheduled_bars;
        let (bars, settings) = (s.bars.clone(), s.settings);
        let plan = plan_bar(&bars, i, &settings, &mut s.rng);
        s.mixer.schedule(&plan, i as f32 * bar_len);
        s.scheduled_bars += 1;
    }
}

fn ui(mut contexts: EguiContexts, mut state: ResMut<AppState>, audio: Res<Audio>) {
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };
    let ctx = ctx.clone();

    // egui 0.35 draws panels into a root `Ui` covering the viewport rather than
    // straight into the context.
    let mut root = egui::Ui::new(
        ctx.clone(),
        "root".into(),
        egui::UiBuilder::new()
            .layer_id(egui::LayerId::background())
            .max_rect(ctx.viewport_rect()),
    );

    egui::Panel::top("tabs").show(&mut root, |ui| {
        ui.horizontal(|ui| {
            ui.heading("The Jazz Improvisation Lab");
            ui.separator();
            ui.selectable_value(&mut state.tab, Tab::PlayAlong, "Play-Along");
            ui.selectable_value(&mut state.tab, Tab::Theory, "Theory");
            ui.selectable_value(&mut state.tab, Tab::Drills, "Drills");
            ui.selectable_value(&mut state.tab, Tab::Ear, "Ear Training");
            ui.selectable_value(&mut state.tab, Tab::Listening, "Masters & Standards");
        });
    });

    egui::CentralPanel::default().show(&mut root, |ui| match state.tab {
        Tab::PlayAlong => play_along_ui(ui, &mut state, &audio),
        Tab::Theory => theory_ui(ui, &mut state, &audio),
        Tab::Drills => drills_ui(ui, &mut state, &audio),
        Tab::Ear => ear_ui(ui, &mut state, &audio),
        Tab::Listening => listening_ui(ui, &mut state, &audio),
    });
}

fn play_along_ui(ui: &mut egui::Ui, state: &mut AppState, audio: &Audio) {
    let mut s = audio.0.lock().unwrap();

    ui.horizontal(|ui| {
        egui::ComboBox::from_label("Progression")
            .selected_text(state.progs[state.prog_index].name)
            .show_ui(ui, |ui| {
                for (i, p) in state.progs.iter().enumerate() {
                    ui.selectable_value(&mut state.prog_index, i, p.name);
                }
            });
        if ui.button(if s.playing { "⏸ Stop" } else { "▶ Play" }).clicked() {
            s.playing = !s.playing;
            if s.playing {
                let bars = state.progs[state.prog_index].bars.clone();
                s.bars = bars;
                s.scheduled_bars = 0;
                s.mixer.clear();
            } else {
                s.mixer.clear();
            }
        }
    });

    ui.add_space(6.0);
    ui.horizontal(|ui| {
        ui.add(egui::Slider::new(&mut s.settings.tempo, 50.0..=280.0).text("bpm"));
        ui.checkbox(&mut s.settings.swing, "Swing");
        ui.checkbox(&mut s.settings.bass, "Bass");
        ui.checkbox(&mut s.settings.chords, "Chords");
        ui.checkbox(&mut s.settings.drums, "Drums");
    });
    ui.horizontal(|ui| {
        egui::ComboBox::from_label("Comping")
            .selected_text(format!("{:?}", s.settings.density))
            .show_ui(ui, |ui| {
                for d in [Density::Sparse, Density::Medium, Density::Busy] {
                    ui.selectable_value(&mut s.settings.density, d, format!("{d:?}"));
                }
            });
        if ui.add(egui::Slider::new(&mut state.volume, 0.0..=1.5).text("Volume")).changed() {
            s.mixer.volume = state.volume;
        }
    });

    ui.separator();

    // Which bar is sounding right now?
    let bars = state.progs[state.prog_index].bars.clone();
    let bar_len = 4.0 * 60.0 / s.settings.tempo;
    let current = if s.playing && !bars.is_empty() {
        ((s.mixer.time() / bar_len) as usize) % bars.len()
    } else {
        0
    };

    ui.label(egui::RichText::new(bars[current].symbol()).size(34.0).strong());
    let slot = bars[current].chord_at(0);
    let use_sharp = use_sharp_for_root(slot.root);
    let tones: Vec<String> = slot
        .quality
        .ints()
        .iter()
        .zip(slot.quality.degs())
        .map(|(&i, d)| {
            format!(
                "{} {}",
                jazz_theory::note::note_name(
                    slot.root + i,
                    d,
                    jazz_theory::note::Note::parse(pc_name(slot.root, use_sharp)),
                    use_sharp
                ),
                d
            )
        })
        .collect();
    ui.label(format!("Aim at: {}", tones.join("   ")));
    ui.label(format!("Suggested scale: {}", slot.scale_name()));

    ui.add_space(10.0);
    egui::ScrollArea::vertical().show(ui, |ui| {
        egui::Grid::new("bars").spacing([8.0, 8.0]).show(ui, |ui| {
            for (i, b) in bars.iter().enumerate() {
                let text = egui::RichText::new(b.symbol());
                let text = if i == current && s.playing {
                    text.strong().color(egui::Color32::from_rgb(224, 166, 60))
                } else {
                    text
                };
                ui.label(text);
                if (i + 1) % 4 == 0 {
                    ui.end_row();
                }
            }
        });
    });
}

fn theory_ui(ui: &mut egui::Ui, state: &mut AppState, audio: &Audio) {
    ui.horizontal(|ui| {
        egui::ComboBox::from_label("Root")
            .selected_text(pc_name(state.scale_root, use_sharp_for_root(state.scale_root)))
            .show_ui(ui, |ui| {
                for r in 0..12 {
                    ui.selectable_value(&mut state.scale_root, r, pc_name(r, use_sharp_for_root(r)));
                }
            });
        egui::ComboBox::from_label("Scale")
            .selected_text(SCALES[state.scale_index].name)
            .show_ui(ui, |ui| {
                for (i, s) in SCALES.iter().enumerate() {
                    ui.selectable_value(&mut state.scale_index, i, s.name);
                }
            });
    });
    ui.separator();
    let scale = &SCALES[state.scale_index];
    let notes = render_scale(scale, state.scale_root, use_sharp_for_root(state.scale_root));
    let root_midi = jazz_theory::note::midi_for(state.scale_root, 4);
    ui.horizontal_wrapped(|ui| {
        for n in &notes {
            let cell = ui.group(|ui| {
                ui.vertical_centered(|ui| {
                    ui.label(egui::RichText::new(&n.name).size(20.0).strong());
                    ui.label(egui::RichText::new(&n.degree).weak());
                });
            });
            // clicking a note sounds it, the same as the browser version
            if cell.response.interact(egui::Sense::click()).clicked() {
                audio.0.lock().unwrap().mixer.play_note(
                    Instrument::Vibes,
                    root_midi + n.semitones,
                    0.20,
                    0.50,
                    0.0,
                );
            }
        }
    });

    ui.add_space(8.0);
    ui.horizontal(|ui| {
        if ui.button("\u{25B6} Play ascending").clicked() {
            let mut s = audio.0.lock().unwrap();
            for (i, n) in notes.iter().enumerate() {
                s.mixer.play_note(Instrument::Vibes, root_midi + n.semitones, 0.20, 0.30, i as f32 * 0.34);
            }
            // top off with the octave so the scale resolves
            s.mixer.play_note(Instrument::Vibes, root_midi + 12, 0.20, 0.40, notes.len() as f32 * 0.34);
        }
        if ui.button("\u{25B6} Play as a chord").clicked() {
            let midis: Vec<i32> = notes.iter().map(|n| root_midi + n.semitones).collect();
            audio.0.lock().unwrap().mixer.play_chord(Instrument::Keys, &midis, 0.07, 1.6, 0.0);
        }
    });
    ui.add_space(8.0);
    ui.label(scale.tip);
    // silence the unused-import warning when the crate is built without the lookup
    let _ = find_scale(scale.name);
}

fn drills_ui(ui: &mut egui::Ui, state: &mut AppState, audio: &Audio) {
    ui.horizontal(|ui| {
        egui::ComboBox::from_label("Drill")
            .selected_text(DRILL_IDS[state.drill_index])
            .show_ui(ui, |ui| {
                for (i, id) in DRILL_IDS.iter().enumerate() {
                    ui.selectable_value(&mut state.drill_index, i, *id);
                }
            });
        egui::ComboBox::from_label("Key")
            .selected_text(pc_name(state.drill_key, use_sharp_for_root(state.drill_key)))
            .show_ui(ui, |ui| {
                for r in 0..12 {
                    ui.selectable_value(&mut state.drill_key, r, pc_name(r, use_sharp_for_root(r)));
                }
            });
    });
    ui.separator();
    if let Some(d) = drill::build(DRILL_IDS[state.drill_index], state.drill_key) {
        ui.label(egui::RichText::new(&d.title).size(20.0).strong());
        ui.label(&d.concept);
        ui.add_space(6.0);
        ui.horizontal(|ui| {
            for b in &d.bars {
                ui.group(|ui| { ui.label(b); });
            }
        });
        ui.add_space(6.0);
        ui.horizontal_wrapped(|ui| {
            for n in &d.notes {
                ui.group(|ui| {
                    ui.vertical_centered(|ui| {
                        ui.label(egui::RichText::new(&n.name).size(18.0).strong());
                        ui.label(egui::RichText::new(&n.degree).weak());
                    });
                });
            }
        });
        ui.add_space(8.0);
        if ui.button("\u{25B6} Play the line").clicked() {
            let mut s = audio.0.lock().unwrap();
            for (i, n) in d.notes.iter().enumerate() {
                s.mixer.play_note(Instrument::Vibes, n.midi, 0.20, 0.30, i as f32 * 0.33);
            }
        }
        ui.add_space(6.0);
        ui.label(egui::RichText::new(&d.how).italics());
    }
}

// ---------------------------------------------------------------- ear training

/// Voice a question through the mixer, using the timing the theory crate specifies.
fn sound_question(q: &Question, audio: &Audio) {
    // one slot per beat-ish; intervals want a clear gap, chords ring longer
    let (slot, dur, gain, instrument) = match q.mode {
        EarMode::Intervals => (0.62, 0.55, 0.22, Instrument::Vibes),
        EarMode::Progressions => (1.0, 0.95, 0.08, Instrument::Keys),
        _ => (1.0, 1.5, 0.085, Instrument::Keys),
    };
    let mut s = audio.0.lock().unwrap();
    for (index, notes) in ear::voicing(q) {
        let at = 0.05 + index as f32 * slot;
        s.mixer.play_chord(instrument, &notes, gain, dur, at);
    }
}

fn ear_ui(ui: &mut egui::Ui, state: &mut AppState, audio: &Audio) {
    ui.horizontal(|ui| {
        let before = state.ear_mode;
        egui::ComboBox::from_label("Mode")
            .selected_text(state.ear_mode.label())
            .show_ui(ui, |ui| {
                for m in EarMode::ALL {
                    ui.selectable_value(&mut state.ear_mode, m, m.label());
                }
            });
        if state.ear_mode != before {
            state.ear.current = None;
            state.ear.picked = None;
        }

        let next = ui.button("\u{1F3B2} New question").clicked();
        if next {
            let previous = state.ear.current.as_ref().map(|q| q.answer);
            let q = ear::draw(
                state.ear_mode,
                previous,
                state.rng.next_u32_pub(),
                state.rng.next_u32_pub(),
            );
            sound_question(&q, audio);
            state.ear.current = Some(q);
            state.ear.picked = None;
        }
        if ui.button("\u{1F501} Replay").clicked() {
            if let Some(q) = state.ear.current.clone() {
                sound_question(&q, audio);
            }
        }
    });

    ui.separator();

    let Some(q) = state.ear.current.clone() else {
        ui.label(
            "Press \u{201C}New question\u{201D} to begin. Sing the sound back before you answer \u{2014} \
             naming comes after hearing.",
        );
        return;
    };

    let answered = state.ear.picked.is_some();
    match state.ear.picked.as_deref() {
        Some(p) if p == q.answer => {
            ui.label(
                egui::RichText::new(format!("\u{2713} Correct \u{2014} {}", q.answer))
                    .color(egui::Color32::from_rgb(87, 201, 138))
                    .strong(),
            );
        }
        Some(_) => {
            ui.label(
                egui::RichText::new(format!("\u{2717} It was {}", q.answer))
                    .color(egui::Color32::from_rgb(224, 107, 107))
                    .strong(),
            );
        }
        None => {
            ui.label(egui::RichText::new("Listening\u{2026}").weak());
        }
    }

    ui.add_space(6.0);
    ui.horizontal_wrapped(|ui| {
        for choice in &q.choices {
            let is_answer = *choice == q.answer;
            let is_pick = state.ear.picked.as_deref() == Some(*choice);
            let mut text = egui::RichText::new(*choice);
            if answered && is_answer {
                text = text.color(egui::Color32::from_rgb(87, 201, 138)).strong();
            } else if is_pick {
                text = text.color(egui::Color32::from_rgb(224, 107, 107));
            }
            if ui.add_enabled(!answered, egui::Button::new(text)).clicked() {
                state.ear.picked = Some((*choice).to_string());
                state.ear.total += 1;
                if is_answer {
                    state.ear.score += 1;
                    state.ear.streak += 1;
                } else {
                    state.ear.streak = 0;
                }
            }
        }
    });

    ui.add_space(10.0);
    ui.label(format!(
        "Score {}   \u{00B7}   Streak {}   \u{00B7}   Asked {}",
        state.ear.score, state.ear.streak, state.ear.total
    ));
}

// ----------------------------------------------------------- masters & standards

fn listening_ui(ui: &mut egui::Ui, state: &mut AppState, audio: &Audio) {
    egui::ScrollArea::vertical().show(ui, |ui| {
        ui.heading("Masters to study");
        ui.label(
            egui::RichText::new(
                "The language is learned by ear. Live with these players, sing along, then \
                 transcribe a single phrase.",
            )
            .weak(),
        );
        ui.add_space(6.0);
        for m in MASTERS {
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new(m.name)
                            .strong()
                            .color(egui::Color32::from_rgb(224, 166, 60)),
                    );
                    ui.label(egui::RichText::new(format!("{} \u{00B7} {}", m.instrument, m.era)).weak());
                });
                ui.label(m.listen_for);
            });
        }
        ui.add_space(4.0);
        ui.label(egui::RichText::new(format!("And more to explore: {ALSO_EXPLORE}")).italics().weak());

        ui.add_space(16.0);
        ui.heading("Essential standards");
        ui.label(
            egui::RichText::new(
                "Learn the melody and the changes, then improvise. Play loads the changes \
                 straight into the Play-Along tab.",
            )
            .weak(),
        );
        ui.add_space(6.0);

        let mut load: Option<&'static str> = None;
        for st in STANDARDS {
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new(st.title).strong());
                    ui.label(egui::RichText::new(format!("\u{2014} {}", st.composer)).weak());
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        match st.progression_id {
                            Some(id) => {
                                if ui.button("\u{25B6} Play").clicked() {
                                    load = Some(id);
                                }
                            }
                            None => {
                                ui.label(egui::RichText::new("study by ear").weak());
                            }
                        }
                    });
                });
                ui.label(egui::RichText::new(st.key_and_form).weak());
                ui.label(st.teaches);
            });
        }

        if let Some(id) = load {
            if let Some(i) = state.progs.iter().position(|p| p.id == id) {
                state.prog_index = i;
                let mut s = audio.0.lock().unwrap();
                s.bars = state.progs[i].bars.clone();
                s.scheduled_bars = 0;
                s.mixer.clear();
                s.playing = true;
                state.tab = Tab::PlayAlong;
            }
        }
    });
}

/// Open the default output device and stream from the shared mixer.
fn start_audio(shared: Arc<Mutex<Shared>>) -> Option<cpal::Stream> {
    use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

    let host = cpal::default_host();
    let device = host.default_output_device()?;
    let config = device.default_output_config().ok()?;
    let channels = config.channels() as usize;

    let stream = device
        .build_output_stream(
            &config.config(),
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                let mut guard = match shared.try_lock() {
                    Ok(g) => g,
                    // Never block the audio thread; a dropped buffer is better than a glitch.
                    Err(_) => {
                        data.fill(0.0);
                        return;
                    }
                };
                for frame in data.chunks_mut(channels) {
                    let s = if guard.playing { guard.mixer.tick() } else { 0.0 };
                    for out in frame.iter_mut() {
                        *out = s;
                    }
                }
            },
            |err| eprintln!("audio stream error: {err}"),
            None,
        )
        .ok()?;
    stream.play().ok()?;
    Some(stream)
}
