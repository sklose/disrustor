use std::sync::{
    atomic::{AtomicI64, Ordering},
    Arc,
};

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

    pub fn compare_exchange(&self, current: Sequence, new: Sequence) -> bool {
        self.offset
            .compare_exchange(current, new, Ordering::SeqCst, Ordering::Acquire)
            .is_ok()
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
    fn publish(&self, lo: Sequence, hi: Sequence);
    fn create_barrier(&mut self, gating_sequences: &[Arc<AtomicSequence>]) -> Self::Barrier;
    fn add_gating_sequence(&mut self, gating_sequence: &Arc<AtomicSequence>);
    fn get_cursor(&self) -> Arc<AtomicSequence>;
    fn drain(self);
}

pub trait WaitStrategy: Send + Sync {
    fn new() -> Self;
    fn wait_for<F: Fn() -> bool, S: AsRef<AtomicSequence>>(
        &self,
        sequence: Sequence,
        dependencies: &[S],
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

pub trait EventProcessorMut<'a, T> {
    fn prepare<B: SequenceBarrier + 'a, D: DataProvider<T> + 'a>(
        self,
        barrier: B,
        data_provider: Arc<D>,
    ) -> Box<dyn Runnable + 'a>;
    fn get_cursor(&self) -> Arc<AtomicSequence>;
}

pub trait EventProcessor<'a, T>: EventProcessorMut<'a, T> {}

pub trait Runnable: Send {
    fn run(self: Box<Self>);
}

pub trait EventProcessorExecutor<'a> {
    type Handle: ExecutorHandle;
    fn with_runnables(items: Vec<Box<dyn Runnable + 'a>>) -> Self;
    fn spawn(self) -> Self::Handle;
}

pub trait ExecutorHandle {
    fn join(self);
}

pub trait EventProducer<'a> {
    type Item;

    fn write<F, U, I, E>(&self, items: I, f: F)
    where
        I: IntoIterator<Item = U, IntoIter = E>,
        E: ExactSizeIterator<Item = U>,
        F: Fn(&mut Self::Item, Sequence, &U);

    fn drain(self);
}
