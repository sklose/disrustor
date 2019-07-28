use crate::barrier::*;
use crate::prelude::*;
use crate::utils::*;
use std::cell::Cell;
use std::sync::Arc;

pub struct SingleProducerSequencer<W: WaitStrategy> {
    cursor: Arc<AtomicSequence>,
    next_write_sequence: Cell<Sequence>,
    cached_available_sequence: Cell<Sequence>,
    wait_strategy: Arc<W>,
    gating_sequences: Vec<Arc<AtomicSequence>>,
    buffer_size: usize,
}

impl<W: WaitStrategy> SingleProducerSequencer<W> {
    pub fn new(buffer_size: usize, wait_strategy: W) -> Self {
        SingleProducerSequencer {
            cursor: Arc::new(AtomicSequence::default()),
            next_write_sequence: Cell::new(0),
            cached_available_sequence: Cell::new(Sequence::default()),
            wait_strategy: Arc::new(wait_strategy),
            gating_sequences: Vec::new(),
            buffer_size,
        }
    }
}

impl<W: WaitStrategy> Sequencer for SingleProducerSequencer<W> {
    type Barrier = ProcessingSequenceBarrier<W>;

    fn next(&self, count: usize) -> (Sequence, Sequence) {
        let mut min_sequence = self.cached_available_sequence.take();
        let next = self.next_write_sequence.take();
        let (start, end) = (next, next + (count - 1) as Sequence);

        while min_sequence + (self.buffer_size as Sequence) < end {
            min_sequence = min_cursor_sequence(&self.gating_sequences);
        }

        self.cached_available_sequence.set(min_sequence);
        self.next_write_sequence.set(end + 1);

        (start, end)
    }

    fn publish(&self, highest: Sequence) {
        self.cursor.set(highest);
        self.wait_strategy.signal();
    }

    fn create_barrier(
        &mut self,
        gating_sequences: Vec<Arc<AtomicSequence>>,
    ) -> ProcessingSequenceBarrier<W> {
        ProcessingSequenceBarrier::new(self.wait_strategy.clone(), gating_sequences)
    }

    fn add_gating_sequence(&mut self, gating_sequence: Arc<AtomicSequence>) {
        self.gating_sequences.push(gating_sequence);
    }

    fn drain(self) {
        let current = self.next_write_sequence.take() - 1;
        while min_cursor_sequence(&self.gating_sequences) < current {
            self.wait_strategy.signal();
        }
    }

    fn get_cursor(&self) -> Arc<AtomicSequence> {
        self.cursor.clone()
    }
}

impl<W: WaitStrategy> SingleProducerSequencer<W> {
    pub fn write<F, T, U, D, I, E>(&self, target: &D, items: I, f: F)
    where
        D: DataProvider<T>,
        I: IntoIterator<Item = U, IntoIter = E>,
        E: ExactSizeIterator<Item = U>,
        F: Fn(&mut T, Sequence, &U),
    {
        let iter = items.into_iter();
        let (start, end) = self.next(iter.len());
        for (idx, item) in iter.enumerate() {
            let seq = start + idx as Sequence;
            let slot = unsafe { target.get_mut(seq) };
            f(slot, seq, &item);
        }
        self.publish(end);
    }
}
