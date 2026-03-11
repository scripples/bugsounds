use fundsp::prelude::*;

/// Shared parameters common to all Bohm kick models.
#[derive(Clone)]
pub struct KickParams {
    pub pitch: Shared,      // Base pitch Hz (C1=32.70 to C2=65.41)
    pub curve: Shared,      // Pitch curve: 0.0=808, 1.0=909
    pub length: Shared,     // Kick duration in seconds
    pub sustain: Shared,    // Sustain level 0..1
    pub attack: Shared,     // Attack / FM mod amount 0..1
    pub velocity: Shared,   // Velocity 0..1
    pub color: Shared,      // Timbre control 0..1
    pub fx_amount: Shared,  // FX wet 0..1
    pub trs_decay: Shared,  // Transient decay 0..1 (10ms..100ms)
    pub trs_tone: Shared,   // Transient tone 0..1 (dark..bright)
    pub trigger: Shared,    // 1.0 to trigger
}

impl KickParams {
    pub fn new() -> Self {
        Self {
            pitch: shared(55.0),
            curve: shared(0.0),
            length: shared(0.3),
            sustain: shared(0.0),
            attack: shared(0.5),
            velocity: shared(1.0),
            color: shared(0.0),
            fx_amount: shared(0.0),
            trs_decay: shared(0.3),
            trs_tone: shared(0.3),
            trigger: shared(0.0),
        }
    }
}

/// Snapshot of parameter values for a single tick.
pub struct ParamSnapshot {
    pub pitch: f32,
    pub curve: f32,
    pub length: f32,
    pub sustain: f32,
    pub attack: f32,
    pub velocity: f32,
    pub color: f32,
    pub fx_amount: f32,
    pub trs_decay: f32,
    pub trs_tone: f32,
}

impl ParamSnapshot {
    pub fn read(p: &KickParams) -> Self {
        Self {
            pitch: p.pitch.value(),
            curve: p.curve.value().clamp(0.0, 1.0),
            length: p.length.value().max(0.01),
            sustain: p.sustain.value().clamp(0.0, 1.0),
            attack: p.attack.value().clamp(0.0, 1.0),
            velocity: p.velocity.value().clamp(0.0, 1.0),
            color: p.color.value().clamp(0.0, 1.0),
            fx_amount: p.fx_amount.value().clamp(0.0, 1.0),
            trs_decay: p.trs_decay.value().clamp(0.0, 1.0),
            trs_tone: p.trs_tone.value().clamp(0.0, 1.0),
        }
    }
}

/// Common kick state shared across all models.
#[derive(Clone)]
pub struct KickState {
    pub env_time: f32,
    pub triggered: bool,
    pub sample_rate: f32,
}

impl KickState {
    pub fn new() -> Self {
        Self {
            env_time: 0.0,
            triggered: false,
            sample_rate: 48000.0,
        }
    }

    pub fn dt(&self) -> f32 {
        1.0 / self.sample_rate
    }

    /// Check trigger and reset state if triggered.
    pub fn check_trigger(&mut self, params: &KickParams) -> bool {
        if params.trigger.value() >= 0.5 {
            self.triggered = true;
            self.env_time = 0.0;
            params.trigger.set(0.0);
            return true;
        }
        false
    }

    /// Advance time by one sample.
    pub fn advance(&mut self) {
        self.env_time += self.dt();
    }
}

/// ADSR envelope for one-shot kick (Attack-Decay-Sustain-Release).
pub fn kick_envelope(t: f32, length: f32, sustain_level: f32) -> f32 {
    let attack_time = 0.005;
    let decay_time = length * 0.3;
    let sustain_end = length * 0.8;
    let release_time = length * 0.2;

    if t < attack_time {
        t / attack_time
    } else if t < attack_time + decay_time {
        let d = (t - attack_time) / decay_time;
        1.0 - d * (1.0 - sustain_level)
    } else if t < sustain_end {
        sustain_level
    } else if t < sustain_end + release_time {
        let r = (t - sustain_end) / release_time;
        sustain_level * (1.0 - r)
    } else {
        0.0
    }
}

/// Returns true if the envelope is finished.
pub fn envelope_done(t: f32, length: f32) -> bool {
    t >= length
}

/// Pitch envelope: starts high, decays to base pitch.
pub fn pitch_envelope(t: f32, base_pitch: f32, curve: f32) -> f32 {
    let pitch_mult = 4.0; // Start 2 octaves above
    let pitch_env = if curve < 0.5 {
        // 808-style: exponential decay
        let rate = 20.0 + (1.0 - curve * 2.0) * 40.0;
        1.0 + (pitch_mult - 1.0) * (-t * rate).exp()
    } else {
        // 909-style: linear decay
        let decay_time = 0.02 + (curve - 0.5) * 2.0 * 0.08;
        let env = (1.0 - t / decay_time).max(0.0);
        1.0 + (pitch_mult - 1.0) * env
    };
    base_pitch * pitch_env
}

/// Soft-clip distortion (tanh-based).
pub fn soft_clip(x: f32, drive: f32) -> f32 {
    let gain = 1.0 + drive * 8.0;
    (x * gain).tanh()
}

/// Apply FX as dry/wet mix of soft clip.
pub fn apply_fx(dry: f32, fx_amount: f32) -> f32 {
    if fx_amount <= 0.0 {
        return dry;
    }
    let wet = soft_clip(dry, fx_amount);
    dry * (1.0 - fx_amount) + wet * fx_amount
}

/// OPL3-style waveforms (used by FM-2X and OLP4).
pub fn opl3_waveform(phase: f32, waveform: u8) -> f32 {
    let p = phase * core::f32::consts::TAU;
    match waveform {
        0 => p.sin(),                                                  // Sine
        1 => { let s = p.sin(); if s >= 0.0 { s } else { 0.0 } }      // Half sine
        2 => p.sin().abs(),                                            // Abs sine
        3 => { let q = phase % 0.5; (q * 2.0 * core::f32::consts::TAU).sin().max(0.0) } // Quarter sine
        4 => { let s = (p * 2.0).sin(); if phase % 1.0 < 0.5 { s } else { 0.0 } } // Alternating sine
        5 => (p * 2.0).sin().abs(),                                    // Camel sine
        6 => if p.sin() >= 0.0 { 1.0 } else { -1.0 },                 // Square
        7 => {                                                         // Derived square (saw-like)
            let m = phase % 1.0;
            if m < 0.5 { m * 4.0 - 1.0 } else { (1.0 - m) * 4.0 - 1.0 }
        }
        _ => p.sin(),
    }
}
