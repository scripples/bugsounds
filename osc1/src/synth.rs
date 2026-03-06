use fundsp::prelude::*;

pub struct Synth {
    graph: Option<Box<dyn AudioUnit>>,
    f0: Shared,
    ratio: Shared,
    m0: Shared,
}

impl Synth {
    pub fn new(freq: f32) -> Self {
        let f0 = shared(freq);
        let ratio = shared(1.0);
        let m0 = shared(1.0);
        let mod_index = var(&m0);
        let modulator = var(&f0) * var(&ratio) >> sine::<f32>() * var(&f0) * mod_index;
        let carrier = (modulator + var(&f0)) >> sine::<f32>();

        Self {
            graph: Some(Box::new(carrier)),
            f0,
            ratio,
            m0,
        }
    }

    pub fn set_f0(&self, hz: f32) {
        self.f0.set(hz);
    }

    pub fn set_ratio(&self, r: f32) {
        self.ratio.set(r);
    }

    pub fn set_m0(&self, val: f32) {
        self.m0.set(val);
    }

    pub fn f0(&self) -> Shared {
        self.f0.clone()
    }

    pub fn ratio(&self) -> Shared {
        self.ratio.clone()
    }

    pub fn m0(&self) -> Shared {
        self.m0.clone()
    }

    /// Extract the audio graph for the audio thread
    pub fn take_graph(&mut self) -> Box<dyn AudioUnit> {
        self.graph.take().expect("graph already taken")
    }
}
