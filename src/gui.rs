use std::collections::HashSet;
use std::sync::{Arc, Mutex};

use anyhow::{Context, Result};
use eframe::egui::{self, Color32, Key, RichText, Sense, Vec2};

use crate::audio::AudioOutput;
use crate::engine::{Engine, VoiceConfig};
use crate::env::AdsrParams;
use crate::keys::{self, PianoKey};
use crate::wave::Waveform;

pub fn run() -> Result<()> {
    let sample_rate = crate::output_sample_rate().unwrap_or(48_000.0);
    let waveform = Waveform::Sine;
    let gain = 0.22;
    let adsr = AdsrParams {
        attack_s: 0.01,
        decay_s: 0.12,
        sustain: 0.7,
        release_s: 0.18,
    };
    let engine = Arc::new(Mutex::new(Engine::new(VoiceConfig {
        sample_rate,
        frequency_hz: 261.63,
        waveform,
        gain,
        adsr,
    })?));
    let output = AudioOutput::start(Arc::clone(&engine)).context("Audio-Ausgabe fehlgeschlagen")?;

    let app = SynthGui {
        engine,
        _output: output,
        octave: 0,
        held: HashSet::new(),
        waveform,
        gain,
        attack: adsr.attack_s,
        decay: adsr.decay_s,
        sustain: adsr.sustain,
        release: adsr.release_s,
    };

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([820.0, 480.0])
            .with_min_inner_size([640.0, 380.0]),
        ..Default::default()
    };

    eframe::run_native(
        "simple-synth",
        options,
        Box::new(|_cc| Ok(Box::new(app))),
    )
    .map_err(|err| anyhow::anyhow!("GUI: {err}"))
}

struct SynthGui {
    engine: Arc<Mutex<Engine>>,
    _output: AudioOutput,
    octave: i32,
    held: HashSet<u8>,
    waveform: Waveform,
    gain: f32,
    attack: f32,
    decay: f32,
    sustain: f32,
    release: f32,
}

impl eframe::App for SynthGui {
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.handle_keyboard(ctx);
        ctx.request_repaint();
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        ui.heading("simple-synth");
        ui.label("Tastatur als Klavier — untere Reihe ab C4 (Z), obere Reihe ab C5 (Q).");
        ui.label("F1–F4 Wellenform, ←/→ Oktave. Akkorde: bis zu 8 Stimmen.");
        ui.add_space(8.0);

        ui.horizontal(|ui| {
            ui.label("Welle");
            for wave in Waveform::ALL {
                let selected = self.waveform == wave;
                if ui.selectable_label(selected, wave.label()).clicked() {
                    self.set_waveform(wave);
                }
            }
        });

        ui.horizontal(|ui| {
            ui.label("Oktave");
            if ui.button("−").clicked() {
                self.shift_octave(-1);
            }
            ui.monospace(format!("{:+}", self.octave));
            if ui.button("+").clicked() {
                self.shift_octave(1);
            }
        });

        if ui
            .add(egui::Slider::new(&mut self.gain, 0.0..=1.0).text("Gain"))
            .changed()
        {
            self.engine.lock().unwrap_or_else(|p| p.into_inner()).set_gain(self.gain);
        }
        if ui
            .add(egui::Slider::new(&mut self.attack, 0.0..=1.0).text("Attack"))
            .changed()
        {
            self.push_adsr();
        }
        if ui
            .add(egui::Slider::new(&mut self.decay, 0.0..=1.0).text("Decay"))
            .changed()
        {
            self.push_adsr();
        }
        if ui
            .add(egui::Slider::new(&mut self.sustain, 0.0..=1.0).text("Sustain"))
            .changed()
        {
            self.push_adsr();
        }
        if ui
            .add(egui::Slider::new(&mut self.release, 0.0..=2.0).text("Release"))
            .changed()
        {
            self.push_adsr();
        }

        ui.add_space(12.0);
        ui.label("Obere Reihe");
        self.draw_row(ui, &keys::UPPER_ROW);
        ui.add_space(6.0);
        ui.label("Untere Reihe");
        self.draw_row(ui, &keys::LOWER_ROW);
    }
}

impl SynthGui {
    fn handle_keyboard(&mut self, ctx: &egui::Context) {
        if ctx.wants_keyboard_input() {
            return;
        }

        ctx.input(|input| {
            if input.key_pressed(Key::ArrowLeft) {
                self.octave = (self.octave - 1).clamp(-2, 3);
            }
            if input.key_pressed(Key::ArrowRight) {
                self.octave = (self.octave + 1).clamp(-2, 3);
            }
            for key in [Key::F1, Key::F2, Key::F3, Key::F4] {
                if input.key_pressed(key) {
                    if let Some(wave) = keys::waveform_from_function_key(key) {
                        self.set_waveform(wave);
                    }
                }
            }
        });

        let mut desired = HashSet::new();
        ctx.input(|input| {
            for event in &input.events {
                if let egui::Event::Key {
                    key,
                    physical_key,
                    pressed,
                    repeat,
                    ..
                } = event
                {
                    if *repeat {
                        continue;
                    }
                    let mapped = physical_key.unwrap_or(*key);
                    if let Some(midi) = keys::midi_for_key(mapped, self.octave) {
                        if *pressed {
                            desired.insert(midi);
                        }
                    }
                }
            }
        });

        // Also treat currently-held piano keys as desired so we do not drop
        // notes when no new event arrives this frame.
        ctx.input(|input| {
            for key in all_piano_keys() {
                if input.key_down(key) {
                    if let Some(midi) = keys::midi_for_key(key, self.octave) {
                        desired.insert(midi);
                    }
                }
            }
        });

        self.sync_notes(desired);
    }

    fn draw_row(&mut self, ui: &mut egui::Ui, row: &[PianoKey]) {
        let mut clicked = HashSet::new();
        ui.horizontal(|ui| {
            for pk in row {
                if let Some(midi) = keys::midi_for_key(pk.key, self.octave) {
                    if piano_key_widget(ui, pk, self.held.contains(&midi)) {
                        clicked.insert(midi);
                    }
                }
            }
        });
        if !clicked.is_empty() {
            let mut desired = self.held.clone();
            desired.extend(clicked);
            self.sync_notes(desired);
        }
    }

    fn sync_notes(&mut self, desired: HashSet<u8>) {
        let mut synth = self.engine.lock().unwrap_or_else(|p| p.into_inner());
        for midi in desired.difference(&self.held) {
            synth.note_on_midi(*midi);
        }
        for midi in self.held.difference(&desired) {
            synth.note_off_midi(*midi);
        }
        drop(synth);
        self.held = desired;
    }

    fn set_waveform(&mut self, waveform: Waveform) {
        self.waveform = waveform;
        self.engine
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .set_waveform(waveform);
    }

    fn shift_octave(&mut self, delta: i32) {
        self.octave = (self.octave + delta).clamp(-2, 3);
    }

    fn push_adsr(&mut self) {
        let params = AdsrParams {
            attack_s: self.attack,
            decay_s: self.decay,
            sustain: self.sustain,
            release_s: self.release,
        };
        self.engine
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .set_adsr(params);
    }
}

fn all_piano_keys() -> impl Iterator<Item = Key> {
    keys::LOWER_ROW
        .iter()
        .chain(keys::UPPER_ROW.iter())
        .map(|k| k.key)
}

fn piano_key_widget(ui: &mut egui::Ui, pk: &PianoKey, held: bool) -> bool {
    let fill = if held {
        Color32::from_rgb(255, 176, 64)
    } else if pk.black {
        Color32::from_rgb(36, 36, 40)
    } else {
        Color32::from_rgb(235, 235, 232)
    };
    let text = if pk.black && !held {
        Color32::from_gray(230)
    } else {
        Color32::from_gray(20)
    };
    let size = if pk.black {
        Vec2::new(34.0, 72.0)
    } else {
        Vec2::new(44.0, 108.0)
    };
    let response = ui.add_sized(
        size,
        egui::Button::new(RichText::new(pk.label).color(text).small()).fill(fill),
    );
    response.interact(Sense::click_and_drag()).is_pointer_button_down_on()
}
