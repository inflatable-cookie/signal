use super::*;

impl RuntimeClipProcessingPipelineStateModel {
    pub(crate) fn render_clip(
        &self,
        request: RuntimeClipRenderRequest,
        media_pipeline: &RuntimeMediaPipelineStateModel,
        warp_pipeline: &RuntimeWarpPipelineStateModel,
        resolved_tempo: &RuntimeResolvedTempo,
        transform_artifact_snapshot: RuntimeTransformArtifactClipSnapshot,
        preview_transform_snapshot: RuntimePreviewTransformClipSnapshot,
    ) -> Result<RuntimeClipRenderResult, RuntimeError> {
        let registration = self.clips.get(&request.clip_id).ok_or_else(|| {
            RuntimeError::new(
                RuntimeErrorKind::InvalidRequest,
                format!(
                    "clip processing render request references unknown clip `{}`",
                    request.clip_id
                ),
            )
        })?;
        let warp_snapshot = warp_pipeline.snapshot(resolved_tempo, media_pipeline);
        let clip_processing_snapshot =
            self.snapshot_clip(registration, media_pipeline, &warp_snapshot.clips);
        let stretch_engine_snapshot =
            RuntimeStretchClipSnapshot::from_clip_processing_snapshot(&clip_processing_snapshot);
        if clip_processing_snapshot.readiness != RuntimeClipProcessingReadiness::Ready {
            return Err(RuntimeError::new(
                RuntimeErrorKind::InvalidState,
                clip_processing_snapshot
                    .last_error
                    .clone()
                    .unwrap_or_else(|| "clip processing render path is not ready".to_string()),
            ));
        }
        if registration.warp_mode != RuntimeWarpMode::Off
            && request.input_stage != RuntimeClipRenderInputStage::PostWarp
        {
            return Err(RuntimeError::new(
                RuntimeErrorKind::UnsupportedCapability,
                "warp-enabled clip render requests currently require post-warp input",
            ));
        }

        let mut output = request.buffer;
        let mut first_frame_gain = None;
        let mut last_frame_gain = None;
        let mut peak_applied_gain: Option<f32> = None;
        let channels = output.channel_count().0;
        for frame_index in 0..output.frames().0 {
            let timeline_position_samples = request
                .timeline_start_samples
                .saturating_add(frame_index as i64);
            let gain = Self::clip_frame_gain(registration, timeline_position_samples);
            if frame_index == 0 {
                first_frame_gain = Some(gain);
            }
            last_frame_gain = Some(gain);
            peak_applied_gain = Some(
                peak_applied_gain
                    .map(|current| current.max(gain))
                    .unwrap_or(gain),
            );
            let frame_start = frame_index.saturating_mul(channels);
            let frame_end = frame_start.saturating_add(channels);
            for sample in &mut output.samples_mut()[frame_start..frame_end] {
                *sample *= gain;
            }
        }

        let timeline_end_samples = request
            .timeline_start_samples
            .saturating_add(output.frames().0 as i64);
        Ok(RuntimeClipRenderResult {
            clip_id: request.clip_id,
            timeline_start_samples: request.timeline_start_samples,
            timeline_end_samples,
            input_stage: request.input_stage,
            clip_processing_snapshot: clip_processing_snapshot.clone(),
            stretch_engine_snapshot: stretch_engine_snapshot.clone(),
            transform_artifact_snapshot: transform_artifact_snapshot.clone(),
            preview_transform_snapshot: preview_transform_snapshot.clone(),
            first_frame_gain,
            last_frame_gain,
            peak_applied_gain,
            output,
        })
    }

    fn clip_frame_gain(
        registration: &RuntimeClipProcessingRegistration,
        timeline_position_samples: i64,
    ) -> f32 {
        let clip_offset_samples =
            timeline_position_samples.saturating_sub(registration.start_samples);
        if clip_offset_samples < 0
            || clip_offset_samples >= i64::from(registration.duration_samples)
        {
            return 0.0;
        }
        let clip_offset_samples = clip_offset_samples as u32;
        let fade_in_gain = Self::fade_in_gain(registration, clip_offset_samples);
        let fade_out_gain = Self::fade_out_gain(registration, clip_offset_samples);
        let gain_shape = Self::gain_shape_gain(registration, clip_offset_samples);
        fade_in_gain * fade_out_gain * gain_shape
    }

    fn fade_in_gain(
        registration: &RuntimeClipProcessingRegistration,
        clip_offset_samples: u32,
    ) -> f32 {
        let fade_in = registration.fade_in.duration_samples;
        if fade_in == 0 {
            return 1.0;
        }
        if clip_offset_samples >= fade_in {
            return 1.0;
        }
        if fade_in == 1 {
            return 1.0;
        }
        let progress = clip_offset_samples as f32 / (fade_in - 1) as f32;
        Self::fade_shape_gain(registration.fade_in.shape, progress)
    }

    fn fade_out_gain(
        registration: &RuntimeClipProcessingRegistration,
        clip_offset_samples: u32,
    ) -> f32 {
        let fade_out = registration.fade_out.duration_samples;
        if fade_out == 0 {
            return 1.0;
        }
        let fade_out_start = registration.duration_samples.saturating_sub(fade_out);
        if clip_offset_samples < fade_out_start {
            return 1.0;
        }
        if fade_out == 1 {
            return 0.0;
        }
        let fade_offset = clip_offset_samples.saturating_sub(fade_out_start) as f32;
        let progress = 1.0 - (fade_offset / (fade_out - 1) as f32);
        Self::fade_shape_gain(registration.fade_out.shape, progress)
    }

    fn fade_shape_gain(shape: RuntimeClipFadeShape, progress: f32) -> f32 {
        let progress = progress.clamp(0.0, 1.0);
        match shape {
            RuntimeClipFadeShape::Linear => progress,
            RuntimeClipFadeShape::EqualPower => (progress * std::f32::consts::FRAC_PI_2).sin(),
            RuntimeClipFadeShape::SmoothStep => progress * progress * (3.0 - 2.0 * progress),
        }
    }

    fn gain_shape_gain(
        registration: &RuntimeClipProcessingRegistration,
        clip_offset_samples: u32,
    ) -> f32 {
        match registration.clip_gain.shape {
            RuntimeClipGainShape::Hold => registration.clip_gain.start_linear,
            RuntimeClipGainShape::Linear => {
                if registration.duration_samples <= 1 {
                    return registration.clip_gain.end_linear;
                }
                let progress =
                    clip_offset_samples as f32 / (registration.duration_samples - 1) as f32;
                registration.clip_gain.start_linear
                    + (registration.clip_gain.end_linear - registration.clip_gain.start_linear)
                        * progress.clamp(0.0, 1.0)
            }
        }
    }
}
