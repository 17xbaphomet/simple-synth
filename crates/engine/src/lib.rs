//! Zweiton DSP-Kern — kein Audio-Host, kein GUI.

mod engine;
mod env;
mod error;
mod note;
mod osc;
mod wave;

pub use engine::{Engine, NoiseDist, NoiseMode, VoiceConfig};
pub use env::AdsrParams;
pub use error::SynthError;
pub use note::{frequency_from_note, midi_to_hz};
pub use osc::OscInteract;
pub use wave::Waveform;
