use crate::env::{AdsrParams, Envelope};
use crate::error::SynthError;
use crate::osc::Oscillator;
use crate::wave::Waveform;

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
pub struct Engine {
    sample_rate: f32,
    osc: Oscillator,
    env: Envelope,
    gain: f32,
}

impl Engine {
    pub fn new(config: VoiceConfig) -> Result<Self, SynthError> {
        config.validate()?;
        Ok(Self {
            sample_rate: config.sample_rate.max(1.0),
            osc: Oscillator::new(config.sample_rate, config.frequency_hz, config.waveform),
            env: Envelope::new(config.sample_rate, config.adsr),
            gain: config.gain.clamp(0.0, 1.0),
        })
    }

    pub fn set_frequency(&mut self, frequency_hz: f32) -> Result<(), SynthError> {
        if frequency_hz <= 0.0 || !frequency_hz.is_finite() {
            return Err(SynthError::InvalidFrequency(frequency_hz));
        }
        self.osc.set_frequency(self.sample_rate, frequency_hz);
        Ok(())
    }

    pub fn set_waveform(&mut self, waveform: Waveform) {
        self.osc.set_waveform(waveform);
    }

    pub fn note_on(&mut self) {
        self.env.note_on();
    }

    pub fn note_off(&mut self) {
        self.env.note_off();
    }

    pub fn is_active(&self) -> bool {
        self.env.is_active()
    }

    pub fn next_sample(&mut self) -> f32 {
        let sample = self.osc.tick() * self.env.tick() * self.gain;
        sample.clamp(-1.0, 1.0)
    }

    pub fn render(&mut self, frames: usize) -> Vec<f32> {
        (0..frames).map(|_| self.next_sample()).collect()
    }
}
