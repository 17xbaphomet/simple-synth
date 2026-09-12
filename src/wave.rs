use std::f32::consts::TAU;
use std::str::FromStr;

use crate::error::SynthError;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Waveform {
    #[default]
    Sine,
    Square,
    Saw,
    Triangle,
}

impl Waveform {
    pub const ALL: [Self; 4] = [Self::Sine, Self::Square, Self::Saw, Self::Triangle];

    pub fn label(self) -> &'static str {
        match self {
            Self::Sine => "sine",
            Self::Square => "square",
            Self::Saw => "saw",
            Self::Triangle => "triangle",
        }
    }

    pub fn sample(self, phase: f32) -> f32 {
        let phase = phase.fract().abs();
        match self {
            Self::Sine => (phase * TAU).sin(),
            Self::Square => {
                if phase < 0.5 {
                    1.0
                } else {
                    -1.0
                }
            }
            Self::Saw => phase.mul_add(2.0, -1.0),
            Self::Triangle => {
                if phase < 0.5 {
                    phase.mul_add(4.0, -1.0)
                } else {
                    phase.mul_add(-4.0, 3.0)
                }
            }
        }
    }
}

impl FromStr for Waveform {
    type Err = SynthError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.trim().to_ascii_lowercase().as_str() {
            "sine" | "sin" | "sinus" => Ok(Self::Sine),
            "square" | "sqr" | "rechteck" => Ok(Self::Square),
            "saw" | "sawtooth" | "saege" | "säge" => Ok(Self::Saw),
            "triangle" | "tri" | "dreieck" => Ok(Self::Triangle),
            other => Err(SynthError::UnknownWaveform(other.to_owned())),
        }
    }
}
