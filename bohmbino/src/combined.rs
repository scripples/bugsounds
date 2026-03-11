//! Combined Bohm + Groove node that mixes both voices into a single output.

use fundsp::prelude::*;
use crate::bohm::MultiModelNode;
use crate::groove::GrooveNode;

#[derive(Clone)]
pub struct CombinedNode {
    bohm: MultiModelNode,
    groove: GrooveNode,
    groove_enabled: Shared,
}

impl CombinedNode {
    pub fn new(
        bohm: MultiModelNode,
        groove: GrooveNode,
        groove_enabled: Shared,
    ) -> Self {
        Self { bohm, groove, groove_enabled }
    }
}

impl AudioNode for CombinedNode {
    const ID: u64 = 0xB00B_0500;
    type Inputs = U0;
    type Outputs = U1;

    fn tick(&mut self, input: &Frame<f32, Self::Inputs>) -> Frame<f32, Self::Outputs> {
        let bohm_out = self.bohm.tick(input)[0];

        if self.groove_enabled.value() >= 0.5 {
            let groove_out = self.groove.tick(input)[0];
            [bohm_out + groove_out].into()
        } else {
            [bohm_out].into()
        }
    }

    fn reset(&mut self) {
        self.bohm.reset();
        self.groove.reset();
    }

    fn set_sample_rate(&mut self, sr: f64) {
        self.bohm.set_sample_rate(sr);
        self.groove.set_sample_rate(sr);
    }
}
