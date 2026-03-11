//! PX3: Wavetable oscillator kick with "weird" wavetables + drum layering.
//! Produces harder and more experimental character.
//! COLOR modulates wavetable position over time.
//! Approximated here with aggressive waveshaping and harmonic stacking.

use fundsp::prelude::*;
use super::common::*;

#[derive(Clone)]
pub struct Px3Node {
    pub params: KickParams,
    pub state: KickState,
    phase_osc: f32,
    phase_layer: f32,
}

impl Px3Node {
    pub fn new(params: KickParams) -> Self {
        Self {
            params,
            state: KickState::new(),
            phase_osc: 0.0,
            phase_layer: 0.0,
        }
    }
}

impl AudioNode for Px3Node {
    const ID: u64 = 0xB00B_0105;
    type Inputs = U0;
    type Outputs = U1;

    fn tick(&mut self, _input: &Frame<f32, Self::Inputs>) -> Frame<f32, Self::Outputs> {
        let dt = self.state.dt();

        if self.state.check_trigger(&self.params) {
            self.phase_osc = 0.0;
            self.phase_layer = 0.0;
        }

        if !self.state.triggered {
            return [0.0].into();
        }

        let p = ParamSnapshot::read(&self.params);
        let t = self.state.env_time;
        let freq = pitch_envelope(t, p.pitch, p.curve);

        // "Weird" wavetable approximation: sum of detuned, waveshaped harmonics
        // COLOR modulates the waveshaping intensity over time
        let color_sweep = p.color * (1.0 + (-t * 8.0).exp()); // Sweep at onset
        let tau = core::f32::consts::TAU;

        let h1 = (self.phase_osc * tau).sin();
        let h2 = (self.phase_osc * 2.003 * tau).sin() * 0.5; // Slightly detuned
        let h3 = (self.phase_osc * 3.01 * tau).sin() * 0.3;
        let raw = h1 + h2 * color_sweep + h3 * color_sweep * 0.5;

        // Aggressive waveshaping for "weird" character
        let fold = ((raw * (1.0 + color_sweep * 3.0)).sin()) * 0.7 + raw * 0.3;

        // Drum layer: synthesized impact (objects hitting surfaces)
        let layer_env = (-t * 30.0).exp();
        let layer_freq = p.pitch * 6.0 + p.trs_tone * p.pitch * 10.0;
        let layer = (self.phase_layer * tau).sin() * layer_env * p.attack;
        // Noise component for "post-processed with reverbs and distortions"
        let noise_seed = ((t * 48000.0) as u32).wrapping_mul(1664525).wrapping_add(1013904223);
        let noise = (noise_seed as f32 / u32::MAX as f32) * 2.0 - 1.0;
        let layer_noise = noise * layer_env * p.trs_decay * 0.3;

        let amp_env = kick_envelope(t, p.length, p.sustain);
        if envelope_done(t, p.length) {
            self.state.triggered = false;
        }

        let sample = apply_fx(
            (fold + layer + layer_noise) * amp_env * p.velocity,
            p.fx_amount,
        );

        self.phase_osc += freq * dt;
        self.phase_osc -= self.phase_osc.floor();
        self.phase_layer += layer_freq * dt;
        self.phase_layer -= self.phase_layer.floor();
        self.state.advance();

        [sample].into()
    }

    fn reset(&mut self) {
        self.phase_osc = 0.0;
        self.phase_layer = 0.0;
        self.state = KickState::new();
    }

    fn set_sample_rate(&mut self, sr: f64) {
        self.state.sample_rate = sr as f32;
    }
}
