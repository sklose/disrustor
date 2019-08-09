mod barrier;
mod consumer;
mod dsl;
mod executor;
mod producer;
mod ringbuffer;
mod utils;
mod wait;

pub mod prelude;
pub use barrier::*;
pub use consumer::*;
pub use executor::*;
pub use producer::*;
pub use ringbuffer::*;
pub use wait::*;

#[cfg(test)]
mod test {
    use super::prelude::*;
    use super::*;
    use std::sync::Arc;

    #[test]
    fn test_blocking_wait_strategy() {
        let ring_buffer: Arc<RingBuffer<i64>> = Arc::new(RingBuffer::new(4096));
        let wait_strategy = BlockingWaitStrategy::new();
        let mut sequencer = SingleProducerSequencer::new(ring_buffer.buffer_size(), wait_strategy);

        let barrier = sequencer.create_barrier(vec![sequencer.get_cursor()]);
        let consumer = BatchEventProcessor::create(|data, sequence, _| {
            if *data != sequence {
                dbg!(*data);
                dbg!(sequence);
                panic!();
            }
        });

        sequencer.add_gating_sequence(consumer.get_cursor());
        let data_provider = ring_buffer.clone();

        let executor =
            ThreadedExecutor::with_runnables(vec![consumer.prepare(barrier, data_provider)]);
        let handle = executor.spawn();

        for _ in 0..10_000 {
            let buffer: Vec<_> = std::iter::repeat(1).take(1000).collect();
            sequencer.write(ring_buffer.as_ref(), buffer, |slot, seq, _| {
                *slot = seq;
            });
        }

        sequencer.drain();
        handle.join();
    }

    #[test]
    fn test_dsl() {
        let ring_buffer: Arc<RingBuffer<i64>> = Arc::new(RingBuffer::new(4096));
        let (executor, sequencer) = dsl::DisrustorBuilder::new(ring_buffer.clone())
            .with_blocking_wait()
            .with_single_producer()
            .handle_events_with(BatchEventProcessor::create_mut(|data, sequence, _| {
                if *data != sequence {
                    dbg!(*data);
                    dbg!(sequence);
                    panic!();
                }
            }))
            .build();

        let handle = executor.spawn();
        for _ in 0..10_000 {
            let buffer: Vec<_> = std::iter::repeat(1).take(1000).collect();
            sequencer.write(ring_buffer.as_ref(), buffer, |slot, seq, _| {
                *slot = seq;
            });
        }
        sequencer.drain();
        handle.join();
    }
}
