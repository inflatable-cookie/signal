use std::collections::BTreeSet;

use super::super::*;

pub(super) fn build_offline_render_stem_targets(
    request: &RuntimeOfflineRenderRequest,
    topology: &RuntimeExecutionTopologySummary,
) -> Result<Vec<RuntimeOfflineRenderStemPreview>, RuntimeError> {
    let mut seen_stem_ids = BTreeSet::new();
    let mut stem_targets = Vec::with_capacity(request.stem_targets.len());
    for stem in &request.stem_targets {
        if stem.stem_id.trim().is_empty() {
            return Err(RuntimeError::new(
                RuntimeErrorKind::InvalidRequest,
                "offline render stem targets require a non-empty stem id",
            ));
        }
        if !seen_stem_ids.insert(stem.stem_id.clone()) {
            return Err(RuntimeError::new(
                RuntimeErrorKind::InvalidRequest,
                format!("offline render stem id `{}` is duplicated", stem.stem_id),
            ));
        }
        stem_targets.push(resolve_offline_render_stem_target(stem, topology)?);
    }
    Ok(stem_targets)
}

fn resolve_offline_render_stem_target(
    stem: &RuntimeOfflineRenderStemTarget,
    topology: &RuntimeExecutionTopologySummary,
) -> Result<RuntimeOfflineRenderStemPreview, RuntimeError> {
    let (target_id, resolved_node_ids, resolved_output_bus_ids) = match stem.target_kind {
        RuntimeOfflineRenderTargetKind::MainMix => (
            None,
            topology
                .nodes
                .iter()
                .map(|node| node.node_id.clone())
                .collect::<Vec<_>>(),
            topology
                .nodes
                .iter()
                .map(|node| node.output_bus_id.clone())
                .collect::<Vec<_>>(),
        ),
        RuntimeOfflineRenderTargetKind::TrackLane => {
            let target_id = stem.target_id.as_deref().ok_or_else(|| {
                RuntimeError::new(
                    RuntimeErrorKind::InvalidRequest,
                    format!(
                        "offline render stem `{}` requires a track lane id",
                        stem.stem_id
                    ),
                )
            })?;
            let summary = topology
                .track_lanes
                .iter()
                .find(|summary| summary.track_lane_id == target_id)
                .ok_or_else(|| {
                    RuntimeError::new(
                        RuntimeErrorKind::InvalidRequest,
                        format!(
                            "offline render stem `{}` references unknown track lane `{}`",
                            stem.stem_id, target_id
                        ),
                    )
                })?;
            (
                Some(target_id.to_string()),
                summary.node_ids.clone(),
                summary.output_bus_ids.clone(),
            )
        }
        RuntimeOfflineRenderTargetKind::BusGroup => {
            let target_id = stem.target_id.as_deref().ok_or_else(|| {
                RuntimeError::new(
                    RuntimeErrorKind::InvalidRequest,
                    format!(
                        "offline render stem `{}` requires a bus group id",
                        stem.stem_id
                    ),
                )
            })?;
            let summary = topology
                .bus_groups
                .iter()
                .find(|summary| summary.bus_group_id == target_id)
                .ok_or_else(|| {
                    RuntimeError::new(
                        RuntimeErrorKind::InvalidRequest,
                        format!(
                            "offline render stem `{}` references unknown bus group `{}`",
                            stem.stem_id, target_id
                        ),
                    )
                })?;
            (
                Some(target_id.to_string()),
                summary.node_ids.clone(),
                summary.output_bus_ids.clone(),
            )
        }
        RuntimeOfflineRenderTargetKind::ConsoleGroup => {
            let target_id = stem.target_id.as_deref().ok_or_else(|| {
                RuntimeError::new(
                    RuntimeErrorKind::InvalidRequest,
                    format!(
                        "offline render stem `{}` requires a console group id",
                        stem.stem_id
                    ),
                )
            })?;
            let summary = topology
                .console_groups
                .iter()
                .find(|summary| summary.console_group_id == target_id)
                .ok_or_else(|| {
                    RuntimeError::new(
                        RuntimeErrorKind::InvalidRequest,
                        format!(
                            "offline render stem `{}` references unknown console group `{}`",
                            stem.stem_id, target_id
                        ),
                    )
                })?;
            (
                Some(target_id.to_string()),
                summary.node_ids.clone(),
                summary.output_bus_ids.clone(),
            )
        }
        RuntimeOfflineRenderTargetKind::SendReturn => {
            let target_id = stem.target_id.as_deref().ok_or_else(|| {
                RuntimeError::new(
                    RuntimeErrorKind::InvalidRequest,
                    format!(
                        "offline render stem `{}` requires a send/return id",
                        stem.stem_id
                    ),
                )
            })?;
            let summary = topology
                .send_returns
                .iter()
                .find(|summary| summary.send_return_id == target_id)
                .ok_or_else(|| {
                    RuntimeError::new(
                        RuntimeErrorKind::InvalidRequest,
                        format!(
                            "offline render stem `{}` references unknown send/return `{}`",
                            stem.stem_id, target_id
                        ),
                    )
                })?;
            let mut node_ids = summary.send_node_ids.clone();
            node_ids.extend(summary.return_node_ids.clone());
            (
                Some(target_id.to_string()),
                node_ids,
                summary.output_bus_ids.clone(),
            )
        }
    };

    let resolved_node_count = resolved_node_ids.len();
    let resolved_output_bus_count = resolved_output_bus_ids.len();
    Ok(RuntimeOfflineRenderStemPreview {
        stem_id: stem.stem_id.clone(),
        target_kind: stem.target_kind,
        target_id,
        resolved_node_ids,
        resolved_output_bus_ids,
        summary: format!(
            "stem={} target={:?}/{:?} nodes={} output_buses={}",
            stem.stem_id,
            stem.target_kind,
            stem.target_id,
            resolved_node_count,
            resolved_output_bus_count,
        ),
    })
}
