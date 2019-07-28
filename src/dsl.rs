use std::marker::PhantomData;
use super::prelude::*;
use super::{BlockingWaitStrategy, SpinLoopWaitStrategy, BatchProcessor};

/*
 * let (p, h) = DisruptorBuilder::with_blocking_wait()
 *     .begin_group()
 *         .add_handler()
 *         .add_handler()
 *         .begin_group()
 *         // ...
 *         .end_group()
 *     .end_group()
 *     .build_single_producer();
 * 
 * h.run();
 * p.produce(...);
 */

#[derive(Debug)]
pub struct DisrustorBuilder {

}

#[derive(Debug)]
struct State<W: WaitStrategy, D: DataProvider<T>, T> {
    wait_strategy: W,
    _element: PhantomData<T>,
}

#[derive(Debug)]
pub struct InitialState<W: WaitStrategy, T> {
    state: State<W, T>,
}

#[derive(Debug)]
pub struct BeginGroupState<W: WaitStrategy, T> {
    state: State<W, T>,
}

#[derive(Debug)]
pub struct EndGroupState<W: WaitStrategy, T> {
    state: State<W, T>,
}

pub trait BeginGroup<W: WaitStrategy, T> {
    fn begin_group(self) -> BeginGroupState<W, T>;
}

impl DisrustorBuilder {
    pub fn new<W: WaitStrategy, T>() -> InitialState<W, T> {
        InitialState {
            state: State {
                wait_strategy: W::new(),
                _element: PhantomData::default(),
            }
        }
    }

    pub fn with_blocking_wait<T>() -> InitialState<BlockingWaitStrategy, T> {
        Self::new()
    }

    pub fn with_spin_wait<T>() -> InitialState<SpinLoopWaitStrategy, T> {
        Self::new()
    }
}

impl<W: WaitStrategy, T> BeginGroup<W, T> for InitialState<W, T> {
    fn begin_group(self) -> BeginGroupState<W, T> {
        BeginGroupState {
            state: self.state,
        }
    }
}

impl<W: WaitStrategy, T> BeginGroupState<W, T> {
    pub fn add_consumer_mut<F>(self, f: F) where F: Fn(&mut T, Sequence) -> EndGroupState<W, T> {
        let consumer = BatchProcessor::new()
    }
}