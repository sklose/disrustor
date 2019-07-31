mod barrier;
mod consumer;
mod producer;
mod ringbuffer;
mod utils;
mod wait;
// mod dsl;

pub mod prelude;
pub use barrier::*;
pub use consumer::*;
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
        let mut ch = consumer.run(barrier, ring_buffer.clone());

        for _ in 0..10_000 {
            let buffer: Vec<_> = std::iter::repeat(1).take(1000).collect();
            sequencer.write(ring_buffer.as_ref(), buffer, |slot, seq, _| {
                *slot = seq;
            });
        }

        sequencer.drain();
        ch.halt();
    }
}
