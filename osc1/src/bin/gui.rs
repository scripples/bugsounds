use bugsound_test::output::run_output;
use bugsound_test::synth::Synth;
use eframe::egui;
use fundsp::prelude::Shared;

struct SynthApp {
    f0: f32,
    f1: f32,
    m0: f32,
    f0s: Shared,
    f1s: Shared,
    m0s: Shared,
}

impl eframe::App for SynthApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Synth");
            if ui
                .add(egui::Slider::new(&mut self.f0, 20.0..=2000.0).text("Frequency (Hz)"))
                .changed()
            {
                self.f0s.set(self.f0);
            }
            if ui
                .add(egui::Slider::new(&mut self.f1, 20.0..=2000.0).text("Frequency (Hz)"))
                .changed()
            {
                self.f1s.set(self.f1);
            }
            if ui
                .add(egui::Slider::new(&mut self.m0, 20.0..=2000.0).text("Frequency (Hz)"))
                .changed()
            {
                self.f0s.set(self.m0);
            }
        });
    }
}

fn main() -> eframe::Result {
    let mut synth = Synth::new(440.0);
    let f0s = synth.f0();
    let f1s = synth.f1();
    let m0s = synth.m0();
    let graph = synth.take_graph();

    run_output(graph);

    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Synth",
        options,
        Box::new(|_cc| {
            Ok(Box::new(SynthApp {
                f0: 440.0,
                f0s,
                f1: 440.0,
                f1s,
                m0: 1.0,
                m0s,
            }))
        }),
    )
}
