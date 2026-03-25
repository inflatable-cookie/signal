use super::math::{
    automation_linear_segment_sample_times_for_block, automation_linear_value_at_time,
    automation_sample_offset_for_block, automation_value_at_time,
    graph_parameter_target_from_runtime_automation_target, sorted_runtime_automation_points,
};
use super::*;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct RuntimeAutomationBatchMetrics {
    pub(crate) projected_segment_count: usize,
    pub(crate) mapped_lane_count: usize,
    pub(crate) unmapped_lane_count: usize,
    pub(crate) hold_lane_count: usize,
    pub(crate) linear_lane_count: usize,
    pub(crate) strategy_max_sub_blocks: usize,
    pub(crate) min_ramp_step_samples: Option<usize>,
    pub(crate) max_sample_offset: Option<usize>,
}

pub(crate) fn graph_parameter_batch(
    projection: Option<&RuntimeAutomationProjection>,
    graph: &GraphProjection,
    transport: Option<TransportProjection>,
    frame_count: usize,
    epoch: u64,
) -> (
    Option<GraphParameterBatch>,
    HashSet<String>,
    RuntimeAutomationBatchMetrics,
) {
    let Some(projection) = projection else {
        return (
            None,
            HashSet::new(),
            RuntimeAutomationBatchMetrics::default(),
        );
    };
    let Some(transport) = transport else {
        return (
            None,
            HashSet::new(),
            RuntimeAutomationBatchMetrics::default(),
        );
    };

    let mut metrics = RuntimeAutomationBatchMetrics::default();
    let mut events = Vec::new();
    let mut mapped_targets = HashSet::new();

    for lane in &projection.lanes {
        metrics.projected_segment_count = metrics
            .projected_segment_count
            .saturating_add(lane.points.len().saturating_sub(1));
        metrics.strategy_max_sub_blocks = metrics
            .strategy_max_sub_blocks
            .max(lane.resolution.max_sub_blocks.max(1));
        match lane.interpolation {
            RuntimeAutomationInterpolation::Hold => {
                metrics.hold_lane_count = metrics.hold_lane_count.saturating_add(1);
            }
            RuntimeAutomationInterpolation::Linear => {
                metrics.linear_lane_count = metrics.linear_lane_count.saturating_add(1);
                metrics.min_ramp_step_samples = Some(
                    metrics
                        .min_ramp_step_samples
                        .map_or(lane.resolution.ramp_step_samples.max(1), |current| {
                            current.min(lane.resolution.ramp_step_samples.max(1))
                        }),
                );
            }
        }

        let Some(target) =
            graph_parameter_target_from_runtime_automation_target(graph, &lane.target)
        else {
            metrics.unmapped_lane_count = metrics.unmapped_lane_count.saturating_add(1);
            continue;
        };
        metrics.mapped_lane_count = metrics.mapped_lane_count.saturating_add(1);
        mapped_targets.insert(lane.target.parameter_path());

        let mut block_events = BTreeMap::new();
        let sorted_points = sorted_runtime_automation_points(&lane.points);
        block_events.insert(
            0usize,
            automation_value_at_time(
                lane.base_normalized_value,
                &sorted_points,
                lane.interpolation,
                transport.timeline_position_samples,
            ),
        );
        for point in &sorted_points {
            if let Some(sample_offset) = automation_sample_offset_for_block(
                transport.timeline_position_samples,
                frame_count,
                transport.loop_state,
                point.time_samples,
            ) {
                block_events.insert(sample_offset, point.normalized_value);
            }
        }
        if lane.interpolation == RuntimeAutomationInterpolation::Linear {
            for window in sorted_points.windows(2) {
                let segment_start = &window[0];
                let segment_end = &window[1];
                if segment_end.time_samples <= segment_start.time_samples {
                    continue;
                }
                for sample_time in automation_linear_segment_sample_times_for_block(
                    transport.timeline_position_samples,
                    frame_count,
                    transport.loop_state,
                    segment_start.time_samples,
                    segment_end.time_samples,
                    lane.resolution.ramp_step_samples.max(1),
                ) {
                    if let Some(sample_offset) = automation_sample_offset_for_block(
                        transport.timeline_position_samples,
                        frame_count,
                        transport.loop_state,
                        sample_time,
                    ) {
                        block_events.insert(
                            sample_offset,
                            automation_linear_value_at_time(
                                segment_start.time_samples,
                                segment_start.normalized_value,
                                segment_end.time_samples,
                                segment_end.normalized_value,
                                sample_time,
                            ),
                        );
                    }
                }
            }
        }

        for (sample_offset, value) in block_events {
            metrics.max_sample_offset = Some(
                metrics
                    .max_sample_offset
                    .map_or(sample_offset, |last| last.max(sample_offset)),
            );
            events.push(GraphParameterEvent {
                sample_offset,
                target: target.clone(),
                value,
            });
        }
    }

    if events.is_empty() {
        return (None, mapped_targets, metrics);
    }

    events.sort_by(|left, right| {
        left.target
            .node_id
            .cmp(&right.target.node_id)
            .then(left.target.stage_index.cmp(&right.target.stage_index))
            .then(
                super::graph_stage_parameter_sort_key(left.target.parameter).cmp(
                    &super::graph_stage_parameter_sort_key(right.target.parameter),
                ),
            )
            .then(left.sample_offset.cmp(&right.sample_offset))
    });

    (
        Some(GraphParameterBatch {
            epoch,
            strategy: GraphParameterApplicationStrategy::SplitAtEvents {
                max_sub_blocks: metrics.strategy_max_sub_blocks.max(1),
            },
            events,
        }),
        mapped_targets,
        metrics,
    )
}
