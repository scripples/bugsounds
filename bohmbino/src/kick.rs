use fundsp::prelude::*;
pub use fundsp::snoop::Snoop;

use crate::bohm::KickParams;
use crate::groove::GrooveNode;

/// Available kick models.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Model {
    /// 2-operator carrier/modulator FM kick
    Fm2x,
    /// Wavetable oscillator + transient synthesizer
    Hz1,
    /// 4-operator FM kick (OPL3 inspired)
    Olp4,
    /// Physical model of an acoustic bass drum
    Pmk1,
    /// Wavetable + weird drum layering
    Px3,
    /// Digital wavetable + FM hihat/snare transients
    Sp6,
    /// Analog wavetable + 4-op FM transient with bandpass
    Vxt,
    /// Analog wavetable + drum layering
    Wt4,
}

impl Model {
    pub const ALL: &[Model] = &[
        Model::Fm2x,
        Model::Hz1,
        Model::Olp4,
        Model::Pmk1,
        Model::Px3,
        Model::Sp6,
        Model::Vxt,
        Model::Wt4,
    ];

    pub fn name(&self) -> &'static str {
        match self {
            Model::Fm2x => "FM-2X",
            Model::Hz1 => "HZ-1",
            Model::Olp4 => "OLP4",
            Model::Pmk1 => "PM-K1",
            Model::Px3 => "PX3",
            Model::Sp6 => "SP-6",
            Model::Vxt => "VX-T",
            Model::Wt4 => "WT-4",
        }
    }

    pub fn index(&self) -> usize {
        *self as usize
    }

    pub fn from_index(i: usize) -> Self {
        Self::ALL[i % Self::ALL.len()]
    }
}

/// Public kick drum synthesizer with Bohm + Groove voices.
pub struct Bohmbino {
    graph: Option<Box<dyn AudioUnit>>,
    snoop: Option<Snoop>,
    // Bohm params
    params: KickParams,
    amp: Shared,
    model_select: Shared,
    // Groove params
    grv_pitch: Shared,
    grv_color: Shared,
    grv_length: Shared,
    grv_fx_amount: Shared,
    grv_fx_type: Shared,
    grv_vol: Shared,
    grv_tap2: Shared,
    grv_tap3: Shared,
    grv_tap4: Shared,
    grv_trigger: Shared,
    grv_clock: Shared,
    groove_enabled: Shared,
}

impl Bohmbino {
    pub fn new() -> Self {
        let params = KickParams::new();
        let amp = shared(0.8);
        let model_select = shared(0.0);

        // Groove params
        let grv_pitch = shared(55.0);
        let grv_color = shared(0.0);
        let grv_length = shared(0.3);
        let grv_fx_amount = shared(0.0);
        let grv_fx_type = shared(0.0);
        let grv_vol = shared(0.5);
        let grv_tap2 = shared(0.7);
        let grv_tap3 = shared(0.4);
        let grv_tap4 = shared(0.2);
        let grv_trigger = shared(0.0);
        let grv_clock = shared(0.0);
        let groove_enabled = shared(0.0);

        let bohm_node = crate::bohm::MultiModelNode::new(params.clone(), model_select.clone());
        let groove_node = GrooveNode::new(
            grv_pitch.clone(), grv_color.clone(), grv_length.clone(),
            grv_fx_amount.clone(), grv_fx_type.clone(), grv_vol.clone(),
            grv_tap2.clone(), grv_tap3.clone(), grv_tap4.clone(),
            grv_trigger.clone(), grv_clock.clone(),
        );

        let combined = crate::combined::CombinedNode::new(bohm_node, groove_node, groove_enabled.clone());

        let (snoop_front, snoop_back) = Snoop::new(4096);
        let graph = An(combined) * var(&amp) >> An(snoop_back);

        Self {
            graph: Some(Box::new(graph)),
            snoop: Some(snoop_front),
            params,
            amp,
            model_select,
            grv_pitch, grv_color, grv_length, grv_fx_amount,
            grv_fx_type, grv_vol, grv_tap2, grv_tap3, grv_tap4,
            grv_trigger, grv_clock, groove_enabled,
        }
    }

    // ── Model ──

    pub fn set_model(&self, model: Model) {
        self.model_select.set(model.index() as f32);
    }
    pub fn model_select(&self) -> Shared { self.model_select.clone() }

    // ── Bohm ──

    pub fn hit(&self) { self.params.trigger.set(1.0); }

    pub fn set_pitch(&self, hz: f32) { self.params.pitch.set(hz); }
    pub fn set_curve(&self, val: f32) { self.params.curve.set(val); }
    pub fn set_length(&self, seconds: f32) { self.params.length.set(seconds); }
    pub fn set_sustain(&self, val: f32) { self.params.sustain.set(val); }
    pub fn set_attack(&self, val: f32) { self.params.attack.set(val); }
    pub fn set_velocity(&self, val: f32) { self.params.velocity.set(val); }
    pub fn set_color(&self, val: f32) { self.params.color.set(val); }
    pub fn set_fx_amount(&self, val: f32) { self.params.fx_amount.set(val); }
    pub fn set_trs_decay(&self, val: f32) { self.params.trs_decay.set(val); }
    pub fn set_trs_tone(&self, val: f32) { self.params.trs_tone.set(val); }
    pub fn set_amp(&self, val: f32) { self.amp.set(val); }

    pub fn pitch(&self) -> Shared { self.params.pitch.clone() }
    pub fn curve(&self) -> Shared { self.params.curve.clone() }
    pub fn length(&self) -> Shared { self.params.length.clone() }
    pub fn sustain(&self) -> Shared { self.params.sustain.clone() }
    pub fn attack(&self) -> Shared { self.params.attack.clone() }
    pub fn velocity(&self) -> Shared { self.params.velocity.clone() }
    pub fn color(&self) -> Shared { self.params.color.clone() }
    pub fn fx_amount(&self) -> Shared { self.params.fx_amount.clone() }
    pub fn trs_decay(&self) -> Shared { self.params.trs_decay.clone() }
    pub fn trs_tone(&self) -> Shared { self.params.trs_tone.clone() }
    pub fn trigger(&self) -> Shared { self.params.trigger.clone() }
    pub fn amp(&self) -> Shared { self.amp.clone() }

    // ── Groove ──

    pub fn set_groove_enabled(&self, on: bool) { self.groove_enabled.set(if on { 1.0 } else { 0.0 }); }
    pub fn groove_hit(&self) { self.grv_trigger.set(1.0); }
    pub fn groove_clock(&self) { self.grv_clock.set(1.0); }

    pub fn set_grv_pitch(&self, hz: f32) { self.grv_pitch.set(hz); }
    pub fn set_grv_color(&self, val: f32) { self.grv_color.set(val); }
    pub fn set_grv_length(&self, val: f32) { self.grv_length.set(val); }
    pub fn set_grv_fx_amount(&self, val: f32) { self.grv_fx_amount.set(val); }
    pub fn set_grv_fx_type(&self, val: f32) { self.grv_fx_type.set(val); }
    pub fn set_grv_vol(&self, val: f32) { self.grv_vol.set(val); }
    pub fn set_grv_tap2(&self, val: f32) { self.grv_tap2.set(val); }
    pub fn set_grv_tap3(&self, val: f32) { self.grv_tap3.set(val); }
    pub fn set_grv_tap4(&self, val: f32) { self.grv_tap4.set(val); }

    pub fn grv_pitch(&self) -> Shared { self.grv_pitch.clone() }
    pub fn grv_color(&self) -> Shared { self.grv_color.clone() }
    pub fn grv_length(&self) -> Shared { self.grv_length.clone() }
    pub fn grv_fx_amount(&self) -> Shared { self.grv_fx_amount.clone() }
    pub fn grv_fx_type(&self) -> Shared { self.grv_fx_type.clone() }
    pub fn grv_vol(&self) -> Shared { self.grv_vol.clone() }
    pub fn grv_tap2(&self) -> Shared { self.grv_tap2.clone() }
    pub fn grv_tap3(&self) -> Shared { self.grv_tap3.clone() }
    pub fn grv_tap4(&self) -> Shared { self.grv_tap4.clone() }
    pub fn grv_trigger(&self) -> Shared { self.grv_trigger.clone() }
    pub fn grv_clock(&self) -> Shared { self.grv_clock.clone() }
    pub fn groove_enabled(&self) -> Shared { self.groove_enabled.clone() }

    // ── Graph ──

    pub fn take_snoop(&mut self) -> Snoop {
        self.snoop.take().expect("snoop already taken")
    }

    pub fn take_graph(&mut self) -> Box<dyn AudioUnit> {
        self.graph.take().expect("graph already taken")
    }
}
