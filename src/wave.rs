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
    /// `phase` im Intervall [0, 1).
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

#[cfg(test)]
mod tests {
    use super::Waveform;

    #[test]
    fn sine_zero_crossing_at_origin() {
        assert!((Waveform::Sine.sample(0.0)).abs() < f32::EPSILON);
    }

    #[test]
    fn square_sign_changes_at_half() {
        assert_eq!(Waveform::Square.sample(0.0), 1.0);
        assert_eq!(Waveform::Square.sample(0.5), -1.0);
    }
}
