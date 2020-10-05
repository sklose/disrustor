use crate::lib::*;

pub(crate) fn min_cursor_sequence<S: AsRef<Sequence>>(sequences: &[S]) -> i64 {
    sequences
        .iter()
        .map(|s| s.as_ref().get())
        .min()
        .unwrap_or_default()
}
