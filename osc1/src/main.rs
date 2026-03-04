use fundsp::prelude::*;

/// This is the main function that is the entry point when we launch the
/// binary, either directly or with `cargo run`.
fn main() {
    // Change the `create_sine_440` function to any of the functions
    // that create a `Box<dyn AudioUnit>` below, to change the
    // sound that's generated.
    let audio_graph = create_sine_440();

    // This function starts the thread that creates the audio and sends
    // it to CPAL so that we can hear it.
    bugsound_test::output::run_output(audio_graph);

    // The audio is being played on a thread, and will run infinitely.
    // As soon as the main function exits, the sound will stop, so we
    // can sleep the main thread for a while so we can hear it.
    // Change the duration to play the sound for more or less time.
    let duration = 5;
    std::thread::sleep(std::time::Duration::from_secs(duration));
}

// ------------------------------------------------------------------
// You can use any of the functions in this section to make the audio
// graph. Just replace the function call in `main` at the top.

/// Simple sine wave at 440 Hz which is standard tuning for A4
fn create_sine_440() -> Box<dyn AudioUnit> {
    let synth = sine_hz::<f32>(440.0);
    Box::new(synth)
}

/// C major chord created by summing waves! Sine by default, but try uncommenting
/// the other wave types.
// fn create_c_major() -> Box<dyn AudioUnit> {
//     let synth = sine_hz::<f32>(261.6) + sine_hz<f32>(329.628) + sine_hz<f32>(391.995);
//     // let synth = square_hz(261.6) + square_hz(329.628) + square_hz(391.995);
//     // let synth = soft_saw_hz(261.6) + soft_saw_hz(329.628) + soft_saw_hz(391.995);
//     // let synth = hammond_hz(261.6) + hammond_hz(329.628) + hammond_hz(391.995);
//
//     Box::new(synth)
// }

// /// Load and play a sample
// fn create_sample() -> Box<dyn AudioUnit> {
//     let wave =
//         Arc::new(Wave64::load("samples/closed_high_hat.wav").expect("Could not find sample file."));
//     let left = wave64(&wave, 0, None);
//     let right = wave64(&wave, 1, None);
//     let synth = left | right;
//
//     Box::new(synth)
// }
//
// /// Load and play a sample, but this time we add reverb
// fn create_sample_with_reverb() -> Box<dyn AudioUnit> {
//     let wave =
//         Arc::new(Wave64::load("samples/closed_high_hat.wav").expect("Could not find sample file."));
//     let left = wave64(&wave, 0, None);
//     let right = wave64(&wave, 1, None);
//     let synth = (left | right) >> (multipass() & (0.2 * reverb_stereo(10.0, 3.0)));
//
//     Box::new(synth)
// }

// Simple FM synthesiser taken from the FunDSP docs
fn create_simple_fm() -> Box<dyn AudioUnit> {
    // Frequency
    let f = 440.0;
    let g = 880;
    // Modulation index
    let m = 5.0;
    let synth = (sine_hz::<f32>(f) * f * m + f) >> sine::<f32>();

    Box::new(synth)
}
