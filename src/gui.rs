use std::collections::HashSet;
use std::sync::{Arc, Mutex};

use anyhow::{Context, Result};
use eframe::egui::{self, Color32, Key, RichText, Sense, Vec2};

use crate::audio::AudioOutput;
use crate::engine::{Engine, VoiceConfig};
use crate::env::AdsrParams;
use crate::keys::{self, Layout, PianoKey};
use crate::osc::OscInteract;
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
        keyboard_notes: HashSet::new(),
        pointer_notes: HashSet::new(),
        layout: Layout::Qwertz,
        wave_a: waveform,
        wave_b: Waveform::Saw,
        mode: OscInteract::Mix,
        mix: 0.35,
        ratio: 1.0,
        detune: 7.0,
        depth: 0.6,
        gain,
        noise: 0.0,
        separate_adsr: false,
        attack: adsr.attack_s,
        decay: adsr.decay_s,
        sustain: adsr.sustain,
        release: adsr.release_s,
        attack_b: adsr.attack_s,
        decay_b: adsr.decay_s,
        sustain_b: adsr.sustain,
        release_b: adsr.release_s,
    };
    app.push_osc_params();
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([860.0, 720.0])
            .with_min_inner_size([680.0, 560.0]),
        ..Default::default()
    };
    eframe::run_native("simple-synth", options, Box::new(|_cc| Ok(Box::new(app))))
        .map_err(|err| anyhow::anyhow!("GUI: {err}"))
}

struct SynthGui {
    engine: Arc<Mutex<Engine>>,
    _output: AudioOutput,
    octave: i32,
    held: HashSet<u8>,
    keyboard_notes: HashSet<u8>,
    pointer_notes: HashSet<u8>,
    layout: Layout,
    wave_a: Waveform,
    wave_b: Waveform,
    mode: OscInteract,
    mix: f32,
    ratio: f32,
    detune: f32,
    depth: f32,
    gain: f32,
    noise: f32,
    separate_adsr: bool,
    attack: f32,
    decay: f32,
    sustain: f32,
    release: f32,
    attack_b: f32,
    decay_b: f32,
    sustain_b: f32,
    release_b: f32,
}

impl eframe::App for SynthGui {
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.handle_keyboard(ctx);
        self.sync_union();
        ctx.request_repaint();
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        ui.heading("simple-synth");
        ui.label("Zwei Oszillatoren: Mix, AM, Ring, FM, Sync. Esc = Panic.");
        ui.horizontal(|ui| {
            ui.label("Osc A");
            for wave in Waveform::ALL {
                if ui.selectable_label(self.wave_a == wave, wave.label()).clicked() {
                    self.wave_a = wave;
                    self.engine.lock().unwrap_or_else(|p| p.into_inner()).set_waveform(wave);
                }
            }
        });
        ui.horizontal(|ui| {
            ui.label("Osc B");
            for wave in Waveform::ALL {
                if ui.selectable_label(self.wave_b == wave, wave.label()).clicked() {
                    self.wave_b = wave;
                    self.engine.lock().unwrap_or_else(|p| p.into_inner()).set_waveform_b(wave);
                }
            }
        });
        ui.horizontal(|ui| {
            ui.label("Modus");
            for mode in OscInteract::ALL {
                if ui.selectable_label(self.mode == mode, mode.label()).clicked() {
                    self.mode = mode;
                    self.engine.lock().unwrap_or_else(|p| p.into_inner()).set_osc_mode(mode);
                }
            }
        });
        if ui.add(egui::Slider::new(&mut self.mix, 0.0..=1.0).text("Mix A->B")).changed() {
            self.engine.lock().unwrap_or_else(|p| p.into_inner()).set_osc_mix(self.mix);
        }
        if ui.add(egui::Slider::new(&mut self.ratio, 0.25..=4.0).text("Verhaeltnis B/A")).changed() {
            self.engine.lock().unwrap_or_else(|p| p.into_inner()).set_osc_ratio(self.ratio);
        }
        if ui.add(egui::Slider::new(&mut self.detune, -50.0..=50.0).text("Detune B (Cent)")).changed() {
            self.engine.lock().unwrap_or_else(|p| p.into_inner()).set_osc_detune(self.detune);
        }
        if ui.add(egui::Slider::new(&mut self.depth, 0.0..=2.0).text("Tiefe AM/FM")).changed() {
            self.engine.lock().unwrap_or_else(|p| p.into_inner()).set_osc_depth(self.depth);
        }
        ui.horizontal(|ui| {
            ui.label("Layout");
            for layout in [Layout::Qwertz, Layout::Qwerty] {
                if ui.selectable_label(self.layout == layout, layout.name()).clicked() {
                    self.layout = layout;
                }
            }
            if ui.button("-").clicked() {
                self.shift_octave(-1);
            }
            ui.monospace(format!("{:+}", self.octave));
            if ui.button("+").clicked() {
                self.shift_octave(1);
            }
        });
        if ui.add(egui::Slider::new(&mut self.gain, 0.0..=1.0).text("Gain")).changed() {
            self.engine.lock().unwrap_or_else(|p| p.into_inner()).set_gain(self.gain);
        }
        if ui
            .add(egui::Slider::new(&mut self.noise, 0.0..=0.5).text("Zufallsabweichung"))
            .changed()
        {
            self.engine
                .lock()
                .unwrap_or_else(|p| p.into_inner())
                .set_noise(self.noise);
        }
        if ui.checkbox(&mut self.separate_adsr, "Getrennte ADSR").changed() {
            let mut synth = self.engine.lock().unwrap_or_else(|p| p.into_inner());
            synth.set_separate_adsr(self.separate_adsr);
            drop(synth);
            if self.separate_adsr {
                self.attack_b = self.attack;
                self.decay_b = self.decay;
                self.sustain_b = self.sustain;
                self.release_b = self.release;
            } else {
                self.push_adsr_shared();
            }
        }
        if self.separate_adsr {
            ui.columns(2, |cols| {
                cols[0].label("ADSR A");
                if cols[0]
                    .add(egui::Slider::new(&mut self.attack, 0.0..=1.0).text("Attack A"))
                    .changed()
                {
                    self.push_adsr_a();
                }
                if cols[0]
                    .add(egui::Slider::new(&mut self.decay, 0.0..=1.0).text("Decay A"))
                    .changed()
                {
                    self.push_adsr_a();
                }
                if cols[0]
                    .add(egui::Slider::new(&mut self.sustain, 0.0..=1.0).text("Sustain A"))
                    .changed()
                {
                    self.push_adsr_a();
                }
                if cols[0]
                    .add(egui::Slider::new(&mut self.release, 0.0..=2.0).text("Release A"))
                    .changed()
                {
                    self.push_adsr_a();
                }
                cols[1].label("ADSR B");
                if cols[1]
                    .add(egui::Slider::new(&mut self.attack_b, 0.0..=1.0).text("Attack B"))
                    .changed()
                {
                    self.push_adsr_b();
                }
                if cols[1]
                    .add(egui::Slider::new(&mut self.decay_b, 0.0..=1.0).text("Decay B"))
                    .changed()
                {
                    self.push_adsr_b();
                }
                if cols[1]
                    .add(egui::Slider::new(&mut self.sustain_b, 0.0..=1.0).text("Sustain B"))
                    .changed()
                {
                    self.push_adsr_b();
                }
                if cols[1]
                    .add(egui::Slider::new(&mut self.release_b, 0.0..=2.0).text("Release B"))
                    .changed()
                {
                    self.push_adsr_b();
                }
            });
        } else {
            if ui.add(egui::Slider::new(&mut self.attack, 0.0..=1.0).text("Attack")).changed() {
                self.push_adsr_shared();
            }
            if ui.add(egui::Slider::new(&mut self.decay, 0.0..=1.0).text("Decay")).changed() {
                self.push_adsr_shared();
            }
            if ui.add(egui::Slider::new(&mut self.sustain, 0.0..=1.0).text("Sustain")).changed() {
                self.push_adsr_shared();
            }
            if ui.add(egui::Slider::new(&mut self.release, 0.0..=2.0).text("Release")).changed() {
                self.push_adsr_shared();
            }
        }
        ui.add_space(8.0);
        ui.label("Obere Reihe");
        let mut pointer = HashSet::new();
        pointer.extend(self.draw_row(ui, &keys::UPPER_ROW));
        ui.label("Untere Reihe");
        pointer.extend(self.draw_row(ui, &keys::LOWER_ROW));
        self.pointer_notes = pointer;
        self.sync_union();
    }
}

impl SynthGui {
    fn push_osc_params(&self) {
        let mut synth = self.engine.lock().unwrap_or_else(|p| p.into_inner());
        synth.set_waveform(self.wave_a);
        synth.set_waveform_b(self.wave_b);
        synth.set_osc_mode(self.mode);
        synth.set_osc_mix(self.mix);
        synth.set_osc_ratio(self.ratio);
        synth.set_osc_detune(self.detune);
        synth.set_osc_depth(self.depth);
        synth.set_noise(self.noise);
        synth.set_separate_adsr(self.separate_adsr);
        synth.set_adsr(self.adsr_a());
        synth.set_adsr_b(self.adsr_b());
    }

    fn adsr_a(&self) -> AdsrParams {
        AdsrParams {
            attack_s: self.attack,
            decay_s: self.decay,
            sustain: self.sustain,
            release_s: self.release,
        }
    }

    fn adsr_b(&self) -> AdsrParams {
        AdsrParams {
            attack_s: self.attack_b,
            decay_s: self.decay_b,
            sustain: self.sustain_b,
            release_s: self.release_b,
        }
    }

    fn push_adsr_a(&mut self) {
        self.engine
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .set_adsr(self.adsr_a());
    }

    fn push_adsr_b(&mut self) {
        self.engine
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .set_adsr_b(self.adsr_b());
    }

    fn push_adsr_shared(&mut self) {
        self.attack_b = self.attack;
        self.decay_b = self.decay;
        self.sustain_b = self.sustain;
        self.release_b = self.release;
        let params = self.adsr_a();
        let mut synth = self.engine.lock().unwrap_or_else(|p| p.into_inner());
        synth.set_adsr(params);
        synth.set_adsr_b(params);
    }

    fn handle_keyboard(&mut self, ctx: &egui::Context) {
        if ctx.input(|input| input.key_pressed(Key::Escape)) {
            self.engine.lock().unwrap_or_else(|p| p.into_inner()).all_notes_off();
            self.held.clear();
            self.keyboard_notes.clear();
            self.pointer_notes.clear();
            return;
        }
        if ctx.egui_wants_keyboard_input() {
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
                        self.wave_a = wave;
                        self.engine.lock().unwrap_or_else(|p| p.into_inner()).set_waveform(wave);
                    }
                }
            }
        });
        let mut desired = HashSet::new();
        ctx.input(|input| {
            for position in all_piano_keys() {
                if input.key_down(self.layout.typed_key(position)) {
                    if let Some(midi) = keys::midi_for_position(position, self.octave) {
                        desired.insert(midi);
                    }
                }
            }
        });
        self.keyboard_notes = desired;
    }

    fn draw_row(&mut self, ui: &mut egui::Ui, row: &[PianoKey]) -> HashSet<u8> {
        let mut pointer_down = HashSet::new();
        ui.horizontal(|ui| {
            for pk in row {
                if let Some(midi) = keys::midi_for_position(pk.key, self.octave) {
                    if piano_key_widget(ui, pk, self.layout, self.held.contains(&midi)) {
                        pointer_down.insert(midi);
                    }
                }
            }
        });
        pointer_down
    }

    fn sync_union(&mut self) {
        let mut desired = self.keyboard_notes.clone();
        desired.extend(self.pointer_notes.iter().copied());
        self.sync_notes(desired);
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

    fn shift_octave(&mut self, delta: i32) {
        self.octave = (self.octave + delta).clamp(-2, 3);
    }
}

fn all_piano_keys() -> impl Iterator<Item = Key> {
    keys::LOWER_ROW.iter().chain(keys::UPPER_ROW.iter()).map(|k| k.key)
}

fn piano_key_widget(ui: &mut egui::Ui, pk: &PianoKey, layout: Layout, held: bool) -> bool {
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
        egui::Button::new(RichText::new(pk.label(layout)).color(text).small()).fill(fill),
    );
    response.interact(Sense::click_and_drag()).is_pointer_button_down_on()
}
