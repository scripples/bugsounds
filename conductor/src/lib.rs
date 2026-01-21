#![no_std]

use crate::{clock::Clock, event::Event, transport::Transport};

pub mod clock;
pub mod event;
pub mod transport;

/// A very small sequencer: Pattern + Transport.
/// No allocations, no_std-friendly.
pub struct Sequencer {
    pub seq: Sequence,
    pub tr: Transport,
}

impl Sequencer {
    pub fn new(seq: Sequence) -> Self {
        Self {
            seq,
            tr: Transport::new(),
        }
    }

    /// `while let Some(ev) = seq.poll(clock) { ... }`
    pub fn poll<C: Clock>(&mut self, clock: &C) -> Option<Event> {
        if !self.tr.running {
            return None;
        }

        let now = clock.now_ticks();
        if now < self.tr.next_step_tick {
            return None;
        }

        // hit or passed a boundary: emit for the CURRENT step,
        // then schedule and advance to the next step.
        let step = self.tr.step;
        let active = self.seq.is_on(step);

        self.tr.advance(clock);

        if active {
            Some(Event::Trigger { step })
        } else {
            None
        }
    }
}

#[derive(Clone, Copy)]
pub struct Sequence(pub [bool; 64]);

impl Sequence {
    #[inline]
    pub fn is_on(&self, step: u8) -> bool {
        self.0[step as usize]
    }

    #[inline]
    pub fn set(&mut self, step: u8, on: bool) {
        self.0[step as usize] = on;
    }

    pub fn clear(&mut self) {
        self.0 = [false; 64];
    }
}

#[cfg(feature = "std")]
extern crate std;

#[cfg(feature = "std")]
pub mod std_clock {
    use super::Clock;

    pub struct StdInstantClock {
        start: std::time::Instant,
    }

    impl StdInstantClock {
        pub fn new() -> Self {
            Self {
                start: std::time::Instant::now(),
            }
        }
    }

    impl Clock for StdInstantClock {
        fn now_ticks(&self) -> u64 {
            self.start.elapsed().as_nanos() as u64
        }

        fn ticks_per_second(&self) -> u64 {
            1_000_000_000
        }
    }
}
