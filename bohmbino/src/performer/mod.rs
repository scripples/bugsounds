//! Performer: Audio effects processor with ducking.
//!
//! Receives the mixed Bohm + Groove signal and optional external audio input.
//! Applies ducking on HIT trigger, then routes through selectable effects:
//! - DJ Filter (LP↔HP sweep)
//! - HP (high-pass)
//! - LP (low-pass)
//! - Beat Roll (repeating buffer)
//! - Slip Roll (resampling beat roll)
//!
//! Channel selector routes ALL, KICK only, or INPUT only to effects.

use fundsp::prelude::*;

/// Performer FX type.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum PerformerFx {
    /// DJ Filter: LP (CCW) → neutral (center) → HP (CW)
    DjFilter = 0,
    /// High-pass filter
    Hp = 1,
    /// Low-pass filter
    Lp = 2,
    /// Beat-synchronized roll
    BeatRoll = 3,
    /// Beat roll with resampling on each HIT
    SlipRoll = 4,
}

impl PerformerFx {
    pub const ALL: &[PerformerFx] = &[
        PerformerFx::DjFilter,
        PerformerFx::Hp,
        PerformerFx::Lp,
        PerformerFx::BeatRoll,
        PerformerFx::SlipRoll,
    ];

    pub fn name(&self) -> &'static str {
        match self {
            PerformerFx::DjFilter => "DJ FILTER",
            PerformerFx::Hp => "HP",
            PerformerFx::Lp => "LP",
            PerformerFx::BeatRoll => "BEAT ROLL",
            PerformerFx::SlipRoll => "SLIP ROLL",
        }
    }
}

/// Channel routing.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Channel {
    /// Both kick and external audio
    All = 0,
    /// Kick only
    Kick = 1,
    /// External input only
    Input = 2,
}

const ROLL_BUFFER_SIZE: usize = 48000; // 1 second at 48kHz

/// Performer AudioNode.
/// Inputs: 0 = kick signal, 1 = external input (optional, pass 0 if unused)
/// Output: 0 = processed mono
#[derive(Clone)]
pub struct PerformerNode {
    duck: Shared,       // Ducking amount 0..1
    fx_amount: Shared,  // FX parameter 0..1
    fx_type: Shared,    // PerformerFx index
    vol: Shared,        // Output volume
    channel: Shared,    // Channel routing (0=ALL, 1=KICK, 2=INPUT)
    reso: Shared,       // DJ filter resonance 0..1
    fx_on: Shared,      // FX bypass 0/1
    trigger: Shared,    // HIT trigger for ducking

    // Ducking state
    duck_env: f32,

    // Filter state
    lp_y: f32,
    hp_y: f32,
    hp_prev_x: f32,

    // Roll buffer
    roll_buf: Vec<f32>,
    roll_write: usize,
    roll_read: f32,
    roll_recording: bool,

    sample_rate: f32,
}

impl PerformerNode {
    pub fn new(
        duck: Shared,
        fx_amount: Shared,
        fx_type: Shared,
        vol: Shared,
        channel: Shared,
        reso: Shared,
        fx_on: Shared,
        trigger: Shared,
    ) -> Self {
        Self {
            duck, fx_amount, fx_type, vol, channel, reso, fx_on, trigger,
            duck_env: 0.0,
            lp_y: 0.0,
            hp_y: 0.0,
            hp_prev_x: 0.0,
            roll_buf: vec![0.0; ROLL_BUFFER_SIZE],
            roll_write: 0,
            roll_read: 0.0,
            roll_recording: true,
            sample_rate: 48000.0,
        }
    }
}

impl AudioNode for PerformerNode {
    const ID: u64 = 0xB00B_0401;
    type Inputs = U2; // kick + external input
    type Outputs = U1;

    fn tick(&mut self, input: &Frame<f32, Self::Inputs>) -> Frame<f32, Self::Outputs> {
        let kick = input[0];
        let ext = input[1];
        let tau = core::f32::consts::TAU;
        let dt = 1.0 / self.sample_rate;

        // Ducking on trigger
        if self.trigger.value() >= 0.5 {
            self.duck_env = 1.0;
            self.trigger.set(0.0);
            // Reset roll on hit for slip roll
            if self.fx_type.value() as usize == 4 {
                self.roll_write = 0;
                self.roll_recording = true;
            }
        }

        let duck_amount = self.duck.value().clamp(0.0, 1.0);
        // Duck envelope: fast attack, slow release
        self.duck_env *= (-dt * 5.0f32).exp();
        let duck_gain = 1.0 - self.duck_env * duck_amount;

        // Channel routing
        let channel_idx = self.channel.value() as usize;
        let signal = match channel_idx {
            1 => kick,
            2 => ext * duck_gain,
            _ => kick + ext * duck_gain, // ALL
        };

        let fx_on = self.fx_on.value() >= 0.5;
        if !fx_on {
            return [signal * self.vol.value()].into();
        }

        let fx_amt = self.fx_amount.value().clamp(0.0, 1.0);
        let fx_idx = self.fx_type.value() as usize;
        let reso = self.reso.value().clamp(0.0, 1.0);

        let processed = match fx_idx {
            0 => {
                // DJ Filter: fx_amt 0=LP, 0.5=neutral, 1=HP
                if (fx_amt - 0.5).abs() < 0.05 {
                    signal // Neutral zone
                } else if fx_amt < 0.5 {
                    // LP: cutoff from 200Hz (0) to full (0.5)
                    let norm = fx_amt * 2.0;
                    let cutoff = 200.0 + norm * 15000.0;
                    let q = 0.707 + reso * 8.0;
                    let w0 = tau * cutoff / self.sample_rate;
                    let alpha = w0.sin() / (2.0 * q);
                    let rc = 1.0 / (tau * cutoff);
                    let a = dt / (rc + dt);
                    self.lp_y = self.lp_y + a * (signal - self.lp_y);
                    // Add resonance feedback
                    self.lp_y + (self.lp_y - signal) * reso * alpha
                } else {
                    // HP: cutoff from 0 (0.5) to 10kHz (1)
                    let norm = (fx_amt - 0.5) * 2.0;
                    let cutoff = 20.0 + norm * 10000.0;
                    let rc = 1.0 / (tau * cutoff);
                    let a = rc / (rc + dt);
                    self.hp_y = a * (self.hp_y + signal - self.hp_prev_x);
                    self.hp_prev_x = signal;
                    self.hp_y
                }
            }
            1 => {
                // HP
                let cutoff = 20.0 + fx_amt * 10000.0;
                let rc = 1.0 / (tau * cutoff);
                let a = rc / (rc + dt);
                self.hp_y = a * (self.hp_y + signal - self.hp_prev_x);
                self.hp_prev_x = signal;
                self.hp_y
            }
            2 => {
                // LP
                let cutoff = 200.0 + (1.0 - fx_amt) * 15000.0;
                let rc = 1.0 / (tau * cutoff);
                let a = dt / (rc + dt);
                self.lp_y = self.lp_y + a * (signal - self.lp_y);
                self.lp_y
            }
            3 | 4 => {
                // Beat Roll / Slip Roll
                // Record into buffer
                if self.roll_recording && self.roll_write < ROLL_BUFFER_SIZE {
                    self.roll_buf[self.roll_write] = signal;
                    self.roll_write += 1;
                }

                // Roll length from fx_amount
                let roll_len = (fx_amt * ROLL_BUFFER_SIZE as f32).max(100.0) as usize;
                let write_max = if self.roll_write > 1 { self.roll_write } else { 1 };
                let roll_len = if roll_len < write_max { roll_len } else { write_max };

                // Read from buffer with wrapping
                let idx = self.roll_read as usize % roll_len;
                let out = self.roll_buf[idx];
                self.roll_read += 1.0;
                if self.roll_read as usize >= roll_len {
                    self.roll_read = 0.0;
                }

                // Mix original and rolled
                signal * (1.0 - fx_amt) + out * fx_amt
            }
            _ => signal,
        };

        [processed * self.vol.value()].into()
    }

    fn reset(&mut self) {
        self.duck_env = 0.0;
        self.lp_y = 0.0;
        self.hp_y = 0.0;
        self.hp_prev_x = 0.0;
        self.roll_buf.fill(0.0);
        self.roll_write = 0;
        self.roll_read = 0.0;
        self.roll_recording = true;
    }

    fn set_sample_rate(&mut self, sr: f64) {
        self.sample_rate = sr as f32;
    }
}

/// Public Performer wrapper.
pub struct Performer {
    graph: Option<Box<dyn AudioUnit>>,
    duck: Shared,
    fx_amount: Shared,
    fx_type: Shared,
    vol: Shared,
    channel: Shared,
    reso: Shared,
    fx_on: Shared,
    trigger: Shared,
}

impl Performer {
    pub fn new() -> Self {
        let duck = shared(0.5);
        let fx_amount = shared(0.5);
        let fx_type = shared(0.0);
        let vol = shared(0.8);
        let channel = shared(0.0);
        let reso = shared(0.0);
        let fx_on = shared(1.0);
        let trigger = shared(0.0);

        let node = PerformerNode::new(
            duck.clone(), fx_amount.clone(), fx_type.clone(),
            vol.clone(), channel.clone(), reso.clone(),
            fx_on.clone(), trigger.clone(),
        );

        Self {
            graph: Some(Box::new(An(node))),
            duck, fx_amount, fx_type, vol, channel, reso, fx_on, trigger,
        }
    }

    pub fn hit(&self) { self.trigger.set(1.0); }

    pub fn set_duck(&self, val: f32) { self.duck.set(val); }
    pub fn set_fx_amount(&self, val: f32) { self.fx_amount.set(val); }
    pub fn set_fx_type(&self, fx: PerformerFx) { self.fx_type.set(fx as i32 as f32); }
    pub fn set_vol(&self, val: f32) { self.vol.set(val); }
    pub fn set_channel(&self, ch: Channel) { self.channel.set(ch as i32 as f32); }
    pub fn set_reso(&self, val: f32) { self.reso.set(val); }
    pub fn set_fx_on(&self, on: bool) { self.fx_on.set(if on { 1.0 } else { 0.0 }); }

    pub fn duck(&self) -> Shared { self.duck.clone() }
    pub fn fx_amount(&self) -> Shared { self.fx_amount.clone() }
    pub fn fx_type(&self) -> Shared { self.fx_type.clone() }
    pub fn vol(&self) -> Shared { self.vol.clone() }
    pub fn channel(&self) -> Shared { self.channel.clone() }
    pub fn reso(&self) -> Shared { self.reso.clone() }
    pub fn fx_on(&self) -> Shared { self.fx_on.clone() }
    pub fn trigger(&self) -> Shared { self.trigger.clone() }

    pub fn take_graph(&mut self) -> Box<dyn AudioUnit> {
        self.graph.take().expect("graph already taken")
    }
}
