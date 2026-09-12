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
        self.increment = (frequency_hz.max(0.0) / safe_rate).clamp(0.0, 0.49);
    }

    pub fn reset_phase(&mut self) {
        self.phase = 0.0;
    }

    pub fn tick(&mut self) -> f32 {
        self.tick_wrap().0
    }

    pub fn tick_wrap(&mut self) -> (f32, bool) {
        let sample = self.waveform.sample(self.phase);
        let next = self.phase + self.increment;
        let wrapped = next >= 1.0;
        self.phase = next.fract().abs();
        (sample, wrapped)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum OscInteract {
    #[default]
    Mix,
    Am,
    Ring,
    Fm,
    Sync,
}

impl OscInteract {
    pub const ALL: [Self; 5] = [Self::Mix, Self::Am, Self::Ring, Self::Fm, Self::Sync];

    pub fn label(self) -> &'static str {
        match self {
            Self::Mix => "Mix",
            Self::Am => "AM",
            Self::Ring => "Ring",
            Self::Fm => "FM",
            Self::Sync => "Sync",
        }
    }
}
