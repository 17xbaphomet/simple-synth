use crate::wave::Waveform;

#[derive(Clone, Debug)]
pub struct Oscillator {
    waveform: Waveform,
    phase: f32,
    increment: f32,
}

impl Oscillator {
    pub fn new(sample_rate: f32, frequency_hz: f32, waveform: Waveform) -> Self {
        let mut osc = Self {
            waveform,
            phase: 0.0,
            increment: 0.0,
        };
        osc.set_frequency(sample_rate, frequency_hz);
        osc
    }

    pub fn set_waveform(&mut self, waveform: Waveform) {
        self.waveform = waveform;
    }

    pub fn set_frequency(&mut self, sample_rate: f32, frequency_hz: f32) {
        let safe_rate = sample_rate.max(1.0);
        self.increment = (frequency_hz.max(0.0) / safe_rate).clamp(0.0, 0.5);
    }

    pub fn tick(&mut self) -> f32 {
        let sample = self.waveform.sample(self.phase);
        self.phase = (self.phase + self.increment) % 1.0;
        sample
    }
}
