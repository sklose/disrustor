use criterion::{criterion_group, criterion_main, Criterion, ParameterizedBenchmark, Throughput};
use disrustor::prelude::*;
use disrustor::{
    BatchEventProcessor, BlockingWaitStrategy, RingBuffer, SingleProducerSequencer,
    SpinLoopWaitStrategy, ThreadedExecutor,
};
use std::sync::mpsc::channel;
use std::sync::Arc;

fn mpsc_channel(n: i64) {
    let (tx, rx) = channel();

    let t = std::thread::spawn(move || loop {
        let next = rx.recv().unwrap();
        if next == n {
            break;
        }
    });

    for i in 1..=n {
        tx.send(i).unwrap();
    }

    t.join().unwrap();
}

fn disrustor_channel<W: WaitStrategy + 'static>(n: i64, b: i64) {
    let capacity = 65536;
    let data: Arc<RingBuffer<i64>> = Arc::new(RingBuffer::new(capacity));
    let mut sequencer = SingleProducerSequencer::new(data.buffer_size(), W::new());

    let barrier = sequencer.create_barrier(vec![sequencer.get_cursor()]);
    let processor = BatchEventProcessor::create(move |data, sequence, _| {
        assert!(*data == sequence);
    });

    sequencer.add_gating_sequence(processor.get_cursor());

    let executor = ThreadedExecutor::with_runnables(vec![processor.prepare(barrier, data.clone())]);

    let handle = executor.spawn();

    let mut counter = 0;
    for _ in 1..=n / b {
        let mut remainder = b;
        while remainder > 0 {
            let (start, end) = sequencer.next(remainder as usize);
            remainder -= end - start + 1;
            for sequence in start..=end {
                counter += 1;
                unsafe { *data.get_mut(sequence) = sequence };
            }
            sequencer.publish(end);
        }
    }

    sequencer.drain();
    handle.join();
    assert!(counter == n);
}

fn criterion_benchmark(c: &mut Criterion) {
    let n = 10_000_000;
    let inputs = vec![1, 10, 50, 100, 1000, 2000, 4000];
    c.bench(
        "channels",
        ParameterizedBenchmark::new("mpsc", move |b, _| b.iter(|| mpsc_channel(n)), inputs)
            .with_function("disrustor spinning", move |b, i| {
                b.iter(|| disrustor_channel::<SpinLoopWaitStrategy>(n, *i));
            })
            .with_function("disrustor blocking", move |b, i| {
                b.iter(|| disrustor_channel::<BlockingWaitStrategy>(n, *i));
            })
            .throughput(move |_| Throughput::Elements(n as u32))
            .sample_size(10),
    );
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
