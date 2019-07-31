use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::Arc;

pub type Sequence = i64;

#[repr(align(64))]
pub struct AtomicSequence {
    _pad: [u8; 56],
    offset: AtomicI64,
}

impl AtomicSequence {
    pub fn get(&self) -> Sequence {
        self.offset.load(Ordering::Acquire)
    }

    pub fn set(&self, value: Sequence) {
        self.offset.store(value, Ordering::Release);
    }
}

impl Default for AtomicSequence {
    fn default() -> Self {
        AtomicSequence::from(-1)
    }
}

impl From<Sequence> for AtomicSequence {
    fn from(offset: Sequence) -> Self {
        AtomicSequence {
            offset: AtomicI64::new(offset),
            _pad: [0u8; 56],
        }
    }
}

pub trait SequenceBarrier: Send + Sync {
    fn wait_for(&self, sequence: Sequence) -> Option<Sequence>;
    fn signal(&self);
}

pub trait Sequencer {
    type Barrier: SequenceBarrier;

    fn next(&self, count: usize) -> (Sequence, Sequence);
    fn publish(&self, highest: Sequence);
    fn create_barrier(&mut self, gating_sequences: Vec<Arc<AtomicSequence>>) -> Self::Barrier;
    fn add_gating_sequence(&mut self, gating_sequence: Arc<AtomicSequence>);
    fn get_cursor(&self) -> Arc<AtomicSequence>;
    fn drain(self);
}

pub trait WaitStrategy: Send + Sync {
    fn new() -> Self;
    fn wait_for<F: Fn() -> bool>(
        &self,
        sequence: Sequence,
        dependencies: &[Arc<AtomicSequence>],
        check_alert: F,
    ) -> Option<Sequence>;
    fn signal(&self);
}

pub trait DataProvider<T>: Sync + Send {
    fn buffer_size(&self) -> usize;
    #[allow(clippy::mut_from_ref)]
    unsafe fn get_mut(&self, sequence: Sequence) -> &mut T;
    unsafe fn get(&self, sequence: Sequence) -> &T;
}

pub trait EventProcessor<T> {
    fn run<B: SequenceBarrier, D: DataProvider<T>>(self, barrier: B, data_provider: &D);
    fn get_cursor(&self) -> Arc<AtomicSequence>;
}
