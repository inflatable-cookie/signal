use super::super::*;

impl RuntimeOfflineRenderContractPreview {
    pub fn from_runtime_state(
        request: &RuntimeOfflineRenderRequest,
        topology: &RuntimeExecutionTopologySummary,
        clip_processing: &RuntimeClipProcessingPipelineSnapshot,
        media_pipeline: &RuntimeMediaPipelineSnapshot,
        tempo_map: &RuntimeTempoMapSnapshot,
        marker_analysis: &RuntimeMarkerAnalysisSnapshot,
        recall_handoff: &RuntimePluginRecallHandoffSnapshot,
    ) -> Result<Self, RuntimeError> {
        if request.request_id.trim().is_empty() {
            return Err(RuntimeError::new(
                RuntimeErrorKind::InvalidRequest,
                "offline render requests require a non-empty request id",
            ));
        }
        if request.duration_samples == 0 {
            return Err(RuntimeError::new(
                RuntimeErrorKind::InvalidRequest,
                "offline render requests require a non-zero duration",
            ));
        }
        if request.export_sample_rate_hz == 0 {
            return Err(RuntimeError::new(
                RuntimeErrorKind::InvalidRequest,
                "offline render requests require a positive export sample rate",
            ));
        }

        let mut seen_stem_ids = std::collections::BTreeSet::new();
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

        let mut freeze_artifacts = Vec::with_capacity(request.freeze_artifacts.len());
        for artifact in &request.freeze_artifacts {
            if artifact.artifact_id.trim().is_empty() {
                return Err(RuntimeError::new(
                    RuntimeErrorKind::InvalidRequest,
                    "offline freeze artifacts require a non-empty artifact id",
                ));
            }
            if !request.include_main_mix
                && !stem_targets
                    .iter()
                    .any(|stem| stem.stem_id == artifact.source_stem_id)
            {
                return Err(RuntimeError::new(
                    RuntimeErrorKind::InvalidRequest,
                    format!(
                        "offline freeze artifact `{}` references unknown stem `{}`",
                        artifact.artifact_id, artifact.source_stem_id
                    ),
                ));
            }
            let resolved_selection = recall_handoff
                .resolve_selection(&artifact.recall_selection)
                .ok_or_else(|| {
                    RuntimeError::new(
                        RuntimeErrorKind::InvalidRequest,
                        format!(
                            "offline freeze artifact `{}` references an unknown recall handoff stage",
                            artifact.artifact_id
                        ),
                    )
                })?;
            freeze_artifacts.push(RuntimeOfflineFreezeArtifactPreview {
                artifact_id: artifact.artifact_id.clone(),
                source_stem_id: artifact.source_stem_id.clone(),
                recall_stage_count: resolved_selection.len(),
                recall_stage_ids: resolved_selection
                    .iter()
                    .map(|stage| stage.stage_id.clone())
                    .collect(),
                recall_states: resolved_selection
                    .iter()
                    .map(|stage| stage.recall_state)
                    .collect(),
                summary: format!(
                    "artifact={} source_stem={} recall_stages={} recall_states={:?}",
                    artifact.artifact_id,
                    artifact.source_stem_id,
                    resolved_selection.len(),
                    resolved_selection
                        .iter()
                        .map(|stage| stage.recall_state)
                        .collect::<Vec<_>>(),
                ),
            });
        }

        let timeline_end_samples = request
            .timeline_start_samples
            .saturating_add(request.duration_samples as i64);
        let chain_contract = Self::chain_contract_from_runtime_state(topology, recall_handoff)?;
        let stretch_engine_snapshot =
            RuntimeStretchEngineSnapshot::from_clip_processing_pipeline(clip_processing);
        let transform_artifact_snapshot =
            RuntimeTransformArtifactSnapshot::from_runtime_transform_state(
                clip_processing,
                &stretch_engine_snapshot,
                marker_analysis,
                media_pipeline,
            );
        let preview_transform_snapshot =
            RuntimePreviewTransformServiceSnapshot::from_runtime_preview_state(
                clip_processing,
                &RuntimeMediaServiceSnapshot {
                    indexed_asset_count: media_pipeline.asset_count,
                    analysis_ready_asset_count: 0,
                    waveform_ready_asset_count: 0,
                    waveform_pending_asset_count: 0,
                    previewable_asset_count: media_pipeline.ready_asset_count,
                    invalidated_asset_count: media_pipeline.invalid_asset_count,
                    invalidation_active: media_pipeline.invalid_asset_count > 0,
                    indexing_state: if media_pipeline.asset_count == 0 {
                        RuntimeMediaIndexingState::Empty
                    } else if media_pipeline.invalid_asset_count > 0 {
                        RuntimeMediaIndexingState::Invalidated
                    } else {
                        RuntimeMediaIndexingState::Ready
                    },
                    preview_state: if media_pipeline.ready_asset_count > 0 {
                        RuntimeMediaPreviewState::Ready
                    } else if media_pipeline.invalid_asset_count > 0 {
                        RuntimeMediaPreviewState::Invalidated
                    } else {
                        RuntimeMediaPreviewState::Unavailable
                    },
                    previewing_asset_id: None,
                    last_invalidated_asset_id: None,
                    last_invalidation_error: None,
                    last_preview_error: None,
                    summary: "offline preview derived from runtime media pipeline".into(),
                },
                &stretch_engine_snapshot,
                marker_analysis,
                &transform_artifact_snapshot,
            );
        let mut preview = Self {
            request_id: request.request_id.clone(),
            timeline_start_samples: request.timeline_start_samples,
            timeline_end_samples,
            duration_samples: request.duration_samples,
            export_sample_rate_hz: request.export_sample_rate_hz,
            include_main_mix: request.include_main_mix,
            clip_count: clip_processing.clip_count,
            ready_clip_count: clip_processing.ready_clip_count,
            stretch_engine_snapshot,
            preview_transform_snapshot,
            transform_artifact_snapshot,
            stem_count: stem_targets.len(),
            freeze_artifact_count: freeze_artifacts.len(),
            resolved_tempo_bpm: tempo_map.resolved_tempo_bpm,
            resolved_tempo_source: tempo_map.tempo_source,
            chain_contract,
            stem_targets,
            freeze_artifacts,
            summary: String::new(),
        };
        preview.summary = format!(
            "request={} timeline={}..{} duration={} export_sample_rate={} clips={}/{} stretch={}/fallback={} preview_transform={}/artifact_backed={}/fallback={} transform_artifacts={}/reusable={} stems={} freeze_artifacts={} tempo={:.3}/{:?} chain_contract={}",
            preview.request_id,
            preview.timeline_start_samples,
            preview.timeline_end_samples,
            preview.duration_samples,
            preview.export_sample_rate_hz,
            preview.ready_clip_count,
            preview.clip_count,
            preview.stretch_engine_snapshot.ready_clip_count,
            preview.stretch_engine_snapshot.fallback_clip_count,
            preview.preview_transform_snapshot.ready_clip_count,
            preview.preview_transform_snapshot.artifact_backed_clip_count,
            preview.preview_transform_snapshot.fallback_clip_count,
            preview.transform_artifact_snapshot.ready_clip_count,
            preview.transform_artifact_snapshot.reusable_clip_count,
            preview.stem_count,
            preview.freeze_artifact_count,
            preview.resolved_tempo_bpm,
            preview.resolved_tempo_source,
            preview.chain_contract.summary,
        );
        Ok(preview)
    }
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
