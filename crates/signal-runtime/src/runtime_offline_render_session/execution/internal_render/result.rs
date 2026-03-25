use super::*;

impl SignalRuntime {
    pub(super) fn finalize_offline_render_synchronous_receipt(
        &self,
        request: RuntimeOfflineRenderRequest,
        collect_checkpoints: bool,
        mut pass: OfflineRenderSynchronousPass,
    ) -> Result<RuntimeOfflineRenderExecutionReceipt, RuntimeError> {
        let stems = pass
            .preview
            .stem_targets
            .iter()
            .map(|stem_preview| {
                let output = pass
                    .stem_outputs
                    .remove(&stem_preview.stem_id)
                    .and_then(|buffer| buffer)
                    .unwrap_or_else(|| {
                        AudioBuffer::new(
                            self.config.sample_rate,
                            ChannelLayout::Stereo,
                            FrameCount(pass.total_frames),
                        )
                    });
                RuntimeOfflineRenderStemResult {
                    stem_id: stem_preview.stem_id.clone(),
                    target_kind: stem_preview.target_kind,
                    target_id: stem_preview.target_id.clone(),
                    peak_level: peak_abs(output.samples()),
                    rms_level: rms(output.samples()),
                    summary: format!(
                        "stem={} target={:?}/{:?} frames={} peak={:.3} rms={:.3}",
                        stem_preview.stem_id,
                        stem_preview.target_kind,
                        stem_preview.target_id,
                        output.frames().0,
                        peak_abs(output.samples()),
                        rms(output.samples()),
                    ),
                    output,
                }
            })
            .collect::<Vec<_>>();

        let freeze_artifacts = pass
            .preview
            .freeze_artifacts
            .iter()
            .map(|artifact_preview| {
                let source_output = stems
                    .iter()
                    .find(|stem| stem.stem_id == artifact_preview.source_stem_id)
                    .map(|stem| stem.output.clone())
                    .or_else(|| pass.main_mix.clone())
                    .ok_or_else(|| {
                        RuntimeError::new(
                            RuntimeErrorKind::InvalidState,
                            format!(
                                "offline freeze artifact `{}` has no rendered source output",
                                artifact_preview.artifact_id
                            ),
                        )
                    })?;
                Ok(RuntimeOfflineFreezeArtifactResult {
                    artifact_id: artifact_preview.artifact_id.clone(),
                    source_stem_id: artifact_preview.source_stem_id.clone(),
                    recall_stage_count: artifact_preview.recall_stage_count,
                    recall_stage_ids: artifact_preview.recall_stage_ids.clone(),
                    recall_states: artifact_preview.recall_states.clone(),
                    peak_level: peak_abs(source_output.samples()),
                    rms_level: rms(source_output.samples()),
                    summary: format!(
                        "artifact={} source_stem={} recall_stages={} frames={} peak={:.3} rms={:.3}",
                        artifact_preview.artifact_id,
                        artifact_preview.source_stem_id,
                        artifact_preview.recall_stage_count,
                        source_output.frames().0,
                        peak_abs(source_output.samples()),
                        rms(source_output.samples()),
                    ),
                    output: source_output,
                })
            })
            .collect::<Result<Vec<_>, RuntimeError>>()?;

        let export_sample_rate = SampleRate(request.export_sample_rate_hz);
        let main_mix = pass
            .main_mix
            .map(|buffer| resample_audio_buffer_linear(&buffer, export_sample_rate));
        let stems = stems
            .into_iter()
            .map(|stem| {
                let output = resample_audio_buffer_linear(&stem.output, export_sample_rate);
                RuntimeOfflineRenderStemResult {
                    peak_level: peak_abs(output.samples()),
                    rms_level: rms(output.samples()),
                    summary: format!(
                        "stem={} target={:?}/{:?} frames={} peak={:.3} rms={:.3}",
                        stem.stem_id,
                        stem.target_kind,
                        stem.target_id,
                        output.frames().0,
                        peak_abs(output.samples()),
                        rms(output.samples()),
                    ),
                    output,
                    ..stem
                }
            })
            .collect::<Vec<_>>();
        let freeze_artifacts = freeze_artifacts
            .into_iter()
            .map(|artifact| {
                let output = resample_audio_buffer_linear(&artifact.output, export_sample_rate);
                RuntimeOfflineFreezeArtifactResult {
                    peak_level: peak_abs(output.samples()),
                    rms_level: rms(output.samples()),
                    summary: format!(
                        "artifact={} source_stem={} recall_stages={} frames={} peak={:.3} rms={:.3}",
                        artifact.artifact_id,
                        artifact.source_stem_id,
                        artifact.recall_stage_count,
                        output.frames().0,
                        peak_abs(output.samples()),
                        rms(output.samples()),
                    ),
                    output,
                    ..artifact
                }
            })
            .collect::<Vec<_>>();

        let main_mix_peak_level = main_mix.as_ref().map(|buffer| peak_abs(buffer.samples()));
        let main_mix_rms_level = main_mix.as_ref().map(|buffer| rms(buffer.samples()));
        let rendered_frame_count = main_mix
            .as_ref()
            .map(|buffer| buffer.frames().0)
            .or_else(|| stems.first().map(|stem| stem.output.frames().0))
            .or_else(|| {
                freeze_artifacts
                    .first()
                    .map(|artifact| artifact.output.frames().0)
            })
            .unwrap_or(0);
        if collect_checkpoints {
            pass.checkpoint_drafts.push(OfflineRenderCheckpointDraft {
                stage: RuntimeOfflineRenderCheckpointStage::MaterializingOutputs,
                rendered_frame_count,
                total_frame_count: pass.total_frames,
                rendered_block_count: pass.block_count,
                total_block_count: pass.total_block_count,
                progress_percent: 95,
                summary: format!(
                    "request={} stage=materializing-outputs main_mix={} stems={} freeze_artifacts={}",
                    request.request_id,
                    request.include_main_mix,
                    pass.preview.stem_count,
                    pass.preview.freeze_artifact_count,
                ),
            });
        }
        let mut result = RuntimeOfflineRenderResult {
            request_id: request.request_id.clone(),
            runtime_frame_count: pass.rendered_frames,
            rendered_frame_count,
            block_count: pass.block_count,
            export_sample_rate_hz: request.export_sample_rate_hz,
            main_mix,
            main_mix_peak_level,
            main_mix_rms_level,
            stems,
            freeze_artifacts,
            manifest: offline_render_manifest(
                &request.request_id,
                request.artifact_root_path.as_deref(),
                Vec::new(),
                None,
                pass.delegated_execution_request.clone(),
                None,
            ),
            plugin_execution_boundary: pass.plugin_execution_boundary,
            contract_preview: pass.preview.clone(),
            summary: format!(
                "request={} runtime_frames={} rendered_frames={} blocks={} main_mix={} stems={} freeze_artifacts={}",
                request.request_id,
                pass.rendered_frames,
                rendered_frame_count,
                pass.block_count,
                request.include_main_mix,
                pass.preview.stem_count,
                pass.preview.freeze_artifact_count,
            ),
        };
        result.manifest = materialize_offline_render_delivery(&result)?;
        if collect_checkpoints {
            pass.checkpoint_drafts.push(OfflineRenderCheckpointDraft {
                stage: RuntimeOfflineRenderCheckpointStage::FinalizingArtifacts,
                rendered_frame_count,
                total_frame_count: pass.total_frames,
                rendered_block_count: pass.block_count,
                total_block_count: pass.total_block_count,
                progress_percent: 99,
                summary: format!(
                    "request={} stage=finalizing-artifacts artifacts={} report={}",
                    request.request_id,
                    result.manifest.artifact_count,
                    result
                        .manifest
                        .report
                        .as_ref()
                        .map(|report| report.report_path.as_str())
                        .unwrap_or("none"),
                ),
            });
        }
        let checkpoints = if collect_checkpoints {
            Self::finalize_offline_render_checkpoints(&request.request_id, pass.checkpoint_drafts)
        } else {
            Vec::new()
        };
        let checkpoint_count = checkpoints.len();
        Ok(RuntimeOfflineRenderExecutionReceipt {
            request_id: request.request_id.clone(),
            checkpoint_count,
            checkpoints,
            result,
            summary: format!(
                "request={} checkpoints={} runtime_frames={} rendered_frames={} blocks={}",
                request.request_id,
                checkpoint_count,
                pass.rendered_frames,
                rendered_frame_count,
                pass.block_count,
            ),
        })
    }
}
