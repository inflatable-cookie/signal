use super::*;

impl SignalRuntime {
    pub(super) fn advance_offline_render_execution_render_progress(
        &mut self,
        request_id: &str,
        session: &mut RuntimeOfflineRenderExecutionSession,
    ) -> Result<Option<RuntimeOfflineRenderExecutionProgressReceipt>, RuntimeError> {
        if session.rendered_frames < session.total_frames {
            while session.rendered_frames < session.total_frames {
                let block_frames = (session.total_frames - session.rendered_frames)
                    .min(self.config.graph.block_size.max(1));
                let block_start_samples = session
                    .request
                    .timeline_start_samples
                    .saturating_add(session.rendered_frames as i64);
                let resolved_tempo = self.resolved_tempo_for_timeline_position(block_start_samples);
                let context = self
                    .offline_render_context((session.block_count + 1) as u64, block_start_samples);
                let (parameter_batch, _) = self.graph_parameter_batch_for_transport(
                    &context,
                    block_frames,
                    Some(transport_projection_from_context(&context)),
                );
                let input = self.offline_render_input_block(
                    block_start_samples,
                    block_frames,
                    session.input_layout,
                    &resolved_tempo,
                    &mut session.decoded_media_assets,
                )?;
                let (output, captured_buses) = self.engine.render_offline_block(
                    context,
                    input,
                    parameter_batch,
                    &session.plugin_overrides,
                    &session.captured_bus_ids,
                )?;
                if session.request.include_main_mix {
                    write_offline_render_block(
                        &mut session.main_mix,
                        session.total_frames,
                        session.rendered_frames,
                        &output,
                    );
                }

                for stem_preview in &session.preview.stem_targets {
                    let block_output =
                        self.offline_render_stem_block(stem_preview, &output, &captured_buses)?;
                    let stem_buffer = session
                        .stem_outputs
                        .get_mut(&stem_preview.stem_id)
                        .expect("stem output slot should exist");
                    write_offline_render_block(
                        stem_buffer,
                        session.total_frames,
                        session.rendered_frames,
                        &block_output,
                    );
                }

                session.rendered_frames = session.rendered_frames.saturating_add(block_frames);
                session.block_count = session.block_count.saturating_add(1);
                if Self::should_emit_offline_render_checkpoint(
                    session.block_count,
                    session.total_block_count,
                ) {
                    let progress_percent = Self::offline_render_checkpoint_progress(
                        session.rendered_frames,
                        session.total_frames,
                    );
                    let rendered_frames = session.rendered_frames;
                    let block_count = session.block_count;
                    let checkpoint = Self::emit_offline_render_session_checkpoint(
                        session,
                        RuntimeOfflineRenderCheckpointStage::RenderingGraph,
                        rendered_frames,
                        block_count,
                        progress_percent,
                    );
                    let emitted_checkpoint_count = session.emitted_checkpoint_count;
                    let checkpoint_count = session.checkpoint_count;
                    self.record_last_offline_render_session_snapshot(
                        Self::offline_render_session_state_snapshot(session),
                    );
                    let parked_session = session.clone();
                    self.offline_render_executions
                        .insert(request_id.to_string(), parked_session);
                    return Ok(Some(RuntimeOfflineRenderExecutionProgressReceipt {
                        request_id: request_id.to_string(),
                        state: RuntimeOfflineRenderExecutionState::Running,
                        interruption_class: Self::offline_render_execution_interruption_class(
                            RuntimeOfflineRenderExecutionState::Running,
                        ),
                        interruption_rebindable: false,
                        emitted_checkpoint_count,
                        checkpoint_count,
                        checkpoint: Some(checkpoint),
                        result: None,
                    }));
                }
            }
        }

        Ok(None)
    }
}
