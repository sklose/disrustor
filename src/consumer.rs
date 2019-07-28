use crate::prelude::*;
use std::marker::PhantomData;
use std::sync::Arc;

pub struct BatchEventProcessor;

impl BatchEventProcessor {
    pub fn create<D, F, T>(data_provider: Arc<D>, handler: F) -> impl EventProcessor<T>
    where
        F: Fn(&T, Sequence, bool) + Send + 'static,
        D: DataProvider<T> + 'static,
    {
        Processor {
            data_provider,
            handler,
            cursor: Default::default(),
            _marker: Default::default(),
        }
    }

    pub fn create_mut<D, F, T>(data_provider: Arc<D>, handler: F) -> impl EventProcessor<T>
    where
        F: Fn(&mut T, Sequence, bool) + Send + 'static,
        D: DataProvider<T> + 'static,
    {
        ProcessorMut {
            data_provider,
            handler,
            cursor: Default::default(),
            _marker: Default::default(),
        }
    }
}

struct Handle<B: SequenceBarrier> {
    barrier: Arc<B>,
    thread: Option<std::thread::JoinHandle<()>>,
}

impl<B: SequenceBarrier> EventProcessorHandle for Handle<B> {
    fn halt(&mut self) {
        self.barrier.alert();
        if let Some(thread) = self.thread.take() {
            thread.join().unwrap();
        }
    }
}

struct Processor<D, F, T> {
    cursor: Arc<AtomicSequence>,
    data_provider: Arc<D>,
    handler: F,
    _marker: PhantomData<T>,
}

struct ProcessorMut<D, F, T> {
    cursor: Arc<AtomicSequence>,
    data_provider: Arc<D>,
    handler: F,
    _marker: PhantomData<T>,
}

impl<F, D, T> EventProcessor<T> for Processor<D, F, T>
where
    D: DataProvider<T> + 'static,
    F: Fn(&T, Sequence, bool) + Send + 'static,
{
    fn run<B>(self, barrier: B) -> Box<dyn EventProcessorHandle>
    where
        B: SequenceBarrier + 'static,
    {
        let f = self.handler;
        let cursor = self.cursor;
        let data_provider = self.data_provider;
        let barrier = Arc::new(barrier);
        let barrier2 = barrier.clone();

        let thread = std::thread::spawn(move || loop {
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
        });

        Box::new(Handle {
            barrier: barrier2,
            thread: Some(thread),
        })
    }

    fn get_cursor(&self) -> Arc<AtomicSequence> {
        self.cursor.clone()
    }
}

impl<F, D, T> EventProcessor<T> for ProcessorMut<D, F, T>
where
    D: DataProvider<T> + 'static,
    F: Fn(&mut T, Sequence, bool) + Send + 'static,
{
    fn run<B>(self, barrier: B) -> Box<dyn EventProcessorHandle>
    where
        B: SequenceBarrier + 'static,
    {
        let f = self.handler;
        let cursor = self.cursor;
        let data_provider = self.data_provider;
        let barrier = Arc::new(barrier);
        let barrier2 = barrier.clone();

        let thread = std::thread::spawn(move || loop {
            let next = cursor.get() + 1;
            let available = match barrier.wait_for(next) {
                Some(seq) => seq,
                None => return,
            };

            for i in next..=available {
                unsafe { f(data_provider.get_mut(i), i, i == available) }
            }

            cursor.set(available);
            barrier.signal();
        });

        Box::new(Handle {
            barrier: barrier2,
            thread: Some(thread),
        })
    }

    fn get_cursor(&self) -> Arc<AtomicSequence> {
        self.cursor.clone()
    }
}
