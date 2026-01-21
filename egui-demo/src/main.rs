use conductor::std_clock::StdInstantClock;
use conductor::{event::Event, Sequence, Sequencer};

use eframe::egui;

struct App {
    seq: Sequencer,
    clock: StdInstantClock,

    // UI pulse state (std-only, fine here)
    last_pulse: std::time::Instant,
    pulse_ms: u64,
}

impl App {
    fn new() -> Self {
        let mut pattern = [false; 64];

        // 4 onna floor
        for i in (0..64).step_by(4) {
            pattern[i] = true;
        }

        let mut seq = Sequencer::new(Sequence(pattern));
        seq.tr.len = 64;
        seq.tr.bpm_x1000 = 120_000;
        seq.tr.steps_per_beat = 4; // 16ths
        seq.tr.swing_permille = 180; // lag offbeats
        seq.emit_steps = false; // pulse only on triggers

        let clock = StdInstantClock::new();
        seq.tr.start(&clock);

        Self {
            seq,
            clock,
            last_pulse: std::time::Instant::now() - std::time::Duration::from_secs(10),
            pulse_ms: 120,
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // poll sequencer and capture triggers
        while let Some(ev) = self.seq.poll(&self.clock) {
            if matches!(ev, Event::Trigger { .. }) {
                self.last_pulse = std::time::Instant::now();
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Sequencer demo");
            ui.horizontal(|ui| {
                ui.label("BPM");
                let mut bpm = self.seq.tr.bpm_x1000 as f32 / 1000.0;
                if ui.add(egui::Slider::new(&mut bpm, 30.0..=240.0)).changed() {
                    self.seq
                        .tr
                        .set_bpm_x1000((bpm * 1000.0).round().max(1.0) as u32);
                }
            });

            ui.horizontal(|ui| {
                ui.label("Swing (permille)");
                let mut s = self.seq.tr.swing_permille as i32;
                if ui.add(egui::Slider::new(&mut s, -10000..=10000)).changed() {
                    self.seq.tr.set_swing_permille(s as i16);
                }
            });

            ui.separator();

            // Draw pulsing circle
            let (rect, _resp) = ui.allocate_exact_size(ui.available_size(), egui::Sense::hover());
            let painter = ui.painter_at(rect);

            let elapsed = self.last_pulse.elapsed();
            let t = (elapsed.as_millis() as u64).min(self.pulse_ms);

            let intensity = 1.0 - (t as f32 / self.pulse_ms as f32);

            let center = rect.center();
            let radius = rect.width().min(rect.height()) * (0.1 + 0.08 * intensity);

            let color = egui::Color32::from_white_alpha(255.0 as u8);
            painter.circle_filled(center, radius, color);

            ui.allocate_space(egui::vec2(0.0, 1.0));
        });

        // keep repainting
        ctx.request_repaint();
    }
}

fn main() -> eframe::Result<()> {
    let opts = eframe::NativeOptions::default();
    eframe::run_native(
        "Sequencer EGUI Demo",
        opts,
        Box::new(|_cc| Box::new(App::new())),
    )
}
