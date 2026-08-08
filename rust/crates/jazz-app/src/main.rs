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
use jazz_theory::progression::{self, Bar, Progression};

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
}

#[derive(PartialEq, Eq, Clone, Copy)]
enum Tab {
    PlayAlong,
    Theory,
    Drills,
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
        });
    });

    egui::CentralPanel::default().show(&mut root, |ui| match state.tab {
        Tab::PlayAlong => play_along_ui(ui, &mut state, &audio),
        Tab::Theory => theory_ui(ui, &mut state),
        Tab::Drills => drills_ui(ui, &mut state),
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

fn theory_ui(ui: &mut egui::Ui, state: &mut AppState) {
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
    ui.horizontal_wrapped(|ui| {
        for n in &notes {
            ui.group(|ui| {
                ui.vertical_centered(|ui| {
                    ui.label(egui::RichText::new(&n.name).size(20.0).strong());
                    ui.label(egui::RichText::new(&n.degree).weak());
                });
            });
        }
    });
    ui.add_space(8.0);
    ui.label(scale.tip);
    // silence the unused-import warning when the crate is built without the lookup
    let _ = find_scale(scale.name);
}

fn drills_ui(ui: &mut egui::Ui, state: &mut AppState) {
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
        ui.label(egui::RichText::new(&d.how).italics());
    }
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
