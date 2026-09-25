use thiserror::Error;

/// Nur DSP-/Notenfehler. Host-Fehler (cpal, hound) leben im Standalone-Binary.
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
}
