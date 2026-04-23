use crate::ParameterAutomationSummary;

/// A single contiguous span of automation data for one parameter within a processing epoch.
#[derive(Clone, Debug, PartialEq)]
pub struct AutomationContinuitySegment {
    /// Processing epoch this segment belongs to.
    pub processing_epoch: u64,
    /// Shared-memory lease ID associated with the blocks in this segment.
    pub lease_id: String,
    /// Aggregated automation statistics for this segment.
    pub summary: ParameterAutomationSummary,
}

/// Tracks automation continuity for a single parameter across multiple epochs and leases.
///
/// Segments are merged when consecutive blocks share the same epoch and lease. A new segment
/// is appended when the epoch or lease changes. `lease_rollovers` counts how many times the
/// lease ID changed, which indicates a shared-memory rebind.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct AutomationContinuityReport {
    /// The parameter this report covers.
    pub parameter_id: u32,
    /// Ordered list of continuity segments.
    pub segments: Vec<AutomationContinuitySegment>,
    /// Number of times the shared-memory lease changed across segments.
    pub lease_rollovers: usize,
}

impl AutomationContinuityReport {
    /// Records a new automation summary into the report, merging it with the last segment when
    /// the epoch and lease ID match.
    pub fn record(
        &mut self,
        processing_epoch: u64,
        lease_id: impl Into<String>,
        summary: ParameterAutomationSummary,
    ) {
        if summary.parameter_id == 0 {
            return;
        }

        if self.parameter_id == 0 {
            self.parameter_id = summary.parameter_id;
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
                self.segments.push(AutomationContinuitySegment {
                    processing_epoch,
                    lease_id,
                    summary,
                });
            }
            None => {
                self.segments.push(AutomationContinuitySegment {
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

    /// Returns a single [`ParameterAutomationSummary`] that aggregates all segments.
    pub fn aggregate(&self) -> ParameterAutomationSummary {
        let mut aggregate = ParameterAutomationSummary {
            parameter_id: self.parameter_id,
            ..ParameterAutomationSummary::default()
        };
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
