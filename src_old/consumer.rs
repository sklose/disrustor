use crate::prelude::*;
use std::marker::PhantomData;
use std::sync::Arc;

pub struct BatchEventProcessor;

impl BatchEventProcessor {
    pub fn create<'a, F, T>(handler: F) -> impl EventProcessor<'a, T>
    where
        T: Send + 'a,
        F: Fn(&T, Sequence, bool) + Send + 'static,
    {
        Processor {
            handler,
            cursor: Default::default(),
            _marker: Default::default(),
        }
    }

    pub fn create_mut<'a, F, T>(handler: F) -> impl EventProcessorMut<'a, T>
    where
        T: Send + 'a,
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

struct RunnableProcessor<F, T, D: DataProvider<T>, B: SequenceBarrier> {
    processor: Processor<F, T>,
    data_provider: Arc<D>,
    barrier: B,
}

struct RunnableProcessorMut<F, T, D: DataProvider<T>, B: SequenceBarrier> {
    processor: ProcessorMut<F, T>,
    data_provider: Arc<D>,
    barrier: B,
}

impl<'a, F, T> EventProcessorMut<'a, T> for Processor<F, T>
where
    F: Fn(&T, Sequence, bool) + Send + 'static,
    T: Send + 'a,
{
    fn prepare<B: SequenceBarrier + 'a, D: DataProvider<T> + 'a>(
        self,
        barrier: B,
        data_provider: Arc<D>,
    ) -> Box<dyn Runnable + 'a> {
        Box::new(RunnableProcessor {
            processor: self,
            data_provider,
            barrier,
        })
    }

    fn get_cursor(&self) -> Arc<AtomicSequence> {
        self.cursor.clone()
    }
}

impl<'a, F, T> EventProcessorMut<'a, T> for ProcessorMut<F, T>
where
    F: Fn(&mut T, Sequence, bool) + Send + 'static,
    T: Send + 'a,
{
    fn prepare<B: SequenceBarrier + 'a, D: DataProvider<T> + 'a>(
        self,
        barrier: B,
        data_provider: Arc<D>,
    ) -> Box<dyn Runnable + 'a> {
        Box::new(RunnableProcessorMut {
            processor: self,
            data_provider,
            barrier,
        })
    }

    fn get_cursor(&self) -> Arc<AtomicSequence> {
        self.cursor.clone()
    }
}

impl<'a, F, T> EventProcessor<'a, T> for Processor<F, T>
where
    F: Fn(&T, Sequence, bool) + Send + 'static,
    T: Send + 'a,
{
}

impl<F, T, D, B> Runnable for RunnableProcessor<F, T, D, B>
where
    F: Fn(&T, Sequence, bool) + Send + 'static,
    D: DataProvider<T>,
    B: SequenceBarrier,
    T: Send,
{
    fn run(self: Box<Self>) {
        let f = &self.processor.handler;
        let cursor = &self.processor.cursor;
        let data_provider = &self.data_provider;
        let barrier = &self.barrier;

        loop {
            let next = cursor.get() + 1;
            let available = match barrier.wait_for(next) {
                Some(seq) => seq,
                None => return,
            };

            for i in next..=available {
                let value = unsafe { data_provider.get(i) };
                f(value, i, i == available);
            }

            cursor.set(available);
            barrier.signal();
        }
    }
}

impl<F, T, D, B> Runnable for RunnableProcessorMut<F, T, D, B>
where
    F: Fn(&mut T, Sequence, bool) + Send + 'static,
    D: DataProvider<T>,
    B: SequenceBarrier,
    T: Send,
{
    fn run(self: Box<Self>) {
        let f = &self.processor.handler;
        let cursor = &self.processor.cursor;
        let data_provider = &self.data_provider;
        let barrier = &self.barrier;

        loop {
            let next = cursor.get() + 1;
            let available = match barrier.wait_for(next) {
                Some(seq) => seq,
                None => return,
            };

            for i in next..=available {
                let value = unsafe { data_provider.get_mut(i) };
                f(value, i, i == available);
            }

            cursor.set(available);
            barrier.signal();
        }
    }
}
