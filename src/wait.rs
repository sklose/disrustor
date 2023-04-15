use crate::prelude::*;
use crate::utils::*;
use std::{
    borrow::Borrow,
    sync::{Condvar, Mutex},
};

pub struct SpinLoopWaitStrategy;

pub struct BlockingWaitStrategy {
    guard: Mutex<()>,
    cvar: Condvar,
}

impl WaitStrategy for SpinLoopWaitStrategy {
    fn new() -> Self {
        SpinLoopWaitStrategy {}
    }

    fn wait_for<F: Fn() -> bool, S: Borrow<AtomicSequence>>(
        &self,
        sequence: Sequence,
        dependencies: &[S],
        check_alert: F,
    ) -> Option<Sequence> {
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

    fn signal(&self) {}
}

impl WaitStrategy for BlockingWaitStrategy {
    fn new() -> Self {
        Self {
            cvar: Condvar::new(),
            guard: Mutex::new(()),
        }
    }

    fn wait_for<F: Fn() -> bool, S: Borrow<AtomicSequence>>(
        &self,
        sequence: Sequence,
        dependencies: &[S],
        check_alert: F,
    ) -> Option<Sequence> {
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

    fn signal(&self) {
        let _guard = self.guard.lock().unwrap();
        self.cvar.notify_all();
        drop(_guard);
    }
}
