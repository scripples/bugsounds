use std::sync::Arc;

use fundsp::prelude::*;
pub use fundsp::snoop::Snoop;

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

        let read_pos = self.phase;

        let sample = self.set.sample_trilinear(x, y, z, read_pos);

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

/// Dual oscillator node (OSC A + OSC B) with oscillator link.
/// When link is enabled, OSC B copies OSC A's frequency plus a fine-tune offset.
/// Outputs a mono mix of both oscillators.
#[derive(Clone)]
struct DualWavetable3DNode {
    set: Arc<WavetableSet>,
    // OSC A
    f0_a: Shared,
    x_a: Shared,
    y_a: Shared,
    z_a: Shared,
    octave_a: Shared,
    amp_a: Shared,
    phase_a: f32,
    // OSC B
    f0_b: Shared,
    x_b: Shared,
    y_b: Shared,
    z_b: Shared,
    octave_b: Shared,
    amp_b: Shared,
    phase_b: f32,
    // Link: when >= 0.5, OSC B freq = OSC A freq + fine_tune_b
    link: Shared,
    fine_tune_b: Shared,
    // Output select: 0=mix, 1=OSC A only, 2=OSC B only
    output_select: Shared,
    // Interpolation mode: 0=trilinear, 1=nearest
    interp_mode: Shared,
    // FM: OSC B modulates OSC A frequency
    fm_amount: Shared,
    sample_rate: f32,
}

impl AudioNode for DualWavetable3DNode {
    const ID: u64 = 0xB005_0002;
    type Inputs = U0;
    type Outputs = U1;

    fn tick(&mut self, _input: &Frame<f32, Self::Inputs>) -> Frame<f32, Self::Outputs> {
        let freq_a = self.f0_a.value() * (2.0_f32).powf(self.octave_a.value());
        let nearest = self.interp_mode.value() >= 0.5;

        // OSC B (computed first so it can modulate A)
        let freq_b = if self.link.value() >= 0.5 {
            freq_a + self.fine_tune_b.value()
        } else {
            self.f0_b.value() * (2.0_f32).powf(self.octave_b.value())
        };
        let read_pos_b = self.phase_b;
        let (xb, yb, zb) = (self.x_b.value(), self.y_b.value(), self.z_b.value());
        let sample_b = if nearest {
            self.set.sample_nearest(xb, yb, zb, read_pos_b)
        } else {
            self.set.sample_trilinear(xb, yb, zb, read_pos_b)
        };
        self.phase_b += freq_b / self.sample_rate;
        self.phase_b -= self.phase_b.floor();

        // OSC A (with FM from B)
        let fm = self.fm_amount.value() * sample_b * freq_a;
        let read_pos_a = self.phase_a;
        let (xa, ya, za) = (self.x_a.value(), self.y_a.value(), self.z_a.value());
        let sample_a = if nearest {
            self.set.sample_nearest(xa, ya, za, read_pos_a)
        } else {
            self.set.sample_trilinear(xa, ya, za, read_pos_a)
        };
        self.phase_a += (freq_a + fm) / self.sample_rate;
        self.phase_a -= self.phase_a.floor();

        let sample_a = sample_a * self.amp_a.value();
        let sample_b = sample_b * self.amp_b.value();
        let out = match self.output_select.value().round() as i32 {
            1 => sample_a,
            2 => sample_b,
            _ => (sample_a + sample_b) * 0.5,
        };
        [out].into()
    }

    fn reset(&mut self) {
        self.phase_a = 0.0;
        self.phase_b = 0.0;
    }

    fn set_sample_rate(&mut self, sample_rate: f64) {
        self.sample_rate = sample_rate as f32;
    }
}

/// Dual oscillator Piston Honda style synth (OSC A + OSC B with link).
pub struct PistonHonda {
    graph: Option<Box<dyn AudioUnit>>,
    snoop: Option<Snoop>,
    // Oscillator A
    f0_a: Shared,
    x_a: Shared,
    y_a: Shared,
    z_a: Shared,
    octave_a: Shared,
    amp_a: Shared,
    // Oscillator B
    f0_b: Shared,
    x_b: Shared,
    y_b: Shared,
    z_b: Shared,
    octave_b: Shared,
    amp_b: Shared,
    link: Shared,
    fine_tune_b: Shared,
    // Common outputs
    amp: Shared,
    output_select: Shared,
    interp_mode: Shared,
    fm_amount: Shared,
}

impl PistonHonda {
    pub fn new(set: WavetableSet, freq: f32) -> Self {
        let f0_a = shared(freq);
        let x_a = shared(0.0);
        let y_a = shared(0.0);
        let z_a = shared(0.0);
        let octave_a = shared(0.0);
        let amp_a = shared(1.0);
        let amp = shared(0.5);

        let f0_b = shared(freq);
        let x_b = shared(0.0);
        let y_b = shared(0.0);
        let z_b = shared(0.0);
        let octave_b = shared(0.0);
        let amp_b = shared(1.0);

        let link = shared(0.0);
        let fine_tune_b = shared(0.0);
        let output_select = shared(0.0);
        let interp_mode = shared(0.0);
        let fm_amount = shared(0.0);

        let node = DualWavetable3DNode {
            set: Arc::new(set),
            f0_a: f0_a.clone(),
            x_a: x_a.clone(),
            y_a: y_a.clone(),
            z_a: z_a.clone(),
            octave_a: octave_a.clone(),
            amp_a: amp_a.clone(),
            phase_a: 0.0,
            f0_b: f0_b.clone(),
            x_b: x_b.clone(),
            y_b: y_b.clone(),
            z_b: z_b.clone(),
            octave_b: octave_b.clone(),
            amp_b: amp_b.clone(),
            phase_b: 0.0,
            link: link.clone(),
            fine_tune_b: fine_tune_b.clone(),
            output_select: output_select.clone(),
            interp_mode: interp_mode.clone(),
            fm_amount: fm_amount.clone(),
            sample_rate: 44100.0,
        };

        let (snoop_front, snoop_back) = Snoop::new(4096);
        let graph = An(node) * var(&amp) >> An(snoop_back);

        Self {
            graph: Some(Box::new(graph)),
            snoop: Some(snoop_front),
            f0_a,
            x_a,
            y_a,
            z_a,
            octave_a,
            amp_a,
            amp,
            f0_b,
            x_b,
            y_b,
            z_b,
            octave_b,
            amp_b,
            link,
            fine_tune_b,
            output_select,
            interp_mode,
            fm_amount,
        }
    }

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

    pub fn set_freq_a(&self, hz: f32) {
        self.f0_a.set(hz);
    }
    pub fn set_x_a(&self, val: f32) {
        self.x_a.set(val);
    }
    pub fn set_y_a(&self, val: f32) {
        self.y_a.set(val);
    }
    pub fn set_z_a(&self, val: f32) {
        self.z_a.set(val);
    }
    pub fn set_octave_a(&self, val: f32) {
        self.octave_a.set(val);
    }
    pub fn set_amp_a(&self, val: f32) {
        self.amp_a.set(val);
    }
    pub fn set_freq_b(&self, hz: f32) {
        self.f0_b.set(hz);
    }
    pub fn set_x_b(&self, val: f32) {
        self.x_b.set(val);
    }
    pub fn set_y_b(&self, val: f32) {
        self.y_b.set(val);
    }
    pub fn set_z_b(&self, val: f32) {
        self.z_b.set(val);
    }
    pub fn set_octave_b(&self, val: f32) {
        self.octave_b.set(val);
    }
    pub fn set_amp_b(&self, val: f32) {
        self.amp_b.set(val);
    }

    pub fn set_amp(&self, val: f32) {
        self.amp.set(val);
    }
    pub fn set_link(&self, enabled: bool) {
        self.link.set(if enabled { 1.0 } else { 0.0 });
    }
    pub fn set_fine_tune_b(&self, hz: f32) {
        self.fine_tune_b.set(hz);
    }
    pub fn set_output_select(&self, val: f32) {
        self.output_select.set(val);
    }
    pub fn set_interp_mode(&self, val: f32) {
        self.interp_mode.set(val);
    }
    pub fn set_fm_amount(&self, val: f32) {
        self.fm_amount.set(val);
    }

    pub fn f0_a(&self) -> Shared {
        self.f0_a.clone()
    }
    pub fn x_a(&self) -> Shared {
        self.x_a.clone()
    }
    pub fn y_a(&self) -> Shared {
        self.y_a.clone()
    }
    pub fn z_a(&self) -> Shared {
        self.z_a.clone()
    }
    pub fn octave_a(&self) -> Shared {
        self.octave_a.clone()
    }
    pub fn amp_a(&self) -> Shared {
        self.amp_a.clone()
    }
    pub fn f0_b(&self) -> Shared {
        self.f0_b.clone()
    }
    pub fn x_b(&self) -> Shared {
        self.x_b.clone()
    }
    pub fn y_b(&self) -> Shared {
        self.y_b.clone()
    }
    pub fn z_b(&self) -> Shared {
        self.z_b.clone()
    }
    pub fn octave_b(&self) -> Shared {
        self.octave_b.clone()
    }
    pub fn amp_b(&self) -> Shared {
        self.amp_b.clone()
    }

    pub fn amp(&self) -> Shared {
        self.amp.clone()
    }
    pub fn link(&self) -> Shared {
        self.link.clone()
    }
    pub fn fine_tune_b(&self) -> Shared {
        self.fine_tune_b.clone()
    }
    pub fn output_select(&self) -> Shared {
        self.output_select.clone()
    }
    pub fn interp_mode(&self) -> Shared {
        self.interp_mode.clone()
    }
    pub fn fm_amount(&self) -> Shared {
        self.fm_amount.clone()
    }

    pub fn take_snoop(&mut self) -> Snoop {
        self.snoop.take().expect("snoop already taken")
    }

    pub fn take_graph(&mut self) -> Box<dyn AudioUnit> {
        self.graph.take().expect("graph already taken")
    }
}

/// Single oscillator wavetable synthesizer.
pub struct Wavetable3D {
    graph: Option<Box<dyn AudioUnit>>,
    snoop: Option<Snoop>,
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

        let (snoop_front, snoop_back) = Snoop::new(4096);
        let graph = An(node) * var(&amp) >> An(snoop_back);

        Self {
            graph: Some(Box::new(graph)),
            snoop: Some(snoop_front),
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

    pub fn take_snoop(&mut self) -> Snoop {
        self.snoop.take().expect("snoop already taken")
    }

    pub fn take_graph(&mut self) -> Box<dyn AudioUnit> {
        self.graph.take().expect("graph already taken")
    }
}
