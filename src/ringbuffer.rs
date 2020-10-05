use crate::lib::*;
use std::cell::UnsafeCell;

pub trait DataProvider<T>: Sync + Send {
    fn buffer_size(&self) -> usize;
    fn get_mut(&self, guard: &mut SequenceGuard) -> &mut T;
    fn get(&self, guard: &SequenceGuard) -> &T;
}

/// Ring based store of reusable entries containing the data representing
/// an event being exchanged between event producer and {@link EventProcessor}s.
///
/// @param <E> implementation storing the data for sharing during exchange or parallel coordination of an event.
pub struct RingBuffer<T> {
    data: Vec<UnsafeCell<T>>,
    capacity: usize,
    mask: usize,
}

impl<T: Default> RingBuffer<T> {
    pub fn new(capacity: usize) -> Self {
        assert!(capacity.is_power_of_two(), "capacity must be power of two");

        let mut data = Vec::with_capacity(capacity);
        for _ in 0..capacity {
            data.push(UnsafeCell::default());
        }

        RingBuffer {
            data,
            capacity,
            mask: capacity - 1,
        }
    }
}

impl<T> DataProvider<T> for RingBuffer<T> {
    fn get_mut(&self, guard: &mut SequenceGuard) -> &mut T {
        let sequence = i64::from(guard);
        let index = sequence as usize & self.mask;
        let cell = unsafe { self.data.get_unchecked(index) };
        unsafe { &mut *cell.get() }
    }

    fn get(&self, guard: &SequenceGuard) -> &T {
        let sequence = i64::from(guard);
        let index = sequence as usize & self.mask;
        let cell = unsafe { self.data.get_unchecked(index) };
        unsafe { &*cell.get() }
    }

    fn buffer_size(&self) -> usize {
        self.capacity
    }
}

unsafe impl<T> Send for RingBuffer<T> {}
unsafe impl<T> Sync for RingBuffer<T> {}

#[cfg(test)]
mod test {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn writes_and_reads_data() {
        const SIZE: i64 = 256;
        let buffer = RingBuffer::new(SIZE as usize);

        for i in 0..SIZE {
            let mut guard = unsafe { SequenceGuard::new(i) };
            *buffer.get_mut(&mut guard) = i;
        }

        for i in 0..SIZE {
            let mut guard = unsafe { SequenceGuard::new(i) };
            *buffer.get_mut(&mut guard) *= 2;
        }

        for i in 0..SIZE {
            let mut guard = unsafe { SequenceGuard::new(i) };
            assert_eq!(*buffer.get_mut(&mut guard), i * 2);
        }
    }

    #[test]
    fn writes_are_visible_across_threads() {
        const SIZE: i64 = 256;
        let buffer: Arc<RingBuffer<i64>> = Arc::new(RingBuffer::new(SIZE as usize));

        let b1 = buffer.clone();
        let t1 = std::thread::spawn(move || {
            for i in 0..SIZE {
                let mut guard = unsafe { SequenceGuard::new(i) };
                *b1.get_mut(&mut guard) = i;
            }
        });

        let b2 = buffer.clone();
        let t2 = std::thread::spawn(move || {
            std::thread::park();
            for i in 0..SIZE {
                let mut guard = unsafe { SequenceGuard::new(i) };
                let value = *b2.get_mut(&mut guard);
                if value != i {
                    panic!("cannot read what was written")
                }
            }
        });

        t1.join().unwrap();
        t2.thread().unpark();
        t2.join().unwrap();
    }
}
