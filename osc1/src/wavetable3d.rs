use std::sync::Arc;

use fundsp::prelude::*;

use crate::wavetable::{FileLoader, NUM_BANKS, WavetableLoader, WavetableSet};

/// Custom AudioNode that reads from a 3D wavetable (X × Y × Z)
/// with trilinear interpolation between neighboring positions.
#[derive(Clone)]
struct Wavetable3DNode {
    set: Arc<WavetableSet>,
    f0: Shared,
    x: Shared,
    y: Shared,
    z: Shared,
    phase: f32,
    sample_rate: f32,
}

impl AudioNode for Wavetable3DNode {
    const ID: u64 = 0xB005_0001;
    type Inputs = U0;
    type Outputs = U1;

    fn tick(&mut self, _input: &Frame<f32, Self::Inputs>) -> Frame<f32, Self::Outputs> {
        let x = self.x.value();
        let y = self.y.value();
        let z = self.z.value();
        let freq = self.f0.value();

        let sample = self.set.sample_trilinear(x, y, z, self.phase);

        self.phase += freq / self.sample_rate;
        self.phase -= self.phase.floor();

        [sample].into()
    }

    fn reset(&mut self) {
        self.phase = 0.0;
    }

    fn set_sample_rate(&mut self, sample_rate: f64) {
        self.sample_rate = sample_rate as f32;
    }
}

/// 3D wavetable synthesizer. Controls X, Y, Z position and frequency
pub struct Wavetable3D {
    graph: Option<Box<dyn AudioUnit>>,
    f0: Shared,
    x: Shared,
    y: Shared,
    z: Shared,
    amp: Shared,
}

impl Wavetable3D {
    pub fn new(set: WavetableSet, freq: f32) -> Self {
        let f0 = shared(freq);
        let x = shared(0.0);
        let y = shared(0.0);
        let z = shared(0.0);
        let amp = shared(0.5);

        let node = Wavetable3DNode {
            set: Arc::new(set),
            f0: f0.clone(),
            x: x.clone(),
            y: y.clone(),
            z: z.clone(),
            phase: 0.0,
            sample_rate: 44100.0,
        };

        let graph = An(node) * var(&amp);

        Self {
            graph: Some(Box::new(graph)),
            f0,
            x,
            y,
            z,
            amp,
        }
    }

    /// Load 8 WAV files from a directory (expected filenames: 1.wav through 8.wav).
    pub fn from_directory(
        dir: &str,
        freq: f32,
    ) -> Result<Self, crate::wavetable::WavetableLoadError> {
        let loader = FileLoader;
        let paths: Vec<String> = (1..=NUM_BANKS).map(|i| format!("{dir}/{i}.wav")).collect();
        let path_refs: Vec<&str> = paths.iter().map(|s| s.as_str()).collect();
        let path_array: &[&str; NUM_BANKS] =
            path_refs.as_slice().try_into().expect("always 8 paths");
        let set = loader.load_set(path_array)?;
        Ok(Self::new(set, freq))
    }

    pub fn set_freq(&self, hz: f32) {
        self.f0.set(hz);
    }

    pub fn set_x(&self, val: f32) {
        self.x.set(val);
    }

    pub fn set_y(&self, val: f32) {
        self.y.set(val);
    }

    pub fn set_z(&self, val: f32) {
        self.z.set(val);
    }

    pub fn set_amp(&self, val: f32) {
        self.amp.set(val);
    }

    pub fn f0(&self) -> Shared {
        self.f0.clone()
    }

    pub fn x(&self) -> Shared {
        self.x.clone()
    }

    pub fn y(&self) -> Shared {
        self.y.clone()
    }

    pub fn z(&self) -> Shared {
        self.z.clone()
    }

    pub fn amp(&self) -> Shared {
        self.amp.clone()
    }

    pub fn take_graph(&mut self) -> Box<dyn AudioUnit> {
        self.graph.take().expect("graph already taken")
    }
}
