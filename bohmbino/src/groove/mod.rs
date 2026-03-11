//! Groove: Secondary kick voice for techno rumbles and kick tops.
//!
//! Contains four sound generators:
//! 1. Kick repetitions (taps from the main kick)
//! 2. Reverb tail
//! 3. Noise burst
//! 4. Gritty sub-frequency noise
//!
//! COLOR blends between these generators.
//! Taps 2/3/4 control individual tap volumes for creating volume envelopes.
//! LENGTH affects repetition tail. PITCH is relative to a base pitch.
//! FX: LP, HP, BP, or DIST selectable post-processing.

use fundsp::prelude::*;

/// Groove FX type.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum GrooveFx {
    /// Low-pass filter (FX controls cutoff)
    Lp = 0,
    /// High-pass filter (FX controls cutoff)
    Hp = 1,
    /// Band-pass filter (FX controls center frequency)
    Bp = 2,
    /// Distortion (FX controls gain)
    Dist = 3,
}

impl GrooveFx {
    pub const ALL: &[GrooveFx] = &[GrooveFx::Lp, GrooveFx::Hp, GrooveFx::Bp, GrooveFx::Dist];

    pub fn name(&self) -> &'static str {
        match self {
            GrooveFx::Lp => "LP",
            GrooveFx::Hp => "HP",
            GrooveFx::Bp => "BP",
            GrooveFx::Dist => "DIST",
        }
    }
}

#[derive(Clone)]
pub struct GrooveNode {
    // Parameters
    pitch: Shared,       // Base pitch Hz (relative to bohm)
    color: Shared,       // Blend between 4 generators: 0..1
    length: Shared,      // Repetition tail length
    fx_amount: Shared,   // FX wet amount 0..1
    fx_type: Shared,     // 0=LP, 1=HP, 2=BP, 3=DIST
    vol: Shared,         // Output volume
    tap2: Shared,        // Tap 2 volume 0..1
    tap3: Shared,        // Tap 3 volume 0..1
    tap4: Shared,        // Tap 4 volume 0..1
    trigger: Shared,     // 1.0 to trigger
    clock: Shared,       // Clock trigger for tap timing

    // Internal state
    phase: f32,
    env_time: f32,
    triggered: bool,
    // Tap timing
    clock_interval: f32, // Samples between clocks
    last_clock_time: f32,
    tap_index: u8,       // Which tap we're on (0-3)
    tap_time: f32,       // Time since current tap
    // Filter state
    lp_y: f32,
    hp_y: f32,
    hp_prev_x: f32,
    bp_y1: f32,
    bp_y2: f32,
    sample_rate: f32,
}

impl GrooveNode {
    pub fn new(
        pitch: Shared,
        color: Shared,
        length: Shared,
        fx_amount: Shared,
        fx_type: Shared,
        vol: Shared,
        tap2: Shared,
        tap3: Shared,
        tap4: Shared,
        trigger: Shared,
        clock: Shared,
    ) -> Self {
        Self {
            pitch, color, length, fx_amount, fx_type, vol,
            tap2, tap3, tap4, trigger, clock,
            phase: 0.0,
            env_time: 0.0,
            triggered: false,
            clock_interval: 12000.0, // ~250ms at 48kHz
            last_clock_time: 0.0,
            tap_index: 0,
            tap_time: 0.0,
            lp_y: 0.0,
            hp_y: 0.0,
            hp_prev_x: 0.0,
            bp_y1: 0.0,
            bp_y2: 0.0,
            sample_rate: 48000.0,
        }
    }
}

impl AudioNode for GrooveNode {
    const ID: u64 = 0xB00B_0301;
    type Inputs = U0;
    type Outputs = U1;

    fn tick(&mut self, _input: &Frame<f32, Self::Inputs>) -> Frame<f32, Self::Outputs> {
        let dt = 1.0 / self.sample_rate;

        // HIT retrigger
        if self.trigger.value() >= 0.5 {
            self.triggered = true;
            self.env_time = 0.0;
            self.phase = 0.0;
            self.tap_index = 0;
            self.tap_time = 0.0;
            self.trigger.set(0.0);
        }

        // Clock: measure interval and advance taps
        if self.clock.value() >= 0.5 {
            if self.triggered && self.tap_index < 4 {
                let elapsed = self.env_time - self.last_clock_time;
                if elapsed > 0.001 {
                    self.clock_interval = elapsed;
                }
                self.tap_index = if self.tap_index < 4 { self.tap_index + 1 } else { 4 };
                self.tap_time = 0.0;
            }
            self.last_clock_time = self.env_time;
            self.clock.set(0.0);
        }

        if !self.triggered {
            return [0.0].into();
        }

        let pitch = self.pitch.value();
        let color = self.color.value().clamp(0.0, 1.0);
        let length = self.length.value().max(0.01);
        let fx_amt = self.fx_amount.value().clamp(0.0, 1.0);
        let vol = self.vol.value().clamp(0.0, 1.0);
        let t = self.env_time;
        let tau = core::f32::consts::TAU;

        // Tap volume envelope
        let tap_vol = match self.tap_index {
            0 => 1.0, // Initial hit
            1 => self.tap2.value().clamp(0.0, 1.0),
            2 => self.tap3.value().clamp(0.0, 1.0),
            3 => self.tap4.value().clamp(0.0, 1.0),
            _ => 0.0,
        };

        // Decay envelope per tap
        let tap_env = (-self.tap_time / length.max(0.05)).exp();

        // Four generators, blended by COLOR
        // 0.0-0.25: kick repetition (sine with pitch envelope)
        // 0.25-0.5: reverb tail (filtered noise)
        // 0.5-0.75: noise burst
        // 0.75-1.0: gritty sub-frequency noise

        let freq = pitch * (1.0 + 3.0 * (-self.tap_time * 30.0).exp());

        // Generator 1: Kick repetition
        let kick_rep = (self.phase * tau).sin() * tap_env;

        // Generator 2: Reverb tail (filtered, longer decay)
        let reverb_decay = (-t / (length * 3.0)).exp();
        let noise_seed = ((t * self.sample_rate) as u32).wrapping_mul(1103515245).wrapping_add(12345);
        let noise = (noise_seed as f32 / u32::MAX as f32) * 2.0 - 1.0;
        let reverb = noise * reverb_decay * 0.3;

        // Generator 3: Noise burst
        let noise_env = (-self.tap_time * 20.0).exp();
        let noise_burst = noise * noise_env;

        // Generator 4: Gritty sub noise
        let sub_phase = self.phase * 0.5;
        let sub = (sub_phase * tau).sin();
        let grit_seed = ((t * self.sample_rate) as u32).wrapping_mul(1664525).wrapping_add(1013904223);
        let grit = (grit_seed as f32 / u32::MAX as f32) * 2.0 - 1.0;
        let sub_grit = (sub * 0.7 + grit * 0.3) * tap_env;

        // Blend by color
        let blended = if color < 0.25 {
            let m = color * 4.0;
            kick_rep * (1.0 - m) + (kick_rep * 0.5 + reverb * 0.5) * m
        } else if color < 0.5 {
            let m = (color - 0.25) * 4.0;
            reverb * (1.0 - m) + noise_burst * m
        } else if color < 0.75 {
            let m = (color - 0.5) * 4.0;
            noise_burst * (1.0 - m) + sub_grit * m
        } else {
            let m = (color - 0.75) * 4.0;
            sub_grit * (1.0 - m) + kick_rep * m
        };

        let dry = blended * tap_vol;

        // FX processing
        let fx_type_idx = self.fx_type.value() as usize;
        let processed = match fx_type_idx {
            0 => {
                // LP filter
                let cutoff = 200.0 + fx_amt * 10000.0;
                let rc = 1.0 / (tau * cutoff);
                let alpha = dt / (rc + dt);
                self.lp_y = self.lp_y + alpha * (dry - self.lp_y);
                self.lp_y
            }
            1 => {
                // HP filter
                let cutoff = 20.0 + fx_amt * 5000.0;
                let rc = 1.0 / (tau * cutoff);
                let alpha = rc / (rc + dt);
                self.hp_y = alpha * (self.hp_y + dry - self.hp_prev_x);
                self.hp_prev_x = dry;
                self.hp_y
            }
            2 => {
                // BP filter
                let center = 200.0 + fx_amt * 8000.0;
                let q = 2.0;
                let w0 = tau * center / self.sample_rate;
                let alpha = w0.sin() / (2.0 * q);
                let b0 = alpha;
                let a0 = 1.0 + alpha;
                let a1 = -2.0 * w0.cos();
                let a2 = 1.0 - alpha;
                let out = (b0 * dry - a1 * self.bp_y1 - a2 * self.bp_y2) / a0;
                self.bp_y2 = self.bp_y1;
                self.bp_y1 = out;
                out
            }
            _ => {
                // Distortion
                let gain = 1.0 + fx_amt * 12.0;
                (dry * gain).tanh()
            }
        };

        let sample = if fx_amt > 0.0 {
            dry * (1.0 - fx_amt) + processed * fx_amt
        } else {
            dry
        };

        // Kill after extended silence
        if t > length * 5.0 && tap_env < 0.001 {
            self.triggered = false;
        }

        self.phase += freq * dt;
        self.phase -= self.phase.floor();
        self.env_time += dt;
        self.tap_time += dt;

        [sample * vol].into()
    }

    fn reset(&mut self) {
        self.phase = 0.0;
        self.env_time = 0.0;
        self.triggered = false;
        self.tap_index = 0;
        self.tap_time = 0.0;
        self.lp_y = 0.0;
        self.hp_y = 0.0;
        self.hp_prev_x = 0.0;
        self.bp_y1 = 0.0;
        self.bp_y2 = 0.0;
    }

    fn set_sample_rate(&mut self, sr: f64) {
        self.sample_rate = sr as f32;
    }
}

/// Public Groove synthesizer wrapper.
pub struct Groove {
    graph: Option<Box<dyn AudioUnit>>,
    pitch: Shared,
    color: Shared,
    length: Shared,
    fx_amount: Shared,
    fx_type: Shared,
    vol: Shared,
    tap2: Shared,
    tap3: Shared,
    tap4: Shared,
    trigger: Shared,
    clock: Shared,
}

impl Groove {
    pub fn new() -> Self {
        let pitch = shared(55.0);
        let color = shared(0.0);
        let length = shared(0.3);
        let fx_amount = shared(0.0);
        let fx_type = shared(0.0);
        let vol = shared(0.5);
        let tap2 = shared(0.7);
        let tap3 = shared(0.4);
        let tap4 = shared(0.2);
        let trigger = shared(0.0);
        let clock = shared(0.0);

        let node = GrooveNode::new(
            pitch.clone(), color.clone(), length.clone(),
            fx_amount.clone(), fx_type.clone(), vol.clone(),
            tap2.clone(), tap3.clone(), tap4.clone(),
            trigger.clone(), clock.clone(),
        );

        Self {
            graph: Some(Box::new(An(node))),
            pitch, color, length, fx_amount, fx_type, vol,
            tap2, tap3, tap4, trigger, clock,
        }
    }

    pub fn hit(&self) { self.trigger.set(1.0); }
    pub fn clock_tick(&self) { self.clock.set(1.0); }

    pub fn set_pitch(&self, hz: f32) { self.pitch.set(hz); }
    pub fn set_color(&self, val: f32) { self.color.set(val); }
    pub fn set_length(&self, s: f32) { self.length.set(s); }
    pub fn set_fx_amount(&self, val: f32) { self.fx_amount.set(val); }
    pub fn set_fx_type(&self, fx: GrooveFx) { self.fx_type.set(fx as i32 as f32); }
    pub fn set_vol(&self, val: f32) { self.vol.set(val); }
    pub fn set_tap2(&self, val: f32) { self.tap2.set(val); }
    pub fn set_tap3(&self, val: f32) { self.tap3.set(val); }
    pub fn set_tap4(&self, val: f32) { self.tap4.set(val); }

    pub fn pitch(&self) -> Shared { self.pitch.clone() }
    pub fn color(&self) -> Shared { self.color.clone() }
    pub fn length(&self) -> Shared { self.length.clone() }
    pub fn fx_amount(&self) -> Shared { self.fx_amount.clone() }
    pub fn fx_type(&self) -> Shared { self.fx_type.clone() }
    pub fn vol(&self) -> Shared { self.vol.clone() }
    pub fn tap2(&self) -> Shared { self.tap2.clone() }
    pub fn tap3(&self) -> Shared { self.tap3.clone() }
    pub fn tap4(&self) -> Shared { self.tap4.clone() }
    pub fn trigger(&self) -> Shared { self.trigger.clone() }
    pub fn clock(&self) -> Shared { self.clock.clone() }

    pub fn take_graph(&mut self) -> Box<dyn AudioUnit> {
        self.graph.take().expect("graph already taken")
    }
}
