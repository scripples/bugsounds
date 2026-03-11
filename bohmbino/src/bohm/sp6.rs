//! SP-6: Wavetable oscillator kick with "digital-sounding" wavetables
//! combined with drum layering samples and synthesized FM transients
//! (hihats and snares).

use fundsp::prelude::*;
use super::common::*;

#[derive(Clone)]
pub struct Sp6Node {
    pub params: KickParams,
    pub state: KickState,
    phase_osc: f32,
    // FM transient operators for hihat/snare synthesis
    phase_t1: f32,
    phase_t2: f32,
}

impl Sp6Node {
    pub fn new(params: KickParams) -> Self {
        Self {
            params,
            state: KickState::new(),
            phase_osc: 0.0,
            phase_t1: 0.0,
            phase_t2: 0.0,
        }
    }
}

impl AudioNode for Sp6Node {
    const ID: u64 = 0xB00B_0106;
    type Inputs = U0;
    type Outputs = U1;

    fn tick(&mut self, _input: &Frame<f32, Self::Inputs>) -> Frame<f32, Self::Outputs> {
        let dt = self.state.dt();

        if self.state.check_trigger(&self.params) {
            self.phase_osc = 0.0;
            self.phase_t1 = 0.0;
            self.phase_t2 = 0.0;
        }

        if !self.state.triggered {
            return [0.0].into();
        }

        let p = ParamSnapshot::read(&self.params);
        let t = self.state.env_time;
        let freq = pitch_envelope(t, p.pitch, p.curve);
        let tau = core::f32::consts::TAU;

        // Digital wavetable osc: quantized harmonics for digital character
        let sine = (self.phase_osc * tau).sin();
        let color_env = (-t * 15.0).exp();
        let effective_color = p.color * color_env;
        // Digital character: bit-reduce the sine slightly
        let quantize_levels = 16.0 + (1.0 - effective_color) * 240.0; // 16..256 levels
        let digital = (sine * quantize_levels).round() / quantize_levels;

        // FM transient: synthesized hihat/snare character
        // Two inharmonic operators create metallic transients
        let trs_decay_s = 0.01 + p.trs_decay * 0.09;
        let trs_env = (-t / trs_decay_s).exp();
        // Inharmonic ratios for metallic sound
        let t1_freq = freq * 7.13;
        let t2_freq = freq * 11.37;
        let t1 = (self.phase_t1 * tau).sin();
        let t2 = (self.phase_t2 * tau).sin();
        // trs_tone blends from snare-like (noise) to hihat-like (metallic)
        let noise_seed = ((t * 48000.0) as u32).wrapping_mul(1103515245).wrapping_add(12345);
        let noise = (noise_seed as f32 / u32::MAX as f32) * 2.0 - 1.0;
        let metallic = t1 * t2; // Ring mod for metallic character
        let transient = (noise * (1.0 - p.trs_tone) + metallic * p.trs_tone)
            * trs_env
            * p.attack
            * 0.5;

        let amp_env = kick_envelope(t, p.length, p.sustain);
        if envelope_done(t, p.length) {
            self.state.triggered = false;
        }

        let sample = apply_fx(
            (digital + transient) * amp_env * p.velocity,
            p.fx_amount,
        );

        self.phase_osc += freq * dt;
        self.phase_osc -= self.phase_osc.floor();
        self.phase_t1 += t1_freq * dt;
        self.phase_t1 -= self.phase_t1.floor();
        self.phase_t2 += t2_freq * dt;
        self.phase_t2 -= self.phase_t2.floor();
        self.state.advance();

        [sample].into()
    }

    fn reset(&mut self) {
        self.phase_osc = 0.0;
        self.phase_t1 = 0.0;
        self.phase_t2 = 0.0;
        self.state = KickState::new();
    }

    fn set_sample_rate(&mut self, sr: f64) {
        self.state.sample_rate = sr as f32;
    }
}
