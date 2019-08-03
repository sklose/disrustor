use super::prelude::*;

pub struct ThreadedExecutor {
    runnables: Vec<Box<dyn Runnable>>,
}

pub struct ThreadedExecutorHandle {
    threads: Vec<std::thread::JoinHandle<()>>,
}

impl EventProcessorExecutor for ThreadedExecutor {
    type Handle = ThreadedExecutorHandle;

    fn with_runnables(runnables: Vec<Box<dyn Runnable>>) -> Self {
        Self { runnables }
    }

    fn spawn(self) -> Self::Handle {
        let mut threads = Vec::new();
        for r in self.runnables.into_iter() {
            threads.push(std::thread::spawn(move || r.run()));
        }

        ThreadedExecutorHandle { threads }
    }
}

impl ExecutorHandle for ThreadedExecutorHandle {
    fn join(self) {
        for t in self.threads {
            t.join().unwrap();
        }
    }
}
