use std::path::Path;

use zweiton_engine::Engine;

use crate::error::HostError;

const WAV_SAMPLE_RATE: u32 = 44_100;

pub fn write_tone(
    engine: &mut Engine,
    path: impl AsRef<Path>,
    gate_s: f32,
    release_s: f32,
) -> Result<(), HostError> {
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: WAV_SAMPLE_RATE,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    let gate_frames = seconds_to_frames(gate_s, WAV_SAMPLE_RATE);
    let tail_frames = seconds_to_frames(release_s + 0.05, WAV_SAMPLE_RATE);

    engine.note_on();
    let mut samples = engine.render(gate_frames);
    engine.note_off();
    samples.extend(engine.render(tail_frames));

    let mut writer = hound::WavWriter::create(path, spec)?;
    for sample in samples {
        writer.write_sample(to_i16(sample))?;
    }
    writer.finalize()?;
    Ok(())
}

fn seconds_to_frames(seconds: f32, sample_rate: u32) -> usize {
    (seconds.max(0.0) * sample_rate as f32).round() as usize
}

fn to_i16(sample: f32) -> i16 {
    let clamped = sample.clamp(-1.0, 1.0);
    (clamped * f32::from(i16::MAX)).round() as i16
}

pub const fn wav_sample_rate() -> f32 {
    WAV_SAMPLE_RATE as f32
}
