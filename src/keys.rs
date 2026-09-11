use eframe::egui::Key;

/// MIDI note number for C4.
pub const C4_MIDI: i32 = 60;

/// Semitone offset from C4 for a computer-keyboard piano key.
/// Uses logical/physical `egui::Key` names in QWERTY positions.
pub fn semitone_offset(key: Key) -> Option<i32> {
    Some(match key {
        Key::Z => 0,
        Key::S => 1,
        Key::X => 2,
        Key::D => 3,
        Key::C => 4,
        Key::V => 5,
        Key::G => 6,
        Key::B => 7,
        Key::H => 8,
        Key::N => 9,
        Key::J => 10,
        Key::M => 11,
        Key::Q => 12,
        Key::Num2 => 13,
        Key::W => 14,
        Key::Num3 => 15,
        Key::E => 16,
        Key::R => 17,
        Key::Num5 => 18,
        Key::T => 19,
        Key::Num6 => 20,
        Key::Y => 21,
        Key::Num7 => 22,
        Key::U => 23,
        Key::I => 24,
        _ => return None,
    })
}

pub fn midi_for_key(key: Key, octave_shift: i32) -> Option<u8> {
    let midi = C4_MIDI + semitone_offset(key)? + octave_shift.clamp(-2, 3) * 12;
    if (0..=127).contains(&midi) {
        Some(midi as u8)
    } else {
        None
    }
}

pub fn waveform_key(key: Key) -> Option<crate::wave::Waveform> {
    match key {
        Key::Num1 => Some(crate::wave::Waveform::Sine),
        Key::Num4 => Some(crate::wave::Waveform::Triangle),
        Key::Num8 => None,
        Key::Num9 => None,
        Key::Num0 => None,
        _ => None,
    }
    .or(match key {
        Key::Num1 => Some(crate::wave::Waveform::Sine),
        Key::Num2 => None,
        Key::Num3 => None,
        _ => None,
    })
}

/// Keys 1/8/9 would collide with black-key numbers. Waveforms use F1–F4 instead.
pub fn waveform_from_function_key(key: Key) -> Option<crate::wave::Waveform> {
    match key {
        Key::F1 => Some(crate::wave::Waveform::Sine),
        Key::F2 => Some(crate::wave::Waveform::Square),
        Key::F3 => Some(crate::wave::Waveform::Saw),
        Key::F4 => Some(crate::wave::Waveform::Triangle),
        _ => None,
    }
}

#[derive(Clone, Copy)]
pub struct PianoKey {
    pub key: Key,
    pub label: &'static str,
    pub black: bool,
}

pub const LOWER_ROW: [PianoKey; 12] = [
    PianoKey { key: Key::Z, label: "Z C", black: false },
    PianoKey { key: Key::S, label: "S C#", black: true },
    PianoKey { key: Key::X, label: "X D", black: false },
    PianoKey { key: Key::D, label: "D D#", black: true },
    PianoKey { key: Key::C, label: "C E", black: false },
    PianoKey { key: Key::V, label: "V F", black: false },
    PianoKey { key: Key::G, label: "G F#", black: true },
    PianoKey { key: Key::B, label: "B G", black: false },
    PianoKey { key: Key::H, label: "H G#", black: true },
    PianoKey { key: Key::N, label: "N A", black: false },
    PianoKey { key: Key::J, label: "J A#", black: true },
    PianoKey { key: Key::M, label: "M B", black: false },
];

pub const UPPER_ROW: [PianoKey; 13] = [
    PianoKey { key: Key::Q, label: "Q C", black: false },
    PianoKey { key: Key::Num2, label: "2 C#", black: true },
    PianoKey { key: Key::W, label: "W D", black: false },
    PianoKey { key: Key::Num3, label: "3 D#", black: true },
    PianoKey { key: Key::E, label: "E E", black: false },
    PianoKey { key: Key::R, label: "R F", black: false },
    PianoKey { key: Key::Num5, label: "5 F#", black: true },
    PianoKey { key: Key::T, label: "T G", black: false },
    PianoKey { key: Key::Num6, label: "6 G#", black: true },
    PianoKey { key: Key::Y, label: "Y A", black: false },
    PianoKey { key: Key::Num7, label: "7 A#", black: true },
    PianoKey { key: Key::U, label: "U B", black: false },
    PianoKey { key: Key::I, label: "I C", black: false },
];
