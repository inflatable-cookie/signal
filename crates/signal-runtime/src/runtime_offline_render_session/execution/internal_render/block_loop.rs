use super::*;

impl SignalRuntime {
    pub(super) fn run_offline_render_synchronous_pass(
        &self,
        request: &RuntimeOfflineRenderRequest,
        collect_checkpoints: bool,
    ) -> Result<OfflineRenderSynchronousPass, RuntimeError> {
        let preview = RuntimeOfflineRenderContractPreview::from_runtime_state(
            request,
            &self.execution_topology_summary(),
            &self.clip_processing_pipeline_snapshot(),
            &self.media_pipeline_snapshot(),
            &self.tempo_map_snapshot(),
            &self.marker_analysis_snapshot(),
            &self.plugin_recall_handoff_snapshot(),
        )?;
        let plugin_execution_boundary =
            self.offline_plugin_execution_boundary_from_preview(request, &preview);
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
        let mut decoded_media_assets = BTreeMap::new();
        let mut main_mix = None;
        let mut stem_outputs = preview
            .stem_targets
            .iter()
            .map(|stem| (stem.stem_id.clone(), None))
            .collect::<BTreeMap<String, Option<AudioBuffer>>>();
        let total_frames = request.duration_samples as usize;
        let block_size = self.config.graph.block_size.max(1);
        let total_block_count = total_frames.div_ceil(block_size);
        let mut rendered_frames = 0usize;
        let mut block_count = 0usize;
        let mut checkpoint_drafts = Vec::new();

        if collect_checkpoints {
            checkpoint_drafts.push(OfflineRenderCheckpointDraft {
                stage: RuntimeOfflineRenderCheckpointStage::PreparingInput,
                rendered_frame_count: 0,
                total_frame_count: total_frames,
                rendered_block_count: 0,
                total_block_count,
                progress_percent: 5,
                summary: format!(
                    "request={} stage=preparing-input total_frames={} blocks={} stems={} freeze_artifacts={}",
                    request.request_id, total_frames, total_block_count, preview.stem_count, preview.freeze_artifact_count
                ),
            });
        }

        while rendered_frames < total_frames {
            let block_frames = (total_frames - rendered_frames).min(block_size);
            let block_start_samples = request
                .timeline_start_samples
                .saturating_add(rendered_frames as i64);
            let resolved_tempo = self.resolved_tempo_for_timeline_position(block_start_samples);
            let context =
                self.offline_render_context((block_count + 1) as u64, block_start_samples);
            let (parameter_batch, _) = self.graph_parameter_batch_for_transport(
                &context,
                block_frames,
                Some(transport_projection_from_context(&context)),
            );
            let input = self.offline_render_input_block(
                block_start_samples,
                block_frames,
                input_layout,
                &resolved_tempo,
                &mut decoded_media_assets,
            )?;
            let (output, captured_buses) = self.engine.render_offline_block(
                context,
                input,
                parameter_batch,
                &plugin_overrides,
                &captured_bus_ids,
            )?;
            if request.include_main_mix {
                write_offline_render_block(&mut main_mix, total_frames, rendered_frames, &output);
            }

            for stem_preview in &preview.stem_targets {
                let block_output =
                    self.offline_render_stem_block(stem_preview, &output, &captured_buses)?;
                let stem_buffer = stem_outputs
                    .get_mut(&stem_preview.stem_id)
                    .expect("stem output slot should exist");
                write_offline_render_block(
                    stem_buffer,
                    total_frames,
                    rendered_frames,
                    &block_output,
                );
            }

            rendered_frames = rendered_frames.saturating_add(block_frames);
            block_count = block_count.saturating_add(1);
            if collect_checkpoints
                && Self::should_emit_offline_render_checkpoint(block_count, total_block_count)
            {
                checkpoint_drafts.push(OfflineRenderCheckpointDraft {
                    stage: RuntimeOfflineRenderCheckpointStage::RenderingGraph,
                    rendered_frame_count: rendered_frames,
                    total_frame_count: total_frames,
                    rendered_block_count: block_count,
                    total_block_count,
                    progress_percent: Self::offline_render_checkpoint_progress(
                        rendered_frames,
                        total_frames,
                    ),
                    summary: format!(
                        "request={} stage=rendering-graph blocks={}/{} frames={}/{}",
                        request.request_id,
                        block_count,
                        total_block_count,
                        rendered_frames,
                        total_frames,
                    ),
                });
            }
        }

        Ok(OfflineRenderSynchronousPass {
            preview,
            plugin_execution_boundary,
            delegated_execution_request,
            main_mix,
            stem_outputs,
            total_frames,
            total_block_count,
            rendered_frames,
            block_count,
            checkpoint_drafts,
        })
    }
}
