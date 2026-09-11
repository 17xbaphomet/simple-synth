use thiserror::Error;

#[derive(Debug, Error)]
pub enum SynthError {
    #[error("unbekannte Wellenform '{0}' (erlaubt: sine, square, saw, triangle)")]
    UnknownWaveform(String),
    #[error("ungültige Note '{0}' (Beispiel: A4, C#5, Bb3)")]
    InvalidNote(String),
    #[error("Frequenz muss > 0 Hz sein, war {0}")]
    InvalidFrequency(f32),
    #[error("Dauer muss > 0 s sein, war {0}")]
    InvalidDuration(f32),
    #[error("kein Audio-Ausgabegerät gefunden")]
    NoOutputDevice,
    #[error("Audio-Backend: {0}")]
    Audio(#[from] cpal::Error),
    #[error("WAV-Export: {0}")]
    Wav(#[from] hound::Error),
    #[error("nicht unterstütztes Sample-Format: {0}")]
    UnsupportedSampleFormat(cpal::SampleFormat),
}
