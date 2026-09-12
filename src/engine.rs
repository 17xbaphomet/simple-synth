use crate::env::{AdsrParams, Envelope};
use crate::error::SynthError;
use crate::note::midi_to_hz;
use crate::osc::{OscInteract, Oscillator};
use crate::wave::Waveform;

const MAX_VOICES: usize = 8;

#[derive(Clone, Debug)]
pub struct VoiceConfig {
    pub sample_rate: f32,
    pub frequency_hz: f32,
    pub waveform: Waveform,
    pub gain: f32,
    pub adsr: AdsrParams,
}

impl VoiceConfig {
    pub fn validate(&self) -> Result<(), SynthError> {
        if self.frequency_hz <= 0.0 || !self.frequency_hz.is_finite() {
            return Err(SynthError::InvalidFrequency(self.frequency_hz));
        }
        Ok(())
    }
}

#[derive(Debug)]
struct Voice {
    osc_a: Oscillator,
    osc_b: Oscillator,
    env: Envelope,
    note: Option<u8>,
    freq: f32,
}

impl Voice {
    fn new(sample_rate: f32, frequency_hz: f32, waveform: Waveform, adsr: AdsrParams) -> Self {
        Self {
            osc_a: Oscillator::new(sample_rate, frequency_hz, waveform),
            osc_b: Oscillator::new(sample_rate, frequency_hz, waveform),
            env: Envelope::new(sample_rate, adsr),
            note: None,
            freq: frequency_hz,
        }
    }
}

#[derive(Debug)]
pub struct Engine {
    sample_rate: f32,
    wave_a: Waveform,
    wave_b: Waveform,
    adsr: AdsrParams,
    gain: f32,
    freq: f32,
    mix: f32,
    ratio: f32,
    detune_cents: f32,
    depth: f32,
    mode: OscInteract,
    voices: Vec<Voice>,
}

impl Engine {
    pub fn new(config: VoiceConfig) -> Result<Self, SynthError> {
        config.validate()?;
        let sample_rate = config.sample_rate.max(1.0);
        let voices = (0..MAX_VOICES)
            .map(|_| Voice::new(sample_rate, config.frequency_hz, config.waveform, config.adsr))
            .collect();
        Ok(Self {
            sample_rate,
            wave_a: config.waveform,
            wave_b: config.waveform,
            adsr: config.adsr.sanitized(),
            gain: config.gain.clamp(0.0, 1.0),
            freq: config.frequency_hz,
            mix: 0.0,
            ratio: 1.0,
            detune_cents: 0.0,
            depth: 0.5,
            mode: OscInteract::Mix,
            voices,
        })
    }

    pub fn set_frequency(&mut self, frequency_hz: f32) -> Result<(), SynthError> {
        if frequency_hz <= 0.0 || !frequency_hz.is_finite() {
            return Err(SynthError::InvalidFrequency(frequency_hz));
        }
        self.freq = frequency_hz;
        let sample_rate = self.sample_rate;
        let ratio = self.ratio;
        let detune = self.detune_cents;
        for voice in &mut self.voices {
            if voice.note.is_none() && voice.env.is_active() {
                voice.freq = frequency_hz;
                apply_freqs(voice, sample_rate, ratio, detune);
            }
        }
        Ok(())
    }

    pub fn set_waveform(&mut self, waveform: Waveform) {
        self.wave_a = waveform;
        for voice in &mut self.voices {
            voice.osc_a.set_waveform(waveform);
        }
    }

    pub fn set_waveform_b(&mut self, waveform: Waveform) {
        self.wave_b = waveform;
        for voice in &mut self.voices {
            voice.osc_b.set_waveform(waveform);
        }
    }

    pub fn set_gain(&mut self, gain: f32) {
        self.gain = gain.clamp(0.0, 1.0);
    }

    pub fn set_adsr(&mut self, adsr: AdsrParams) {
        self.adsr = adsr.sanitized();
    }

    pub fn set_osc_mix(&mut self, mix: f32) {
        self.mix = mix.clamp(0.0, 1.0);
    }

    pub fn set_osc_ratio(&mut self, ratio: f32) {
        self.ratio = ratio.clamp(0.25, 8.0);
    }

    pub fn set_osc_detune(&mut self, cents: f32) {
        self.detune_cents = cents.clamp(-100.0, 100.0);
    }

    pub fn set_osc_depth(&mut self, depth: f32) {
        self.depth = depth.clamp(0.0, 2.0);
    }

    pub fn set_osc_mode(&mut self, mode: OscInteract) {
        self.mode = mode;
    }

    pub fn note_on(&mut self) {
        self.start_voice(None, self.freq);
    }

    pub fn note_off(&mut self) {
        for voice in &mut self.voices {
            if voice.note.is_none() && voice.env.is_active() {
                voice.env.note_off();
            }
        }
    }

    pub fn note_on_midi(&mut self, midi: u8) {
        let freq = midi_to_hz(f32::from(midi));
        self.freq = freq;
        self.start_voice(Some(midi), freq);
    }

    pub fn note_off_midi(&mut self, midi: u8) {
        for voice in &mut self.voices {
            if voice.note == Some(midi) {
                voice.env.note_off();
            }
        }
    }

    pub fn all_notes_off(&mut self) {
        for voice in &mut self.voices {
            voice.env.reset();
            voice.note = None;
        }
    }

    pub fn is_active(&self) -> bool {
        self.voices.iter().any(|voice| voice.env.is_active())
    }

    pub fn next_sample(&mut self) -> f32 {
        let mix_ab = self.mix;
        let depth = self.depth;
        let mode = self.mode;
        let sample_rate = self.sample_rate;
        let ratio = self.ratio;
        let detune = self.detune_cents;
        let mix: f32 = self
            .voices
            .iter_mut()
            .map(|voice| {
                if !voice.env.is_active() {
                    voice.note = None;
                    return 0.0;
                }
                let freq_b = freq_b_of(voice.freq, ratio, detune);
                voice.osc_b.set_frequency(sample_rate, freq_b);
                let pair = tick_pair(voice, sample_rate, mode, mix_ab, depth);
                pair * voice.env.tick()
            })
            .sum();
        (mix * self.gain).clamp(-1.0, 1.0)
    }

    pub fn render(&mut self, frames: usize) -> Vec<f32> {
        (0..frames).map(|_| self.next_sample()).collect()
    }

    fn start_voice(&mut self, note: Option<u8>, freq: f32) {
        let index = self.allocate(note);
        let sample_rate = self.sample_rate;
        let ratio = self.ratio;
        let detune = self.detune_cents;
        let wave_a = self.wave_a;
        let wave_b = self.wave_b;
        let adsr = self.adsr;
        let voice = &mut self.voices[index];
        voice.freq = freq;
        voice.osc_a.set_waveform(wave_a);
        voice.osc_b.set_waveform(wave_b);
        voice.osc_a.reset_phase();
        voice.osc_b.reset_phase();
        apply_freqs(voice, sample_rate, ratio, detune);
        voice.env = Envelope::new(sample_rate, adsr);
        voice.env.note_on();
        voice.note = note;
    }

    fn allocate(&self, note: Option<u8>) -> usize {
        if let Some(note) = note {
            if let Some(index) = self.voices.iter().position(|voice| voice.note == Some(note)) {
                return index;
            }
        }
        if let Some(index) = self.voices.iter().position(|voice| !voice.env.is_active()) {
            return index;
        }
        0
    }
}

fn apply_freqs(voice: &mut Voice, sample_rate: f32, ratio: f32, detune_cents: f32) {
    voice.osc_a.set_frequency(sample_rate, voice.freq);
    voice
        .osc_b
        .set_frequency(sample_rate, freq_b_of(voice.freq, ratio, detune_cents));
}

fn freq_b_of(freq_a: f32, ratio: f32, detune_cents: f32) -> f32 {
    freq_a * ratio.clamp(0.25, 8.0) * 2_f32.powf(detune_cents / 1200.0)
}

fn tick_pair(voice: &mut Voice, sample_rate: f32, mode: OscInteract, mix: f32, depth: f32) -> f32 {
    match mode {
        OscInteract::Mix => {
            let a = voice.osc_a.tick();
            let b = voice.osc_b.tick();
            a.mul_add(1.0 - mix, b * mix)
        }
        OscInteract::Am => {
            let a = voice.osc_a.tick();
            let b = voice.osc_b.tick();
            (a * (1.0 + depth * b)).clamp(-1.0, 1.0)
        }
        OscInteract::Ring => voice.osc_a.tick() * voice.osc_b.tick(),
        OscInteract::Fm => {
            let b = voice.osc_b.tick();
            let freq = (voice.freq * (1.0 + depth * 4.0 * b)).max(0.0);
            voice.osc_a.set_frequency(sample_rate, freq);
            voice.osc_a.tick()
        }
        OscInteract::Sync => {
            let (a, wrapped) = voice.osc_a.tick_wrap();
            if wrapped {
                voice.osc_b.reset_phase();
            }
            let b = voice.osc_b.tick();
            a.mul_add(1.0 - mix, b * mix)
        }
    }
}
