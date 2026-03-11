use std::time::Instant;

use bohmbino::groove::GrooveFx;
use bohmbino::kick::{Bohmbino, Model, Snoop};
use bohmbino::output::run_output;
use eframe::egui;
use fundsp::prelude::Shared;

const SCOPE_DISPLAY: usize = 512;
const SNAPSHOT_SIZE: usize = 4096;

#[derive(Clone, Copy, PartialEq)]
enum TriggerMode {
    Pulse,
    Run,
}

struct BohmbApp {
    // Model
    model_idx: usize,
    model_select_s: Shared,
    // Bohm params
    pitch: f32,
    curve: f32,
    length: f32,
    sustain: f32,
    attack: f32,
    velocity: f32,
    color: f32,
    fx_amount: f32,
    trs_decay: f32,
    trs_tone: f32,
    amp: f32,
    pitch_s: Shared,
    curve_s: Shared,
    length_s: Shared,
    sustain_s: Shared,
    attack_s: Shared,
    velocity_s: Shared,
    color_s: Shared,
    fx_amount_s: Shared,
    trs_decay_s: Shared,
    trs_tone_s: Shared,
    amp_s: Shared,
    trigger_s: Shared,
    // Groove params
    groove_on: bool,
    groove_enabled_s: Shared,
    grv_pitch: f32,
    grv_color: f32,
    grv_length: f32,
    grv_fx_amount: f32,
    grv_fx_idx: usize,
    grv_vol: f32,
    grv_tap2: f32,
    grv_tap3: f32,
    grv_tap4: f32,
    grv_pitch_s: Shared,
    grv_color_s: Shared,
    grv_length_s: Shared,
    grv_fx_amount_s: Shared,
    grv_fx_type_s: Shared,
    grv_vol_s: Shared,
    grv_tap2_s: Shared,
    grv_tap3_s: Shared,
    grv_tap4_s: Shared,
    grv_trigger_s: Shared,
    grv_clock_s: Shared,
    // Trigger
    trigger_mode: TriggerMode,
    bpm: f32,
    last_trigger: Instant,
    last_clock: Instant,
    // Scope
    scope_scale: f32,
    scope_freeze: bool,
    scope_refresh_ms: u64,
    scope_snapshot: Vec<f32>,
    scope_last_capture: Instant,
    snoop: Snoop,
}

impl BohmbApp {
    fn slider(
        ui: &mut egui::Ui,
        val: &mut f32,
        range: std::ops::RangeInclusive<f32>,
        label: &str,
        shared: &Shared,
    ) {
        if ui
            .add(egui::Slider::new(val, range).text(label))
            .changed()
        {
            shared.set(*val);
        }
    }
}

impl eframe::App for BohmbApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Auto-trigger in Run mode
        if self.trigger_mode == TriggerMode::Run {
            let interval_ms = (60_000.0 / self.bpm) as u128;
            if self.last_trigger.elapsed().as_millis() >= interval_ms {
                self.trigger_s.set(1.0);
                if self.groove_on {
                    self.grv_trigger_s.set(1.0);
                }
                self.last_trigger = Instant::now();
            }
            // Clock at 4x BPM for groove taps
            let clock_interval = interval_ms / 4;
            if self.groove_on && self.last_clock.elapsed().as_millis() >= clock_interval {
                self.grv_clock_s.set(1.0);
                self.last_clock = Instant::now();
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Bohmbino");

            // ── Model Selector ──
            ui.separator();
            ui.horizontal(|ui| {
                ui.label("Model:");
                for (i, model) in Model::ALL.iter().enumerate() {
                    if ui
                        .selectable_label(self.model_idx == i, model.name())
                        .clicked()
                    {
                        self.model_idx = i;
                        self.model_select_s.set(i as f32);
                    }
                }
            });

            // ── Trigger ──
            ui.separator();
            ui.horizontal(|ui| {
                if ui.button("HIT").clicked() {
                    self.trigger_s.set(1.0);
                    if self.groove_on {
                        self.grv_trigger_s.set(1.0);
                    }
                    self.last_trigger = Instant::now();
                }

                ui.separator();

                ui.radio_value(&mut self.trigger_mode, TriggerMode::Pulse, "Pulse");
                ui.radio_value(&mut self.trigger_mode, TriggerMode::Run, "Run");

                if self.trigger_mode == TriggerMode::Run {
                    ui.add(egui::Slider::new(&mut self.bpm, 20.0..=300.0).text("BPM"));
                }
            });

            // ── Bohm Parameters ──
            ui.separator();
            ui.heading("Bohm");

            ui.label("Oscillator");
            Self::slider(ui, &mut self.pitch, 32.70..=130.81, "Pitch (Hz)", &self.pitch_s);
            Self::slider(ui, &mut self.curve, 0.0..=1.0, "Curve (808 ← → 909)", &self.curve_s);
            Self::slider(ui, &mut self.color, 0.0..=1.0, "Color", &self.color_s);

            ui.separator();
            ui.label("Envelope");
            Self::slider(ui, &mut self.attack, 0.0..=1.0, "Attack", &self.attack_s);
            Self::slider(ui, &mut self.sustain, 0.0..=1.0, "Sustain", &self.sustain_s);
            Self::slider(ui, &mut self.length, 0.01..=2.0, "Length (s)", &self.length_s);
            Self::slider(ui, &mut self.velocity, 0.0..=1.0, "Velocity", &self.velocity_s);

            ui.separator();
            ui.label("Transient");
            Self::slider(ui, &mut self.trs_decay, 0.0..=1.0, "TRS Decay", &self.trs_decay_s);
            Self::slider(ui, &mut self.trs_tone, 0.0..=1.0, "TRS Tone", &self.trs_tone_s);

            ui.separator();
            ui.label("Output");
            Self::slider(ui, &mut self.fx_amount, 0.0..=1.0, "FX Amount", &self.fx_amount_s);
            Self::slider(ui, &mut self.amp, 0.0..=1.0, "Amplitude", &self.amp_s);

            // ── Groove ──
            ui.separator();
            ui.horizontal(|ui| {
                ui.heading("Groove");
                if ui
                    .checkbox(&mut self.groove_on, "Enable")
                    .changed()
                {
                    self.groove_enabled_s
                        .set(if self.groove_on { 1.0 } else { 0.0 });
                }
            });

            if self.groove_on {
                Self::slider(ui, &mut self.grv_pitch, 20.0..=130.81, "Pitch (Hz)", &self.grv_pitch_s);
                Self::slider(ui, &mut self.grv_color, 0.0..=1.0, "Color", &self.grv_color_s);
                Self::slider(ui, &mut self.grv_length, 0.01..=2.0, "Length (s)", &self.grv_length_s);
                Self::slider(ui, &mut self.grv_vol, 0.0..=1.0, "Volume", &self.grv_vol_s);

                ui.separator();
                ui.label("Taps");
                Self::slider(ui, &mut self.grv_tap2, 0.0..=1.0, "Tap 2", &self.grv_tap2_s);
                Self::slider(ui, &mut self.grv_tap3, 0.0..=1.0, "Tap 3", &self.grv_tap3_s);
                Self::slider(ui, &mut self.grv_tap4, 0.0..=1.0, "Tap 4", &self.grv_tap4_s);

                ui.separator();
                ui.horizontal(|ui| {
                    ui.label("FX:");
                    for (i, fx) in GrooveFx::ALL.iter().enumerate() {
                        if ui
                            .selectable_label(self.grv_fx_idx == i, fx.name())
                            .clicked()
                        {
                            self.grv_fx_idx = i;
                            self.grv_fx_type_s.set(i as f32);
                        }
                    }
                });
                Self::slider(ui, &mut self.grv_fx_amount, 0.0..=1.0, "FX Amount", &self.grv_fx_amount_s);
            }

            // ── Oscilloscope ──
            ui.separator();
            ui.label("Oscilloscope");

            self.snoop.update();
            if !self.scope_freeze
                && self.scope_last_capture.elapsed().as_millis()
                    >= self.scope_refresh_ms as u128
            {
                self.scope_snapshot.clear();
                for i in (0..SNAPSHOT_SIZE).rev() {
                    self.scope_snapshot.push(self.snoop.at(i));
                }
                self.scope_last_capture = Instant::now();
            }

            ui.horizontal(|ui| {
                ui.add(
                    egui::Slider::new(&mut self.scope_scale, 1.0..=32.0)
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

            let desired = egui::vec2(ui.available_width(), 150.0);
            let (rect, _response) =
                ui.allocate_exact_size(desired, egui::Sense::hover());
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
            let trigger = {
                let threshold = 0.01;
                let mut t = 0;
                for i in 0..snap.len().saturating_sub(1) {
                    if snap[i].abs() > threshold {
                        t = i;
                        break;
                    }
                }
                t
            };

            let base = trigger;
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
                    let x = rect.left()
                        + (i as f32 / SCOPE_DISPLAY as f32) * rect.width();
                    let y = mid_y - sample * (rect.height() * 0.45);
                    Some(egui::pos2(x, y))
                })
                .collect();

            let stroke =
                egui::Stroke::new(1.5, egui::Color32::from_rgb(255, 120, 0));
            for window in points.windows(2) {
                painter.line_segment([window[0], window[1]], stroke);
            }
        });

        ctx.request_repaint();
    }
}

fn main() -> eframe::Result {
    let mut synth = Bohmbino::new();

    let pitch_s = synth.pitch();
    let curve_s = synth.curve();
    let length_s = synth.length();
    let sustain_s = synth.sustain();
    let attack_s = synth.attack();
    let velocity_s = synth.velocity();
    let color_s = synth.color();
    let fx_amount_s = synth.fx_amount();
    let trs_decay_s = synth.trs_decay();
    let trs_tone_s = synth.trs_tone();
    let amp_s = synth.amp();
    let trigger_s = synth.trigger();
    let model_select_s = synth.model_select();

    let groove_enabled_s = synth.groove_enabled();
    let grv_pitch_s = synth.grv_pitch();
    let grv_color_s = synth.grv_color();
    let grv_length_s = synth.grv_length();
    let grv_fx_amount_s = synth.grv_fx_amount();
    let grv_fx_type_s = synth.grv_fx_type();
    let grv_vol_s = synth.grv_vol();
    let grv_tap2_s = synth.grv_tap2();
    let grv_tap3_s = synth.grv_tap3();
    let grv_tap4_s = synth.grv_tap4();
    let grv_trigger_s = synth.grv_trigger();
    let grv_clock_s = synth.grv_clock();

    let snoop = synth.take_snoop();
    let graph = synth.take_graph();
    run_output(graph);

    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Bohmbino",
        options,
        Box::new(|_cc| {
            Ok(Box::new(BohmbApp {
                model_idx: 0,
                model_select_s,
                pitch: 55.0,
                curve: 0.0,
                length: 0.3,
                sustain: 0.0,
                attack: 0.5,
                velocity: 1.0,
                color: 0.0,
                fx_amount: 0.0,
                trs_decay: 0.3,
                trs_tone: 0.3,
                amp: 0.8,
                pitch_s,
                curve_s,
                length_s,
                sustain_s,
                attack_s,
                velocity_s,
                color_s,
                fx_amount_s,
                trs_decay_s,
                trs_tone_s,
                amp_s,
                trigger_s,
                groove_on: false,
                groove_enabled_s,
                grv_pitch: 55.0,
                grv_color: 0.0,
                grv_length: 0.3,
                grv_fx_amount: 0.0,
                grv_fx_idx: 0,
                grv_vol: 0.5,
                grv_tap2: 0.7,
                grv_tap3: 0.4,
                grv_tap4: 0.2,
                grv_pitch_s,
                grv_color_s,
                grv_length_s,
                grv_fx_amount_s,
                grv_fx_type_s,
                grv_vol_s,
                grv_tap2_s,
                grv_tap3_s,
                grv_tap4_s,
                grv_trigger_s,
                grv_clock_s,
                trigger_mode: TriggerMode::Pulse,
                bpm: 120.0,
                last_trigger: Instant::now(),
                last_clock: Instant::now(),
                scope_scale: 4.0,
                scope_freeze: false,
                scope_refresh_ms: 100,
                scope_snapshot: vec![0.0; SNAPSHOT_SIZE],
                scope_last_capture: Instant::now(),
                snoop,
            }))
        }),
    )
}
