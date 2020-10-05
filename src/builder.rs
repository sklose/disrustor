/// A DSL-style API for setting up the disruptor pattern around a ring buffer
/// (aka the Builder pattern).
///
/// A simple example of setting up the disruptor with two event handlers that
/// must process events in order:
///
/// ```
/// Disruptor&lt;MyEvent&gt; disruptor = new Disruptor&lt;MyEvent&gt;(MyEvent.FACTORY, 32, Executors.newCachedThreadPool());
/// EventHandler&lt;MyEvent&gt; handler1 = new EventHandler&lt;MyEvent&gt;() { ... };
/// EventHandler&lt;MyEvent&gt; handler2 = new EventHandler&lt;MyEvent&gt;() { ... };
/// disruptor.handleEventsWith(handler1);
/// disruptor.after(handler1).handleEventsWith(handler2);
///
/// RingBuffer ringBuffer = disruptor.start();
/// ```
#[derive(Debug)]
pub struct Disrustor<State> {
    state: State,
}

impl<State> Disrustor<State> {
    pub fn builder() -> Disrustor<state::Initial> {
        Disrustor {
            state: Default::default(),
        }
    }
}

#[doc(hidden)]
pub mod state {
    use crate::*;
    use std::marker::PhantomData;
    use std::sync::Arc;

    #[derive(Debug, Default)]
    pub struct Initial;
}
