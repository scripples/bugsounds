//! Multi-model node: wraps all kick models and switches between them
//! at runtime via a Shared selector.

use fundsp::prelude::*;
use super::common::KickParams;
use super::*;

#[derive(Clone)]
pub struct MultiModelNode {
    pub params: KickParams,
    pub model_select: Shared, // 0..7 maps to Model variants
    fm2x: Fm2xNode,
    hz1: Hz1Node,
    olp4: Olp4Node,
    pmk1: Pmk1Node,
    px3: Px3Node,
    sp6: Sp6Node,
    vxt: VxtNode,
    wt4: Wt4Node,
}

impl MultiModelNode {
    pub fn new(params: KickParams, model_select: Shared) -> Self {
        Self {
            fm2x: Fm2xNode::new(params.clone()),
            hz1: Hz1Node::new(params.clone()),
            olp4: Olp4Node::new(params.clone()),
            pmk1: Pmk1Node::new(params.clone()),
            px3: Px3Node::new(params.clone()),
            sp6: Sp6Node::new(params.clone()),
            vxt: VxtNode::new(params.clone()),
            wt4: Wt4Node::new(params.clone()),
            params,
            model_select,
        }
    }
}

impl AudioNode for MultiModelNode {
    const ID: u64 = 0xB00B_0200;
    type Inputs = U0;
    type Outputs = U1;

    fn tick(&mut self, input: &Frame<f32, Self::Inputs>) -> Frame<f32, Self::Outputs> {
        let idx = self.model_select.value() as usize;
        match idx {
            0 => self.fm2x.tick(input),
            1 => self.hz1.tick(input),
            2 => self.olp4.tick(input),
            3 => self.pmk1.tick(input),
            4 => self.px3.tick(input),
            5 => self.sp6.tick(input),
            6 => self.vxt.tick(input),
            _ => self.wt4.tick(input),
        }
    }

    fn reset(&mut self) {
        self.fm2x.reset();
        self.hz1.reset();
        self.olp4.reset();
        self.pmk1.reset();
        self.px3.reset();
        self.sp6.reset();
        self.vxt.reset();
        self.wt4.reset();
    }

    fn set_sample_rate(&mut self, sr: f64) {
        self.fm2x.set_sample_rate(sr);
        self.hz1.set_sample_rate(sr);
        self.olp4.set_sample_rate(sr);
        self.pmk1.set_sample_rate(sr);
        self.px3.set_sample_rate(sr);
        self.sp6.set_sample_rate(sr);
        self.vxt.set_sample_rate(sr);
        self.wt4.set_sample_rate(sr);
    }
}
