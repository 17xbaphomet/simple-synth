use eframe::egui::Key;
use zweiton_engine::Waveform;

pub const C4_MIDI: i32 = 60;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Layout {
    #[default]
    Qwertz,
    Qwerty,
}

impl Layout {
    pub fn name(self) -> &'static str {
        match self {
            Self::Qwertz => "QWERTZ",
            Self::Qwerty => "QWERTY",
        }
    }

    pub fn typed_key(self, position: Key) -> Key {
        match (self, position) {
            (Self::Qwertz, Key::Z) => Key::Y,
            (Self::Qwertz, Key::Y) => Key::Z,
            _ => position,
        }
    }

    pub fn position_key(self, typed: Key) -> Key {
        self.typed_key(typed)
    }
}

pub fn semitone_offset(position: Key) -> Option<i32> {
    Some(match position {
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

pub fn midi_for_position(position: Key, octave_shift: i32) -> Option<u8> {
    let midi = C4_MIDI + semitone_offset(position)? + octave_shift.clamp(-2, 3) * 12;
    (0..=127).contains(&midi).then_some(midi as u8)
}

pub fn waveform_from_function_key(key: Key) -> Option<Waveform> {
    match key {
        Key::F1 => Some(Waveform::Sine),
        Key::F2 => Some(Waveform::Square),
        Key::F3 => Some(Waveform::Saw),
        Key::F4 => Some(Waveform::Triangle),
        _ => None,
    }
}

#[derive(Clone, Copy)]
pub struct PianoKey {
    pub key: Key,
    pub label_qwerty: &'static str,
    pub label_qwertz: &'static str,
    pub black: bool,
}

impl PianoKey {
    pub fn label(self, layout: Layout) -> &'static str {
        match layout {
            Layout::Qwerty => self.label_qwerty,
            Layout::Qwertz => self.label_qwertz,
        }
    }
}

pub const LOWER_ROW: [PianoKey; 12] = [
    PianoKey { key: Key::Z, label_qwerty: "Z C", label_qwertz: "Y C", black: false },
    PianoKey { key: Key::S, label_qwerty: "S C#", label_qwertz: "S C#", black: true },
    PianoKey { key: Key::X, label_qwerty: "X D", label_qwertz: "X D", black: false },
    PianoKey { key: Key::D, label_qwerty: "D D#", label_qwertz: "D D#", black: true },
    PianoKey { key: Key::C, label_qwerty: "C E", label_qwertz: "C E", black: false },
    PianoKey { key: Key::V, label_qwerty: "V F", label_qwertz: "V F", black: false },
    PianoKey { key: Key::G, label_qwerty: "G F#", label_qwertz: "G F#", black: true },
    PianoKey { key: Key::B, label_qwerty: "B G", label_qwertz: "B G", black: false },
    PianoKey { key: Key::H, label_qwerty: "H G#", label_qwertz: "H G#", black: true },
    PianoKey { key: Key::N, label_qwerty: "N A", label_qwertz: "N A", black: false },
    PianoKey { key: Key::J, label_qwerty: "J A#", label_qwertz: "J A#", black: true },
    PianoKey { key: Key::M, label_qwerty: "M B", label_qwertz: "M B", black: false },
];

pub const UPPER_ROW: [PianoKey; 13] = [
    PianoKey { key: Key::Q, label_qwerty: "Q C", label_qwertz: "Q C", black: false },
    PianoKey { key: Key::Num2, label_qwerty: "2 C#", label_qwertz: "2 C#", black: true },
    PianoKey { key: Key::W, label_qwerty: "W D", label_qwertz: "W D", black: false },
    PianoKey { key: Key::Num3, label_qwerty: "3 D#", label_qwertz: "3 D#", black: true },
    PianoKey { key: Key::E, label_qwerty: "E E", label_qwertz: "E E", black: false },
    PianoKey { key: Key::R, label_qwerty: "R F", label_qwertz: "R F", black: false },
    PianoKey { key: Key::Num5, label_qwerty: "5 F#", label_qwertz: "5 F#", black: true },
    PianoKey { key: Key::T, label_qwerty: "T G", label_qwertz: "T G", black: false },
    PianoKey { key: Key::Num6, label_qwerty: "6 G#", label_qwertz: "6 G#", black: true },
    PianoKey { key: Key::Y, label_qwerty: "Y A", label_qwertz: "Z A", black: false },
    PianoKey { key: Key::Num7, label_qwerty: "7 A#", label_qwertz: "7 A#", black: true },
    PianoKey { key: Key::U, label_qwerty: "U B", label_qwertz: "U B", black: false },
    PianoKey { key: Key::I, label_qwerty: "I C", label_qwertz: "I C", black: false },
];
