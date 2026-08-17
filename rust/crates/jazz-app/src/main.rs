//! The Jazz Improvisation Lab — native front end.
//!
//! Bevy drives the frame loop and UI; [`jazz_theory`] supplies the musical
//! content and [`jazz_audio`] renders the rhythm section. Audio goes out through
//! `cpal` on a dedicated thread, fed by a lock-free-ish shared mixer, because the
//! one thing an audio callback must never do is block on the render loop.
//!
//! Run with `cargo run -p jazz-app --release`.

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPlugin, EguiPrimaryContextPass};

use jazz_audio::engine::{plan_bar, BassStyle, CompStyle, Density, Rng, Settings};
use jazz_audio::stream::{AudioCore, AudioHandle};
use jazz_theory::drill::{self, DRILL_IDS};
use jazz_theory::harmony::{render_scale, find_scale, SCALES};
use jazz_theory::note::{pc_name, use_sharp_for_root};
use jazz_theory::ear::{self, Mode as EarMode, Question};
use jazz_theory::listening::{Standard, ALSO_EXPLORE, MASTERS, STANDARDS};
use jazz_theory::progression::{self, Bar, Progression};
use jazz_audio::voice::Instrument;

/// UI-side handle to the audio thread. Sending a command never blocks, which is
/// what keeps playback free of dropouts.
#[derive(Resource, Clone)]
struct Audio(AudioHandle);

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
    /// Theme is applied once, on the first frame that has a context.
    styled: bool,
    // ---- transport, owned here now that the audio thread owns the mixer ----
    bars: Vec<Bar>,
    settings: Settings,
    /// How many bars of the form have been queued to the audio thread.
    scheduled_bars: usize,
    /// The tempo shown while the slider is mid-drag. `settings.tempo` — which
    /// the scheduler reads live — only changes when the drag commits, because
    /// changing `bar_len` under an un-requeued transport schedules bars into the
    /// past and they all sound at once.
    pending_tempo: Option<f32>,
    /// The last bar the chart auto-scrolled to, so it follows the music once
    /// per bar instead of pinning the view against the user's own scrolling.
    chart_bar: usize,
    /// Which master's biography page is open, if any.
    selected_master: Option<usize>,
    /// Audio-clock time (seconds) at which the current transport started. The
    /// mixer clock free-runs and is never reset, so bars must be scheduled
    /// relative to this epoch rather than to absolute zero — otherwise a restart
    /// after the clock has advanced queues every bar in the past and the mixer
    /// dumps them all at once.
    epoch: f32,
    sched_rng: Rng,
}

#[derive(PartialEq, Eq, Clone, Copy)]
enum Tab {
    PlayAlong,
    Theory,
    Drills,
    Ear,
    Masters,
    Standards,
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

/// The horn section is being reworked; its mixer row and jam presets are
/// parked behind this flag. Flip it to bring the horns back — the engine
/// support and its tests are all still in place.
const HORNS_ENABLED: bool = false;

fn main() {
    #[cfg(target_arch = "wasm32")]
    console_error_panic_hook::set_once();

    let progs = progression::all();
    let bars = progs[0].bars.clone();
    let (handle, core) = AudioCore::new(SAMPLE_RATE);

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
        .insert_resource(Audio(handle))
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
            styled: false,
            bars,
            settings: Settings::default(),
            scheduled_bars: 0,
            pending_tempo: None,
            chart_bar: 0,
            selected_master: None,
            epoch: 0.0,
            sched_rng: Rng::new(0xC0FFEE),
        })
        .insert_resource(ClearColor(Color::srgb(0.06, 0.07, 0.09)))
        .insert_non_send(AudioStream { stream: None, core: Some(core) })
        .add_systems(Startup, setup_camera)
        .add_systems(Update, (start_audio_on_first_interaction, keep_audio_fed).chain())
        .add_systems(EguiPrimaryContextPass, ui)
        .run();
}


/// A fixed-width note cell.
///
/// `vertical_centered` claims all remaining width, so a chip built with it inside
/// a wrapped row swallows the line and hides every note after the first. Setting an
/// explicit width is what makes the row wrap correctly.
fn note_chip(ui: &mut egui::Ui, name: &str, degree: &str) -> egui::Response {
    let r = ui.group(|ui| {
        ui.set_width(52.0);
        ui.vertical_centered(|ui| {
            ui.label(egui::RichText::new(name).size(19.0).strong());
            ui.label(
                egui::RichText::new(degree)
                    .size(12.0)
                    .color(egui::Color32::from_rgb(75, 179, 167)),
            );
        });
    });
    r.response.interact(egui::Sense::click())
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

/// The live output stream. `cpal::Stream` is `!Send`, and on the web the audio
/// context may not be created before a user gesture, so it is a non-send resource
/// opened lazily rather than at start-up.
struct AudioStream {
    stream: Option<cpal::Stream>,
    /// Held until a user gesture lets us open the device.
    core: Option<AudioCore>,
}

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
    if stream.stream.is_some() {
        return;
    }
    let interacted = buttons.get_just_pressed().next().is_some()
        || keys.get_just_pressed().next().is_some()
        || touches.iter_just_pressed().next().is_some();
    if interacted {
        if let Some(core) = stream.core.take() {
            // The SoundFont is loaded inside `start_audio`, once the device's real
            // sample rate is known — loading it at the wrong rate plays every
            // sample transposed.
            match start_audio(core) {
                Ok(s) => stream.stream = Some(s),
                Err((core, why)) => {
                    eprintln!("audio unavailable: {why}");
                    stream.core = Some(core); // keep it, so a later click can retry
                }
            }
        }
    }
    let _ = &audio;
}

/// Find the bundled SoundFont, looking next to the executable first (how a
/// shipped build carries it) and then in the dev-tree locations.
fn find_soundfont() -> Option<std::path::PathBuf> {
    use std::path::PathBuf;
    let mut candidates: Vec<PathBuf> = Vec::new();
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            candidates.push(dir.join("soundfont.sf2"));
            candidates.push(dir.join("assets/soundfont.sf2"));
        }
    }
    candidates.push(PathBuf::from("assets/soundfont.sf2"));
    candidates.push(PathBuf::from("crates/jazz-app/assets/soundfont.sf2"));
    candidates.into_iter().find(|p| p.is_file())
}

/// Keep at least a couple of bars queued ahead of the play head.
///
/// This runs on the Bevy schedule and only ever *sends*; the audio thread is never
/// made to wait for it.
fn keep_audio_fed(audio: Res<Audio>, mut state: ResMut<AppState>) {
    if !audio.0.is_playing() || state.bars.is_empty() {
        return;
    }
    // Borrow fields directly rather than cloning the form every frame; this
    // runs 60x a second and usually schedules nothing.
    let state = &mut *state;
    let bar_len = state.settings.bar_len();
    let now = audio.0.time();
    let epoch = state.epoch;
    let settings = state.settings;
    // Keep a few bars queued ahead of the play head. Bar `i` sounds at
    // `epoch + i * bar_len`, so the whole form stays anchored to wherever the
    // free-running mixer clock was when this transport started.
    while epoch + (state.scheduled_bars as f32) * bar_len < now + bar_len * 4.0 {
        let i = state.scheduled_bars;
        let plan = plan_bar(&state.bars, i, &settings, &mut state.sched_rng);
        audio.0.schedule(plan, epoch + i as f32 * bar_len);
        state.scheduled_bars += 1;
    }
}

fn ui(mut contexts: EguiContexts, mut state: ResMut<AppState>, audio: Res<Audio>) {
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };
    let ctx = ctx.clone();

    if !state.styled {
        apply_style(&ctx);
        state.styled = true;
    }

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
            ui.selectable_value(&mut state.tab, Tab::Masters, "Masters");
            ui.selectable_value(&mut state.tab, Tab::Standards, "Standards");
        });
    });

    egui::CentralPanel::default().show(&mut root, |ui| match state.tab {
        Tab::PlayAlong => play_along_ui(ui, &mut state, &audio),
        Tab::Theory => theory_ui(ui, &mut state, &audio),
        Tab::Drills => drills_ui(ui, &mut state, &audio),
        Tab::Ear => ear_ui(ui, &mut state, &audio),
        Tab::Masters => masters_ui(ui, &mut state),
        Tab::Standards => standards_ui(ui, &mut state, &audio),
    });
}

/// Match the browser build's dark palette and give the text room to breathe.
fn apply_style(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::dark();
    visuals.panel_fill = egui::Color32::from_rgb(15, 18, 22);
    visuals.window_fill = egui::Color32::from_rgb(23, 27, 34);
    visuals.extreme_bg_color = egui::Color32::from_rgb(12, 15, 19);
    visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(30, 36, 46);
    visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(35, 43, 54);
    visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(45, 55, 68);
    visuals.selection.bg_fill = egui::Color32::from_rgb(224, 166, 60);
    visuals.selection.stroke.color = egui::Color32::from_rgb(26, 18, 6);
    ctx.set_visuals(visuals);

    // egui 0.35 keeps a style per theme, so mutate them all rather than one.
    ctx.all_styles_mut(|style| {
        for (_, font) in style.text_styles.iter_mut() {
            font.size *= 1.25;
        }
        style.spacing.item_spacing = egui::vec2(8.0, 7.0);
        style.spacing.button_padding = egui::vec2(10.0, 5.0);
    });
}

fn play_along_ui(ui: &mut egui::Ui, state: &mut AppState, audio: &Audio) {
    let playing = audio.0.is_playing();

    ui.horizontal(|ui| {
        ui.label("Progression");
        let before = state.prog_index;
        egui::ComboBox::from_id_salt("progression")
            .selected_text(state.progs[state.prog_index].name)
            .show_ui(ui, |ui| {
                for (i, p) in state.progs.iter().enumerate() {
                    ui.selectable_value(&mut state.prog_index, i, p.name);
                }
            });
        if state.prog_index != before {
            load_progression(state, audio, state.prog_index, playing);
        }
        if ui.button(if playing { "\u{23F8} Stop" } else { "\u{25B6} Play" }).clicked() {
            if playing {
                audio.0.set_playing(false);
            } else {
                load_progression(state, audio, state.prog_index, true);
            }
        }
    });

    ui.add_space(6.0);
    ui.horizontal(|ui| {
        // The slider edits a staging copy; `settings.tempo` (which the scheduler
        // reads every frame) only changes at the commit points below. Mutating it
        // mid-drag would shrink `bar_len` under the already-anchored transport
        // and queue a burst of bars into the past.
        let mut tempo = state.pending_tempo.unwrap_or(state.settings.tempo);
        let resp = ui.add(egui::Slider::new(&mut tempo, 50.0..=280.0).text("bpm"));
        if resp.dragged() {
            state.pending_tempo = Some(tempo);
        } else if resp.drag_stopped() || resp.changed() {
            // Commit on release or on a direct (typed/clicked) edit — and only
            // requeue when the value actually changed, so a click that moves
            // nothing does not restart the form.
            state.pending_tempo = None;
            if (tempo - state.settings.tempo).abs() > 0.01 {
                state.settings.tempo = tempo;
                requeue(state, audio);
            }
        }
        if ui.checkbox(&mut state.settings.swing, "Swing").changed() {
            resync(state, audio);
        }
        if ui
            .add(egui::Slider::new(&mut state.volume, 0.0..=1.5).text("Volume"))
            .changed()
        {
            audio.0.set_volume(state.volume);
        }
    });

    // ---- per-part mixer: pick the instrument and on/off for each part ----
    let mut dirty = false;
    ui.horizontal(|ui| {
        dirty |= ui.checkbox(&mut state.settings.bass, "Bass").changed();
        dirty |= instrument_combo(ui, "bass_inst", &mut state.settings.bass_inst, &Instrument::BASS_CHOICES);
        ui.label("  Line");
        egui::ComboBox::from_id_salt("bass_style")
            .selected_text(state.settings.bass_style.label())
            .show_ui(ui, |ui| {
                for s in BassStyle::ALL {
                    dirty |= ui.selectable_value(&mut state.settings.bass_style, s, s.label()).changed();
                }
            });
    });
    ui.horizontal(|ui| {
        dirty |= ui.checkbox(&mut state.settings.chords, "Comp").changed();
        dirty |= instrument_combo(ui, "comp_inst", &mut state.settings.comp_inst, &Instrument::COMP_CHOICES);
        ui.label("  Style");
        egui::ComboBox::from_id_salt("comp_style")
            .selected_text(state.settings.comp_style.label())
            .show_ui(ui, |ui| {
                for s in CompStyle::ALL {
                    dirty |= ui.selectable_value(&mut state.settings.comp_style, s, s.label()).changed();
                }
            });
        ui.label("  Feel");
        egui::ComboBox::from_id_salt("comping")
            .selected_text(format!("{:?}", state.settings.density))
            .show_ui(ui, |ui| {
                for d in [Density::Sparse, Density::Medium, Density::Busy] {
                    dirty |= ui.selectable_value(&mut state.settings.density, d, format!("{d:?}")).changed();
                }
            });
    });
    if HORNS_ENABLED {
        ui.horizontal(|ui| {
            dirty |= ui.checkbox(&mut state.settings.horns, "Horns").changed();
            dirty |= instrument_combo(ui, "horn_inst", &mut state.settings.horn_inst, &Instrument::HORN_CHOICES);
        });
    }
    ui.horizontal(|ui| {
        dirty |= ui.checkbox(&mut state.settings.drums, "Drums").changed();
    });
    if dirty {
        resync(state, audio);
    }

    ui.separator();

    let bars = state.bars.clone();
    if bars.is_empty() {
        return;
    }
    let bar_len = state.settings.bar_len();
    let current = if playing {
        let elapsed = (audio.0.time() - state.epoch).max(0.0);
        ((elapsed / bar_len) as usize) % bars.len()
    } else {
        0
    };

    let slot = bars[current].chord_at(0);
    ui.label(egui::RichText::new(bars[current].symbol()).size(34.0).strong());
    let use_sharp = slot.use_sharp();
    let root_note = jazz_theory::note::Note::parse(pc_name(slot.root, use_sharp));
    let tones: Vec<String> = slot
        .quality
        .ints()
        .iter()
        .zip(slot.quality.degs())
        .map(|(&i, d)| {
            format!(
                "{} {}",
                jazz_theory::note::note_name(slot.root + i, d, root_note, use_sharp),
                d
            )
        })
        .collect();
    ui.label(format!("Aim at: {}", tones.join("   ")));
    ui.label(format!("Suggested scale: {}", slot.scale_name()));

    ui.add_space(10.0);
    // ---- the chart: four-bar systems that read like a lead sheet ----
    let sections = state.progs[state.prog_index].sections;
    let gold = egui::Color32::from_rgb(224, 166, 60);
    ui.label(
        egui::RichText::new(format!(
            "{} \u{00B7} {} bars",
            state.settings.meter.label(),
            bars.len()
        ))
        .weak(),
    );
    egui::ScrollArea::vertical().show(ui, |ui| {
        for (sys, chunk) in bars.chunks(4).enumerate() {
            let start = sys * 4;
            let row = ui.horizontal(|ui| {
                // Section letter in the margin where a section begins.
                let label = sections
                    .iter()
                    .find(|(at, _)| *at == start)
                    .map(|(_, l)| *l)
                    .unwrap_or("");
                ui.add_sized(
                    [22.0, 36.0],
                    egui::Label::new(egui::RichText::new(label).strong().size(18.0).color(gold)),
                );
                for (j, b) in chunk.iter().enumerate() {
                    let i = start + j;
                    ui.label(egui::RichText::new("|").weak().size(22.0));
                    // Fake-book shorthand: a bar identical to the one before it
                    // reads as a repeat, but the symbol returns at a new system
                    // so a line always starts readable.
                    let sym = if j > 0 && bars[i - 1].symbol() == b.symbol() {
                        "%".to_string()
                    } else {
                        b.symbol()
                    };
                    let here = playing && i == current;
                    let (fill, color) = if here {
                        (gold, egui::Color32::from_rgb(26, 18, 6))
                    } else {
                        (egui::Color32::TRANSPARENT, ui.visuals().text_color())
                    };
                    egui::Frame::default()
                        .fill(fill)
                        .corner_radius(egui::CornerRadius::same(4))
                        .inner_margin(egui::Margin::symmetric(4, 6))
                        .show(ui, |ui| {
                            ui.set_width(108.0);
                            ui.vertical_centered(|ui| {
                                ui.label(egui::RichText::new(sym).size(17.0).strong().color(color));
                            });
                        });
                }
                ui.label(egui::RichText::new("|").weak().size(22.0));
            });
            // Follow along: bring the sounding system into view once per bar,
            // so the reader tracks the form but can still scroll around freely.
            if playing && (start..start + chunk.len()).contains(&current) && state.chart_bar != current {
                row.response.scroll_to_me(Some(egui::Align::Center));
                state.chart_bar = current;
            }
        }
    });
}

/// Load a progression's changes and (re)start the transport from its top.
/// The tune's meter comes with its bars — a waltz arrives in three.
fn load_progression(state: &mut AppState, audio: &Audio, index: usize, play: bool) {
    state.bars = state.progs[index].bars.clone();
    state.settings.meter = state.progs[index].meter;
    restart_transport(state, audio, play);
}

/// Requeue from the top after something that changes future timing.
fn requeue(state: &mut AppState, audio: &Audio) {
    if audio.0.is_playing() {
        restart_transport(state, audio, true);
    }
}

/// Re-queue the *upcoming* bars after a live mix change (instrument swap, part
/// toggle, feel/swing change) without restarting the form, moving the clock, or
/// cutting the note currently under the player's ear. The change is audible from
/// the next bar.
fn resync(state: &mut AppState, audio: &Audio) {
    if !audio.0.is_playing() {
        return;
    }
    let bar_len = state.settings.bar_len();
    let elapsed = (audio.0.time() - state.epoch).max(0.0);
    let next_bar = (elapsed / bar_len) as usize + 1;
    // Drop only what is queued from the next bar boundary on — the remainder of
    // the current bar keeps playing with the old settings — then reschedule from
    // that boundary.
    audio.0.clear_pending_from(state.epoch + next_bar as f32 * bar_len);
    state.scheduled_bars = next_bar;
}

/// A dropdown of instruments for one part. Returns whether the choice changed.
fn instrument_combo(
    ui: &mut egui::Ui,
    id: &str,
    current: &mut Instrument,
    choices: &[Instrument],
) -> bool {
    let mut changed = false;
    egui::ComboBox::from_id_salt(id)
        .selected_text(current.label())
        .show_ui(ui, |ui| {
            for &opt in choices {
                changed |= ui.selectable_value(current, opt, opt.label()).changed();
            }
        });
    changed
}

/// Reset the transport to bar 1, anchored to the current clock position.
///
/// The mixer clock free-runs and is never rewound, so the new form must start
/// from `now`, not from absolute zero — otherwise every re-queued bar lands in
/// the past and sounds at once.
fn restart_transport(state: &mut AppState, audio: &Audio, play: bool) {
    audio.0.clear();
    state.scheduled_bars = 0;
    state.epoch = audio.0.time();
    audio.0.set_playing(play);
}

fn theory_ui(ui: &mut egui::Ui, state: &mut AppState, audio: &Audio) {
    ui.horizontal(|ui| {
        ui.label("Root");
        egui::ComboBox::from_id_salt("root")
            .selected_text(pc_name(state.scale_root, use_sharp_for_root(state.scale_root)))
            .show_ui(ui, |ui| {
                for r in 0..12 {
                    ui.selectable_value(&mut state.scale_root, r, pc_name(r, use_sharp_for_root(r)));
                }
            });
        ui.label("Scale");
        egui::ComboBox::from_id_salt("scale")
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
            // clicking a note sounds it, the same as the browser version
            if note_chip(ui, &n.name, &n.degree).clicked() {
                audio.0.note(Instrument::Vibes, root_midi + n.semitones, 0.20, 0.50, 0.0);
            }
        }
    });

    ui.add_space(8.0);
    ui.horizontal(|ui| {
        if ui.button("\u{25B6} Play ascending").clicked() {
            for (i, n) in notes.iter().enumerate() {
                audio.0.note(Instrument::Vibes, root_midi + n.semitones, 0.20, 0.30, i as f32 * 0.34);
            }
            // top off with the octave so the scale resolves
            audio.0.note(Instrument::Vibes, root_midi + 12, 0.20, 0.40, notes.len() as f32 * 0.34);
        }
        if ui.button("\u{25B6} Play as a chord").clicked() {
            let midis: Vec<i32> = notes.iter().map(|n| root_midi + n.semitones).collect();
            audio.0.chord(Instrument::Keys, midis, 0.07, 1.6, 0.0);
        }
    });
    ui.add_space(8.0);
    ui.label(scale.tip);
    // silence the unused-import warning when the crate is built without the lookup
    let _ = find_scale(scale.name);
}

fn drills_ui(ui: &mut egui::Ui, state: &mut AppState, audio: &Audio) {
    ui.horizontal(|ui| {
        ui.label("Drill");
        egui::ComboBox::from_id_salt("drill")
            .selected_text(DRILL_IDS[state.drill_index])
            .show_ui(ui, |ui| {
                for (i, id) in DRILL_IDS.iter().enumerate() {
                    ui.selectable_value(&mut state.drill_index, i, *id);
                }
            });
        ui.label("Key");
        egui::ComboBox::from_id_salt("key")
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
            let root = jazz_theory::note::midi_for(state.drill_key, 4);
            let _ = root;
            for n in &d.notes {
                if note_chip(ui, &n.name, &n.degree).clicked() {
                    audio.0.note(Instrument::Vibes, n.midi, 0.20, 0.45, 0.0);
                }
            }
        });
        ui.add_space(8.0);
        if ui.button("\u{25B6} Play the line").clicked() {
            for (i, n) in d.notes.iter().enumerate() {
                audio.0.note(Instrument::Vibes, n.midi, 0.20, 0.30, i as f32 * 0.33);
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
    for (index, notes) in ear::voicing(q) {
        let at = 0.05 + index as f32 * slot;
        audio.0.chord(instrument, notes, gain, dur, at);
    }
}

fn ear_ui(ui: &mut egui::Ui, state: &mut AppState, audio: &Audio) {
    ui.horizontal(|ui| {
        let before = state.ear_mode;
        ui.label("Mode");
        egui::ComboBox::from_id_salt("mode")
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

/// The Masters tab: a gallery of players, each opening onto a biography page —
/// who they were, what happened to them, and what to listen for.
fn masters_ui(ui: &mut egui::Ui, state: &mut AppState) {
    let gold = egui::Color32::from_rgb(224, 166, 60);

    // ---- the biography page ----
    if let Some(i) = state.selected_master {
        let m = &MASTERS[i];
        egui::ScrollArea::vertical().show(ui, |ui| {
            if ui.button("\u{25C0} All masters").clicked() {
                state.selected_master = None;
            }
            ui.add_space(8.0);
            ui.label(egui::RichText::new(m.name).size(30.0).strong().color(gold));
            let dates = if m.died.is_empty() {
                format!("born {}", m.born)
            } else {
                format!("{} \u{2013} {}", m.born, m.died)
            };
            ui.label(
                egui::RichText::new(format!("{} \u{00B7} {} \u{00B7} {}", m.instrument, m.era, dates))
                    .weak(),
            );
            ui.add_space(10.0);
            ui.add(egui::Label::new(egui::RichText::new(m.bio).size(15.0)).wrap());
            ui.add_space(10.0);
            ui.group(|ui| {
                ui.label(egui::RichText::new("Listen for").strong().color(gold));
                ui.add(egui::Label::new(m.listen_for).wrap());
            });
            ui.add_space(12.0);
            ui.heading("A life in music");
            ui.add_space(4.0);
            for (year, event) in m.life {
                ui.horizontal_top(|ui| {
                    ui.add_sized(
                        [52.0, 20.0],
                        egui::Label::new(egui::RichText::new(*year).strong().color(gold)),
                    );
                    ui.add(egui::Label::new(*event).wrap());
                });
                ui.add_space(3.0);
            }
        });
        return;
    }

    // ---- the gallery ----
    egui::ScrollArea::vertical().show(ui, |ui| {
        ui.heading("Masters to study");
        ui.label(
            egui::RichText::new(
                "The language is learned by ear. Live with these players, sing along, then \
                 transcribe a single phrase. Click a name for the life behind the sound.",
            )
            .weak(),
        );
        ui.add_space(6.0);
        let mut open = None;
        for (i, m) in MASTERS.iter().enumerate() {
            let card = ui.group(|ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new(m.name).strong().color(gold));
                    ui.label(egui::RichText::new(format!("{} \u{00B7} {}", m.instrument, m.era)).weak());
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(egui::RichText::new("their story \u{25B6}").weak());
                    });
                });
                ui.label(m.listen_for);
            });
            let resp = card
                .response
                .interact(egui::Sense::click())
                .on_hover_cursor(egui::CursorIcon::PointingHand);
            if resp.clicked() {
                open = Some(i);
            }
        }
        if let Some(i) = open {
            state.selected_master = Some(i);
        }
        ui.add_space(4.0);
        ui.label(egui::RichText::new(format!("And more to explore: {ALSO_EXPLORE}")).italics().weak());
    });
}

/// The Standards tab: the repertoire, each tune one click from a jam.
fn standards_ui(ui: &mut egui::Ui, state: &mut AppState, audio: &Audio) {
    egui::ScrollArea::vertical().show(ui, |ui| {
        ui.heading("Essential standards");
        ui.label(
            egui::RichText::new(
                "Learn the melody and the changes, then improvise. \"Comp me\" starts a \
                 sparse trio behind your solo; \"Full band\" brings the busier rhythm \
                 section. Both load the tune into Play-Along at its own tempo and feel.",
            )
            .weak(),
        );
        ui.add_space(6.0);

        // Defer the load until after the loop so `state` is not borrowed twice.
        let mut load: Option<(&'static Standard, bool)> = None;
        for st in STANDARDS {
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new(st.title).strong());
                    ui.label(egui::RichText::new(format!("\u{2014} {}", st.composer)).weak());
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        match st.progression_id {
                            Some(_) => {
                                if ui.button("\u{25B6} Full band").clicked() {
                                    load = Some((st, true));
                                }
                                if ui.button("Comp me").clicked() {
                                    load = Some((st, false));
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

        if let Some((st, full_band)) = load {
            start_jam(state, audio, st, full_band);
        }
    });
}

/// Start jamming a standard: apply its idiomatic tempo and feel, set the band
/// up for the chosen mode — a sparse comping trio that leaves room for the
/// student's solo, or the full band with the horn section — and drop into the
/// Play-Along tab already playing. The chosen instruments (comp voice, bass,
/// horns) are the user's own from the per-part mixer.
fn start_jam(state: &mut AppState, audio: &Audio, st: &Standard, full_band: bool) {
    let Some(id) = st.progression_id else { return };
    let Some(i) = state.progs.iter().position(|p| p.id == id) else { return };
    state.prog_index = i;
    state.settings.tempo = st.tempo;
    state.pending_tempo = None;
    state.settings.swing = st.swing;
    state.settings.bass = true;
    state.settings.chords = true;
    state.settings.drums = true;
    // While the horn section is being reworked, "Full band" means the busier
    // rhythm section rather than added horns.
    state.settings.horns = full_band && HORNS_ENABLED;
    state.settings.density = if full_band { Density::Busy } else { Density::Sparse };
    load_progression(state, audio, i, true);
    state.tab = Tab::PlayAlong;
}

/// Open the default output device and hand the core to the callback.
///
/// On failure the core is returned so a later gesture can try again — losing the
/// device should never cost you the queued music.
fn start_audio(mut core: AudioCore) -> Result<cpal::Stream, (AudioCore, String)> {
    use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

    let host = cpal::default_host();
    let Some(device) = host.default_output_device() else {
        return Err((core, "no output device".into()));
    };
    let config = match device.default_output_config() {
        Ok(c) => c,
        Err(e) => return Err((core, format!("no usable output config: {e}"))),
    };
    let channels = config.channels() as usize;
    let rate = config.sample_rate().0 as f32;
    println!(
        "[audio] output device: {} @ {} Hz",
        device.name().unwrap_or_else(|_| "default device".into()),
        config.sample_rate().0
    );
    // Follow the device rather than assuming 44.1 kHz; a 48 kHz device would
    // otherwise play everything about a tone sharp and correspondingly fast.
    core.set_sample_rate(rate);

    // Load the SoundFont at the real device rate so its samples are not
    // transposed. Pitched instruments then play from FluidR3; drums stay synth.
    match find_soundfont() {
        Some(path) => match jazz_audio::soundfont::SoundfontVoices::load(&path, rate) {
            Ok(sf) => {
                println!("[audio] using soundfont {}", path.display());
                core.set_soundfont(Some(sf));
            }
            Err(why) => println!("[audio] built-in synth ({why})"),
        },
        None => println!("[audio] built-in synth (no soundfont found)"),
    }

    let mut core = core;
    let built = device.build_output_stream(
        &config.config(),
        move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
            core.fill(data, channels);
        },
        |err| eprintln!("audio stream error: {err}"),
        None,
    );
    match built {
        Ok(stream) => match stream.play() {
            Ok(()) => Ok(stream),
            Err(e) => Err((AudioCore::new(44_100.0).1, format!("could not start stream: {e}"))),
        },
        Err(e) => Err((AudioCore::new(44_100.0).1, format!("could not open stream: {e}"))),
    }
}
