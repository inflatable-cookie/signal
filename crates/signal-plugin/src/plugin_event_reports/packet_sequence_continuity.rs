use crate::EventPacketSummary;

/// A contiguous span of observed blocks within a single processing epoch and lease.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BlockSequenceContinuitySegment {
    /// Processing epoch this segment belongs to.
    pub processing_epoch: u64,
    /// Shared-memory lease ID for the blocks in this segment.
    pub lease_id: String,
    /// Block sequence number of the first block in this segment.
    pub first_block_sequence: u64,
    /// Block sequence number of the last block in this segment.
    pub last_block_sequence: u64,
    /// Number of blocks observed in this segment.
    pub observed_blocks: u64,
}

/// A contiguous span of event packets within a single processing epoch and lease.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EventPacketContinuitySegment {
    /// Processing epoch this segment belongs to.
    pub processing_epoch: u64,
    /// Shared-memory lease ID for the packets in this segment.
    pub lease_id: String,
    /// Aggregated event-type counts for this segment.
    pub summary: EventPacketSummary,
}

/// Tracks event-packet continuity across epochs and lease boundaries.
///
/// Segments are merged when consecutive packets share the same epoch and lease. Empty packets
/// (zero events) are ignored. `lease_rollovers` counts lease-ID changes between segments.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct EventPacketContinuityReport {
    /// Ordered list of continuity segments.
    pub segments: Vec<EventPacketContinuitySegment>,
    /// Number of times the shared-memory lease changed across segments.
    pub lease_rollovers: usize,
}

impl EventPacketContinuityReport {
    /// Records a packet summary, merging it with the last segment when epoch and lease match.
    ///
    /// Packets with zero total events are silently ignored.
    pub fn record(
        &mut self,
        processing_epoch: u64,
        lease_id: impl Into<String>,
        summary: EventPacketSummary,
    ) {
        if summary.total_events == 0 {
            return;
        }

        let lease_id = lease_id.into();
        match self.segments.last_mut() {
            Some(last)
                if last.processing_epoch == processing_epoch && last.lease_id == lease_id =>
            {
                last.summary.merge(summary);
            }
            Some(last) => {
                if last.lease_id != lease_id {
                    self.lease_rollovers = self.lease_rollovers.saturating_add(1);
                }
                self.segments.push(EventPacketContinuitySegment {
                    processing_epoch,
                    lease_id,
                    summary,
                });
            }
            None => {
                self.segments.push(EventPacketContinuitySegment {
                    processing_epoch,
                    lease_id,
                    summary,
                });
            }
        }
    }

    /// Merges all segments from `other` into this report.
    pub fn merge(&mut self, other: Self) {
        for segment in other.segments {
            self.record(segment.processing_epoch, segment.lease_id, segment.summary);
        }
    }

    /// Returns a single [`EventPacketSummary`] that aggregates all segments.
    pub fn aggregate(&self) -> EventPacketSummary {
        let mut aggregate = EventPacketSummary::default();
        for segment in &self.segments {
            aggregate.merge(segment.summary);
        }
        aggregate
    }

    /// Returns the number of segments in this report.
    pub fn segment_count(&self) -> usize {
        self.segments.len()
    }

    /// Returns the processing epoch of the first segment, or `None` if the report is empty.
    pub fn first_epoch(&self) -> Option<u64> {
        self.segments
            .first()
            .map(|segment| segment.processing_epoch)
    }

    /// Returns the processing epoch of the last segment, or `None` if the report is empty.
    pub fn last_epoch(&self) -> Option<u64> {
        self.segments.last().map(|segment| segment.processing_epoch)
    }

    /// Returns the processing epoch of every segment in order.
    pub fn segment_epochs(&self) -> Vec<u64> {
        self.segments
            .iter()
            .map(|segment| segment.processing_epoch)
            .collect()
    }
}

/// Tracks block-sequence continuity across epochs and lease boundaries.
///
/// Each new block sequence number is recorded and checked for monotonic continuity. Gaps
/// (non-consecutive sequence numbers within the same epoch/lease) increment `sequence_gaps`.
/// Lease changes increment `lease_rollovers`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BlockSequenceContinuityReport {
    /// Ordered list of continuity segments.
    pub segments: Vec<BlockSequenceContinuitySegment>,
    /// Number of times the shared-memory lease changed across segments.
    pub lease_rollovers: usize,
    /// Number of non-consecutive sequence numbers observed within a single epoch/lease.
    pub sequence_gaps: usize,
}

impl BlockSequenceContinuityReport {
    /// Records a block sequence number, extending the last segment or creating a new one.
    pub fn record(
        &mut self,
        processing_epoch: u64,
        lease_id: impl Into<String>,
        block_sequence: u64,
    ) {
        let lease_id = lease_id.into();
        match self.segments.last_mut() {
            Some(last)
                if last.processing_epoch == processing_epoch && last.lease_id == lease_id =>
            {
                if block_sequence == last.last_block_sequence.saturating_add(1) {
                    last.last_block_sequence = block_sequence;
                    last.observed_blocks = last.observed_blocks.saturating_add(1);
                } else {
                    self.sequence_gaps = self.sequence_gaps.saturating_add(1);
                    self.segments.push(BlockSequenceContinuitySegment {
                        processing_epoch,
                        lease_id,
                        first_block_sequence: block_sequence,
                        last_block_sequence: block_sequence,
                        observed_blocks: 1,
                    });
                }
            }
            Some(last) => {
                if last.lease_id != lease_id {
                    self.lease_rollovers = self.lease_rollovers.saturating_add(1);
                }
                self.segments.push(BlockSequenceContinuitySegment {
                    processing_epoch,
                    lease_id,
                    first_block_sequence: block_sequence,
                    last_block_sequence: block_sequence,
                    observed_blocks: 1,
                });
            }
            None => {
                self.segments.push(BlockSequenceContinuitySegment {
                    processing_epoch,
                    lease_id,
                    first_block_sequence: block_sequence,
                    last_block_sequence: block_sequence,
                    observed_blocks: 1,
                });
            }
        }
    }

    /// Merges all segments from `other` by replaying each block sequence number.
    pub fn merge(&mut self, other: Self) {
        for segment in other.segments {
            for block_sequence in segment.first_block_sequence..=segment.last_block_sequence {
                self.record(
                    segment.processing_epoch,
                    segment.lease_id.clone(),
                    block_sequence,
                );
            }
        }
    }

    /// Returns the number of segments in this report.
    pub fn segment_count(&self) -> usize {
        self.segments.len()
    }

    /// Returns the processing epoch of every segment in order.
    pub fn segment_epochs(&self) -> Vec<u64> {
        self.segments
            .iter()
            .map(|segment| segment.processing_epoch)
            .collect()
    }

    /// Returns the first block sequence number observed, or `None` if the report is empty.
    pub fn first_block_sequence(&self) -> Option<u64> {
        self.segments
            .first()
            .map(|segment| segment.first_block_sequence)
    }

    /// Returns the last block sequence number observed, or `None` if the report is empty.
    pub fn last_block_sequence(&self) -> Option<u64> {
        self.segments
            .last()
            .map(|segment| segment.last_block_sequence)
    }
}
