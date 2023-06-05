use disrustor::{
    internal::{BlockingWaitStrategy, SpinLoopWaitStrategy},
    *,
};
use log::*;

const MAX: i64 = 200i64;

struct Doubler;

impl EventHandlerMut<u32> for Doubler {
    fn handle_event(&mut self, data: &mut u32, sequence: Sequence, _: bool) {
        let val = *data;
        if i64::from(val) != sequence {
            panic!(
                "concurrency problem detected (p1), expected {}, but got {}",
                sequence, val
            );
        }
        debug!("updating sequence {} from {} to {}", sequence, val, val * 2);
        *data = val * 2;
    }
}

struct Checker;

impl EventHandler<u32> for Checker {
    fn handle_event(&mut self, data: &u32, sequence: Sequence, _: bool) {
        let val = *data;
        if i64::from(val) != sequence * 2 {
            panic!(
                "concurrency problem detected (p2), expected {}, but got {}",
                sequence * 2,
                val
            );
        }
    }
}

fn follow_sequence<W: WaitStrategy + 'static>() {
    let (executor, producer) = DisrustorBuilder::with_ring_buffer(128)
        .with_wait_strategy::<W>()
        .with_single_producer()
        .with_barrier(|b| {
            b.handle_events_mut(Doubler {});
        })
        .with_barrier(|b| {
            b.handle_events(Checker{});
        })
        .build();

    let handle = executor.spawn();
    for i in 1..=MAX / 20 {
        let range = ((i - 1) * 20)..=((i - 1) * 20 + 19);
        let items: Vec<_> = range.collect();
        producer.write(items, |d, _, v| {
            *d = *v as u32;
        });
    }

    producer.drain();
    handle.join();
}

fn main() {
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}[{}][{:?}][{}] {}",
                chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                record.target(),
                std::thread::current().id(),
                record.level(),
                message
            ))
        })
        .level(log::LevelFilter::Debug)
        .chain(std::io::stdout())
        .chain(fern::log_file("output.log").unwrap())
        .apply()
        .unwrap();

    info!("running blocking wait strategy");
    follow_sequence::<BlockingWaitStrategy>();

    info!("running spinning wait strategy");
    follow_sequence::<SpinLoopWaitStrategy>();
}
