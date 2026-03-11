//! HZ-1: Wavetable oscillator kick combined with a transient synthesizer.
//! COLOR adjusts high-frequency content via harmonic waveshaping.
//! At minimum = pure sine, increasing adds distortion during attack.
//! Transient synth provides click/pop/tick/toc variations.

use fundsp::prelude::*;
use super::common::*;

#[derive(Clone)]
pub struct Hz1Node {
    pub params: KickParams,
    pub state: KickState,
    phase_osc: f32,
    phase_click: f32,
}

impl Hz1Node {
    pub fn new(params: KickParams) -> Self {
        Self {
            params,
            state: KickState::new(),
            phase_osc: 0.0,
            phase_click: 0.0,
        }
    }
}

impl AudioNode for Hz1Node {
    const ID: u64 = 0xB00B_0102;
    type Inputs = U0;
    type Outputs = U1;

    fn tick(&mut self, _input: &Frame<f32, Self::Inputs>) -> Frame<f32, Self::Outputs> {
        let dt = self.state.dt();

        if self.state.check_trigger(&self.params) {
            self.phase_osc = 0.0;
            self.phase_click = 0.0;
        }

        if !self.state.triggered {
            return [0.0].into();
        }

        let p = ParamSnapshot::read(&self.params);
        let t = self.state.env_time;
        let freq = pitch_envelope(t, p.pitch, p.curve);

        // Main oscillator: sine with harmonic distortion controlled by COLOR
        let sine = (self.phase_osc * core::f32::consts::TAU).sin();
        // COLOR adds overtones during attack, fading to sine in tail
        let color_env = (-t * 15.0).exp(); // Color effect strongest at onset
        let effective_color = p.color * color_env;
        let harmonics = (sine * (1.0 + effective_color * 6.0)).tanh();
        let osc = sine * (1.0 - effective_color) + harmonics * effective_color;

        // Transient synthesizer: short noise burst shaped by trs_tone
        let trs_decay_s = 0.01 + p.trs_decay * 0.09;
        let trs_env = (-t / trs_decay_s).exp();
        // trs_tone: 0=dark click (low freq), 1=bright tick (high freq)
        let click_freq = freq * (4.0 + p.trs_tone * 20.0);
        let click = (self.phase_click * core::f32::consts::TAU).sin();
        // Add some noise for texture
        let noise = ((t * 48000.0) as u32).wrapping_mul(1103515245).wrapping_add(12345);
        let noise_val = (noise as f32 / u32::MAX as f32) * 2.0 - 1.0;
        let transient = (click * 0.7 + noise_val * 0.3 * p.trs_tone) * trs_env * p.attack;

        let amp_env = kick_envelope(t, p.length, p.sustain);
        if envelope_done(t, p.length) {
            self.state.triggered = false;
        }

        let sample = apply_fx((osc + transient) * amp_env * p.velocity, p.fx_amount);

        self.phase_osc += freq * dt;
        self.phase_osc -= self.phase_osc.floor();
        self.phase_click += click_freq * dt;
        self.phase_click -= self.phase_click.floor();
        self.state.advance();

        [sample].into()
    }

    fn reset(&mut self) {
        self.phase_osc = 0.0;
        self.phase_click = 0.0;
        self.state = KickState::new();
    }

    fn set_sample_rate(&mut self, sr: f64) {
        self.state.sample_rate = sr as f32;
    }
}
