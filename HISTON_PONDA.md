# Histon Ponda: Wavetable Loader Plan

## Piston Honda WAV Format

- Mono 16-bit WAV
- 64 single-cycle waves, each 256 samples
- 8 WAV files = 8 Z-axis positions
- Within each file: 8 groups of 8 waves
  - X axis: wave index within a group (0..8)
  - Y axis: group index (0..8)

## Step 1: Data Model

Create `osc1/src/wavetable.rs` with internal representations.

Store samples as `f32` internally (convert from i16 on load) so there's no
per-tick conversion at playback time.

```
Wave            — single cycle: [f32; 256]
WavetableBank   — one WAV file: [[Wave; 8]; 8]  (Y x X, 64 waves total)
WavetableSet    — full Z axis:  [WavetableBank; 8]
```

Indexing: `set.banks[z].waves[y][x]` gives you a single 256-sample wave.

## Step 2: I/O Trait

Define a trait to abstract loading, keeping it swappable between native
filesystem and embedded (SD card) later.

```rust
pub trait WavetableLoader {
    type Error;
    fn load_bank(&self, path: &str) -> Result<WavetableBank, Self::Error>;
    fn load_set(&self, paths: &[&str; 8]) -> Result<WavetableSet, Self::Error>;
}
```

### Native Implementation: `FileLoader`

- Uses `std::fs` for file access
- WAV parsing options:
  - `hound` crate — simple, reads 16-bit PCM directly
  - Manual — WAV PCM header is 44 bytes, format is known and fixed
- Implement `WavetableLoader` for a `FileLoader` struct

### Future: `SdCardLoader` (embedded)

- Same trait, different backing storage
- Not in scope yet, but the trait boundary is set up for it

## Step 3: Playback Oscillator (future)

Once loading works, build a wavetable oscillator that:

- Takes X, Y, Z coordinates (0.0..1.0 or discrete 0..7)
- Reads the appropriate wave from the set
- Steps through the 256-sample cycle based on frequency/phase
- Optionally interpolates between adjacent waves for smooth morphing
