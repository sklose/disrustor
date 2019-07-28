use crate::prelude::*;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

pub struct ProcessingSequenceBarrier<W: WaitStrategy> {
    gating_sequences: Vec<Arc<AtomicSequence>>,
    wait_strategy: Arc<W>,
    is_alerted: AtomicBool,
}

impl<W: WaitStrategy> ProcessingSequenceBarrier<W> {
    pub fn new(wait_strategy: Arc<W>, gating_sequences: Vec<Arc<AtomicSequence>>) -> Self {
        ProcessingSequenceBarrier {
            wait_strategy,
            gating_sequences,
            is_alerted: AtomicBool::new(false),
        }
    }
}

impl<W: WaitStrategy> SequenceBarrier for ProcessingSequenceBarrier<W> {
    fn wait_for(&self, sequence: Sequence) -> Option<Sequence> {
        self.wait_strategy
            .wait_for(sequence, &self.gating_sequences, || {
                self.is_alerted.load(Ordering::Relaxed)
            })
    }

    fn signal(&self) {
        self.wait_strategy.signal();
    }

    fn alert(&self) {
        self.is_alerted.store(true, Ordering::Relaxed);
        self.signal();
    }
}
