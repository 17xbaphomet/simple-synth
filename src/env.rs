#[derive(Clone, Copy, Debug)]
pub struct AdsrParams {
    pub attack_s: f32,
    pub decay_s: f32,
    pub sustain: f32,
    pub release_s: f32,
}

impl Default for AdsrParams {
    fn default() -> Self {
        Self {
            attack_s: 0.01,
            decay_s: 0.10,
            sustain: 0.70,
            release_s: 0.20,
        }
    }
}

impl AdsrParams {
    pub fn sanitized(self) -> Self {
        Self {
            attack_s: self.attack_s.max(0.0),
            decay_s: self.decay_s.max(0.0),
            sustain: self.sustain.clamp(0.0, 1.0),
            release_s: self.release_s.max(0.0),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum Stage {
    #[default]
    Idle,
    Attack,
    Decay,
    Sustain,
    Release,
}

#[derive(Clone, Debug)]
pub struct Envelope {
    params: AdsrParams,
    sample_rate: f32,
    stage: Stage,
    level: f32,
}

impl Envelope {
    pub fn new(sample_rate: f32, params: AdsrParams) -> Self {
        Self {
            params: params.sanitized(),
            sample_rate: sample_rate.max(1.0),
            stage: Stage::Idle,
            level: 0.0,
        }
    }

    pub fn note_on(&mut self) {
        self.stage = Stage::Attack;
    }

    pub fn note_off(&mut self) {
        self.stage = match self.stage {
            Stage::Idle => Stage::Idle,
            _ => Stage::Release,
        };
    }

    pub fn is_active(&self) -> bool {
        self.stage != Stage::Idle
    }

    pub fn tick(&mut self) -> f32 {
        match self.stage {
            Stage::Idle => {
                self.level = 0.0;
            }
            Stage::Attack => {
                self.level = advance(self.level, 1.0, self.params.attack_s, self.sample_rate);
                if self.level >= 1.0 {
                    self.level = 1.0;
                    self.stage = Stage::Decay;
                }
            }
            Stage::Decay => {
                self.level = advance(
                    self.level,
                    self.params.sustain,
                    self.params.decay_s,
                    self.sample_rate,
                );
                if (self.level - self.params.sustain).abs() <= f32::EPSILON {
                    self.level = self.params.sustain;
                    self.stage = Stage::Sustain;
                }
            }
            Stage::Sustain => {
                self.level = self.params.sustain;
            }
            Stage::Release => {
                self.level = advance(self.level, 0.0, self.params.release_s, self.sample_rate);
                if self.level <= f32::EPSILON {
                    self.level = 0.0;
                    self.stage = Stage::Idle;
                }
            }
        }
        self.level
    }
}

fn advance(current: f32, target: f32, time_s: f32, sample_rate: f32) -> f32 {
    if time_s <= f32::EPSILON {
        return target;
    }
    let delta = (target - current).signum() / (time_s * sample_rate);
    let next = current + delta;
    if (target - current).signum() != (target - next).signum() {
        target
    } else {
        next
    }
}
