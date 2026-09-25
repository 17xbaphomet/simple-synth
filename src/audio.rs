use std::sync::{Arc, Mutex};
use std::time::Duration;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{FromSample, Sample, SizedSample};
use zweiton_engine::Engine;

use crate::error::HostError;

pub struct AudioOutput {
    _stream: cpal::Stream,
}

impl AudioOutput {
    pub fn start(engine: Arc<Mutex<Engine>>) -> Result<Self, HostError> {
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .ok_or(HostError::NoOutputDevice)?;
        println!("Ausgabegerät: {device}");

        let supported = device.default_output_config()?;
        let sample_format = supported.sample_format();
        let config = supported.config();

        let stream = match sample_format {
            cpal::SampleFormat::F32 => build_stream::<f32>(&device, config, engine)?,
            cpal::SampleFormat::F64 => build_stream::<f64>(&device, config, engine)?,
            cpal::SampleFormat::I16 => build_stream::<i16>(&device, config, engine)?,
            cpal::SampleFormat::I32 => build_stream::<i32>(&device, config, engine)?,
            cpal::SampleFormat::U16 => build_stream::<u16>(&device, config, engine)?,
            other => return Err(HostError::UnsupportedSampleFormat(other)),
        };

        stream.play()?;
        Ok(Self { _stream: stream })
    }
}

fn build_stream<T>(
    device: &cpal::Device,
    config: cpal::StreamConfig,
    engine: Arc<Mutex<Engine>>,
) -> Result<cpal::Stream, HostError>
where
    T: SizedSample + Sample + FromSample<f32>,
{
    let channels = config.channels as usize;
    let err_fn = |err: cpal::Error| match err.kind() {
        cpal::ErrorKind::DeviceChanged | cpal::ErrorKind::Xrun => eprintln!("{err}"),
        _ => eprintln!("Audio-Stream-Fehler: {err}"),
    };

    Ok(device.build_output_stream(
        config,
        move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
            write_frames(data, channels, &engine);
        },
        err_fn,
        None,
    )?)
}

fn write_frames<T>(output: &mut [T], channels: usize, engine: &Arc<Mutex<Engine>>)
where
    T: Sample + FromSample<f32>,
{
    for frame in output.chunks_mut(channels) {
        let sample = engine
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .next_sample();
        let value = T::from_sample(sample);
        for slot in frame.iter_mut() {
            *slot = value;
        }
    }
}

pub fn play_for(engine: Arc<Mutex<Engine>>, gate_s: f32, release_s: f32) -> Result<(), HostError> {
    let _output = AudioOutput::start(Arc::clone(&engine))?;
    {
        let mut synth = engine
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        synth.note_on();
    }
    std::thread::sleep(Duration::from_secs_f32(gate_s.max(0.01)));
    {
        let mut synth = engine
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        synth.note_off();
    }
    std::thread::sleep(Duration::from_secs_f32(release_s.max(0.02) + 0.05));
    Ok(())
}
