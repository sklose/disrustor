use crate::prelude::*;
use std::sync::Arc;

pub fn min_cursor_sequence(sequences: &[Arc<AtomicSequence>]) -> Sequence {
    sequences.iter().map(|s| s.get()).min().unwrap_or_default()
}
