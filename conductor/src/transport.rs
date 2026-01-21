use crate::clock::Clock;

#[derive(Clone, Copy, Debug)]
pub struct Transport {
    pub running: bool,
    pub len: u8,
    pub step: u8,

    // user params
    pub bpm: f32,
    pub steps_per_beat: u8,
    pub swing_permille: i16, // [-500..500]? idk lol

    // schedule
    pub next_step_tick: u64,

    // cached timing
    cached_tps: u64,
    base_step_ticks: u64,
    swing_delta_ticks: i64,
    dirty: bool,
}

impl Default for Transport {
    fn default() -> Self {
        Self {
            running: false,
            len: 16,
            step: 0,
            bpm: 120.0,
            steps_per_beat: 4,
            swing_permille: 0,
            next_step_tick: 0,
            cached_tps: 0,
            base_step_ticks: 1,
            swing_delta_ticks: 0,
            dirty: true,
        }
    }
}

impl Transport {
    pub fn new() -> Self {
        Self {
            running: false,
            len: 16,
            step: 0,
            bpm: 120.0,
            steps_per_beat: 4,
            swing_permille: 0,
            next_step_tick: 0,
            cached_tps: 0,
            base_step_ticks: 1,
            swing_delta_ticks: 0,
            dirty: true,
        }
    }

    pub fn set_bpm<C: Clock>(&mut self, clock: &C) {}

    pub fn start<C: Clock>(&mut self, clock: &C) {
        self.step = 0;
        self.running = true;
    }

    pub fn advance<C: Clock>(&mut self, clock: &C) {
        let next = (self.step + 1) % self.len;
        self.step = next;
    }
}
