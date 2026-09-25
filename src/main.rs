mod audio;
mod error;
mod gui;
mod keys;
mod wav;

use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use anyhow::{Context, Result, bail};
use clap::{Parser, Subcommand};

use crate::error::HostError;
use zweiton_engine::{
    AdsrParams, Engine, SynthError, VoiceConfig, Waveform, frequency_from_note,
};

#[derive(Parser, Debug)]
#[command(
    name = "zweiton",
    version,
    about = "Zweiton: Dual-Osc Synthesizer + ADSR + GUI"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    Play(ToneArgs),
    Wav(WavArgs),
    Melody(MelodyArgs),
    Gui,
}

#[derive(clap::Args, Debug)]
struct ToneArgs {
    #[arg(long)]
    freq: Option<f32>,
    #[arg(long, conflicts_with = "freq")]
    note: Option<String>,
    #[arg(long, default_value = "sine")]
    wave: String,
    #[arg(long, default_value_t = 1.5)]
    duration: f32,
    #[arg(long, default_value_t = 0.25)]
    gain: f32,
    #[arg(long, default_value_t = 0.01)]
    attack: f32,
    #[arg(long, default_value_t = 0.10)]
    decay: f32,
    #[arg(long, default_value_t = 0.70)]
    sustain: f32,
    #[arg(long, default_value_t = 0.20)]
    release: f32,
}

#[derive(clap::Args, Debug)]
struct WavArgs {
    #[command(flatten)]
    tone: ToneArgs,
    #[arg(long, short, default_value = "tone.wav")]
    out: String,
}

#[derive(clap::Args, Debug)]
struct MelodyArgs {
    #[arg(long, default_value = "square")]
    wave: String,
    #[arg(long, default_value_t = 0.20)]
    gain: f32,
}

fn main() -> Result<()> {
    match Cli::parse().command {
        Command::Play(args) => play(args),
        Command::Wav(args) => export_wav(args),
        Command::Melody(args) => play_melody(args),
        Command::Gui => gui::run().context("GUI fehlgeschlagen"),
    }
}

fn play(args: ToneArgs) -> Result<()> {
    let sample_rate = output_sample_rate().unwrap_or(48_000.0);
    let config = voice_config(&args, sample_rate)?;
    let release = config.adsr.release_s;
    let engine = Arc::new(Mutex::new(Engine::new(config)?));
    audio::play_for(engine, args.duration, release).context("Realtime-Wiedergabe fehlgeschlagen")
}

fn export_wav(args: WavArgs) -> Result<()> {
    let config = voice_config(&args.tone, wav::wav_sample_rate())?;
    let release = config.adsr.release_s;
    let mut engine = Engine::new(config)?;
    wav::write_tone(&mut engine, &args.out, args.tone.duration, release)
        .with_context(|| format!("WAV konnte nicht nach {} geschrieben werden", args.out))?;
    println!("geschrieben: {}", args.out);
    Ok(())
}

fn play_melody(args: MelodyArgs) -> Result<()> {
    let sample_rate = output_sample_rate().unwrap_or(48_000.0);
    let waveform = Waveform::from_str(&args.wave)?;
    let notes = ["C4", "E4", "G4", "C5", "G4", "E4", "C4"];
    let config = VoiceConfig {
        sample_rate,
        frequency_hz: frequency_from_note(notes[0])?,
        waveform,
        gain: args.gain,
        adsr: AdsrParams {
            attack_s: 0.01,
            decay_s: 0.08,
            sustain: 0.65,
            release_s: 0.12,
        },
    };
    let release = config.adsr.release_s;
    let engine = Arc::new(Mutex::new(Engine::new(config)?));
    let _output = audio::AudioOutput::start(Arc::clone(&engine))?;
    for note in notes {
        let freq = frequency_from_note(note)?;
        {
            let mut synth = engine.lock().unwrap_or_else(|p| p.into_inner());
            synth.set_frequency(freq)?;
            synth.note_on();
        }
        std::thread::sleep(Duration::from_millis(280));
        {
            let mut synth = engine.lock().unwrap_or_else(|p| p.into_inner());
            synth.note_off();
        }
        std::thread::sleep(Duration::from_secs_f32(release + 0.04));
    }
    Ok(())
}

fn voice_config(args: &ToneArgs, sample_rate: f32) -> Result<VoiceConfig> {
    if args.duration <= 0.0 {
        bail!(HostError::Engine(SynthError::InvalidDuration(args.duration)));
    }
    let frequency_hz = match (&args.note, args.freq) {
        (Some(note), _) => frequency_from_note(note)?,
        (None, Some(freq)) => freq,
        (None, None) => 440.0,
    };
    Ok(VoiceConfig {
        sample_rate,
        frequency_hz,
        waveform: Waveform::from_str(&args.wave)?,
        gain: args.gain,
        adsr: AdsrParams {
            attack_s: args.attack,
            decay_s: args.decay,
            sustain: args.sustain,
            release_s: args.release,
        },
    })
}

pub(crate) fn output_sample_rate() -> Option<f32> {
    use cpal::traits::{DeviceTrait, HostTrait};
    cpal::default_host()
        .default_output_device()
        .and_then(|device| device.default_output_config().ok())
        .map(|config| {
            let stream: cpal::StreamConfig = config.into();
            stream.sample_rate as f32
        })
}
