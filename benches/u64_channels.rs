use criterion::{
    black_box, criterion_group, criterion_main, BenchmarkId, Criterion, SamplingMode, Throughput,
};
use disrustor::{internal::*, *};
use std::sync::mpsc::channel;
use std::sync::Arc;
use std::time::Duration;

fn mpsc_channel(n: u64) {
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

fn disrustor_channel<S: Sequencer, F: FnOnce(&RingBuffer<i64>) -> S>(n: u64, b: u64, f: F) {
    let capacity = 65536;
    let data: Arc<RingBuffer<i64>> = Arc::new(RingBuffer::new(capacity));
    let mut sequencer = f(data.as_ref());

    let gating_sequence = vec![sequencer.get_cursor()];
    let barrier = sequencer.create_barrier(&gating_sequence);
    let processor = BatchEventProcessor::create(move |data, sequence, _| {
        assert!(*data == sequence);
    });

    sequencer.add_gating_sequence(&processor.get_cursor());

    let executor = ThreadedExecutor::with_runnables(vec![processor.prepare(barrier, data.clone())]);

    let handle = executor.spawn();

    let mut counter = 0;
    for _ in 1..=n / b {
        let mut remainder = b as i64;
        while remainder > 0 {
            let (start, end) = sequencer.next(remainder as usize);
            let count = end - start + 1;
            remainder -= count;
            for sequence in start..=end {
                counter += 1;
                unsafe { *data.get_mut(sequence) = sequence };
            }
            sequencer.publish(start, end);
        }
    }

    sequencer.drain();
    handle.join();
    assert!(counter == n);
}

fn criterion_benchmark(c: &mut Criterion) {
    const N: u64 = 1_000_000;

    let mut group = c.benchmark_group("mpsc");
    group.throughput(Throughput::Elements(N));
    group.warm_up_time(Duration::from_secs(10));
    group.bench_with_input(BenchmarkId::from_parameter(1), &1, |b, _| {
        b.iter(|| mpsc_channel(black_box(N)));
    });
    group.finish();

    let mut group = c.benchmark_group("single_producer_spinning");
    group.throughput(Throughput::Elements(N));
    group.warm_up_time(Duration::from_secs(10));
    group.sampling_mode(SamplingMode::Flat);
    for batch_size in [1, 10, 50, 100, 1000] {
        group.bench_with_input(
            BenchmarkId::from_parameter(batch_size),
            &batch_size,
            |b, batch_size| {
                b.iter(|| {
                    disrustor_channel(black_box(N), *batch_size, |d| {
                        SingleProducerSequencer::new(d.buffer_size(), SpinLoopWaitStrategy::new())
                    })
                });
            },
        );
    }
    group.finish();

    let mut group = c.benchmark_group("single_producer_blocking");
    group.throughput(Throughput::Elements(N));
    group.warm_up_time(Duration::from_secs(10));
    group.sampling_mode(SamplingMode::Flat);
    for batch_size in [10, 50, 100, 1000] {
        group.bench_with_input(
            BenchmarkId::from_parameter(batch_size),
            &batch_size,
            |b, batch_size| {
                b.iter(|| {
                    disrustor_channel(black_box(N), *batch_size, |d| {
                        SingleProducerSequencer::new(d.buffer_size(), BlockingWaitStrategy::new())
                    })
                });
            },
        );
    }
    group.finish();

    let mut group = c.benchmark_group("multi_producer_spinning");
    group.throughput(Throughput::Elements(N));
    group.warm_up_time(Duration::from_secs(10));
    group.sampling_mode(SamplingMode::Flat);
    for batch_size in [1, 10, 50, 100, 1000] {
        group.bench_with_input(
            BenchmarkId::from_parameter(batch_size),
            &batch_size,
            |b, batch_size| {
                b.iter(|| {
                    disrustor_channel(black_box(N), *batch_size, |d| {
                        MultiProducerSequencer::new(d.buffer_size(), SpinLoopWaitStrategy::new())
                    })
                });
            },
        );
    }
    group.finish();

    let mut group = c.benchmark_group("multi_producer_blocking");
    group.throughput(Throughput::Elements(N));
    group.warm_up_time(Duration::from_secs(10));
    group.sampling_mode(SamplingMode::Flat);
    for batch_size in [10, 50, 100, 1000] {
        group.bench_with_input(
            BenchmarkId::from_parameter(batch_size),
            &batch_size,
            |b, batch_size| {
                b.iter(|| {
                    disrustor_channel(black_box(N), *batch_size, |d| {
                        MultiProducerSequencer::new(d.buffer_size(), BlockingWaitStrategy::new())
                    })
                });
            },
        );
    }
    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
