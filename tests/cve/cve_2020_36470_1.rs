#![forbid(unsafe_code)]

use disrustor::internal::RingBuffer;

use std::marker::PhantomData;
use std::thread;

struct NonSend {
    created_thread: thread::ThreadId,
    // Mark this struct as `NonSend`
    _marker: PhantomData<*mut ()>,
}

impl Default for NonSend {
    fn default() -> Self {
        NonSend {
            created_thread: thread::current().id(),
            _marker: PhantomData,
        }
    }
}

impl Drop for NonSend {
    fn drop(&mut self) {
        if thread::current().id() != self.created_thread {
            panic!("NonSend destructor is running on a wrong thread!");
        }
    }
}

fn main() {
    let buffer = RingBuffer::<NonSend>::new(1);

    let handle = thread::spawn(move || {
        drop(buffer);
    });

    handle.join().unwrap();
}