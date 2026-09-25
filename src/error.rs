use thiserror::Error;
use zweiton_engine::SynthError;

/// Host-Fehler (cpal, hound) plus durchgereichte Engine-Fehler.
#[derive(Debug, Error)]
pub enum HostError {
    #[error(transparent)]
    Engine(#[from] SynthError),
    #[error("kein Audio-Ausgabegerät gefunden")]
    NoOutputDevice,
    #[error("Audio-Backend: {0}")]
    Audio(#[from] cpal::Error),
    #[error("WAV-Export: {0}")]
    Wav(#[from] hound::Error),
    #[error("nicht unterstütztes Sample-Format: {0}")]
    UnsupportedSampleFormat(cpal::SampleFormat),
}
