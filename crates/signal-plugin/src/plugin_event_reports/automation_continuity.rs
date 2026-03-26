use crate::ParameterAutomationSummary;

#[derive(Clone, Debug, PartialEq)]
pub struct AutomationContinuitySegment {
    pub processing_epoch: u64,
    pub lease_id: String,
    pub summary: ParameterAutomationSummary,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct AutomationContinuityReport {
    pub parameter_id: u32,
    pub segments: Vec<AutomationContinuitySegment>,
    pub lease_rollovers: usize,
}

impl AutomationContinuityReport {
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

    pub fn merge(&mut self, other: Self) {
        for segment in other.segments {
            self.record(segment.processing_epoch, segment.lease_id, segment.summary);
        }
    }

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
