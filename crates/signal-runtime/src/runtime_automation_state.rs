#[path = "runtime_automation_state/batch.rs"]
mod batch;
#[path = "runtime_automation_state/math.rs"]
mod math;

use super::*;
use batch::graph_parameter_batch;
pub(crate) use batch::RuntimeAutomationBatchMetrics;
pub(crate) use math::{graph_parameter_target_from_runtime_target, graph_stage_parameter_sort_key};

pub(crate) struct RuntimeAutomationExecutionRecord {
    pub(crate) block_sequence: u64,
    pub(crate) timeline_position_samples: Option<i64>,
    pub(crate) transport_playing: Option<bool>,
    pub(crate) parameter_epoch: Option<u64>,
    pub(crate) parameter_event_count: usize,
    pub(crate) parameter_ignored_event_count: usize,
    pub(crate) parameter_sub_block_count: usize,
    pub(crate) parameter_coalesced_event_count: usize,
    pub(crate) metrics: RuntimeAutomationBatchMetrics,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct RuntimeAutomationState {
    projection: Option<RuntimeAutomationProjection>,
    projected_segment_count: usize,
    mapped_lane_count: usize,
    unmapped_lane_count: usize,
    hold_lane_count: usize,
    linear_lane_count: usize,
    last_batch_epoch: Option<u64>,
    last_batch_event_count: usize,
    last_batch_ignored_event_count: usize,
    last_batch_sub_block_count: usize,
    last_batch_coalesced_event_count: usize,
    last_batch_strategy_max_sub_blocks: usize,
    last_batch_min_ramp_step_samples: Option<usize>,
    last_batch_max_sample_offset: Option<usize>,
    last_block_sequence: Option<u64>,
    last_timeline_position_samples: Option<i64>,
    transport_playing: Option<bool>,
}

impl RuntimeAutomationState {
    pub(crate) fn apply_projection(&mut self, mut projection: RuntimeAutomationProjection) {
        projection.lane_count = projection.lanes.len();
        projection.point_count = projection
            .lanes
            .iter()
            .map(|lane| lane.points.len())
            .sum::<usize>();
        for lane in &mut projection.lanes {
            lane.point_count = lane.points.len();
        }
        self.projection = Some(projection);
    }

    pub(crate) fn graph_parameter_batch(
        &self,
        graph: &GraphProjection,
        transport: Option<TransportProjection>,
        frame_count: usize,
        epoch: u64,
    ) -> (
        Option<GraphParameterBatch>,
        HashSet<String>,
        RuntimeAutomationBatchMetrics,
    ) {
        graph_parameter_batch(
            self.projection.as_ref(),
            graph,
            transport,
            frame_count,
            epoch,
        )
    }

    pub(crate) fn record_execution(&mut self, record: RuntimeAutomationExecutionRecord) {
        self.projected_segment_count = record.metrics.projected_segment_count;
        self.mapped_lane_count = record.metrics.mapped_lane_count;
        self.unmapped_lane_count = record.metrics.unmapped_lane_count;
        self.hold_lane_count = record.metrics.hold_lane_count;
        self.linear_lane_count = record.metrics.linear_lane_count;
        self.last_batch_epoch = record.parameter_epoch;
        self.last_batch_event_count = record.parameter_event_count;
        self.last_batch_ignored_event_count = record.parameter_ignored_event_count;
        self.last_batch_sub_block_count = record.parameter_sub_block_count;
        self.last_batch_coalesced_event_count = record.parameter_coalesced_event_count;
        self.last_batch_strategy_max_sub_blocks = record.metrics.strategy_max_sub_blocks;
        self.last_batch_min_ramp_step_samples = record.metrics.min_ramp_step_samples;
        self.last_batch_max_sample_offset = record.metrics.max_sample_offset;
        self.last_block_sequence = Some(record.block_sequence);
        self.last_timeline_position_samples = record.timeline_position_samples;
        self.transport_playing = record.transport_playing;
    }

    pub(crate) fn reset(&mut self) {
        *self = Self::default();
    }

    pub(crate) fn snapshot(&self) -> RuntimeAutomationSnapshot {
        let lane_count = self
            .projection
            .as_ref()
            .map_or(0, |projection| projection.lane_count);
        let point_count = self
            .projection
            .as_ref()
            .map_or(0, |projection| projection.point_count);
        RuntimeAutomationSnapshot {
            lane_count,
            point_count,
            projected_segment_count: self.projected_segment_count,
            mapped_lane_count: self.mapped_lane_count,
            unmapped_lane_count: self.unmapped_lane_count,
            hold_lane_count: self.hold_lane_count,
            linear_lane_count: self.linear_lane_count,
            last_batch_epoch: self.last_batch_epoch,
            last_batch_event_count: self.last_batch_event_count,
            last_batch_ignored_event_count: self.last_batch_ignored_event_count,
            last_batch_sub_block_count: self.last_batch_sub_block_count,
            last_batch_coalesced_event_count: self.last_batch_coalesced_event_count,
            last_batch_strategy_max_sub_blocks: self.last_batch_strategy_max_sub_blocks,
            last_batch_min_ramp_step_samples: self.last_batch_min_ramp_step_samples,
            last_batch_max_sample_offset: self.last_batch_max_sample_offset,
            last_block_sequence: self.last_block_sequence,
            last_timeline_position_samples: self.last_timeline_position_samples,
            transport_playing: self.transport_playing,
        }
    }
}
