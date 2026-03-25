use super::*;

impl SignalRuntime {
    pub(in super::super) fn offline_render_context(
        &self,
        block_sequence: u64,
        timeline_position_samples: i64,
    ) -> GraphExecutionContext {
        let transport = self.offline_render_transport(timeline_position_samples);
        GraphExecutionContext {
            processing_epoch: 1,
            block_sequence,
            projection_epoch: self.projection_epoch,
            parameter_epoch: self.latest_parameter_epoch,
            configured_block_size: self.config.graph.block_size,
            anticipative_enabled: false,
            transport_playing: transport.playing,
            transport_tempo_bpm: transport.tempo_bpm,
            timeline_position_samples: transport.timeline_position_samples,
        }
    }

    pub(in super::super) fn offline_render_transport(
        &self,
        timeline_position_samples: i64,
    ) -> TransportProjection {
        let resolved_tempo = self.resolved_tempo_for_timeline_position(timeline_position_samples);
        TransportProjection {
            playing: true,
            timeline_position_samples,
            tempo_bpm: resolved_tempo.tempo_bpm,
            loop_state: None,
        }
    }

    pub(in super::super) fn resolved_tempo_for_timeline_position(
        &self,
        timeline_position_samples: i64,
    ) -> RuntimeResolvedTempo {
        let projected_transport = Some(TransportProjection {
            playing: true,
            timeline_position_samples,
            tempo_bpm: self
                .applied_transport
                .map(|transport| transport.tempo_bpm)
                .unwrap_or(120.0),
            loop_state: None,
        });
        self.tempo_map.resolve(
            Some(timeline_position_samples),
            projected_transport,
            self.timeline.last_transport_tempo_bpm,
        )
    }

    pub(in super::super) fn offline_render_input_block(
        &self,
        timeline_start_samples: i64,
        frame_count: usize,
        input_layout: ChannelLayout,
        resolved_tempo: &RuntimeResolvedTempo,
        decoded_media_assets: &mut BTreeMap<String, AudioBuffer>,
    ) -> Result<AudioBuffer, RuntimeError> {
        let clip_processing = self.clip_processing_pipeline.snapshot(
            &self.media_pipeline,
            &self.warp_pipeline,
            resolved_tempo,
        );
        let mut input = AudioBuffer::new(
            self.config.sample_rate,
            input_layout,
            FrameCount(frame_count),
        );
        for clip in clip_processing
            .clips
            .iter()
            .filter(|clip| clip.readiness == RuntimeClipProcessingReadiness::Ready)
        {
            let registration = self
                .clip_processing_pipeline
                .clips
                .get(&clip.clip_id)
                .ok_or_else(|| {
                    RuntimeError::new(
                        RuntimeErrorKind::InvalidState,
                        format!(
                            "offline render clip `{}` is missing registration state",
                            clip.clip_id
                        ),
                    )
                })?;
            let source = self.offline_render_clip_source_block(
                registration,
                clip,
                timeline_start_samples,
                frame_count,
                decoded_media_assets,
            )?;
            let rendered = self.render_clip_processing_buffer_with_resolved_tempo(
                RuntimeClipRenderRequest {
                    clip_id: clip.clip_id.clone(),
                    timeline_start_samples,
                    input_stage: RuntimeClipRenderInputStage::PostWarp,
                    buffer: source,
                },
                resolved_tempo,
            )?;
            let adapted = adapt_audio_buffer_layout(&rendered.output, input_layout);
            mix_audio_buffer(&mut input, &adapted);
        }
        Ok(input)
    }

    pub(in super::super) fn offline_render_clip_source_block(
        &self,
        registration: &RuntimeClipProcessingRegistration,
        clip: &RuntimeClipProcessingSnapshot,
        timeline_start_samples: i64,
        frame_count: usize,
        decoded_media_assets: &mut BTreeMap<String, AudioBuffer>,
    ) -> Result<AudioBuffer, RuntimeError> {
        let media_asset_id = registration.media_asset_id.as_deref().ok_or_else(|| {
            RuntimeError::new(
                RuntimeErrorKind::UnsupportedCapability,
                format!(
                    "offline render clip `{}` requires a runtime-owned media asset",
                    registration.clip_id
                ),
            )
        })?;
        let asset =
            decode_runtime_media_asset(&self.media_pipeline, media_asset_id, decoded_media_assets)?;
        let ratio = clip.realized_warp_ratio.unwrap_or(1.0);
        let asset_frame_ratio = asset.sample_rate().0 as f64 / self.config.sample_rate.0 as f64;
        let mut output = AudioBuffer::new(
            asset.sample_rate(),
            asset.channels(),
            FrameCount(frame_count),
        );
        let channel_count = output.channel_count().0;
        let asset_frames = asset.frames().0;
        for frame_index in 0..frame_count {
            let timeline_position_samples =
                timeline_start_samples.saturating_add(frame_index as i64);
            let clip_offset_samples =
                timeline_position_samples.saturating_sub(registration.start_samples);
            if clip_offset_samples < 0
                || clip_offset_samples >= i64::from(registration.duration_samples)
            {
                continue;
            }
            let source_frame = (clip_offset_samples as f64 * ratio * asset_frame_ratio).max(0.0);
            for channel_index in 0..channel_count {
                output.samples_mut()[frame_index * channel_count + channel_index] =
                    sample_audio_buffer_linear(&asset, source_frame, channel_index, asset_frames);
            }
        }
        Ok(output)
    }

    pub(in super::super) fn offline_render_stem_block(
        &self,
        stem: &RuntimeOfflineRenderStemPreview,
        main_output: &AudioBuffer,
        captured_buses: &[GraphCapturedBusOutput],
    ) -> Result<AudioBuffer, RuntimeError> {
        if stem.target_kind == crate::interfaces::RuntimeOfflineRenderTargetKind::MainMix {
            return Ok(main_output.clone());
        }
        if stem.resolved_output_bus_ids.is_empty() {
            return Ok(main_output.clone());
        }
        let mut mixed: Option<AudioBuffer> = None;
        for bus_id in &stem.resolved_output_bus_ids {
            let Some(captured) = captured_buses.iter().find(|bus| &bus.bus_id == bus_id) else {
                continue;
            };
            if let Some(buffer) = mixed.as_mut() {
                let adapted = adapt_audio_buffer_layout(&captured.buffer, buffer.channels());
                mix_audio_buffer(buffer, &adapted);
            } else {
                mixed = Some(captured.buffer.clone());
            }
        }
        Ok(mixed.unwrap_or_else(|| {
            AudioBuffer::new(
                self.config.sample_rate,
                main_output.channels(),
                main_output.frames(),
            )
        }))
    }
}
