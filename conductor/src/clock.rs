/// monotonic clock source.
pub trait Clock {
    fn now_ticks(&self) -> u64;

    fn ticks_per_second(&self) -> u64;
}
