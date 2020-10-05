mod barrier;
mod consumer;
mod dsl;
mod executor;
mod prelude;
mod producer;
mod ringbuffer;
mod utils;
mod wait;

pub use dsl::*;
pub use prelude::*;
pub mod internal {
    pub use super::barrier::*;
    pub use super::consumer::*;
    pub use super::executor::*;
    pub use super::producer::*;
    pub use super::ringbuffer::*;
    pub use super::wait::*;
}

#[cfg(test)]
mod test {
    use super::internal::*;
    use super::*;
    use std::sync::Arc;

    #[test]
    fn test_blocking_wait_strategy() {
        let ring_buffer: Arc<RingBuffer<i64>> = Arc::new(RingBuffer::new(4096));
        let wait_strategy = BlockingWaitStrategy::new();
        let mut sequencer = SingleProducerSequencer::new(ring_buffer.buffer_size(), wait_strategy);

        let gating_sequences = vec![sequencer.get_cursor()];
        let barrier = sequencer.create_barrier(&gating_sequences);
        let consumer = BatchEventProcessor::create(|data, sequence, _| {
            if *data != sequence {
                dbg!(*data);
                dbg!(sequence);
                panic!();
            }
        });

        sequencer.add_gating_sequence(&consumer.get_cursor());
        let data_provider = ring_buffer.clone();
        let producer = Producer::new(data_provider.clone(), sequencer);

        let executor =
            ThreadedExecutor::with_runnables(vec![consumer.prepare(barrier, data_provider)]);
        let handle = executor.spawn();

        for _ in 0..10_000 {
            let buffer: Vec<_> = std::iter::repeat(1).take(1000).collect();
            producer.write(buffer, |slot, seq, _| {
                *slot = seq;
            });
        }

        producer.drain();
        handle.join();
    }

    #[test]
    fn test_dsl() {
        let ring_buffer: Arc<RingBuffer<i64>> = Arc::new(RingBuffer::new(4096));
        let (executor, producer) = dsl::DisrustorBuilder::new(ring_buffer.clone())
            .with_blocking_wait()
            .with_single_producer()
            .with_barrier(|b| {
                b.handle_events_mut(|data, sequence, _| {
                    if *data != sequence {
                        dbg!(*data);
                        dbg!(sequence);
                        panic!();
                    }
                });
            })
            .build();

        let handle = executor.spawn();
        for _ in 0..10_000 {
            let buffer: Vec<_> = std::iter::repeat(1).take(1000).collect();
            producer.write(buffer, |slot, seq, _| {
                *slot = seq;
            });
        }
        producer.drain();
        handle.join();
    }
}
