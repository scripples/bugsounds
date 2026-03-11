//! VX-T: Wavetable oscillator kick with analog-style wavetables
//! combined with a 4-operator FM transient synthesizer.
//! TRS DECAY: toc (CCW) to hihat (CW), fed through bandpass filter.
//! TRS TONE: bandpass center frequency.

use fundsp::prelude::*;
use super::common::*;

#[derive(Clone)]
pub struct VxtNode {
    pub params: KickParams,
    pub state: KickState,
    phase_osc: f32,
    // 4-operator FM transient
    phase_ops: [f32; 4],
    // Bandpass filter state (2-pole)
    bp_y1: f32,
    bp_y2: f32,
}

impl VxtNode {
    pub fn new(params: KickParams) -> Self {
        Self {
            params,
            state: KickState::new(),
            phase_osc: 0.0,
            phase_ops: [0.0; 4],
            bp_y1: 0.0,
            bp_y2: 0.0,
        }
    }
}

impl AudioNode for VxtNode {
    const ID: u64 = 0xB00B_0107;
    type Inputs = U0;
    type Outputs = U1;

    fn tick(&mut self, _input: &Frame<f32, Self::Inputs>) -> Frame<f32, Self::Outputs> {
        let dt = self.state.dt();

        if self.state.check_trigger(&self.params) {
            self.phase_osc = 0.0;
            self.phase_ops = [0.0; 4];
            self.bp_y1 = 0.0;
            self.bp_y2 = 0.0;
        }

        if !self.state.triggered {
            return [0.0].into();
        }

        let p = ParamSnapshot::read(&self.params);
        let t = self.state.env_time;
        let freq = pitch_envelope(t, p.pitch, p.curve);
        let tau = core::f32::consts::TAU;

        // Analog-style wavetable osc: smooth waveshaping
        let sine = (self.phase_osc * tau).sin();
        let color_env = (-t * 15.0).exp();
        let effective_color = p.color * color_env;
        let osc = (sine * (1.0 + effective_color * 4.0)).tanh();

        // 4-operator FM transient synthesizer
        let trs_decay_s = 0.01 + p.trs_decay * 0.09;
        let trs_env = (-t / trs_decay_s).exp();

        // Operator frequencies: inharmonic ratios for metallic character
        // trs_decay controls toc (low ratios) vs hihat (high ratios)
        let ratio_base = 1.0 + p.trs_decay * 8.0;
        let op_freqs = [
            freq * ratio_base,
            freq * ratio_base * 1.47,
            freq * ratio_base * 2.09,
            freq * ratio_base * 2.83,
        ];

        // Simple serial FM: 1→2→3→4
        let op1 = (self.phase_ops[0] * tau).sin();
        let op2 = (self.phase_ops[1] * tau + op1 * 0.5).sin();
        let op3 = (self.phase_ops[2] * tau + op2 * 0.3).sin();
        let op4 = (self.phase_ops[3] * tau + op3 * 0.2).sin();
        let fm_raw = op4 * trs_env * p.attack;

        // Bandpass filter: TRS TONE controls center frequency
        // Simple 2-pole bandpass approximation
        let center = 500.0 + p.trs_tone * 8000.0; // 500Hz to 8500Hz
        let q = 2.0;
        let w0 = tau * center / self.state.sample_rate;
        let alpha = w0.sin() / (2.0 * q);
        let b0 = alpha;
        let a0 = 1.0 + alpha;
        let a1 = -2.0 * w0.cos();
        let a2 = 1.0 - alpha;

        let bp_out = (b0 * fm_raw - a1 * self.bp_y1 - a2 * self.bp_y2) / a0;
        self.bp_y2 = self.bp_y1;
        self.bp_y1 = bp_out;

        let transient = bp_out;

        let amp_env = kick_envelope(t, p.length, p.sustain);
        if envelope_done(t, p.length) {
            self.state.triggered = false;
        }

        let sample = apply_fx(
            (osc + transient) * amp_env * p.velocity,
            p.fx_amount,
        );

        self.phase_osc += freq * dt;
        self.phase_osc -= self.phase_osc.floor();
        for i in 0..4 {
            self.phase_ops[i] += op_freqs[i] * dt;
            self.phase_ops[i] -= self.phase_ops[i].floor();
        }
        self.state.advance();

        [sample].into()
    }

    fn reset(&mut self) {
        self.phase_osc = 0.0;
        self.phase_ops = [0.0; 4];
        self.bp_y1 = 0.0;
        self.bp_y2 = 0.0;
        self.state = KickState::new();
    }

    fn set_sample_rate(&mut self, sr: f64) {
        self.state.sample_rate = sr as f32;
    }
}
