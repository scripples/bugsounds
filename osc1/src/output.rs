use cpal::{
    Device, FromSample, SampleFormat, SizedSample, StreamConfig,
    traits::{DeviceTrait, HostTrait, StreamTrait},
};
use fundsp::prelude::AudioUnit;

/// This function determines the sample format, which depends on your system,
/// then starts the synth, passing along the audio graph that will generate
/// the sound to be played.
pub fn run_output(audio_graph: Box<dyn AudioUnit>) {
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .expect("failed to find a default output device");
    let config = device.default_output_config().unwrap();
    match config.sample_format() {
        SampleFormat::F32 => run_synth::<f32>(audio_graph, device, config.into()),
        SampleFormat::I16 => run_synth::<i16>(audio_graph, device, config.into()),
        SampleFormat::U16 => run_synth::<u16>(audio_graph, device, config.into()),
        _ => panic!("Unsupported format"),
    }
}

/// This function takes an audio graph as an input, along with some the audio
/// device and config, and starts a thread that will play the audio. The thread
/// will loop infinitely until the programme exits.
fn run_synth<T: SizedSample + FromSample<f64>>(
    mut audio_graph: Box<dyn AudioUnit>,
    device: Device,
    config: StreamConfig,
) {
    std::thread::spawn(move || {
        let sample_rate = config.sample_rate as f64;
        audio_graph.set_sample_rate(sample_rate);

        // This is a function that is used to get the next audio sample. It is
        // written using the closure syntax, so looks a bit different from
        // normal function definition.
        let mut next_value = move || audio_graph.get_stereo();

        let channels = config.channels as usize;
        let err_fn = |err| eprintln!("an error occurred on stream: {err}");
        let stream = device
            .build_output_stream(
                &config,
                move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                    write_data(data, channels, &mut next_value)
                },
                err_fn,
                None,
            )
            .unwrap();

        stream.play().unwrap();
        loop {
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
    });
}

/// This function is passed to the output audio stream and is used to generate
/// audio samples and send them to the audio device.
fn write_data<T: SizedSample + FromSample<f32>>(
    output: &mut [T],
    channels: usize,
    next_sample: &mut dyn FnMut() -> (f32, f32),
) {
    for frame in output.chunks_mut(channels) {
        let sample = next_sample();
        let left: T = T::from_sample(sample.0);
        let right: T = T::from_sample(sample.1);

        for (channel, sample) in frame.iter_mut().enumerate() {
            *sample = if channel & 1 == 0 { left } else { right };
        }
    }
}
