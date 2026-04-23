use super::super::*;
use super::freeze_artifacts::build_offline_freeze_artifact_previews;
use super::stem_targets::build_offline_render_stem_targets;
use super::validation::validate_offline_preview_request;

impl RuntimeOfflineRenderContractPreview {
    /// Assembles a full offline render contract preview from the current runtime state.
    pub fn from_runtime_state(
        request: &RuntimeOfflineRenderRequest,
        topology: &RuntimeExecutionTopologySummary,
        clip_processing: &RuntimeClipProcessingPipelineSnapshot,
        media_pipeline: &RuntimeMediaPipelineSnapshot,
        tempo_map: &RuntimeTempoMapSnapshot,
        marker_analysis: &RuntimeMarkerAnalysisSnapshot,
        recall_handoff: &RuntimePluginRecallHandoffSnapshot,
    ) -> Result<Self, RuntimeError> {
        validate_offline_preview_request(request)?;
        let stem_targets = build_offline_render_stem_targets(request, topology)?;
        let freeze_artifacts =
            build_offline_freeze_artifact_previews(request, &stem_targets, recall_handoff)?;

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
