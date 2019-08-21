use crate::prelude::*;

pub fn min_cursor_sequence<S: AsRef<AtomicSequence>>(sequences: &[S]) -> Sequence {
    sequences
        .iter()
        .map(|s| s.as_ref().get())
        .min()
        .unwrap_or_default()
}
