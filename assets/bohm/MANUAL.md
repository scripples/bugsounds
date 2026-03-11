# Bohm Eurorack Manual

Firmware V2025.0827.1600 — Ohm Force

Bohm is a stereo dual-voice Eurorack kick set of modules.

## Installation

### Groove and Performer Expanders

1. Position the modules face down
2. Use supplied ribbon cables to connect expanders: Groove connects on the **left** of the Bohm module, Performer on the **right**

### Bohm

1. Power down the Eurorack system
2. Locate space: Bohm requires 18HP, Groove requires 10HP, Performer 8HP. The entire set requires 36HP
3. Connect the 10-pin connector, ensuring the **red stripe on the ribbon cable** is on the **left** of the module
4. Connect the 16-pin connector to the rack, matching the **red stripe** on the ribbon cable to the **−12V** pin on the header
5. Position the module in the rack
6. Secure using supplied screws — **do not overtighten**
7. Power up the Eurorack system

Calibrate expanders after installation for optimal performance (see Calibration Mode).

## Technical Specifications

### General

| Parameter | Bohm | Groove | Performer |
|-----------|------|--------|-----------|
| Width | 18HP | 10HP | 8HP |
| Height | 3U | 3U | 3U |
| Depth (incl. connectors) | 28mm | 26mm | 26mm |
| +12V consumption | 130mA | 20mA | 10mA |
| -12V consumption | 10mA | 10mA | 5mA |
| +5V consumption | 0mA | 0mA | 0mA |

### Audio

- Sample rate: 48kHz
- Hardware audio converters: 24-bit
- Internal processing: 32-bit floating point
- True stereo audio
- High fidelity Texas Instruments Burr-Brown audio converters
- Latency: 0.33ms

### Controls

- Knobs resolution: 16-bit (65536 distinct values)
- CV inputs: ±5V (0..5V compatible), 16-bit resolution

### Memory

- Current model with variations for up to 12 models
- 32 programs of 16 steps each
- System settings

### SD Port

- Type: microSD, SDHC class 10 or greater
- Capacity: up to 32GB
- Format: FAT32 with MBR (Master Boot Record) only
- Used for firmware updates, kick model storage, and custom wavetables/samples

## Overview

### Bohm (Main Module)

Primary kick drum voice with 12 controls:

- **HIT**: Kick trigger
- **VELOCITY**: Kick velocity
- **LENGTH**: Kick duration
- **SUSTAIN**: Kick volume
- **ATTACK**: Kick attack amount
- **PITCH**: Range C1 (32.70Hz) to C2 (65.41Hz)
- **CURVE**: Pitch curve, from 808-style (counterclockwise) to 909-style (clockwise)
- **TRS DECAY**: Transient (click) duration
- **COLOR**: Timbre of the sub-bass oscillator
- **FX**: Amount of kick post-effect
- **TRS TONE**: Transient (click) tone, dark (counterclockwise) to bright (clockwise)
- **FUNCTION**: Model and model variations selector

Additional hardware: microSD slot, stereo audio output.

### Groove (Expander)

Secondary kick voice for techno rumbles or kick tops, with seven controls:
- Clock trigger
- Volume envelope taps (2, 3, 4)
- Length, pitch (relative to Bohm), color, effects, volume

### Performer (Expander)

Audio input section with ducking, performance effects, activation toggle, and master volume for both voices.

## Functions

The Bohm engine processes models from a patching environment. Each model file contains around 400 modules including oscillators, filters, and functions. All knobs and CVs control macros within these models, with each model interpreting controls uniquely.

### Routing Architecture

- **Bohm**: Primary kick voice triggered by HIT input/button
- **Groove** (optional): Secondary kick triggered by CLOCK input, balanced via Groove VOL
- **Performer** (optional): Receives both signals, blends with external audio, applies effects

Both Bohm and Groove feed into Performer if present, otherwise directly to audio OUT.

### Bohm Controls

**PITCH**: Ranges from C1 (32.70Hz) to C2 (65.41Hz), following adjustable curve (808-style counterclockwise to 909-style clockwise).

**COLOR**: Controls wavetable position curve over time.

**ATTACK**: Affects envelope and transient synthesizer volume.

**LENGTH**: Total kick duration.

**SUSTAIN**: Classical ADSR envelope parameter.

**HIT**: Functions as trigger or gate; sustains while held.

**TRS DECAY / TRS TONE**: Controls transient synthesizer decay time and tonal character (dark to bright).

**VELOCITY**: Volume control applied pre-effects.

**FX**: Controls effect amount.

### Groove Controls

Contains four sound generators: kick repetitions, reverb, noise, and gritty sub-frequency noise.

**COLOR**: Selects and blends sound generators.

**2, 3, 4 knobs + TAPS CV**: Control individual tap volumes and create volume envelopes.

**LENGTH**: Affects repetitions only (approximately 3 o'clock to fully clockwise).

**PITCH**: Relative to Bohm pitch; center position matches Bohm.

**Volume Envelope Behavior**:
- Retriggered when HIT activates
- CLOCK triggers determine speed to subsequent taps
- `GRV ENV` system setting controls post-envelope behavior (FALL or SUSTAIN)

**VOL**: Final output volume control.

### Performer Controls

**Audio IN**: Stereo external input, ducked on HIT trigger.

**DUCK**: Controls ducking amount.

**VOL**: Controls Bohm/Groove output level before mixing with audio input.

**Channel Selector**: Routes ALL (both signals), KICK only, or INPUT only to effect section.

**ON/OFF**: Effect activation/deactivation (immediate or HIT-synchronized via system settings).

**FX**: Controls effect parameter.

### Model Variations

Access via FUNCTION button in Studio mode. Features hierarchical menus with Bohm variations at root level, plus separate Groove and Performer submenus.

#### Randomization

- `BOHM + GROOVE`: Excludes Performer
- `ALL`: Includes all sections

Accessible via FUNCTION CV input or model variations menu.

#### Snapshots

- **SAVE**: Store current variations to program slots with optional naming
- **LOAD**: Recall saved variations (knob positions not included)

### Bohm Common Variations — FX Types

- `TUBE`: Slight distortion with analog character
- `BASS`: Selective bass distortion for lengthy kicks
- `SOFT`: Soft-clipper
- `HARD`: Hard-clipper
- `WAVEFOLD`: Wavefolder distortion
- `BITCRUSH`: Bit crusher (select models only)
- `DECIM`: Sampling rate reduction (select models only)

### Bohm Stereo Variation

`STEREO` parameter: 0% (mono) to 100% (full stereo width).

### Groove Variations — FX Types

- `LP`: Low-pass filter (FX controls cutoff)
- `HP`: High-pass filter (FX controls cutoff)
- `BP`: Band-pass filter (FX controls center frequency)
- `DIST`: Distortion (FX controls gain)

### Groove Stereo Variation

Same as Bohm: 0% (mono) to 100% (full stereo).

### Performer Variations — FX Types

- `DJ FILTER`: Low-pass counterclockwise to high-pass clockwise; center neutral
- `HP`: High-pass (cutoff from all frequencies to blocking)
- `LP`: Low-pass (cutoff from blocking all frequencies to passing)
- `BEAT ROLL`: Beat-synchronized roll (FX controls amount)
- `SLIP ROLL`: Beat roll with resampling on each HIT

### Performer DJ RESO

When using DJ FILTER, HP, or LP: Controls filter resonance from 0% (none) to 100% (pronounced rave effect).

### Performer CHN (Channel Selection)

- `ALL`: Routes both kick and external audio to effects
- `KICK`: Routes only kick to effects
- `INPUT`: Routes only external audio to effects

## Core Models Library

### FM-2X

2-operator carrier/modulator FM kick with a sub-bass oscillator. Carrier frequency responds to PITCH and CURVE controls; amplitude shaped by ATTACK, SUSTAIN, LENGTH, and VELOCITY.

Carrier uses a wavetable oscillator where COLOR adjusts position from square (counterclockwise) through sine (center) to triangle (clockwise), with interpolation between positions.

Modulator oscillator creates FM-based transients. ATTACK controls frequency modulation amount. TRS DECAY spans 10ms to 100ms. TRS TONE selects from eight waveforms: Square, Derived Square, Quarter Sinus, Sinus, Half Sinus, Alternating Sinus, Camel Sinus, Positive Sinus — similar to OPL3 waveforms.

Carrier-to-modulator ratio adjustable via RATIO menu: 0.5, 1–10, 12, 15. Groove expander compatible.

### HZ-1

Wavetable oscillator kick combined with a transient synthesizer. Frequency controlled by PITCH/CURVE; amplitude by ATTACK, SUSTAIN, LENGTH, VELOCITY.

COLOR adjusts high-frequency transients by varying wavetable position. At minimum, output is pure sine; increasing clockwise adds distortion during attack while maintaining sinusoidal tail; maximum requires extended LENGTH for sine presence.

WT menu offers analog-style waveforms. CLK menu provides click, pop, tick, and toc variations. Groove expander uses wavetable oscillator without transient synthesis.

### OLP4

4-operator FM kick, inspired by the OPL3 chip. Probably the most experimental model. Uses non-interpolated wavetables selectable via WF1 and WF2 menus (WF1 for operators 1/3, WF2 for operators 2/4).

Six algorithm configurations: 12, 1//2, 1234, 12//34, 1//234, 1//23//4. TRS TONE controls FM feedback from operator 3 to operator 1 for noise character. COLOR inactive. Groove expander unsupported.

### PM-K1

Physical model of an acoustic bass drum — entirely different parameter mapping.

- PITCH: Drum size/tension
- ATTACK: Beater volume
- TRS TONE: Beater decay (dark to bright)
- SUSTAIN: Ambient microphone volume
- LENGTH: Room size
- FX: Stereo spread (mono to wide)

All other controls inactive. Groove expander unsupported.

### PX3

Wavetable oscillator kick with weird wavetables combined with drum layering samples. Incorporates various objects hitting diverse surfaces, post-processed with reverbs and distortions. Produces harder and more experimental character.

Standard controls: PITCH, CURVE, ATTACK, SUSTAIN, LENGTH, VELOCITY. COLOR modulates wavetable position. WT menu adjusts wavetable; LAYER menu selects drum samples. Groove expander uses oscillator without layering sampler.

### SP-6

Wavetable oscillator kick with digital-sounding wavetables combined with drum layering samples, featuring synthesized transients (FM hihats and snares).

Standard frequency/amplitude controls. COLOR creates transients: sine at minimum, increasing distortion clockwise during attack while tail remains sinusoidal. WT and LAYER menus customizable. Groove expander available without layering.

### VX-T

Wavetable oscillator kick combined with a transient synthesizer. Standard controls apply. COLOR produces transients identically to SP-6.

WT menu provides analog-style options. Transient synthesizer uses 4-operator FM configuration where TRS DECAY creates toc sounds (counterclockwise) to hihat sounds (clockwise), fed through band-pass filter with TRS TONE controlling center frequency. Groove expander excludes transient synthesis.

### WT-4

Wavetable oscillator kick with analog-sounding wavetables combined with drum layering samples (synthesized transients: FM hihats/snares).

Standard controls. COLOR behavior matches SP-6/VX-T. WT and LAYER menus adjustable. Groove expander available without layering.

### XT-88

Wavetable oscillator kick combined with drum layering samples, supporting user-loaded content.

Two SD card folders: `wavetables` and `samples` (WAVE format only).

**Wavetable requirements**:
- Either generated from Xfer Records Serum or Serum 2 (2048 cycle length assumed)
- Or mono, 32-bit float, 1024 cycle length
- Maximum 16 wavetables; 1.4 MB capacity

**Sample requirements**:
- Mono or stereo, 16/24/32-bit, 48kHz
- Maximum 256 samples; 14 MB capacity

Standard controls apply. WT menu displays loaded wavetables; COLOR adjusts wavetable position with modulated EQ filter (brightness via BRIGHT menu). LAYER menu shows loaded samples. Groove expander uses oscillator only.

## Running Modes

Three operational modes: **Studio** (default), **Song**, and **Jam**.

### Studio Mode

Default mode on power-on. Enables immediate parameter changes and rapid model cycling via the encoder. Best for home and studio environments.

Access from other modes: hold FUNCTION encoder for 2+ seconds, select `STUDIO`.

### Live Modes

Both Song and Jam modes rely on **kick snapshots** that capture model variations and all knob positions. Create snapshots by pressing HIT, holding it, then clicking FUNCTION to select and optionally name a step. Alternatively: `SNAPSHOT` → `SAVE` in Studio mode.

#### Programs

Bohm contains 32 programs, each defined by:
- Number of steps (1–16)
- Step snapshots containing complete kick data
- Knob options: Latch, Relative, or Override
- Follow actions (None, Loop, or next program number)
- Performer mode settings (Include or Exclude)

**Knob behavior**:
- **Latch** (`LAT`): Parameter jumps to current knob position immediately
- **Relative** (`REL`): Parameter adjusts relative to knob position when step loaded
- **Override** (`OVR`): Step parameter ignored; current knob position used instead

The `EXCL` Performer option applies active settings across all program steps. `INCL` recalls saved Performer settings per step.

### Song Mode

Treats each program as a step sequence with two decks — one playing while the other preloads the next kick. Trigger FUNCTION to advance steps; the kick activates upon HIT press.

Follow actions determine end-of-sequence behavior:
- **Loop**: Returns to step one
- **End**: Stops and maintains final step
- **Program number**: Automatically starts specified program

### Jam Mode

Displays programs as kick collections. Rotate FUNCTION encoder to select and load kicks automatically. Click FUNCTION to cue a kick; it activates upon HIT press.

### Program Clear

Programs can be reset: navigate to `LIVE` → `PRG NBR` → list end → `CLEAR` → confirm.

## System Settings

Access by holding FUNCTION for 2+ seconds.

### Pitch CV Option

Default: PITCH CV controls kick octave across full range (±5V). Alternative: track Volt/Octave pitch, selecting which 1V range maps to the octave (0..1V, 1..2V, or 2..3V).

### ATTVERT 2 Option

SUSTAIN attenuverter normally maps to SUSTAIN CV input. Can be reassigned to VELOCITY CV.

### Func Rand Option

Controls randomization on FUNCTION trigger in Studio mode:
- `ALL`: Randomizes Bohm, Groove, and Performer
- `B+G`: Randomizes only Bohm and Groove

### Perf FX Option

Performer FX section activation:
- `INSTANT`: Toggle immediately
- `SYNCED`: Synchronize to next HIT

### Grv Env Option

Groove envelope behavior after 4th tap:
- `FALL`: Envelope falls (default)
- `SUSTAIN`: Envelope sustains at 4th tap level

### Panning Option

Applies to BOHM, GROOVE, and PERFORMER:
- `LEFT`: Hard-panned left channel only
- `CENTER`: Stereo untouched
- `RIGHT`: Hard-panned right channel only

### Screen Saver

Default ON. Disable for video recording or live performances.

### Shop Mode

Prevents module from remembering previous settings. For retailer demos.

### Backup/Restore

- `BACKUP`: Saves entire memory to `backup.bohm` on SD card
- `RESTORE`: Loads `.bohm` file from SD card (one file only)

Backup preserves calibration; restoration excludes it. Module auto-restarts after restoration.

### Factory Reset

Resets all system settings, programs, and snapshots to factory state. Preserves calibration data. Auto-restarts module.

## Firmware Update

### Checking Version

Hold FUNCTION encoder for 2 seconds → navigate right to `ABOUT` → observe version.

### Update Process

1. Download firmware zip
2. Extract archive
3. Power down Eurorack system
4. Remove SD card
5. Copy archive contents to SD card root (`Bohm.bin` must be at top level)
6. Safely eject card
7. Reinstall SD card
8. Power on system

Update takes approximately 20 seconds with no screen activity. Subsequent power cycles skip the update.

## Calibration Mode

### Entering Calibration

1. Power down the Eurorack system
2. Disconnect all patch cables
3. Press and hold FUNCTION while powering on
4. Follow onscreen instructions

Choose to calibrate expanders only (brief FUNCTION press) or all modules (1+ second FUNCTION press).

### Equipment

- Expander calibration: no special equipment needed
- Bohm calibration: requires a stable 3V source (Volt/Octave device)

### Procedure

Turn each knob fully left then right, pressing FUNCTION after each. Complete attenuverter trims. Confirm cable removal for automatic CV calibration. For Bohm, connect 3V source to PITCH CV and adjust to approximately 3000mV.

Calibration data only saves at the final step — cancel at any point if equipment is unavailable.

## Changelog

### V2025.0827.1600

- System setting to "talk" to the model
- `GRV ENV` setting: FALL (default) or SUSTAIN for drone behavior
- Shop Mode for consistent showroom experience
- Immediate program/snapshot saving (was 1 second delay)
- Factory Reset option (excludes calibration)
- Backup/Restore to SD card
- End-of-chain soft clipper prevents hard clipping above 0dB (4.6dB headroom)
- DJ RESO controls resonance 0–100% in 10% increments
- Stereo width variation for Groove and Bohm
- Panning options for each signal path

### V2025.0722.1625

Initial release.
