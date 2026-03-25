use super::super::*;
use crate::interfaces::RuntimeAutomationTargetProjection;
use signal_graph::{GraphParameterTarget, GraphStageParameter};

pub(super) fn sorted_runtime_automation_points(
    points: &[crate::interfaces::RuntimeAutomationPointProjection],
) -> Vec<crate::interfaces::RuntimeAutomationPointProjection> {
    let mut points = points.to_vec();
    points.sort_by_key(|point| point.time_samples);
    points
}

pub(super) fn automation_value_at_time(
    base_normalized_value: f32,
    points: &[crate::interfaces::RuntimeAutomationPointProjection],
    interpolation: RuntimeAutomationInterpolation,
    time_samples: i64,
) -> f32 {
    let mut previous_time = None;
    let mut previous_value = base_normalized_value;

    for point in points {
        if point.time_samples > time_samples {
            return match (interpolation, previous_time) {
                (RuntimeAutomationInterpolation::Linear, Some(previous_time))
                    if point.time_samples > previous_time =>
                {
                    automation_linear_value_at_time(
                        previous_time,
                        previous_value,
                        point.time_samples,
                        point.normalized_value,
                        time_samples,
                    )
                }
                _ => previous_value,
            };
        }
        previous_time = Some(point.time_samples);
        previous_value = point.normalized_value;
    }

    previous_value
}

pub(super) fn automation_linear_value_at_time(
    start_time_samples: i64,
    start_value: f32,
    end_time_samples: i64,
    end_value: f32,
    time_samples: i64,
) -> f32 {
    if end_time_samples <= start_time_samples {
        return end_value;
    }
    let span = (end_time_samples - start_time_samples) as f32;
    let offset =
        (time_samples - start_time_samples).clamp(0, end_time_samples - start_time_samples) as f32;
    start_value + ((end_value - start_value) * (offset / span))
}

pub(super) fn automation_linear_segment_sample_times_for_block(
    block_start_samples: i64,
    frame_count: usize,
    loop_state: Option<crate::interfaces::LoopRegion>,
    segment_start_samples: i64,
    segment_end_samples: i64,
    ramp_step_samples: usize,
) -> Vec<i64> {
    if segment_end_samples <= segment_start_samples {
        return Vec::new();
    }

    let mut sample_times = Vec::new();
    let step = ramp_step_samples.max(1) as i64;
    let block_end_samples = block_start_samples.saturating_add(frame_count as i64);
    let first_sample = if block_start_samples < segment_start_samples {
        segment_start_samples.saturating_add(step)
    } else {
        let delta = block_start_samples.saturating_sub(segment_start_samples);
        let steps = delta.div_euclid(step).saturating_add(1);
        segment_start_samples.saturating_add(steps.saturating_mul(step))
    };

    let mut sample_time = first_sample;
    while sample_time < segment_end_samples && sample_time < block_end_samples {
        if automation_sample_offset_for_block(
            block_start_samples,
            frame_count,
            loop_state,
            sample_time,
        )
        .is_some()
        {
            sample_times.push(sample_time);
        }
        sample_time = sample_time.saturating_add(step);
    }
    sample_times
}

pub(super) fn automation_sample_offset_for_block(
    block_start_samples: i64,
    frame_count: usize,
    loop_state: Option<crate::interfaces::LoopRegion>,
    point_time_samples: i64,
) -> Option<usize> {
    let block_end_samples = block_start_samples.saturating_add(frame_count as i64);
    if point_time_samples >= block_start_samples && point_time_samples < block_end_samples {
        return usize::try_from(point_time_samples.saturating_sub(block_start_samples)).ok();
    }

    let Some(loop_state) = loop_state else {
        return None;
    };
    if loop_state.end_samples <= loop_state.start_samples
        || block_end_samples <= loop_state.end_samples
    {
        return None;
    }

    let wrapped_span = block_end_samples.saturating_sub(loop_state.end_samples);
    if point_time_samples < loop_state.start_samples
        || point_time_samples >= loop_state.start_samples.saturating_add(wrapped_span)
    {
        return None;
    }

    usize::try_from(
        loop_state
            .end_samples
            .saturating_sub(block_start_samples)
            .saturating_add(point_time_samples.saturating_sub(loop_state.start_samples)),
    )
    .ok()
}

pub(super) fn graph_parameter_target_from_runtime_automation_target(
    graph: &GraphProjection,
    target: &RuntimeAutomationTargetProjection,
) -> Option<GraphParameterTarget> {
    graph_parameter_target_from_runtime_parameter_path(graph, &target.node_id, &target.parameter_id)
}

pub(crate) fn graph_parameter_target_from_runtime_target(
    graph: &GraphProjection,
    target: &str,
) -> Option<GraphParameterTarget> {
    let (node_id, parameter_id) = target.rsplit_once('.')?;
    graph_parameter_target_from_runtime_parameter_path(graph, node_id, parameter_id)
}

fn graph_parameter_target_from_runtime_parameter_path(
    graph: &GraphProjection,
    node_id: &str,
    parameter_id: &str,
) -> Option<GraphParameterTarget> {
    let parameter = graph_stage_parameter_from_runtime_parameter_id(parameter_id)?;
    let stage_index = graph
        .nodes
        .iter()
        .find(|node| node.node_id == node_id)
        .and_then(|node| {
            node.stages
                .iter()
                .position(|stage| graph_stage_parameter_applies_to(parameter, stage))
        })?;
    Some(GraphParameterTarget {
        node_id: node_id.to_string(),
        stage_index,
        parameter,
    })
}

fn graph_stage_parameter_from_runtime_parameter_id(
    parameter_id: &str,
) -> Option<GraphStageParameter> {
    match parameter_id {
        "gain" => Some(GraphStageParameter::GainLinear),
        "bias" => Some(GraphStageParameter::BiasAmount),
        "drive" => Some(GraphStageParameter::TanhDrive),
        "balance" => Some(GraphStageParameter::StereoBalance),
        "threshold" => Some(GraphStageParameter::HardClipThreshold),
        "cutoff_hz" => Some(GraphStageParameter::LowPassCutoffHz),
        "feedback" => Some(GraphStageParameter::DelayFeedback),
        _ => None,
    }
}

fn graph_stage_parameter_applies_to(
    parameter: GraphStageParameter,
    stage: &signal_graph::GraphStageSpec,
) -> bool {
    matches!(
        (parameter, stage),
        (
            GraphStageParameter::GainLinear,
            signal_graph::GraphStageSpec::Gain { .. }
        ) | (
            GraphStageParameter::BiasAmount,
            signal_graph::GraphStageSpec::Bias { .. }
        ) | (
            GraphStageParameter::TanhDrive,
            signal_graph::GraphStageSpec::TanhDrive { .. }
        ) | (
            GraphStageParameter::StereoBalance,
            signal_graph::GraphStageSpec::StereoBalance { .. }
        ) | (
            GraphStageParameter::HardClipThreshold,
            signal_graph::GraphStageSpec::HardClip { .. }
        ) | (
            GraphStageParameter::LowPassCutoffHz,
            signal_graph::GraphStageSpec::LowPass { .. }
        ) | (
            GraphStageParameter::DelayFeedback,
            signal_graph::GraphStageSpec::Delay { .. }
        )
    )
}

pub(crate) fn graph_stage_parameter_sort_key(parameter: GraphStageParameter) -> usize {
    match parameter {
        GraphStageParameter::GainLinear => 0,
        GraphStageParameter::BiasAmount => 1,
        GraphStageParameter::TanhDrive => 2,
        GraphStageParameter::StereoBalance => 3,
        GraphStageParameter::HardClipThreshold => 4,
        GraphStageParameter::LowPassCutoffHz => 5,
        GraphStageParameter::DelayFeedback => 6,
    }
}
