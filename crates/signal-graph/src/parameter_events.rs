//! Parameter event processing for graph execution.
//!
//! This module provides utilities for processing parameter events, including
//! filtering events for specific nodes/stages and computing parameter
//! application reports.

use crate::{
    GraphExecutionPlan, GraphNodeSpec, GraphParameterBatch, GraphStageParameter, GraphStageSpec,
    StageParameterEvent,
};
use std::collections::BTreeSet;

/// Collect parameter events for a specific node stage.
pub fn stage_parameter_events_for_node(
    parameter_batch: Option<&GraphParameterBatch>,
    node: &GraphNodeSpec,
    stage_index: usize,
    stage: &GraphStageSpec,
    frame_count: usize,
) -> Vec<StageParameterEvent> {
    let Some(parameter_batch) = parameter_batch else {
        return Vec::new();
    };

    let mut events = parameter_batch
        .events
        .iter()
        .filter(|event| {
            event.target.node_id == node.node_id
                && event.target.stage_index == stage_index
                && event.target.parameter.applies_to(stage)
                && event.sample_offset < frame_count
        })
        .map(|event| StageParameterEvent {
            sample_offset: event.sample_offset,
            value: event.value,
        })
        .collect::<Vec<_>>();
    events.sort_by_key(|event| event.sample_offset);
    events
}

/// Report structure for parameter application tracking.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ParameterApplicationReport {
    pub event_count: usize,
    pub targeted_node_count: usize,
    pub ignored_event_count: usize,
    pub sub_block_count: usize,
    pub coalesced_event_count: usize,
}

/// Compute a parameter application report for a given batch.
pub fn parameter_application_report(
    plan: &GraphExecutionPlan,
    frame_count: usize,
    parameter_batch: Option<&GraphParameterBatch>,
) -> ParameterApplicationReport {
    let Some(parameter_batch) = parameter_batch else {
        return ParameterApplicationReport::default();
    };

    let mut report = ParameterApplicationReport {
        event_count: parameter_batch.events.len(),
        ..ParameterApplicationReport::default()
    };
    let mut targeted_nodes = BTreeSet::new();

    for event in &parameter_batch.events {
        let Some(node) = plan
            .nodes
            .iter()
            .find(|node| node.node_id == event.target.node_id)
        else {
            report.ignored_event_count += 1;
            continue;
        };
        let Some(stage) = node.stages.get(event.target.stage_index) else {
            report.ignored_event_count += 1;
            continue;
        };
        if !event.target.parameter.applies_to(stage) || event.sample_offset >= frame_count {
            report.ignored_event_count += 1;
            continue;
        }
        targeted_nodes.insert(node.node_id.clone());
    }

    for node in &plan.nodes {
        for (stage_index, stage) in node.stages.iter().enumerate() {
            let stage_events = stage_parameter_events_for_node(
                Some(parameter_batch),
                node,
                stage_index,
                stage,
                frame_count,
            );
            if stage_events.is_empty() {
                continue;
            }
            let (bounded, coalesced) = crate::stage_processor::bounded_stage_events(
                &stage_events,
                parameter_batch.strategy,
            );
            let boundary_count = bounded
                .iter()
                .filter(|event| event.sample_offset > 0 && event.sample_offset < frame_count)
                .map(|event| event.sample_offset)
                .collect::<BTreeSet<_>>()
                .len();
            report.sub_block_count += boundary_count.saturating_add(1);
            report.coalesced_event_count += coalesced;
        }
    }

    report.targeted_node_count = targeted_nodes.len();
    report
}

/// Extension methods on [`GraphStageParameter`].
///
/// Implemented for [`GraphStageParameter`] to add compatibility queries that
/// are not part of the core enum definition.
pub trait GraphStageParameterExt {
    /// Returns `true` if this parameter variant is valid for the given stage.
    ///
    /// Use this before dispatching a [`GraphParameterEvent`] to verify that the
    /// parameter kind matches the stage kind — e.g. `GainLinear` only applies
    /// to a `Gain` stage. Mismatched events are silently ignored during
    /// execution but flagged in [`GraphBlockReport::parameter_ignored_event_count`].
    fn applies_to(self, stage: &GraphStageSpec) -> bool;
}

impl GraphStageParameterExt for GraphStageParameter {
    fn applies_to(self, stage: &GraphStageSpec) -> bool {
        matches!(
            (self, stage),
            (GraphStageParameter::GainLinear, GraphStageSpec::Gain { .. })
                | (GraphStageParameter::BiasAmount, GraphStageSpec::Bias { .. })
                | (
                    GraphStageParameter::TanhDrive,
                    GraphStageSpec::TanhDrive { .. }
                )
                | (
                    GraphStageParameter::StereoBalance,
                    GraphStageSpec::StereoBalance { .. }
                )
                | (
                    GraphStageParameter::HardClipThreshold,
                    GraphStageSpec::HardClip { .. }
                )
        )
    }
}
