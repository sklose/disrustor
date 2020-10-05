use std::sync::atomic::*;

/// Reference to a sequence that is valid for a specific lifetime.
///
/// A SequenceGuard ensures that the underlying data that is referenced
/// by the sequence can only be accessed for a certain time. This mechanism
/// is used to provide a sound API that prevents data races. SequenceGuards
/// can only be constructed from within this library.
pub struct SequenceGuard {
    sequence: i64,
}

impl SequenceGuard {
    pub unsafe fn new(seq: i64) -> Self {
        Self { sequence: seq }
    }
}

impl From<&SequenceGuard> for i64 {
    fn from(v: &SequenceGuard) -> Self {
        v.sequence
    }
}

impl From<&mut SequenceGuard> for i64 {
    fn from(v: &mut SequenceGuard) -> Self {
        v.sequence
    }
}

impl std::fmt::Display for SequenceGuard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.sequence.fmt(f)
    }
}

impl std::fmt::Debug for SequenceGuard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.sequence.fmt(f)
    }
}

/// Concurrent sequence class used for tracking the progress of
/// the ring buffer and event processors.
///
/// It supports a number of concurrent operations including CAS and
/// order writes. Also attempts to be more efficient with regards to
/// false sharing by adding padding around the atomic field.
#[repr(align(64))]
pub struct Sequence {
    _pad: [u8; 56],
    offset: AtomicI64,
}

impl Sequence {
    /// Perform a volatile read of this sequence's value.
    pub fn get(&self) -> i64 {
        self.offset.load(Ordering::Acquire)
    }

    pub fn set(&self, value: i64) {
        self.offset.store(value, Ordering::Release);
    }

    pub fn compare_exchange(&self, current: i64, new: i64) -> bool {
        self.offset
            .compare_exchange(current, new, Ordering::SeqCst, Ordering::Acquire)
            .is_ok()
    }
}

impl Default for Sequence {
    fn default() -> Self {
        Self {
            offset: AtomicI64::new(-1),
            _pad: [0; 56],
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn offset_can_be_used_as_i64() {
        let o = unsafe { SequenceGuard::new(42) };
        assert_eq!(i64::from(&o), 42);
    }

    #[test]
    fn offset_renders_like_i64() {
        let o = unsafe { SequenceGuard::new(42) };
        assert_eq!(format!("{}", o), "42");
        assert_eq!(format!("{:?}", o), "42");
        assert_eq!(format!("{:#?}", o), "42");
    }
}
