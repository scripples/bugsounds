use fundsp::prelude::*;

pub struct Synth {
    graph: Option<Box<dyn AudioUnit>>,
    f0: Shared,
    f1: Shared,
    m0: Shared,
}

impl Synth {
    pub fn new(freq: f32) -> Self {
        let f0 = shared(freq);
        let f1 = shared(freq);
        let m0 = shared(1.0);
        let synth = sine_hz::<f32>(440.0) * var(&f1) * var(&m0) + var(&f0) >> sine::<f32>();

        Self {
            graph: Some(Box::new(synth)),
            f0,
            f1,
            m0,
        }
    }

    /// Extract the audio graph for the audio thread
    pub fn fm(&mut self) {
        // let modulator = var(&self.f0) >> sine::<f32>();
        let synth = var(&self.f1) * var(&self.m0) >> sine::<f32>();
        self.graph = Some(Box::new(synth));
    }

    pub fn set_f0(&self, hz: f32) {
        self.f0.set(hz);
    }

    pub fn set_f1(&self, hz: f32) {
        self.f1.set(hz);
    }

    pub fn set_m0(&self, hz: f32) {
        self.m0.set(hz);
    }

    pub fn f0_val(&self) -> f32 {
        self.f0.value()
    }

    pub fn f1_val(&self) -> f32 {
        self.f1.value()
    }

    /// Get a clone of the shared frequency handle
    pub fn f0(&self) -> Shared {
        self.f0.clone()
    }

    pub fn f1(&self) -> Shared {
        self.f1.clone()
    }

    pub fn m0(&self) -> Shared {
        self.m0.clone()
    }

    /// Extract the audio graph for the audio thread
    pub fn take_graph(&mut self) -> Box<dyn AudioUnit> {
        self.graph.take().expect("graph already taken")
    }
}
