# Piston Honda Mark III - Operations Manual Summary

Firmware V1.1 — Industrial Music Electronics (Model 1991 Mark III)

## Overview

Dual wavetable oscillator module with three-axis morphing and nonlinear waveshapers. Contains two complete oscillators (OSC A, OSC B) and a preset controller.

Wavetable synthesis reads digital waveform data from memory instead of generating classic analog shapes. The results are harmonically complex, less dependent on filtering, and respond well to chorus and unison effects.

## Oscillator Operation

- Two frequency controls: coarse tune (large knob) and fine tune (small knob). Roles can be swapped in the Global Options menu.
- Frequency is determined by: coarse + fine + 1V/Oct input + CV input + octave shift + preset data.
- Minimum frequency ~16.35 Hz (C0) at lowest coarse setting, middle fine tune, 0V input.
- 1V/Oct input tracks pitch. CV input is bipolar with attenuverter.
- FM input provides audio-rate thru-zero FM. Normalled to the other oscillator's output.

## External Input Mode

Press MODE to switch an oscillator to external input. The FM input (or normalled other oscillator) replaces the internal oscillator. The wavetable acts as a **waveshaper/transfer function** on the input signal.

For the classic nonlinear waveshaping sound: feed a sine wave in, select a wavetable, and modulate with an envelope on the CV input.

## Waveform Selection

512 waveforms organized as an 8x8x8 cube across axes X, Y, and Z. Each axis has:
- A manual control slider
- A dedicated CV input with attenuverter

The two SELECT buttons near the sliders determine which oscillator receives waveform changes. Controls lock when SELECT buttons are pressed or a preset is loaded; move a slider to unlock.

## Waveform Axes

- **Z axis**: Selects which of the 8 loaded WAV files (banks) to read from.
- **Y axis**: Selects which group of 8 waves within a file.
- **X axis**: Selects which wave within a group.

When morphing is enabled, positions between integer values crossfade (lerp) between adjacent waveforms.

## Oscillator Link

OSC B can follow OSC A's frequency via the LINK button. OSC B's fine tune and CV still apply on top. FM on OSC A does not affect the linked frequency.

## Oscillator Options Menu

Access by holding encoder + pressing OSC MODE button.

### Unison
Adds a second oscillator running alongside the main one with a slight frequency deviation. The displayed number is the detune amount. +OCT/-OCT options set unison one octave above/below.

### Octave Shift
+/- 2 octave range. Oscillator goes silent below ~14 Hz.

### Morph Enable
Each axis (X, Y, Z) can have morphing independently enabled or disabled. Disabled = hard transitions between the 8 waveforms (glitchy). Enabled = smooth interpolation between neighbors.

### Waveform CV
Can be set to "Off" per oscillator to ignore incoming X/Y/Z CV.

### Tone
Several subtle distortion flavors. "Orthodox" = full 16-bit resolution. Other settings add quantization noise characteristic of earlier Piston Honda generations. Best heard with sine waves at low frequencies.

## File Format

- 1-channel (mono), 16-bit WAV
- 256 samples per single-cycle waveform
- 64 waveforms per file (8 groups of 8)
- 8 files total = 8 Z positions
- 32 Kbytes per file (256 samples * 2 bytes * 64 waveforms)

### Layout within each file

Waveforms are stored sequentially:
- Waves 0-7: X=0..7 at Y=0
- Waves 8-15: X=0..7 at Y=1
- ...
- Waves 56-63: X=0..7 at Y=7

File index = `Y * 8 + X`

### Loading

Files must be named `1.wav` through `8.wav` on a FAT-formatted microSD card. All 8 must be present. Load via Global Options > "Load Waves From SD".

## Preset Manager

8 presets stored in onboard EEPROM (not SD card). Enable with PRESET button.

### Preset Morph
Press PRESET again to enter morph mode. Apply CV to smoothly morph between presets. Base preset set by encoder.

### Preset Scope
- "Waves Only": presets control waveform sliders, CV/XYZ attenuverters, unison/octave.
- "All Params": additionally controls coarse/fine frequency.

### Preset CV Control (CTL)
- CV+OFFSET: CV cycles through presets, knob = manual offset.
- CV+ATTEN: like CV+OFFSET but knob attenuates incoming CV.
- TRIG+OFFSET: trigger advances preset, knob scrolls manually.
- TRIG RANDOM: trigger randomizes parameters.

## Calibration

1. Prepare 1V and 7V sources.
2. Set coarse to minimum, fine to center.
3. Adjust fine tune until display reads C1 with 1V input.
4. Apply 7V, adjust rear trimmer (OSC1SCALE/OSC2SCALE) until display centers on C7.
5. Repeat until stable.

## References

- WaveEdit editor: http://synthtech.com/waveedit/
- SCW editor: http://scw.sheetsofsound.com/
- Product page: http://www.industrialmusicelectronics.com/products/21
- Support: support@industrialmusicelectronics.com
