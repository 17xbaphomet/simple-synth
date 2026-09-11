use crate::env::{AdsrParams, Envelope};
use crate::error::SynthError;
use crate::note::midi_to_hz;
use crate::osc::Oscillator;
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
    osc: Oscillator,
    env: Envelope,
    note: Option<u8>,
}

impl Voice {
    fn new(sample_rate: f32, frequency_hz: f32, waveform: Waveform, adsr: AdsrParams) -> Self {
        Self {
            osc: Oscillator::new(sample_rate, frequency_hz, waveform),
            env: Envelope::new(sample_rate, adsr),
            note: None,
        }
    }
}

#[derive(Debug)]
pub struct Engine {
    sample_rate: f32,
    waveform: Waveform,
    adsr: AdsrParams,
    gain: f32,
    freq: f32,
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
            waveform: config.waveform,
            adsr: config.adsr.sanitized(),
            gain: config.gain.clamp(0.0, 1.0),
            freq: config.frequency_hz,
            voices,
        })
    }

    pub fn set_frequency(&mut self, frequency_hz: f32) -> Result<(), SynthError> {
        if frequency_hz <= 0.0 || !frequency_hz.is_finite() {
            return Err(SynthError::InvalidFrequency(frequency_hz));
        }
        self.freq = frequency_hz;
        for voice in &mut self.voices {
            if voice.note.is_none() && voice.env.is_active() {
                voice.osc.set_frequency(self.sample_rate, frequency_hz);
            }
        }
        Ok(())
    }

    pub fn set_waveform(&mut self, waveform: Waveform) {
        self.waveform = waveform;
        for voice in &mut self.voices {
            voice.osc.set_waveform(waveform);
        }
    }

    pub fn set_gain(&mut self, gain: f32) {
        self.gain = gain.clamp(0.0, 1.0);
    }

    pub fn set_adsr(&mut self, adsr: AdsrParams) {
        self.adsr = adsr.sanitized();
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

    pub fn is_active(&self) -> bool {
        self.voices.iter().any(|voice| voice.env.is_active())
    }

    pub fn next_sample(&mut self) -> f32 {
        let mix: f32 = self
            .voices
            .iter_mut()
            .map(|voice| {
                if !voice.env.is_active() {
                    voice.note = None;
                    return 0.0;
                }
                voice.osc.tick() * voice.env.tick()
            })
            .sum();
        (mix * self.gain).clamp(-1.0, 1.0)
    }

    pub fn render(&mut self, frames: usize) -> Vec<f32> {
        (0..frames).map(|_| self.next_sample()).collect()
    }

    fn start_voice(&mut self, note: Option<u8>, freq: f32) {
        let index = self.allocate(note);
        let voice = &mut self.voices[index];
        voice.osc.set_frequency(self.sample_rate, freq);
        voice.osc.set_waveform(self.waveform);
        voice.env = Envelope::new(self.sample_rate, self.adsr);
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
