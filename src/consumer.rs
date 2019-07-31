use crate::prelude::*;
use std::marker::PhantomData;
use std::sync::Arc;

pub struct BatchEventProcessor;

impl BatchEventProcessor {
    pub fn create<F, T>(handler: F) -> impl EventProcessor<T>
    where
        F: Fn(&T, Sequence, bool) + Send + 'static,
    {
        Processor {
            handler,
            cursor: Default::default(),
            _marker: Default::default(),
        }
    }

    pub fn create_mut<F, T>(handler: F) -> impl EventProcessor<T>
    where
        F: Fn(&mut T, Sequence, bool) + Send + 'static,
    {
        ProcessorMut {
            handler,
            cursor: Default::default(),
            _marker: Default::default(),
        }
    }
}

struct Processor<F, T> {
    handler: F,
    cursor: Arc<AtomicSequence>,
    _marker: PhantomData<T>,
}

struct ProcessorMut<F, T> {
    handler: F,
    cursor: Arc<AtomicSequence>,
    _marker: PhantomData<T>,
}

impl<F, T> EventProcessor<T> for Processor<F, T>
where
    F: Fn(&T, Sequence, bool) + Send + 'static,
{
    fn run<B: SequenceBarrier, D: DataProvider<T>>(self, barrier: B, data_provider: &D) {
        let f = self.handler;
        let cursor = self.cursor;

        loop {
            let next = cursor.get() + 1;
            let available = match barrier.wait_for(next) {
                Some(seq) => seq,
                None => return,
            };

            for i in next..=available {
                unsafe { f(data_provider.get(i), i, i == available) }
            }

            cursor.set(available);
            barrier.signal();
        }
    }

    fn get_cursor(&self) -> Arc<AtomicSequence> {
        self.cursor.clone()
    }
}

impl<F, T> EventProcessor<T> for ProcessorMut<F, T>
where
    F: Fn(&mut T, Sequence, bool) + Send + 'static,
{
    fn run<B: SequenceBarrier, D: DataProvider<T>>(self, barrier: B, data_provider: &D) {
        let f = self.handler;
        let cursor = self.cursor;

        loop {
            let next = cursor.get() + 1;
            let available = match barrier.wait_for(next) {
                Some(seq) => seq,
                None => break,
            };

            for i in next..=available {
                unsafe { f(data_provider.get_mut(i), i, i == available) }
            }

            cursor.set(available);
            barrier.signal();
        }
    }

    fn get_cursor(&self) -> Arc<AtomicSequence> {
        self.cursor.clone()
    }
}
