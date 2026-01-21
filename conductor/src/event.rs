/// What the sequencer emits when a step boundary is reached.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Event {
    Trigger { step: u8 },
}
