use crate::error::SynthError;

pub fn frequency_from_note(name: &str) -> Result<f32, SynthError> {
    let raw = name.trim();
    if raw.is_empty() {
        return Err(SynthError::InvalidNote(name.to_owned()));
    }

    let mut chars = raw.chars();
    let letter = chars
        .next()
        .map(|c| c.to_ascii_uppercase())
        .ok_or_else(|| SynthError::InvalidNote(name.to_owned()))?;

    let base = match letter {
        'C' => 0,
        'D' => 2,
        'E' => 4,
        'F' => 5,
        'G' => 7,
        'A' => 9,
        'B' | 'H' => 11,
        _ => return Err(SynthError::InvalidNote(name.to_owned())),
    };

    let rest: String = chars.collect();
    let (accidental, octave_str) = if rest.starts_with('#') || rest.starts_with('♯') {
        let skip = rest.chars().next().map(|c| c.len_utf8()).unwrap_or(1);
        (1, &rest[skip..])
    } else if rest.starts_with('b') || rest.starts_with('♭') {
        (-1, &rest[1..])
    } else {
        (0, rest.as_str())
    };

    let octave: i32 = octave_str
        .parse()
        .map_err(|_| SynthError::InvalidNote(name.to_owned()))?;

    let midi = (octave + 1) * 12 + base + accidental;
    Ok(midi_to_hz(midi as f32))
}

pub fn midi_to_hz(midi: f32) -> f32 {
    440.0 * 2_f32.powf((midi - 69.0) / 12.0)
}

#[cfg(test)]
mod tests {
    use super::{frequency_from_note, midi_to_hz};

    #[test]
    fn a4_is_440() {
        assert!((frequency_from_note("A4").unwrap() - 440.0).abs() < 0.001);
        assert!((midi_to_hz(69.0) - 440.0).abs() < f32::EPSILON);
    }

    #[test]
    fn c4_is_middle_c() {
        let hz = frequency_from_note("C4").unwrap();
        assert!((hz - midi_to_hz(60.0)).abs() < 0.001);
    }

    #[test]
    fn german_h4_is_b4() {
        let h = frequency_from_note("H4").unwrap();
        let b = frequency_from_note("B4").unwrap();
        assert!((h - b).abs() < 0.001);
    }
}
