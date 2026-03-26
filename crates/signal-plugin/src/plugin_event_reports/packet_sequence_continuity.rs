use crate::EventPacketSummary;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BlockSequenceContinuitySegment {
    pub processing_epoch: u64,
    pub lease_id: String,
    pub first_block_sequence: u64,
    pub last_block_sequence: u64,
    pub observed_blocks: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EventPacketContinuitySegment {
    pub processing_epoch: u64,
    pub lease_id: String,
    pub summary: EventPacketSummary,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct EventPacketContinuityReport {
    pub segments: Vec<EventPacketContinuitySegment>,
    pub lease_rollovers: usize,
}

impl EventPacketContinuityReport {
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

    pub fn merge(&mut self, other: Self) {
        for segment in other.segments {
            self.record(segment.processing_epoch, segment.lease_id, segment.summary);
        }
    }

    pub fn aggregate(&self) -> EventPacketSummary {
        let mut aggregate = EventPacketSummary::default();
        for segment in &self.segments {
            aggregate.merge(segment.summary);
        }
        aggregate
    }

    pub fn segment_count(&self) -> usize {
        self.segments.len()
    }

    pub fn first_epoch(&self) -> Option<u64> {
        self.segments
            .first()
            .map(|segment| segment.processing_epoch)
    }

    pub fn last_epoch(&self) -> Option<u64> {
        self.segments.last().map(|segment| segment.processing_epoch)
    }

    pub fn segment_epochs(&self) -> Vec<u64> {
        self.segments
            .iter()
            .map(|segment| segment.processing_epoch)
            .collect()
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BlockSequenceContinuityReport {
    pub segments: Vec<BlockSequenceContinuitySegment>,
    pub lease_rollovers: usize,
    pub sequence_gaps: usize,
}

impl BlockSequenceContinuityReport {
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

    pub fn segment_count(&self) -> usize {
        self.segments.len()
    }

    pub fn segment_epochs(&self) -> Vec<u64> {
        self.segments
            .iter()
            .map(|segment| segment.processing_epoch)
            .collect()
    }

    pub fn first_block_sequence(&self) -> Option<u64> {
        self.segments
            .first()
            .map(|segment| segment.first_block_sequence)
    }

    pub fn last_block_sequence(&self) -> Option<u64> {
        self.segments
            .last()
            .map(|segment| segment.last_block_sequence)
    }
}
