use bugsound_test::output::run_output;
use bugsound_test::wavetable3d::Wavetable3D;
use eframe::egui;
use fundsp::prelude::Shared;

struct WavetableApp {
    freq: f32,
    x: f32,
    y: f32,
    z: f32,
    amp: f32,
    freq_s: Shared,
    x_s: Shared,
    y_s: Shared,
    z_s: Shared,
    amp_s: Shared,
}

impl eframe::App for WavetableApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Wavetable 3D");

            ui.separator();

            if ui
                .add(egui::Slider::new(&mut self.freq, 20.0..=2000.0).text("Frequency (Hz)"))
                .changed()
            {
                self.freq_s.set(self.freq);
            }

            if ui
                .add(egui::Slider::new(&mut self.amp, 0.0..=1.0).text("Amplitude"))
                .changed()
            {
                self.amp_s.set(self.amp);
            }

            ui.separator();
            ui.label("Wavetable Position");

            if ui
                .add(egui::Slider::new(&mut self.x, 0.0..=7.0).text("X"))
                .changed()
            {
                self.x_s.set(self.x);
            }

            if ui
                .add(egui::Slider::new(&mut self.y, 0.0..=7.0).text("Y"))
                .changed()
            {
                self.y_s.set(self.y);
            }

            if ui
                .add(egui::Slider::new(&mut self.z, 0.0..=7.0).text("Z"))
                .changed()
            {
                self.z_s.set(self.z);
            }
        });
    }
}

fn main() -> eframe::Result {
    let mut synth = Wavetable3D::from_directory("assets/piston-honda-factory", 440.0)
        .expect("failed to load wavetables");

    let freq_s = synth.f0();
    let x_s = synth.x();
    let y_s = synth.y();
    let z_s = synth.z();
    let amp_s = synth.amp();

    let graph = synth.take_graph();
    run_output(graph);

    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Wavetable 3D",
        options,
        Box::new(|_cc| {
            Ok(Box::new(WavetableApp {
                freq: 440.0,
                x: 0.0,
                y: 0.0,
                z: 0.0,
                amp: 0.5,
                freq_s,
                x_s,
                y_s,
                z_s,
                amp_s,
            }))
        }),
    )
}
