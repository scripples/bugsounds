pub const WAVE_SIZE: usize = 256;
pub const BANK_DIM: usize = 8;
pub const WAVES_PER_BANK: usize = BANK_DIM * BANK_DIM;
pub const NUM_BANKS: usize = 8;

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

/// A single wavetable bank loaded from one WAV file.
/// Contains 64 single-cycle waves (8 Y rows x 8 X columns),
/// each 256 samples long.
#[derive(Clone)]
pub struct WavetableBank {
    /// Flat storage: samples[(y * 8 + x) * 256 + i]
    samples: Vec<f32>,
}

impl WavetableBank {
    pub fn from_samples(samples: Vec<f32>) -> Self {
        assert_eq!(samples.len(), WAVES_PER_BANK * WAVE_SIZE);
        Self { samples }
    }

    /// Read a sample from wave (x, y) at a fractional phase [0.0, 1.0),
    /// using nearest-neighbor lookup (no interpolation).
    pub fn sample_at(&self, x: usize, y: usize, phase: f32) -> f32 {
        let offset = (y * BANK_DIM + x) * WAVE_SIZE;
        // nearest-neighbor interpolation
        let idx = (phase * WAVE_SIZE as f32) as usize % WAVE_SIZE;
        self.samples[offset + idx]
    }

    pub fn sample_at_interpolate(&self, x: usize, y: usize, phase: f32) -> f32 {
        let offset = (y * BANK_DIM + x) * WAVE_SIZE;
        let pos = phase * WAVE_SIZE as f32;
        let idx0 = pos as usize % WAVE_SIZE;
        let idx1 = (idx0 + 1) % WAVE_SIZE;
        let frac = pos - pos.floor();
        lerp(self.samples[offset + idx0], self.samples[offset + idx1], frac)
    }
}

/// Complete 3D wavetable: 8 banks along the Z axis.
/// Each bank corresponds to one WAV file / one Z position.
#[derive(Clone)]
pub struct WavetableSet {
    pub banks: Vec<WavetableBank>,
}

impl WavetableSet {
    /// Sample the 3D wavetable with trilinear interpolation.
    /// x, y, z should be in [0.0, 7.0]. Values between integer
    /// positions are linearly interpolated across all three axes.
    pub fn sample_trilinear(&self, x: f32, y: f32, z: f32, phase: f32) -> f32 {
        let max = (BANK_DIM - 1) as f32;

        let x = x.clamp(0.0, max + 1.0);
        let y = y.clamp(0.0, max + 1.0);
        let z = z.clamp(0.0, max + 1.0);

        let xi = (x as usize).min(BANK_DIM - 2);
        let yi = (y as usize).min(BANK_DIM - 2);
        let zi = (z as usize).min(BANK_DIM - 2);

        let xf = x - xi as f32;
        let yf = y - yi as f32;
        let zf = z - zi as f32;

        // Sample all 8 corners of the cube
        let s000 = self.banks[zi].sample_at_interpolate(xi, yi, phase);
        let s100 = self.banks[zi].sample_at_interpolate(xi + 1, yi, phase);
        let s010 = self.banks[zi].sample_at_interpolate(xi, yi + 1, phase);
        let s110 = self.banks[zi].sample_at_interpolate(xi + 1, yi + 1, phase);
        let s001 = self.banks[zi + 1].sample_at_interpolate(xi, yi, phase);
        let s101 = self.banks[zi + 1].sample_at_interpolate(xi + 1, yi, phase);
        let s011 = self.banks[zi + 1].sample_at_interpolate(xi, yi + 1, phase);
        let s111 = self.banks[zi + 1].sample_at_interpolate(xi + 1, yi + 1, phase);

        // Interpolate along X
        let c00 = lerp(s000, s100, xf);
        let c10 = lerp(s010, s110, xf);
        let c01 = lerp(s001, s101, xf);
        let c11 = lerp(s011, s111, xf);

        // Interpolate along Y
        let c0 = lerp(c00, c10, yf);
        let c1 = lerp(c01, c11, yf);

        // Interpolate along Z
        lerp(c0, c1, zf)
    }

    /// Sample the nearest whole waveform (no crossfading between positions).
    pub fn sample_nearest(&self, x: f32, y: f32, z: f32, phase: f32) -> f32 {
        let max = (BANK_DIM - 1) as f32;
        let xi = (x.round() as usize).min(BANK_DIM - 1);
        let yi = (y.round() as usize).min(BANK_DIM - 1);
        let zi = (z.round() as usize).min(BANK_DIM - 1);
        self.banks[zi].sample_at_interpolate(xi, yi, phase)
    }
}

// I/O trait + native filesystem implementation

pub trait WavetableLoader {
    type Error: std::fmt::Debug;
    fn load_bank(&self, path: &str) -> Result<WavetableBank, Self::Error>;
    fn load_set(&self, paths: &[&str; NUM_BANKS]) -> Result<WavetableSet, Self::Error>;
}

#[derive(Debug)]
pub enum WavetableLoadError {
    Io(String),
    Format(String),
}

impl std::fmt::Display for WavetableLoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(msg) => write!(f, "IO error: {msg}"),
            Self::Format(msg) => write!(f, "format error: {msg}"),
        }
    }
}

impl std::error::Error for WavetableLoadError {}

/// Loads wavetable WAV files from the native filesystem using `hound`.
pub struct FileLoader;

impl WavetableLoader for FileLoader {
    type Error = WavetableLoadError;

    fn load_bank(&self, path: &str) -> Result<WavetableBank, Self::Error> {
        let reader =
            hound::WavReader::open(path).map_err(|e| WavetableLoadError::Io(e.to_string()))?;

        let spec = reader.spec();
        if spec.channels != 1 {
            return Err(WavetableLoadError::Format("expected mono".into()));
        }
        if spec.bits_per_sample != 16 {
            return Err(WavetableLoadError::Format("expected 16-bit".into()));
        }

        let expected = WAVES_PER_BANK * WAVE_SIZE;
        // Some WAV files have incorrect data size headers (claiming more
        // data than exists). Read the exact number of samples.
        let mut samples = Vec::with_capacity(expected);
        for s in reader.into_samples::<i16>().take(expected) {
            match s {
                Ok(v) => samples.push(v as f32 / i16::MAX as f32),
                Err(_) if samples.len() == expected => break,
                Err(e) => return Err(WavetableLoadError::Io(e.to_string())),
            }
        }

        if samples.len() != expected {
            return Err(WavetableLoadError::Format(format!(
                "expected {} samples, got {}",
                expected,
                samples.len()
            )));
        }

        Ok(WavetableBank::from_samples(samples))
    }

    fn load_set(&self, paths: &[&str; NUM_BANKS]) -> Result<WavetableSet, Self::Error> {
        let banks: Vec<WavetableBank> = paths
            .iter()
            .map(|p| self.load_bank(p))
            .collect::<Result<_, _>>()?;
        Ok(WavetableSet { banks })
    }
}
