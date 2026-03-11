//! PM-K1: Physical model of an acoustic bass drum.
//! Entirely different parameter mapping:
//! - PITCH: drum size/tension
//! - ATTACK: beater volume
//! - TRS TONE: beater decay (dark to bright)
//! - SUSTAIN: ambient mic volume
//! - LENGTH: room size
//! - FX: stereo spread (mono output here, ignored)
//!
//! Uses modal synthesis: a struck membrane with multiple resonant modes.

use fundsp::prelude::*;
use super::common::*;

const NUM_MODES: usize = 6;

// Relative frequency ratios for circular membrane modes (Bessel function zeros)
const MODE_RATIOS: [f32; NUM_MODES] = [1.0, 1.593, 2.136, 2.296, 2.653, 3.156];
// Relative amplitudes (higher modes are quieter)
const MODE_AMPS: [f32; NUM_MODES] = [1.0, 0.5, 0.3, 0.25, 0.15, 0.1];

#[derive(Clone)]
pub struct Pmk1Node {
    pub params: KickParams,
    pub state: KickState,
    mode_phases: [f32; NUM_MODES],
    beater_phase: f32,
}

impl Pmk1Node {
    pub fn new(params: KickParams) -> Self {
        Self {
            params,
            state: KickState::new(),
            mode_phases: [0.0; NUM_MODES],
            beater_phase: 0.0,
        }
    }
}

impl AudioNode for Pmk1Node {
    const ID: u64 = 0xB00B_0104;
    type Inputs = U0;
    type Outputs = U1;

    fn tick(&mut self, _input: &Frame<f32, Self::Inputs>) -> Frame<f32, Self::Outputs> {
        let dt = self.state.dt();

        if self.state.check_trigger(&self.params) {
            self.mode_phases = [0.0; NUM_MODES];
            self.beater_phase = 0.0;
        }

        if !self.state.triggered {
            return [0.0].into();
        }

        let p = ParamSnapshot::read(&self.params);
        let t = self.state.env_time;

        // PITCH controls fundamental (drum size/tension)
        let fundamental = p.pitch;

        // Membrane: sum of decaying modal resonances
        // Higher modes decay faster
        let mut membrane = 0.0f32;
        let base_decay = p.length * 2.0; // Room size scales decay
        for i in 0..NUM_MODES {
            let mode_freq = fundamental * MODE_RATIOS[i];
            let decay_rate = (i as f32 + 1.0) * 3.0 / base_decay;
            let mode_env = (-t * decay_rate).exp();
            let mode_val = (self.mode_phases[i] * core::f32::consts::TAU).sin();
            membrane += mode_val * MODE_AMPS[i] * mode_env;

            self.mode_phases[i] += mode_freq * dt;
            self.mode_phases[i] -= self.mode_phases[i].floor();
        }
        // Normalize
        let mode_sum: f32 = MODE_AMPS.iter().sum();
        membrane /= mode_sum;

        // Beater impact: short broadband burst
        // ATTACK = beater volume, TRS TONE = beater brightness
        let beater_decay = 0.003 + (1.0 - p.trs_tone) * 0.015; // 3ms bright to 18ms dark
        let beater_env = (-t / beater_decay).exp();
        let beater_freq = fundamental * (3.0 + p.trs_tone * 12.0);
        let beater_tone = (self.beater_phase * core::f32::consts::TAU).sin();
        // Add noise component for realistic beater
        let noise_seed = ((t * 48000.0) as u32).wrapping_mul(1103515245).wrapping_add(12345);
        let noise = (noise_seed as f32 / u32::MAX as f32) * 2.0 - 1.0;
        let beater = (beater_tone * 0.6 + noise * 0.4) * beater_env * p.attack;

        self.beater_phase += beater_freq * dt;
        self.beater_phase -= self.beater_phase.floor();

        // SUSTAIN = ambient microphone volume (room resonance)
        let room_env = (-t / (base_decay * 0.5)).exp();
        let room = membrane * room_env * p.sustain * 0.3;

        // Overall amplitude: membrane decays naturally, no ADSR needed for physical model
        // But we still need to stop the node eventually
        let total_decay = base_decay * 1.5;
        if t > total_decay {
            self.state.triggered = false;
        }

        let sample = (membrane + beater + room) * p.velocity;

        self.state.advance();

        [sample].into()
    }

    fn reset(&mut self) {
        self.mode_phases = [0.0; NUM_MODES];
        self.beater_phase = 0.0;
        self.state = KickState::new();
    }

    fn set_sample_rate(&mut self, sr: f64) {
        self.state.sample_rate = sr as f32;
    }
}
