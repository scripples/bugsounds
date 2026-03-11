use std::time::Instant;

use bugsound_test::output::run_output;
use bugsound_test::wavetable3d::{PistonHonda, Snoop};
use eframe::egui;
use fundsp::prelude::Shared;

const SCOPE_DISPLAY: usize = 512;
const SNAPSHOT_SIZE: usize = 4096;

struct WavetableApp {
    // OSC A
    freq_a: f32,
    x_a: f32,
    y_a: f32,
    z_a: f32,
    octave_a: i32,
    amp_a: f32,
    freq_a_s: Shared,
    x_a_s: Shared,
    y_a_s: Shared,
    z_a_s: Shared,
    octave_a_s: Shared,
    amp_a_s: Shared,
    // OSC B
    freq_b: f32,
    x_b: f32,
    y_b: f32,
    z_b: f32,
    octave_b: i32,
    amp_b: f32,
    freq_b_s: Shared,
    x_b_s: Shared,
    y_b_s: Shared,
    z_b_s: Shared,
    octave_b_s: Shared,
    amp_b_s: Shared,
    // Link
    link: bool,
    fine_tune_b: f32,
    link_s: Shared,
    fine_tune_b_s: Shared,
    // Output
    output_select: i32,
    output_select_s: Shared,
    // Interpolation
    interp_nearest: bool,
    interp_mode_s: Shared,
    // FM
    fm_amount: f32,
    fm_amount_s: Shared,
    // Shared
    amp: f32,
    amp_s: Shared,
    // Scope
    scope_scale: f32,
    scope_offset: usize,
    scope_refresh_ms: u64,
    scope_freeze: bool,
    scope_snapshot: Vec<f32>,
    scope_last_capture: Instant,
    snoop: Snoop,
}

impl eframe::App for WavetableApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.spacing_mut().slider_width = ui.available_width() - 200.0;
            ui.heading("Piston Honda");

            ui.separator();

            if ui
                .add(egui::Slider::new(&mut self.amp, 0.0..=1.0).text("Amplitude"))
                .changed()
            {
                self.amp_s.set(self.amp);
            }

            ui.horizontal(|ui| {
                ui.label("Output:");
                let changed = ui.radio_value(&mut self.output_select, 0, "Mix").changed()
                    | ui.radio_value(&mut self.output_select, 1, "OSC A").changed()
                    | ui.radio_value(&mut self.output_select, 2, "OSC B").changed();
                if changed {
                    self.output_select_s.set(self.output_select as f32);
                }
            });

            if ui
                .checkbox(&mut self.interp_nearest, "Nearest waveform (no morphing)")
                .changed()
            {
                self.interp_mode_s
                    .set(if self.interp_nearest { 1.0 } else { 0.0 });
            }

            if ui
                .add(egui::Slider::new(&mut self.fm_amount, 0.0..=10.0).text("FM (B→A)"))
                .changed()
            {
                self.fm_amount_s.set(self.fm_amount);
            }

            // ── OSC A ──
            ui.separator();
            ui.heading("OSC A");

            if ui
                .add(egui::Slider::new(&mut self.freq_a, 20.0..=2000.0).text("Frequency (Hz)"))
                .changed()
            {
                self.freq_a_s.set(self.freq_a);
            }
            if ui
                .add(egui::Slider::new(&mut self.octave_a, -2..=2).text("Octave"))
                .changed()
            {
                self.octave_a_s.set(self.octave_a as f32);
            }
            if ui
                .add(egui::Slider::new(&mut self.amp_a, 0.0..=1.0).text("Level"))
                .changed()
            {
                self.amp_a_s.set(self.amp_a);
            }

            ui.label("Wavetable Position");
            if ui
                .add(egui::Slider::new(&mut self.x_a, 1.0..=8.0).text("X"))
                .changed()
            {
                self.x_a_s.set(self.x_a - 1.0);
            }
            if ui
                .add(egui::Slider::new(&mut self.y_a, 1.0..=8.0).text("Y"))
                .changed()
            {
                self.y_a_s.set(self.y_a - 1.0);
            }
            if ui
                .add(egui::Slider::new(&mut self.z_a, 1.0..=8.0).text("Z"))
                .changed()
            {
                self.z_a_s.set(self.z_a - 1.0);
            }
            // ── OSC B ──
            ui.separator();
            ui.heading("OSC B");

            // Link toggle
            if ui
                .checkbox(&mut self.link, "LINK (OSC B follows OSC A freq)")
                .changed()
            {
                self.link_s.set(if self.link { 1.0 } else { 0.0 });
            }

            if self.link {
                // Fine tune slider when linked
                if ui
                    .add(
                        egui::Slider::new(&mut self.fine_tune_b, -50.0..=50.0)
                            .text("Fine Tune (Hz)"),
                    )
                    .changed()
                {
                    self.fine_tune_b_s.set(self.fine_tune_b);
                }
            } else {
                // Independent frequency when not linked
                if ui
                    .add(egui::Slider::new(&mut self.freq_b, 20.0..=2000.0).text("Frequency (Hz)"))
                    .changed()
                {
                    self.freq_b_s.set(self.freq_b);
                }
            }
            if ui
                .add(egui::Slider::new(&mut self.octave_b, -2..=2).text("Octave"))
                .changed()
            {
                self.octave_b_s.set(self.octave_b as f32);
            }
            if ui
                .add(egui::Slider::new(&mut self.amp_b, 0.0..=1.0).text("Level"))
                .changed()
            {
                self.amp_b_s.set(self.amp_b);
            }

            ui.label("Wavetable Position");
            if ui
                .add(egui::Slider::new(&mut self.x_b, 1.0..=8.0).text("X"))
                .changed()
            {
                self.x_b_s.set(self.x_b - 1.0);
            }
            if ui
                .add(egui::Slider::new(&mut self.y_b, 1.0..=8.0).text("Y"))
                .changed()
            {
                self.y_b_s.set(self.y_b - 1.0);
            }
            if ui
                .add(egui::Slider::new(&mut self.z_b, 1.0..=8.0).text("Z"))
                .changed()
            {
                self.z_b_s.set(self.z_b - 1.0);
            }
            // ── Oscilloscope ──
            ui.separator();
            ui.label("Oscilloscope");

            self.snoop.update();
            if !self.scope_freeze
                && self.scope_last_capture.elapsed().as_millis() >= self.scope_refresh_ms as u128
            {
                self.scope_snapshot.clear();
                for i in (0..SNAPSHOT_SIZE).rev() {
                    self.scope_snapshot.push(self.snoop.at(i));
                }
                self.scope_last_capture = Instant::now();
            }

            ui.horizontal(|ui| {
                ui.add(
                    egui::Slider::new(&mut self.scope_scale, 0.01..=32.0)
                        .logarithmic(true)
                        .text("Zoom"),
                );
                ui.checkbox(&mut self.scope_freeze, "Freeze");
            });
            ui.add(
                egui::Slider::new(&mut self.scope_refresh_ms, 16..=1000)
                    .logarithmic(true)
                    .text("Refresh (ms)"),
            );
            ui.add(egui::Slider::new(&mut self.scope_offset, 0..=256).text("Offset"));

            let desired = egui::vec2(ui.available_width(), 150.0);
            let (rect, _response) = ui.allocate_exact_size(desired, egui::Sense::hover());
            let painter = ui.painter_at(rect);

            painter.rect_filled(rect, 2.0, egui::Color32::from_gray(20));

            let mid_y = rect.center().y;
            painter.line_segment(
                [
                    egui::pos2(rect.left(), mid_y),
                    egui::pos2(rect.right(), mid_y),
                ],
                egui::Stroke::new(0.5, egui::Color32::from_gray(60)),
            );

            let snap = &self.scope_snapshot;
            let period_samples = 44100.0 / self.freq_a;
            let trigger = {
                let search_len = (period_samples as usize + 1).min(snap.len().saturating_sub(1));
                let mut t = 0;
                for i in 1..search_len {
                    if snap[i - 1] <= 0.0 && snap[i] > 0.0 {
                        t = i;
                        break;
                    }
                }
                t
            };
            let base = trigger + self.scope_offset;
            let points: Vec<egui::Pos2> = (0..SCOPE_DISPLAY)
                .filter_map(|i| {
                    let src = i as f32 / self.scope_scale;
                    let idx0 = base + src as usize;
                    let idx1 = idx0 + 1;
                    if idx1 >= snap.len() {
                        return None;
                    }
                    let frac = src - src.floor();
                    let sample = snap[idx0] + (snap[idx1] - snap[idx0]) * frac;
                    let x = rect.left() + (i as f32 / SCOPE_DISPLAY as f32) * rect.width();
                    let y = mid_y - sample * (rect.height() * 0.45);
                    Some(egui::pos2(x, y))
                })
                .collect();

            let stroke = egui::Stroke::new(1.5, egui::Color32::from_rgb(0, 255, 100));
            for window in points.windows(2) {
                painter.line_segment([window[0], window[1]], stroke);
            }
        });

        ctx.request_repaint();
    }
}

fn main() -> eframe::Result {
    let mut synth = PistonHonda::from_directory("assets/piston-honda-factory", 69.30)
        .expect("failed to load wavetables");

    let freq_a_s = synth.f0_a();
    let x_a_s = synth.x_a();
    let y_a_s = synth.y_a();
    let z_a_s = synth.z_a();
    let octave_a_s = synth.octave_a();
    let amp_a_s = synth.amp_a();

    let freq_b_s = synth.f0_b();
    let x_b_s = synth.x_b();
    let y_b_s = synth.y_b();
    let z_b_s = synth.z_b();
    let octave_b_s = synth.octave_b();
    let amp_b_s = synth.amp_b();

    let amp_s = synth.amp();
    let output_select_s = synth.output_select();
    let interp_mode_s = synth.interp_mode();
    let fm_amount_s = synth.fm_amount();
    let link_s = synth.link();
    let fine_tune_b_s = synth.fine_tune_b();
    let snoop = synth.take_snoop();

    let graph = synth.take_graph();
    run_output(graph);

    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Piston Honda",
        options,
        Box::new(|_cc| {
            Ok(Box::new(WavetableApp {
                freq_a: 69.30,
                x_a: 1.0,
                y_a: 1.0,
                z_a: 1.0,
                octave_a: 0,
                amp_a: 1.0,
                freq_a_s,
                x_a_s,
                y_a_s,
                z_a_s,
                octave_a_s,
                amp_a_s,
                freq_b: 69.30,
                x_b: 1.0,
                y_b: 1.0,
                z_b: 1.0,
                octave_b: 0,
                amp_b: 1.0,
                freq_b_s,
                x_b_s,
                y_b_s,
                z_b_s,
                octave_b_s,
                amp_b_s,
                link: false,
                fine_tune_b: 0.0,
                link_s,
                fine_tune_b_s,
                output_select: 0,
                output_select_s,
                interp_nearest: false,
                interp_mode_s,
                fm_amount: 0.0,
                fm_amount_s,
                amp: 0.5,
                amp_s,
                scope_scale: 0.27,
                scope_offset: 0,
                scope_refresh_ms: 100,
                scope_freeze: false,
                scope_snapshot: vec![0.0; SNAPSHOT_SIZE],
                scope_last_capture: Instant::now(),
                snoop,
            }))
        }),
    )
}
