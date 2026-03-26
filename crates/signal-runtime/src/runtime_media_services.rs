use super::*;

impl SignalRuntime {
    pub fn render_clip_processing_buffer(
        &self,
        request: RuntimeClipRenderRequest,
    ) -> Result<RuntimeClipRenderResult, RuntimeError> {
        if request.clip_id.is_empty() {
            return Err(RuntimeError::new(
                RuntimeErrorKind::InvalidRequest,
                "clip processing render requests require a non-empty clip id",
            ));
        }
        if request.buffer.sample_rate() != self.config.sample_rate {
            return Err(RuntimeError::new(
                RuntimeErrorKind::InvalidRequest,
                format!(
                    "clip processing render buffer sample rate {} does not match runtime sample rate {}",
                    request.buffer.sample_rate().0,
                    self.config.sample_rate.0,
                ),
            ));
        }
        self.render_clip_processing_buffer_with_resolved_tempo(
            request,
            &self.current_resolved_tempo(),
        )
    }

    pub fn prepare_offline_plugin_execution_boundary(
        &self,
        request: &RuntimeOfflineRenderRequest,
    ) -> Result<RuntimeOfflinePluginExecutionBoundary, RuntimeError> {
        let preview = RuntimeOfflineRenderContractPreview::from_runtime_state(
            request,
            &self.execution_topology_summary(),
            &self.clip_processing_pipeline_snapshot(),
            &self.media_pipeline_snapshot(),
            &self.tempo_map_snapshot(),
            &self.marker_analysis_snapshot(),
            &self.plugin_recall_handoff_snapshot(),
        )?;
        Ok(self.offline_plugin_execution_boundary_from_preview(request, &preview))
    }

    pub(crate) fn offline_render_input_layout(&self) -> Result<ChannelLayout, RuntimeError> {
        let graph = self.engine.graph.as_ref().ok_or_else(|| {
            RuntimeError::new(
                RuntimeErrorKind::InvalidState,
                "offline render requires an applied executable graph",
            )
        })?;
        let contract = graph.contract_summary();
        let mut layout = None;
        for node in contract
            .node_contracts
            .iter()
            .filter(|node| node.input_bus_id == "main:in")
        {
            match layout {
                Some(existing) if existing != node.input_channels => {
                    return Err(RuntimeError::new(
                        RuntimeErrorKind::UnsupportedCapability,
                        "offline render requires a consistent main:in channel layout",
                    ));
                }
                None => layout = Some(node.input_channels),
                _ => {}
            }
        }
        Ok(layout.unwrap_or(ChannelLayout::Stereo))
    }

    pub fn start_recording_capture(
        &mut self,
        request: RuntimeRecordingCaptureStartRequest,
    ) -> Result<(), RuntimeError> {
        self.recording_capture.start_capture(
            request,
            self.config.sample_rate.0,
            self.control.configured,
            &self.readiness,
        )
    }

    pub fn finish_recording_capture(
        &mut self,
    ) -> Result<RuntimeRecordingCaptureCommitReceipt, RuntimeError> {
        self.recording_capture.finish_capture()
    }

    pub fn cancel_recording_capture(&mut self) -> Result<(), RuntimeError> {
        self.recording_capture.cancel_capture()
    }

    pub fn reconcile_media_assets(
        &mut self,
        assets: Vec<RuntimeMediaAssetRegistration>,
    ) -> Result<(), RuntimeError> {
        self.media_pipeline.reconcile_assets(assets)
    }

    pub fn start_media_preview(&mut self, asset_id: &str) -> Result<(), RuntimeError> {
        self.media_pipeline.start_preview(asset_id)
    }

    pub fn stop_media_preview(&mut self) -> Result<(), RuntimeError> {
        self.media_pipeline.stop_preview();
        Ok(())
    }

    pub fn reconcile_warp_clips(
        &mut self,
        clips: Vec<RuntimeWarpClipRegistration>,
    ) -> Result<(), RuntimeError> {
        self.warp_pipeline.reconcile_clips(clips);
        Ok(())
    }

    pub fn reconcile_clip_processing_clips(
        &mut self,
        clips: Vec<RuntimeClipProcessingRegistration>,
    ) -> Result<(), RuntimeError> {
        self.clip_processing_pipeline.reconcile_clips(clips);
        Ok(())
    }
}
