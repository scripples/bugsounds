//! FM-2X: 2-operator carrier/modulator FM kick with sub-bass oscillator.
//! Carrier uses wavetable-style waveshaping (COLOR: square→sine→triangle).
//! Modulator creates FM transients with OPL3-style waveforms.

use fundsp::prelude::*;
use super::common::*;

#[derive(Clone)]
pub struct Fm2xNode {
    pub params: KickParams,
    pub state: KickState,
    phase_carrier: f32,
    phase_mod: f32,
}

impl Fm2xNode {
    pub fn new(params: KickParams) -> Self {
        Self {
            params,
            state: KickState::new(),
            phase_carrier: 0.0,
            phase_mod: 0.0,
        }
    }
}

impl AudioNode for Fm2xNode {
    const ID: u64 = 0xB00B_0101;
    type Inputs = U0;
    type Outputs = U1;

    fn tick(&mut self, _input: &Frame<f32, Self::Inputs>) -> Frame<f32, Self::Outputs> {
        let dt = self.state.dt();

        if self.state.check_trigger(&self.params) {
            self.phase_carrier = 0.0;
            self.phase_mod = 0.0;
        }

        if !self.state.triggered {
            return [0.0].into();
        }

        let p = ParamSnapshot::read(&self.params);
        let t = self.state.env_time;

        let freq = pitch_envelope(t, p.pitch, p.curve);

        // Modulator: FM transient with OPL3 waveform selected by trs_tone
        let trs_decay_s = 0.01 + p.trs_decay * 0.09;
        let mod_env = (-t / trs_decay_s).exp();
        let mod_freq = freq * (1.0 + p.trs_tone * 7.0);
        let mod_index = p.attack * 8.0 * mod_env;
        let waveform = (p.trs_tone * 7.0) as u8;
        let mod_signal = opl3_waveform(self.phase_mod, waveform) * mod_index;

        // Carrier with FM and COLOR waveshaping (square→sine→triangle)
        let carrier_phase = self.phase_carrier * core::f32::consts::TAU + mod_signal;
        let sine_val = carrier_phase.sin();
        // COLOR: 0=square-ish, 0.5=sine, 1.0=triangle-ish
        let carrier = if p.color < 0.5 {
            let mix = p.color * 2.0;
            let sq = sine_val.signum();
            sq * (1.0 - mix) + sine_val * mix
        } else {
            let mix = (p.color - 0.5) * 2.0;
            let tri = (self.phase_carrier * 4.0 - 1.0).abs() * 2.0 - 1.0;
            sine_val * (1.0 - mix) + tri * mix
        };

        let amp_env = kick_envelope(t, p.length, p.sustain);
        if envelope_done(t, p.length) {
            self.state.triggered = false;
        }

        let sample = apply_fx(carrier * amp_env * p.velocity, p.fx_amount);

        self.phase_carrier += freq * dt;
        self.phase_carrier -= self.phase_carrier.floor();
        self.phase_mod += mod_freq * dt;
        self.phase_mod -= self.phase_mod.floor();
        self.state.advance();

        [sample].into()
    }

    fn reset(&mut self) {
        self.phase_carrier = 0.0;
        self.phase_mod = 0.0;
        self.state = KickState::new();
    }

    fn set_sample_rate(&mut self, sr: f64) {
        self.state.sample_rate = sr as f32;
    }
}
