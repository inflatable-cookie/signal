use super::*;
#[path = "runtime_offline_render_session/control.rs"]
mod control;
#[path = "runtime_offline_render_session/delegated_execution.rs"]
mod delegated_execution;
#[path = "runtime_offline_render_session/execution.rs"]
mod execution;
#[path = "runtime_offline_render_session/materialization.rs"]
mod materialization;
#[path = "runtime_offline_render_session/preparation.rs"]
mod preparation;
#[path = "runtime_offline_render_session/snapshot.rs"]
mod snapshot;

#[derive(Clone, Debug)]
pub(super) struct OfflineRenderCheckpointDraft {
    pub(super) stage: RuntimeOfflineRenderCheckpointStage,
    pub(super) rendered_frame_count: usize,
    pub(super) total_frame_count: usize,
    pub(super) rendered_block_count: usize,
    pub(super) total_block_count: usize,
    pub(super) progress_percent: u8,
    pub(super) summary: String,
}

#[derive(Clone, Debug)]
pub(super) struct RuntimeOfflineRenderExecutionSession {
    pub(super) request: RuntimeOfflineRenderRequest,
    pub(super) state: RuntimeOfflineRenderExecutionState,
    pub(super) preview: RuntimeOfflineRenderContractPreview,
    pub(super) plugin_execution_boundary: RuntimeOfflinePluginExecutionBoundary,
    pub(super) delegated_execution_request: RuntimeOfflinePluginDelegatedExecutionRequest,
    pub(super) input_layout: ChannelLayout,
    pub(super) plugin_overrides: Vec<GraphNodeRenderOverride>,
    pub(super) captured_bus_ids: Vec<String>,
    pub(super) decoded_media_assets: BTreeMap<String, AudioBuffer>,
    pub(super) main_mix: Option<AudioBuffer>,
    pub(super) stem_outputs: BTreeMap<String, Option<AudioBuffer>>,
    pub(super) total_frames: usize,
    pub(super) total_block_count: usize,
    pub(super) rendered_frames: usize,
    pub(super) block_count: usize,
    pub(super) checkpoint_count: usize,
    pub(super) emitted_checkpoint_count: usize,
    pub(super) interruption_count: usize,
    pub(super) interruption_class_override: Option<RuntimeInterruptionClass>,
    pub(super) last_checkpoint: Option<RuntimeOfflineRenderCheckpointReceipt>,
    pub(super) last_state_summary: String,
    pub(super) materialized_result: Option<RuntimeOfflineRenderResult>,
    pub(super) finalizing_checkpoint_emitted: bool,
}

impl SignalRuntime {
    pub(super) fn should_emit_offline_render_checkpoint(
        rendered_block_count: usize,
        total_block_count: usize,
    ) -> bool {
        if total_block_count == 0 || rendered_block_count == 0 {
            return false;
        }
        if rendered_block_count >= total_block_count {
            return true;
        }
        let stride = total_block_count
            .div_ceil(OFFLINE_RENDER_PROGRESS_CHECKPOINT_TARGET_COUNT)
            .max(1);
        rendered_block_count % stride == 0
    }

    pub(super) fn offline_render_checkpoint_count(total_block_count: usize) -> usize {
        let rendering_checkpoint_count = (1..=total_block_count)
            .filter(|rendered_block_count| {
                Self::should_emit_offline_render_checkpoint(
                    *rendered_block_count,
                    total_block_count,
                )
            })
            .count();
        1usize
            .saturating_add(rendering_checkpoint_count)
            .saturating_add(2)
    }

    pub(super) fn offline_render_checkpoint_progress(
        rendered_frame_count: usize,
        total_frame_count: usize,
    ) -> u8 {
        if total_frame_count == 0 {
            return 90;
        }
        let scaled = 10usize
            .saturating_add((rendered_frame_count.saturating_mul(80)) / total_frame_count.max(1));
        scaled.clamp(10, 90) as u8
    }

    pub(super) fn finalize_offline_render_checkpoints(
        request_id: &str,
        drafts: Vec<OfflineRenderCheckpointDraft>,
    ) -> Vec<RuntimeOfflineRenderCheckpointReceipt> {
        let checkpoint_count = drafts.len();
        drafts
            .into_iter()
            .enumerate()
            .map(
                |(checkpoint_index, draft)| RuntimeOfflineRenderCheckpointReceipt {
                    request_id: request_id.to_string(),
                    stage: draft.stage,
                    checkpoint_index,
                    checkpoint_count,
                    rendered_frame_count: draft.rendered_frame_count,
                    total_frame_count: draft.total_frame_count,
                    rendered_block_count: draft.rendered_block_count,
                    total_block_count: draft.total_block_count,
                    progress_percent: draft.progress_percent,
                    summary: draft.summary,
                },
            )
            .collect()
    }

    pub(super) fn build_offline_render_execution_session(
        &self,
        request: RuntimeOfflineRenderRequest,
    ) -> Result<RuntimeOfflineRenderExecutionSession, RuntimeError> {
        let preview = RuntimeOfflineRenderContractPreview::from_runtime_state(
            &request,
            &self.execution_topology_summary(),
            &self.clip_processing_pipeline_snapshot(),
            &self.media_pipeline_snapshot(),
            &self.tempo_map_snapshot(),
            &self.marker_analysis_snapshot(),
            &self.plugin_recall_handoff_snapshot(),
        )?;
        let plugin_execution_boundary =
            self.offline_plugin_execution_boundary_from_preview(&request, &preview);
        let delegated_execution_request =
            Self::offline_plugin_delegated_execution_request(&plugin_execution_boundary);
        let input_layout = self.offline_render_input_layout()?;
        let plugin_overrides = self.offline_render_plugin_node_overrides()?;
        let captured_bus_ids = preview
            .stem_targets
            .iter()
            .flat_map(|stem| stem.resolved_output_bus_ids.iter().cloned())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
        let total_frames = request.duration_samples as usize;
        let total_block_count = total_frames.div_ceil(self.config.graph.block_size.max(1));
        let stem_outputs = preview
            .stem_targets
            .iter()
            .map(|stem| (stem.stem_id.clone(), None))
            .collect::<BTreeMap<String, Option<AudioBuffer>>>();

        Ok(RuntimeOfflineRenderExecutionSession {
            request,
            state: RuntimeOfflineRenderExecutionState::Running,
            preview,
            plugin_execution_boundary,
            delegated_execution_request,
            input_layout,
            plugin_overrides,
            captured_bus_ids,
            decoded_media_assets: BTreeMap::new(),
            main_mix: None,
            stem_outputs,
            total_frames,
            total_block_count,
            rendered_frames: 0,
            block_count: 0,
            checkpoint_count: Self::offline_render_checkpoint_count(total_block_count),
            emitted_checkpoint_count: 0,
            interruption_count: 0,
            interruption_class_override: None,
            last_checkpoint: None,
            last_state_summary: "state=running checkpoints=0".to_string(),
            materialized_result: None,
            finalizing_checkpoint_emitted: false,
        })
    }

    pub(super) fn emit_offline_render_session_checkpoint(
        session: &mut RuntimeOfflineRenderExecutionSession,
        stage: RuntimeOfflineRenderCheckpointStage,
        rendered_frame_count: usize,
        rendered_block_count: usize,
        progress_percent: u8,
        summary: String,
    ) -> RuntimeOfflineRenderCheckpointReceipt {
        let checkpoint = RuntimeOfflineRenderCheckpointReceipt {
            request_id: session.request.request_id.clone(),
            stage,
            checkpoint_index: session.emitted_checkpoint_count,
            checkpoint_count: session.checkpoint_count,
            rendered_frame_count,
            total_frame_count: session.total_frames,
            rendered_block_count,
            total_block_count: session.total_block_count,
            progress_percent,
            summary,
        };
        session.emitted_checkpoint_count = session.emitted_checkpoint_count.saturating_add(1);
        session.last_checkpoint = Some(checkpoint.clone());
        checkpoint
    }
}
