use crate::prelude::*;
use std::{
    iter::*,
    sync::atomic::{AtomicU64, Ordering},
};

pub fn min_cursor_sequence<S: AsRef<AtomicSequence>>(sequences: &[S]) -> Sequence {
    sequences
        .iter()
        .map(|s| s.as_ref().get())
        .min()
        .unwrap_or_default()
}

pub struct BitMap {
    slots: Vec<AtomicU64>,
    index_mask: i64,
    index_shift: i64,
    word_bits_mask: usize,
}

impl BitMap {
    pub fn new(capacity: usize) -> Self {
        fn log2(mut n: i64) -> i64 {
            let mut r = 0;
            loop {
                n >>= 1;
                if n == 0 {
                    return r;
                }
                r += 1;
            }
        }

        let word_bits = std::mem::size_of::<AtomicU64>() * 8;
        let remainder = capacity % word_bits;
        let len = capacity / word_bits + if remainder == 0 { 0 } else { 1 };
        let slots = Vec::from_iter(repeat_with(AtomicU64::default).take(len));
        Self {
            slots,
            index_mask: (capacity - 1) as i64,
            index_shift: log2(word_bits as i64),
            word_bits_mask: word_bits - 1,
        }
    }

    pub fn is_set(&self, sequence: Sequence) -> bool {
        let index = (sequence & self.index_mask >> self.index_shift) as usize;
        let slot = unsafe { self.slots.get_unchecked(index) };
        let val = slot.load(Ordering::SeqCst);
        val & (1 << (index & self.word_bits_mask)) != 0
    }

    pub fn set(&self, sequence: Sequence) {
        let index = (sequence & self.index_mask >> self.index_shift) as usize;
        let slot = unsafe { self.slots.get_unchecked(index) };
        let val = 1 << (index & self.word_bits_mask);
        slot.fetch_or(val, Ordering::SeqCst);
    }

    pub fn unset(&self, sequence: Sequence) {
        let index = (sequence & self.index_mask >> self.index_shift) as usize;
        let slot = unsafe { self.slots.get_unchecked(index) };
        let val = !(1 << (index & self.word_bits_mask));
        slot.fetch_and(val, Ordering::SeqCst);
    }
}

#[cfg(test)]
mod test {
    use super::*;
    mod bitmap {
        use super::*;

        #[test]
        pub fn is_unset_by_default() {
            let bitmap = BitMap::new(100);
            for n in 0..100 {
                assert!(!bitmap.is_set(n));
            }
        }

        #[test]
        pub fn sets_and_gets_individual_bits() {
            let bitmap = BitMap::new(100);
            for n in 0..100 {
                if n % 2 == 0 {
                    bitmap.set(n);
                } else {
                    bitmap.unset(n);
                }
            }

            for n in 0..100 {
                assert_eq!(bitmap.is_set(n), n % 2 == 0);
            }
        }
    }
}
