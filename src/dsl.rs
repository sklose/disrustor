use super::prelude::*;
use super::*;
use std::marker::PhantomData;
use std::sync::Arc;

/*
 * let (p, h) = DisruptorBuilder::with_ring_buffer(128)
 *     .with_blocking_wait()
 *     .with_single_producer()
 *     .handle_events_with(batch_processor(||))
 *     .handle_events_with(batch_processor_mut(||))
 *     .handle_events_with_group(vec![
 *          batch_processor(||)
 *          batch_processor(||)
 *      ])
 *     .handle_events_with_pool(3, || batch_processor(||))
 *     .build();
 *
 * h.run();
 * p.produce(...);
 */

#[derive(Debug)]
pub struct DisrustorBuilder {}

pub struct WithDataProvider<D: DataProvider<T>, T> {
    data_provider: Arc<D>,
    _element: PhantomData<T>,
}

pub struct WithWaitStrategy<W: WaitStrategy, D: DataProvider<T>, T> {
    with_data_provider: WithDataProvider<D, T>,
    _wait_strategy: PhantomData<W>,
}

pub struct WithSequencer<'a, S: Sequencer + 'a, W: WaitStrategy, D: DataProvider<T> + 'a, T> {
    with_wait_strategy: WithWaitStrategy<W, D, T>,
    sequencer: S,
    event_handlers: Vec<Box<dyn Runnable + 'a>>,
    next_cursors: Vec<Arc<AtomicSequence>>,
}

impl DisrustorBuilder {
    pub fn new<D: DataProvider<T>, T>(data_provider: Arc<D>) -> WithDataProvider<D, T> {
        WithDataProvider {
            data_provider: data_provider,
            _element: Default::default(),
        }
    }

    pub fn with_ring_buffer<T: Default>(capacity: usize) -> WithDataProvider<RingBuffer<T>, T> {
        Self::new(Arc::new(RingBuffer::new(capacity)))
    }
}

impl<D: DataProvider<T>, T> WithDataProvider<D, T> {
    pub fn with_wait_strategy<W: WaitStrategy>(self) -> WithWaitStrategy<W, D, T> {
        WithWaitStrategy {
            with_data_provider: self,
            _wait_strategy: Default::default(),
        }
    }

    pub fn with_blocking_wait(self) -> WithWaitStrategy<BlockingWaitStrategy, D, T> {
        self.with_wait_strategy()
    }

    pub fn with_spin_wait(self) -> WithWaitStrategy<SpinLoopWaitStrategy, D, T> {
        self.with_wait_strategy()
    }
}

impl<W: WaitStrategy, D: DataProvider<T>, T> WithWaitStrategy<W, D, T> {
    pub fn with_sequencer<'a, S: Sequencer + 'a>(
        self,
        sequencer: S,
    ) -> WithSequencer<'a, S, W, D, T> {
        WithSequencer {
            with_wait_strategy: self,
            event_handlers: Vec::new(),
            next_cursors: vec![sequencer.get_cursor()],
            sequencer,
        }
    }

    pub fn with_single_producer<'a>(
        self,
    ) -> WithSequencer<'a, SingleProducerSequencer<W>, W, D, T> {
        let buffer_size = self.with_data_provider.data_provider.buffer_size();
        self.with_sequencer(SingleProducerSequencer::new(buffer_size, W::new()))
    }
}

impl<'a, S: Sequencer + 'a, W: WaitStrategy, D: DataProvider<T> + 'a, T>
    WithSequencer<'a, S, W, D, T>
{
    pub fn handle_events_with<E: EventProcessorMut<'a, T>>(mut self, processor: E) -> Self {
        let barrier = self.sequencer.create_barrier(self.next_cursors);
        self.next_cursors = vec![processor.get_cursor()];
        let runable = processor.prepare(
            barrier,
            self.with_wait_strategy
                .with_data_provider
                .data_provider
                .clone(),
        );
        self.event_handlers.push(runable);
        self
    }

    pub fn build(mut self) -> (impl EventProcessorExecutor<'a>, S) {
        let gating_sequences = std::mem::replace(&mut self.next_cursors, Vec::new());
        for gs in gating_sequences.into_iter() {
            self.sequencer.add_gating_sequence(gs);
        }
        let executor = ThreadedExecutor::with_runnables(self.event_handlers);
        (executor, self.sequencer)
    }
}
