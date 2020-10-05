use crate::prelude::*;
use std::cell::UnsafeCell;

pub struct RingBuffer<T> {
    data: Vec<UnsafeCell<T>>,
    capacity: usize,
    mask: usize,
}

impl<T: Default> RingBuffer<T> {
    pub fn new(capacity: usize) -> Self {
        assert!(
            (capacity != 0) && ((capacity & (capacity - 1)) == 0),
            "capacity must be power of two"
        );

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
    unsafe fn get_mut(&self, sequence: Sequence) -> &mut T {
        let index = sequence as usize & self.mask;
        let cell = self.data.get_unchecked(index);
        &mut *cell.get()
    }

    unsafe fn get(&self, sequence: Sequence) -> &T {
        let index = sequence as usize & self.mask;
        let cell = self.data.get_unchecked(index);
        &*cell.get()
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
            unsafe {
                *buffer.get_mut(i) = i;
            }
        }

        for i in 0..SIZE {
            unsafe {
                *buffer.get_mut(i) *= 2;
            }
        }

        for i in 0..SIZE {
            unsafe {
                assert_eq!(*buffer.get_mut(i), i * 2);
            }
        }
    }

    #[test]
    fn writes_are_visible_across_threads() {
        const SIZE: i64 = 256;
        let buffer: Arc<RingBuffer<i64>> = Arc::new(RingBuffer::new(SIZE as usize));

        let b1 = buffer.clone();
        let t1 = std::thread::spawn(move || {
            for i in 0..SIZE {
                unsafe {
                    *b1.get_mut(i) = i;
                }
            }
        });

        let b2 = buffer.clone();
        let t2 = std::thread::spawn(move || {
            std::thread::park();
            for i in 0..SIZE {
                unsafe {
                    let value = *b2.get_mut(i);
                    if value != i {
                        panic!("cannot read what was written")
                    }
                }
            }
        });

        t1.join().unwrap();
        t2.thread().unpark();
        t2.join().unwrap();
    }
}
