use crate::lib::*;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

/// Coordination barrier for tracking the cursor for publishers and sequence of
/// dependent {@link EventProcessor}s for processing a data structure
pub trait SequenceBarrier: Send + Sync {
    /// Wait for the given sequence to be available for consumption.
    fn wait_for(&self, sequence: i64) -> Option<i64>;
    fn signal(&self);
}

/// {@link SequenceBarrier} handed out for gating {@link EventProcessor}s on
/// cursor sequence and optional dependent {@link EventProcessor}(s), using the given WaitStrategy.
pub struct ProcessingSequenceBarrier<W: WaitStrategy> {
    gating_sequences: Vec<Arc<Sequence>>,
    wait_strategy: Arc<W>,
    is_alerted: Arc<AtomicBool>,
}

impl<W: WaitStrategy> ProcessingSequenceBarrier<W> {
    pub fn new(
        wait_strategy: Arc<W>,
        gating_sequences: Vec<Arc<Sequence>>,
        is_alerted: Arc<AtomicBool>,
    ) -> Self {
        ProcessingSequenceBarrier {
            wait_strategy,
            gating_sequences,
            is_alerted,
        }
    }
}

impl<W: WaitStrategy> SequenceBarrier for ProcessingSequenceBarrier<W> {
    fn wait_for(&self, sequence: i64) -> Option<i64> {
        self.wait_strategy
            .wait_for(sequence, &self.gating_sequences, || {
                self.is_alerted.load(Ordering::Relaxed)
            })
    }

    fn signal(&self) {
        self.wait_strategy.signal_all_when_blocking();
    }
}
