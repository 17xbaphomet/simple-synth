use crate::env::{AdsrParams, Envelope};
use crate::error::SynthError;
use crate::note::midi_to_hz;
use crate::osc::{OscInteract, Oscillator};
use crate::wave::Waveform;

const MAX_VOICES: usize = 8;

#[derive(Clone, Debug)]
pub struct VoiceConfig {
    pub sample_rate: f32,
    pub frequency_hz: f32,
    pub waveform: Waveform,
    pub gain: f32,
    pub adsr: AdsrParams,
}

impl VoiceConfig {
    pub fn validate(&self) -> Result<(), SynthError> {
        if self.frequency_hz <= 0.0 || !self.frequency_hz.is_finite() {
            return Err(SynthError::InvalidFrequency(self.frequency_hz));
        }
        Ok(())
    }
}

#[derive(Debug)]
struct Voice {
    osc_a: Oscillator,
    osc_b: Oscillator,
    env_a: Envelope,
    env_b: Envelope,
    note: Option<u8>,
    freq: f32,
}

impl Voice {
    fn new(sample_rate: f32, frequency_hz: f32, waveform: Waveform, adsr: AdsrParams) -> Self {
        Self {
            osc_a: Oscillator::new(sample_rate, frequency_hz, waveform),
            osc_b: Oscillator::new(sample_rate, frequency_hz, waveform),
            env_a: Envelope::new(sample_rate, adsr),
            env_b: Envelope::new(sample_rate, adsr),
            note: None,
            freq: frequency_hz,
        }
    }

    fn is_live(&self, separate: bool) -> bool {
        self.env_a.is_active() || (separate && self.env_b.is_active())
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum NoiseMode {
    #[default]
    Additive,
    Multiplicative,
    /// `out = sample * (val_prev * (1 + n))`. `val_prev` is a running gain, starts at 1.
    Walk,
    /// `out = sample * val_prev * (2 + n - val_prev)` = `sample * (2v + v*n - v²)`.
    Parabolic,
}

impl NoiseMode {
    pub const ALL: [Self; 4] = [Self::Additive, Self::Multiplicative, Self::Walk, Self::Parabolic];

    pub fn label(self) -> &'static str {
        match self {
            Self::Additive => "Additiv",
            Self::Multiplicative => "×(1±val)",
            Self::Walk => "s×(prev×(1±))",
            Self::Parabolic => "s×v(2+n−v)",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum NoiseDist {
    #[default]
    Uniform,
    Gauss,
}

impl NoiseDist {
    pub const ALL: [Self; 2] = [Self::Uniform, Self::Gauss];

    pub fn label(self) -> &'static str {
        match self {
            Self::Uniform => "Gleich",
            Self::Gauss => "Gauß",
        }
    }
}

#[derive(Debug)]
struct XorShift32(u32);

impl XorShift32 {
    fn new(seed: u32) -> Self {
        Self(seed | 1)
    }

    fn next_u32(&mut self) -> u32 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.0 = x;
        x
    }

    fn next_unit(&mut self) -> f32 {
        (self.next_u32() as f32) * (1.0 / 4_294_967_296.0)
    }

    fn next_bipolar(&mut self) -> f32 {
        self.next_unit().mul_add(2.0, -1.0)
    }

    /// Marsaglia polar method → N(0, 1). Rejection loop is ~1.27 iterations on average.
    fn next_gauss(&mut self) -> f32 {
        loop {
            let u = self.next_bipolar();
            let v = self.next_bipolar();
            let s = u * u + v * v;
            if s > 0.0 && s < 1.0 {
                return u * (-2.0 * s.ln() / s).sqrt();
            }
        }
    }

    fn next_dev(&mut self, dist: NoiseDist) -> f32 {
        match dist {
            NoiseDist::Uniform => self.next_bipolar(),
            NoiseDist::Gauss => self.next_gauss(),
        }
    }
}

#[derive(Debug)]
pub struct Engine {
    sample_rate: f32,
    wave_a: Waveform,
    wave_b: Waveform,
    adsr_a: AdsrParams,
    adsr_b: AdsrParams,
    separate_adsr: bool,
    gain: f32,
    freq: f32,
    mix: f32,
    ratio: f32,
    detune_cents: f32,
    depth: f32,
    noise: f32,
    noise_mode: NoiseMode,
    noise_dist: NoiseDist,
    mode: OscInteract,
    rng: XorShift32,
    walk_gain: f32,
    voices: Vec<Voice>,
}

impl Engine {
    pub fn new(config: VoiceConfig) -> Result<Self, SynthError> {
        config.validate()?;
        let sample_rate = config.sample_rate.max(1.0);
        let voices = (0..MAX_VOICES)
            .map(|_| Voice::new(sample_rate, config.frequency_hz, config.waveform, config.adsr))
            .collect();
        Ok(Self {
            sample_rate,
            wave_a: config.waveform,
            wave_b: config.waveform,
            adsr_a: config.adsr.sanitized(),
            adsr_b: config.adsr.sanitized(),
            separate_adsr: false,
            gain: config.gain.clamp(0.0, 1.0),
            freq: config.frequency_hz,
            mix: 0.0,
            ratio: 1.0,
            detune_cents: 0.0,
            depth: 0.5,
            noise: 0.0,
            noise_mode: NoiseMode::Additive,
            noise_dist: NoiseDist::Uniform,
            mode: OscInteract::Mix,
            rng: XorShift32::new(0xA5A5_C3C3),
            walk_gain: 1.0,
            voices,
        })
    }

    pub fn set_frequency(&mut self, frequency_hz: f32) -> Result<(), SynthError> {
        if frequency_hz <= 0.0 || !frequency_hz.is_finite() {
            return Err(SynthError::InvalidFrequency(frequency_hz));
        }
        self.freq = frequency_hz;
        let sample_rate = self.sample_rate;
        let ratio = self.ratio;
        let detune = self.detune_cents;
        let separate = self.separate_adsr;
        for voice in &mut self.voices {
            if voice.note.is_none() && voice.is_live(separate) {
                voice.freq = frequency_hz;
                apply_freqs(voice, sample_rate, ratio, detune);
            }
        }
        Ok(())
    }

    pub fn set_waveform(&mut self, waveform: Waveform) {
        self.wave_a = waveform;
        for voice in &mut self.voices {
            voice.osc_a.set_waveform(waveform);
        }
    }

    pub fn set_waveform_b(&mut self, waveform: Waveform) {
        self.wave_b = waveform;
        for voice in &mut self.voices {
            voice.osc_b.set_waveform(waveform);
        }
    }

    pub fn set_gain(&mut self, gain: f32) {
        self.gain = gain.clamp(0.0, 1.0);
    }

    pub fn set_adsr(&mut self, adsr: AdsrParams) {
        self.adsr_a = adsr.sanitized();
    }

    pub fn set_adsr_b(&mut self, adsr: AdsrParams) {
        self.adsr_b = adsr.sanitized();
    }

    pub fn set_separate_adsr(&mut self, enabled: bool) {
        self.separate_adsr = enabled;
    }

    pub fn set_noise(&mut self, noise: f32) {
        self.noise = noise.clamp(0.0, 0.5);
    }

    pub fn set_noise_mode(&mut self, mode: NoiseMode) {
        self.noise_mode = mode;
    }

    pub fn set_noise_dist(&mut self, dist: NoiseDist) {
        self.noise_dist = dist;
    }

    pub fn set_osc_mix(&mut self, mix: f32) {
        self.mix = mix.clamp(0.0, 1.0);
    }

    pub fn set_osc_ratio(&mut self, ratio: f32) {
        self.ratio = ratio.clamp(0.25, 8.0);
    }

    pub fn set_osc_detune(&mut self, cents: f32) {
        self.detune_cents = cents.clamp(-100.0, 100.0);
    }

    pub fn set_osc_depth(&mut self, depth: f32) {
        self.depth = depth.clamp(0.0, 2.0);
    }

    pub fn set_osc_mode(&mut self, mode: OscInteract) {
        self.mode = mode;
    }

    pub fn note_on(&mut self) {
        self.start_voice(None, self.freq);
    }

    pub fn note_off(&mut self) {
        for voice in &mut self.voices {
            if voice.note.is_none() && voice.is_live(self.separate_adsr) {
                voice.env_a.note_off();
                voice.env_b.note_off();
            }
        }
    }

    pub fn note_on_midi(&mut self, midi: u8) {
        let freq = midi_to_hz(f32::from(midi));
        self.freq = freq;
        self.start_voice(Some(midi), freq);
    }

    pub fn note_off_midi(&mut self, midi: u8) {
        for voice in &mut self.voices {
            if voice.note == Some(midi) {
                voice.env_a.note_off();
                voice.env_b.note_off();
            }
        }
    }

    pub fn all_notes_off(&mut self) {
        for voice in &mut self.voices {
            voice.env_a.reset();
            voice.env_b.reset();
            voice.note = None;
        }
        self.walk_gain = 1.0;
    }

    pub fn is_active(&self) -> bool {
        let separate = self.separate_adsr;
        self.voices.iter().any(|voice| voice.is_live(separate))
    }

    pub fn next_sample(&mut self) -> f32 {
        let mix_ab = self.mix;
        let depth = self.depth;
        let mode = self.mode;
        let sample_rate = self.sample_rate;
        let ratio = self.ratio;
        let detune = self.detune_cents;
        let separate = self.separate_adsr;
        let mix: f32 = self
            .voices
            .iter_mut()
            .map(|voice| {
                if !voice.is_live(separate) {
                    voice.note = None;
                    return 0.0;
                }
                let freq_b = freq_b_of(voice.freq, ratio, detune);
                voice.osc_b.set_frequency(sample_rate, freq_b);
                let env_a = voice.env_a.tick();
                let env_b = voice.env_b.tick();
                tick_pair(voice, sample_rate, mode, mix_ab, depth, env_a, env_b, separate)
            })
            .sum();
        let dry = mix * self.gain;
        let mut sample = dry;
        if self.noise > 0.0 {
            let n = self.rng.next_dev(self.noise_dist) * self.noise;
            sample = match self.noise_mode {
                NoiseMode::Additive => dry + n,
                NoiseMode::Multiplicative => dry * (1.0 + n),
                NoiseMode::Walk => apply_walk(self.is_active(), &mut self.walk_gain, dry, n),
                NoiseMode::Parabolic => apply_parabolic(self.is_active(), &mut self.walk_gain, dry, n),
            };
        }
        sample.clamp(-1.0, 1.0)
    }

    pub fn render(&mut self, frames: usize) -> Vec<f32> {
        (0..frames).map(|_| self.next_sample()).collect()
    }

    fn start_voice(&mut self, note: Option<u8>, freq: f32) {
        let index = self.allocate(note);
        let sample_rate = self.sample_rate;
        let ratio = self.ratio;
        let detune = self.detune_cents;
        let wave_a = self.wave_a;
        let wave_b = self.wave_b;
        let adsr_a = self.adsr_a;
        let adsr_b = self.adsr_b;
        let voice = &mut self.voices[index];
        voice.freq = freq;
        voice.osc_a.set_waveform(wave_a);
        voice.osc_b.set_waveform(wave_b);
        voice.osc_a.reset_phase();
        voice.osc_b.reset_phase();
        apply_freqs(voice, sample_rate, ratio, detune);
        voice.env_a = Envelope::new(sample_rate, adsr_a);
        voice.env_b = Envelope::new(sample_rate, adsr_b);
        voice.env_a.note_on();
        voice.env_b.note_on();
        voice.note = note;
    }

    fn allocate(&self, note: Option<u8>) -> usize {
        if let Some(note) = note {
            if let Some(index) = self.voices.iter().position(|voice| voice.note == Some(note)) {
                return index;
            }
        }
        let separate = self.separate_adsr;
        if let Some(index) = self
            .voices
            .iter()
            .position(|voice| !voice.is_live(separate))
        {
            return index;
        }
        0
    }
}

fn apply_freqs(voice: &mut Voice, sample_rate: f32, ratio: f32, detune_cents: f32) {
    voice.osc_a.set_frequency(sample_rate, voice.freq);
    voice
        .osc_b
        .set_frequency(sample_rate, freq_b_of(voice.freq, ratio, detune_cents));
}

fn freq_b_of(freq_a: f32, ratio: f32, detune_cents: f32) -> f32 {
    freq_a * ratio.clamp(0.25, 8.0) * 2_f32.powf(detune_cents / 1200.0)
}

fn apply_walk(active: bool, walk_gain: &mut f32, dry: f32, n: f32) -> f32 {
    if !active {
        *walk_gain = 1.0;
        return 0.0;
    }
    let factor = *walk_gain * (1.0 + n);
    *walk_gain = factor.clamp(0.05, 8.0);
    dry * factor
}

fn apply_parabolic(active: bool, walk_gain: &mut f32, dry: f32, n: f32) -> f32 {
    if !active {
        *walk_gain = 1.0;
        return 0.0;
    }
    let v = *walk_gain;
    let factor = v * (2.0 + n - v);
    *walk_gain = factor.clamp(0.05, 8.0);
    dry * factor
}

fn tick_pair(
    voice: &mut Voice,
    sample_rate: f32,
    mode: OscInteract,
    mix: f32,
    depth: f32,
    env_a: f32,
    env_b: f32,
    separate: bool,
) -> f32 {
    match mode {
        OscInteract::Mix => {
            let a = voice.osc_a.tick();
            let b = voice.osc_b.tick();
            if separate {
                a.mul_add(env_a * (1.0 - mix), b * env_b * mix)
            } else {
                a.mul_add(1.0 - mix, b * mix) * env_a
            }
        }
        OscInteract::Am => {
            let a = voice.osc_a.tick();
            let b = voice.osc_b.tick();
            let shaped = if separate {
                (a * env_a) * (1.0 + depth * b * env_b)
            } else {
                a * (1.0 + depth * b) * env_a
            };
            shaped.clamp(-1.0, 1.0)
        }
        OscInteract::Ring => {
            let a = voice.osc_a.tick();
            let b = voice.osc_b.tick();
            if separate {
                a * env_a * b * env_b
            } else {
                a * b * env_a
            }
        }
        OscInteract::Fm => {
            let b = voice.osc_b.tick();
            let mod_level = if separate { b * env_b } else { b };
            let freq = (voice.freq * (1.0 + depth * 4.0 * mod_level)).max(0.0);
            voice.osc_a.set_frequency(sample_rate, freq);
            voice.osc_a.tick() * env_a
        }
        OscInteract::Sync => {
            let (a, wrapped) = voice.osc_a.tick_wrap();
            if wrapped {
                voice.osc_b.reset_phase();
            }
            let b = voice.osc_b.tick();
            if separate {
                a.mul_add(env_a * (1.0 - mix), b * env_b * mix)
            } else {
                a.mul_add(1.0 - mix, b * mix) * env_a
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{apply_parabolic, NoiseDist, NoiseMode};

    #[test]
    fn noise_mode_labels() {
        assert_eq!(NoiseMode::ALL.len(), 4);
        assert_eq!(NoiseMode::Additive.label(), "Additiv");
        assert_eq!(NoiseMode::Multiplicative.label(), "×(1±val)");
        assert_eq!(NoiseMode::Walk.label(), "s×(prev×(1±))");
        assert_eq!(NoiseMode::Parabolic.label(), "s×v(2+n−v)");
        assert_eq!(NoiseMode::default(), NoiseMode::Additive);
    }

    #[test]
    fn noise_dist_labels() {
        assert_eq!(NoiseDist::ALL.len(), 2);
        assert_eq!(NoiseDist::Uniform.label(), "Gleich");
        assert_eq!(NoiseDist::Gauss.label(), "Gauß");
        assert_eq!(NoiseDist::default(), NoiseDist::Uniform);
    }

    #[test]
    fn parabolic_mean_reverts_to_one() {
        let mut v = 0.5;
        let out = apply_parabolic(true, &mut v, 1.0, 0.0);
        assert!((out - 0.75).abs() < 1e-6);
        assert!((v - 0.75).abs() < 1e-6);
        let out = apply_parabolic(true, &mut v, 1.0, 0.0);
        assert!((out - 0.9375).abs() < 1e-6);
        let mut v = 1.0;
        let out = apply_parabolic(true, &mut v, 0.4, 0.0);
        assert!((out - 0.4).abs() < 1e-6);
        assert!((v - 1.0).abs() < 1e-6);
    }

    #[test]
    fn parabolic_resets_when_silent() {
        let mut v = 3.0;
        let out = apply_parabolic(false, &mut v, 0.5, 0.2);
        assert_eq!(out, 0.0);
        assert_eq!(v, 1.0);
    }
}
