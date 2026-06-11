/// Per-parameter automation snapshot: lane counts and batch statistics.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RuntimeAutomationSnapshot {
    /// Total number of automation lanes for this parameter.
    pub lane_count: usize,
    /// Total number of automation breakpoint points.
    pub point_count: usize,
    /// Number of projected automation segments.
    pub projected_segment_count: usize,
    /// Number of lanes that are mapped to a parameter.
    pub mapped_lane_count: usize,
    /// Number of lanes that are not mapped to any parameter.
    pub unmapped_lane_count: usize,
    /// Number of lanes using hold (step) interpolation.
    pub hold_lane_count: usize,
    /// Number of lanes using linear interpolation.
    pub linear_lane_count: usize,
    /// Processing epoch of the last dispatched automation batch.
    pub last_batch_epoch: Option<u64>,
    /// Number of events in the last dispatched batch.
    pub last_batch_event_count: usize,
    /// Number of events ignored in the last batch.
    pub last_batch_ignored_event_count: usize,
    /// Number of sub-blocks generated in the last batch.
    pub last_batch_sub_block_count: usize,
    /// Number of events coalesced in the last batch.
    pub last_batch_coalesced_event_count: usize,
    /// Maximum sub-blocks allowed by the current batch strategy.
    pub last_batch_strategy_max_sub_blocks: usize,
    /// Minimum ramp step size in samples for the last batch.
    pub last_batch_min_ramp_step_samples: Option<usize>,
    /// Maximum sample offset of any event in the last batch.
    pub last_batch_max_sample_offset: Option<usize>,
    /// Block sequence of the most recent block that received automation.
    pub last_block_sequence: Option<u64>,
    /// Timeline position in samples at the most recent block.
    pub last_timeline_position_samples: Option<i64>,
    /// Whether the transport was playing at the most recent block.
    pub transport_playing: Option<bool>,
}
