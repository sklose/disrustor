use crate::lib::*;
use std::sync::Arc;

pub struct ClaimedRange {
    start: i64,
    end: i64,
}

/// Coordinates claiming sequences for access to a data structure while tracking dependent {@link Sequence}s
pub trait Sequencer {
    type Barrier: SequenceBarrier;

    fn next(&self, count: usize) -> ClaimedRange;
    fn publish(&self, range: ClaimedRange);

    ///
    /// Create a new SequenceBarrier to be used by an EventProcessor to track which messages
    /// are available to be read from the ring buffer given a list of sequences to track.
    ///
    /// @param sequencesToTrack All of the sequences that the newly constructed barrier will wait on.
    /// @return A sequence barrier that will track the specified sequences.
    /// @see SequenceBarrier
    fn create_barrier(&mut self, gating_sequences: &[Arc<Sequence>]) -> Self::Barrier;

    fn add_gating_sequence(&mut self, gating_sequence: &Arc<Sequence>);

    /// Get the current cursor value.
    fn get_cursor(&self) -> Arc<Sequence>;
    fn drain(self);
}
