use nice_plug::midi::Key;
use nice_plug::prelude::*;
use std::sync::Arc;
use zweiton_engine::{
    AdsrParams, Engine, NoiseDist, NoiseMode, OscInteract, VoiceConfig, Waveform,
};

pub struct Zweiton {
    params: Arc<ZweitonParams>,
    engine: Engine,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Enum)]
enum WaveChoice {
    #[default]
    #[name = "sine"]
    Sine,
    #[name = "square"]
    Square,
    #[name = "saw"]
    Saw,
    #[name = "triangle"]
    Triangle,
}

impl From<WaveChoice> for Waveform {
    fn from(value: WaveChoice) -> Self {
        match value {
            WaveChoice::Sine => Waveform::Sine,
            WaveChoice::Square => Waveform::Square,
            WaveChoice::Saw => Waveform::Saw,
            WaveChoice::Triangle => Waveform::Triangle,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Enum)]
enum ModeChoice {
    #[default]
    Mix,
    AM,
    Ring,
    FM,
    Sync,
}

impl From<ModeChoice> for OscInteract {
    fn from(value: ModeChoice) -> Self {
        match value {
            ModeChoice::Mix => OscInteract::Mix,
            ModeChoice::AM => OscInteract::Am,
            ModeChoice::Ring => OscInteract::Ring,
            ModeChoice::FM => OscInteract::Fm,
            ModeChoice::Sync => OscInteract::Sync,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Enum)]
enum NoiseModeChoice {
    #[default]
    #[name = "Additiv"]
    Additive,
    #[name = "×(1±val)"]
    Multiplicative,
    #[name = "s×(prev×(1±))"]
    Walk,
    #[name = "s×v(2+n−v)"]
    Parabolic,
}

impl From<NoiseModeChoice> for NoiseMode {
    fn from(value: NoiseModeChoice) -> Self {
        match value {
            NoiseModeChoice::Additive => NoiseMode::Additive,
            NoiseModeChoice::Multiplicative => NoiseMode::Multiplicative,
            NoiseModeChoice::Walk => NoiseMode::Walk,
            NoiseModeChoice::Parabolic => NoiseMode::Parabolic,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Enum)]
enum NoiseDistChoice {
    #[default]
    #[name = "Gleich"]
    Uniform,
    #[name = "Gauß"]
    Gauss,
}

impl From<NoiseDistChoice> for NoiseDist {
    fn from(value: NoiseDistChoice) -> Self {
        match value {
            NoiseDistChoice::Uniform => NoiseDist::Uniform,
            NoiseDistChoice::Gauss => NoiseDist::Gauss,
        }
    }
}

#[derive(Params)]
struct ZweitonParams {
    #[id = "gain"]
    pub gain: FloatParam,
    #[id = "wave_a"]
    pub wave_a: EnumParam<WaveChoice>,
    #[id = "wave_b"]
    pub wave_b: EnumParam<WaveChoice>,
    #[id = "mode"]
    pub mode: EnumParam<ModeChoice>,
    #[id = "mix"]
    pub mix: FloatParam,
    #[id = "ratio"]
    pub ratio: FloatParam,
    #[id = "detune"]
    pub detune: FloatParam,
    #[id = "depth"]
    pub depth: FloatParam,
    #[id = "sep_adsr"]
    pub separate_adsr: BoolParam,
    #[id = "atk_a"]
    pub attack_a: FloatParam,
    #[id = "dec_a"]
    pub decay_a: FloatParam,
    #[id = "sus_a"]
    pub sustain_a: FloatParam,
    #[id = "rel_a"]
    pub release_a: FloatParam,
    #[id = "atk_b"]
    pub attack_b: FloatParam,
    #[id = "dec_b"]
    pub decay_b: FloatParam,
    #[id = "sus_b"]
    pub sustain_b: FloatParam,
    #[id = "rel_b"]
    pub release_b: FloatParam,
    #[id = "noise"]
    pub noise: FloatParam,
    #[id = "nmode"]
    pub noise_mode: EnumParam<NoiseModeChoice>,
    #[id = "ndist"]
    pub noise_dist: EnumParam<NoiseDistChoice>,
}

impl Default for ZweitonParams {
    fn default() -> Self {
        Self {
            gain: FloatParam::new("Gain", 0.25, FloatRange::Linear { min: 0.0, max: 1.0 })
                .with_smoother(SmoothingStyle::Linear(20.0)),
            wave_a: EnumParam::new("Welle A", WaveChoice::Sine),
            wave_b: EnumParam::new("Welle B", WaveChoice::Sine),
            mode: EnumParam::new("Modus", ModeChoice::Mix),
            mix: FloatParam::new("Mix", 0.0, FloatRange::Linear { min: 0.0, max: 1.0 })
                .with_smoother(SmoothingStyle::Linear(20.0)),
            ratio: FloatParam::new(
                "Ratio",
                1.0,
                FloatRange::Skewed {
                    min: 0.25,
                    max: 8.0,
                    factor: FloatRange::skew_factor(-1.0),
                },
            ),
            detune: FloatParam::new(
                "Detune",
                0.0,
                FloatRange::Linear {
                    min: -100.0,
                    max: 100.0,
                },
            )
            .with_unit(" ct"),
            depth: FloatParam::new("Tiefe", 0.5, FloatRange::Linear { min: 0.0, max: 2.0 }),
            separate_adsr: BoolParam::new("Getrennte ADSR", false),
            attack_a: FloatParam::new(
                "A Attack",
                0.01,
                FloatRange::Skewed {
                    min: 0.001,
                    max: 2.0,
                    factor: FloatRange::skew_factor(-1.5),
                },
            )
            .with_unit(" s"),
            decay_a: FloatParam::new(
                "A Decay",
                0.10,
                FloatRange::Skewed {
                    min: 0.001,
                    max: 2.0,
                    factor: FloatRange::skew_factor(-1.5),
                },
            )
            .with_unit(" s"),
            sustain_a: FloatParam::new("A Sustain", 0.70, FloatRange::Linear { min: 0.0, max: 1.0 }),
            release_a: FloatParam::new(
                "A Release",
                0.20,
                FloatRange::Skewed {
                    min: 0.001,
                    max: 3.0,
                    factor: FloatRange::skew_factor(-1.5),
                },
            )
            .with_unit(" s"),
            attack_b: FloatParam::new(
                "B Attack",
                0.01,
                FloatRange::Skewed {
                    min: 0.001,
                    max: 2.0,
                    factor: FloatRange::skew_factor(-1.5),
                },
            )
            .with_unit(" s"),
            decay_b: FloatParam::new(
                "B Decay",
                0.10,
                FloatRange::Skewed {
                    min: 0.001,
                    max: 2.0,
                    factor: FloatRange::skew_factor(-1.5),
                },
            )
            .with_unit(" s"),
            sustain_b: FloatParam::new("B Sustain", 0.70, FloatRange::Linear { min: 0.0, max: 1.0 }),
            release_b: FloatParam::new(
                "B Release",
                0.20,
                FloatRange::Skewed {
                    min: 0.001,
                    max: 3.0,
                    factor: FloatRange::skew_factor(-1.5),
                },
            )
            .with_unit(" s"),
            noise: FloatParam::new("Zufall",
                0.0,
                FloatRange::Linear { min: 0.0, max: 0.5 },
            ),
            noise_mode: EnumParam::new("Rauschmodus", NoiseModeChoice::Additive),
            noise_dist: EnumParam::new("Verteilung", NoiseDistChoice::Uniform),
        }
    }
}

fn default_engine() -> Engine {
    Engine::new(VoiceConfig {
        sample_rate: 48_000.0,
        frequency_hz: 440.0,
        waveform: Waveform::Sine,
        gain: 0.25,
        adsr: AdsrParams::default(),
    })
    .expect("default VoiceConfig is valid")
}

impl Default for Zweiton {
    fn default() -> Self {
        Self {
            params: Arc::new(ZweitonParams::default()),
            engine: default_engine(),
        }
    }
}

impl Zweiton {
    fn sync_params(&mut self) {
        let p = &*self.params;
        self.engine.set_gain(p.gain.smoothed.next());
        self.engine.set_waveform(p.wave_a.value().into());
        self.engine.set_waveform_b(p.wave_b.value().into());
        self.engine.set_osc_mode(p.mode.value().into());
        self.engine.set_osc_mix(p.mix.smoothed.next());
        self.engine.set_osc_ratio(p.ratio.value());
        self.engine.set_osc_detune(p.detune.value());
        self.engine.set_osc_depth(p.depth.value());
        self.engine.set_separate_adsr(p.separate_adsr.value());
        self.engine.set_adsr(AdsrParams {
            attack_s: p.attack_a.value(),
            decay_s: p.decay_a.value(),
            sustain: p.sustain_a.value(),
            release_s: p.release_a.value(),
        });
        self.engine.set_adsr_b(AdsrParams {
            attack_s: p.attack_b.value(),
            decay_s: p.decay_b.value(),
            sustain: p.sustain_b.value(),
            release_s: p.release_b.value(),
        });
        self.engine.set_noise(p.noise.value());
        self.engine.set_noise_mode(p.noise_mode.value().into());
        self.engine.set_noise_dist(p.noise_dist.value().into());
    }
}

impl Plugin for Zweiton {
    const NAME: &'static str = "Zweiton";
    const VENDOR: &'static str = "17xbaphomet";
    const URL: &'static str = "https://github.com/17xbaphomet/simple-synth";
    const EMAIL: &'static str = "zweiton@example.com";
    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[
        AudioIOLayout {
            main_input_channels: None,
            main_output_channels: NonZeroU32::new(2),
            ..AudioIOLayout::const_default()
        },
        AudioIOLayout {
            main_input_channels: None,
            main_output_channels: NonZeroU32::new(1),
            ..AudioIOLayout::const_default()
        },
    ];

    const MIDI_INPUT: MidiConfig = MidiConfig::Basic;
    const SAMPLE_ACCURATE_AUTOMATION: bool = true;

    type Editor = ();
    type SysExMessage = ();
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn activate(
        &mut self,
        _audio_io_layout: &AudioIOLayout,
        buffer_config: &BufferConfig,
        _context: &mut impl ActivateContext<Self>,
    ) -> bool {
        self.engine.set_sample_rate(buffer_config.sample_rate);
        true
    }

    fn reset(&mut self) {
        self.engine.all_notes_off();
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        let mut next_event = context.next_event();
        for (sample_id, channel_samples) in buffer.iter_samples().enumerate() {
            while let Some(event) = next_event {
                if event.timing() > sample_id as u32 {
                    break;
                }
                match event {
                    NoteEvent::NoteOn { key, .. } => {
                        self.engine.note_on_midi(key.number_or_middle_c());
                    }
                    NoteEvent::NoteOff { key, .. } => {
                        self.engine.note_off_midi(key.number_or_middle_c());
                    }
                    NoteEvent::Choke { key, .. } => {
                        if key.is_wildcard() {
                            self.engine.all_notes_off();
                        } else {
                            self.engine.note_off_midi(key.number_or_middle_c());
                        }
                    }
                    _ => {}
                }
                next_event = context.next_event();
            }

            self.sync_params();
            let sample = self.engine.next_sample();
            for slot in channel_samples {
                *slot = sample;
            }
        }

        if self.engine.is_active() {
            ProcessStatus::Normal
        } else {
            ProcessStatus::KeepAlive
        }
    }
}

impl ClapPlugin for Zweiton {
    const CLAP_ID: &'static str = "com.17xbaphomet.zweiton";
    const CLAP_DESCRIPTION: Option<&'static str> =
        Some("Dual-Oscillator mit Mix/AM/Ring/FM/Sync und Rauschmodi");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;
    const CLAP_FEATURES: &'static [ClapFeature] = &[
        ClapFeature::Instrument,
        ClapFeature::Synthesizer,
        ClapFeature::Stereo,
        ClapFeature::Mono,
    ];
}

impl Vst3Plugin for Zweiton {
    const VST3_CLASS_ID: [u8; 16] = *b"ZweitonSynth0001";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Instrument, Vst3SubCategory::Synth];
}

nice_export_clap!(Zweiton);
nice_export_vst3!(Zweiton);
