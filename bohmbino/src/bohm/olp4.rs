//! OLP4: 4-operator FM kick inspired by the OPL3 chip.
//! Uses non-interpolated OPL3 waveforms.
//! Six algorithm configurations.
//! TRS TONE controls FM feedback from op3→op1.
//! COLOR is inactive. Most experimental model.

use fundsp::prelude::*;
use super::common::*;

/// Algorithm configurations for 4 operators.
#[derive(Clone, Copy)]
enum Algorithm {
    /// 1→2 (serial pair)
    A12,
    /// 1//2 (parallel pair)
    A1P2,
    /// 1→2→3→4 (full serial)
    A1234,
    /// 1→2 // 3→4 (two serial pairs in parallel)
    A12P34,
    /// 1 // 2→3→4 (one solo + serial triple)
    A1P234,
    /// 1 // 2→3 // 4 (one solo + serial pair + one solo)
    A1P23P4,
}

impl Algorithm {
    fn from_index(i: usize) -> Self {
        match i % 6 {
            0 => Algorithm::A12,
            1 => Algorithm::A1P2,
            2 => Algorithm::A1234,
            3 => Algorithm::A12P34,
            4 => Algorithm::A1P234,
            _ => Algorithm::A1P23P4,
        }
    }
}

#[derive(Clone)]
pub struct Olp4Node {
    pub params: KickParams,
    pub state: KickState,
    phase: [f32; 4],
}

impl Olp4Node {
    pub fn new(params: KickParams) -> Self {
        Self {
            params,
            state: KickState::new(),
            phase: [0.0; 4],
        }
    }
}

impl AudioNode for Olp4Node {
    const ID: u64 = 0xB00B_0103;
    type Inputs = U0;
    type Outputs = U1;

    fn tick(&mut self, _input: &Frame<f32, Self::Inputs>) -> Frame<f32, Self::Outputs> {
        let dt = self.state.dt();

        if self.state.check_trigger(&self.params) {
            self.phase = [0.0; 4];
        }

        if !self.state.triggered {
            return [0.0].into();
        }

        let p = ParamSnapshot::read(&self.params);
        let t = self.state.env_time;
        let freq = pitch_envelope(t, p.pitch, p.curve);

        // Algorithm selected by attack knob position (6 algorithms)
        let algo = Algorithm::from_index((p.attack * 5.99) as usize);

        // Waveforms: trs_decay selects wf for ops 1,3; color(repurposed) for ops 2,4
        // Since COLOR is inactive per spec, use fixed sine for 2,4
        let wf13 = (p.trs_decay * 7.0) as u8;
        let wf24: u8 = 0; // sine

        // Operator frequencies: ratios based on operator index
        let freqs = [freq, freq * 2.0, freq * 3.0, freq * 4.0];

        // FM feedback: trs_tone controls op3→op1 feedback
        let feedback = p.trs_tone * 4.0;

        // Modulation envelope: all operators decay
        let mod_env = (-t * 20.0).exp();

        // Compute operators
        let op3 = opl3_waveform(self.phase[2], wf13) * mod_env;
        let fb_signal = op3 * feedback;

        let op1 = opl3_waveform(self.phase[0] + fb_signal * 0.1, wf13) * mod_env;
        let op2 = opl3_waveform(self.phase[1], wf24) * mod_env;
        let op4 = opl3_waveform(self.phase[3], wf24) * mod_env;

        // Mix based on algorithm
        let out = match algo {
            Algorithm::A12 => {
                // 1 modulates 2
                opl3_waveform(self.phase[1] + op1 * 0.3, wf24) * mod_env
            }
            Algorithm::A1P2 => {
                // 1 and 2 in parallel
                (op1 + op2) * 0.5
            }
            Algorithm::A1234 => {
                // 1→2→3→4 serial
                let m2 = opl3_waveform(self.phase[1] + op1 * 0.3, wf24) * mod_env;
                let m3 = opl3_waveform(self.phase[2] + m2 * 0.3, wf13) * mod_env;
                opl3_waveform(self.phase[3] + m3 * 0.3, wf24) * mod_env
            }
            Algorithm::A12P34 => {
                // (1→2) // (3→4)
                let m12 = opl3_waveform(self.phase[1] + op1 * 0.3, wf24) * mod_env;
                let m34 = opl3_waveform(self.phase[3] + op3 * 0.3, wf24) * mod_env;
                (m12 + m34) * 0.5
            }
            Algorithm::A1P234 => {
                // 1 // (2→3→4)
                let m23 = opl3_waveform(self.phase[2] + op2 * 0.3, wf13) * mod_env;
                let m234 = opl3_waveform(self.phase[3] + m23 * 0.3, wf24) * mod_env;
                (op1 + m234) * 0.5
            }
            Algorithm::A1P23P4 => {
                // 1 // (2→3) // 4
                let m23 = opl3_waveform(self.phase[2] + op2 * 0.3, wf13) * mod_env;
                (op1 + m23 + op4) / 3.0
            }
        };

        let amp_env = kick_envelope(t, p.length, p.sustain);
        if envelope_done(t, p.length) {
            self.state.triggered = false;
        }

        let sample = apply_fx(out * amp_env * p.velocity, p.fx_amount);

        for i in 0..4 {
            self.phase[i] += freqs[i] * dt;
            self.phase[i] -= self.phase[i].floor();
        }
        self.state.advance();

        [sample].into()
    }

    fn reset(&mut self) {
        self.phase = [0.0; 4];
        self.state = KickState::new();
    }

    fn set_sample_rate(&mut self, sr: f64) {
        self.state.sample_rate = sr as f32;
    }
}
