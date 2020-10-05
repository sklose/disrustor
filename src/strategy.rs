use crate::lib::*;
use std::sync::{Condvar, Mutex};

/// Strategy employed for making {@link EventProcessor}s wait on a cursor {@link Sequence}.
pub trait WaitStrategy: Send + Sync {
    fn new() -> Self;

    /// Wait for the given sequence to be available. It is possible for this method to return a value
    /// less than the sequence number supplied depending on the implementation of the WaitStrategy. A common
    /// use for this is to signal a timeout. Any EventProcessor that is using a WaitStrategy to get notifications
    /// about message becoming available should remember to handle this case. The {@link BatchEventProcessor} explicitly
    /// handles this case and will signal a timeout if required.
    fn wait_for<F: Fn() -> bool, S: AsRef<Sequence>>(
        &self,
        sequence: i64,
        dependencies: &[S],
        check_alert: F,
    ) -> Option<i64>;

    /// Implementations should signal the waiting {@link EventProcessor}s that the cursor has advanced
    fn signal_all_when_blocking(&self) {
        // most wait strategies do not needs this
    }
}

/// Busy Spin strategy that uses a busy spin loop for {@link com.lmax.disruptor.EventProcessor}s waiting on a barrier.
///
/// This strategy will use CPU resource to avoid syscalls which can introduce latency jitter.  It is best
/// used when threads can be bound to specific CPU cores.
pub struct BusySpinWaitStrategy;

/// Blocking strategy that uses a lock and condition variable for {@link EventProcessor}s waiting on a barrier.
///
/// This strategy can be used when throughput and low-latency are not as important as CPU resource.
pub struct BlockingWaitStrategy {
    guard: Mutex<()>,
    cvar: Condvar,
}

impl WaitStrategy for BusySpinWaitStrategy {
    fn new() -> Self {
        Self {}
    }

    fn wait_for<F: Fn() -> bool, S: AsRef<Sequence>>(
        &self,
        sequence: i64,
        dependencies: &[S],
        check_alert: F,
    ) -> Option<i64> {
        loop {
            let available = min_cursor_sequence(dependencies);
            if available >= sequence {
                return Some(available);
            }
            if check_alert() {
                return None;
            }
        }
    }
}

impl WaitStrategy for BlockingWaitStrategy {
    fn new() -> Self {
        Self {
            cvar: Condvar::new(),
            guard: Mutex::new(()),
        }
    }

    fn wait_for<F: Fn() -> bool, S: AsRef<Sequence>>(
        &self,
        sequence: i64,
        dependencies: &[S],
        check_alert: F,
    ) -> Option<i64> {
        loop {
            let blocked = self.guard.lock().unwrap();
            if check_alert() {
                return None;
            }

            let available = min_cursor_sequence(dependencies);
            if available >= sequence {
                return Some(available);
            } else {
                let _guard = self.cvar.wait(blocked).unwrap();
            }
        }
    }

    fn signal_all_when_blocking(&self) {
        let _guard = self.guard.lock().unwrap();
        self.cvar.notify_all();
    }
}
