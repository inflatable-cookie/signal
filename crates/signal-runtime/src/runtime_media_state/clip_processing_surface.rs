use super::*;

impl RuntimeClipProcessingPipelineStateModel {
    pub(crate) fn treatment_stages(
        &self,
        registration: &RuntimeClipProcessingRegistration,
    ) -> Vec<RuntimeClipProcessingStage> {
        let mut stages = Vec::new();
        if registration.warp_mode != RuntimeWarpMode::Off {
            stages.push(RuntimeClipProcessingStage::Warp);
        }
        if registration.fade_in.duration_samples > 0 {
            stages.push(RuntimeClipProcessingStage::FadeIn);
        }
        if registration.clip_gain.shape != RuntimeClipGainShape::Hold
            || (registration.clip_gain.start_linear - 1.0).abs() > f32::EPSILON
            || (registration.clip_gain.end_linear - 1.0).abs() > f32::EPSILON
        {
            stages.push(RuntimeClipProcessingStage::GainShape);
        }
        if registration.fade_out.duration_samples > 0 {
            stages.push(RuntimeClipProcessingStage::FadeOut);
        }
        stages
    }

    pub(crate) fn warp_context(
        &self,
        registration: &RuntimeClipProcessingRegistration,
        warp_clips: &[RuntimeWarpClipSnapshot],
    ) -> (Option<f64>, Option<RuntimeTempoSource>, Option<String>) {
        warp_clips
            .iter()
            .find(|clip| clip.clip_id == registration.clip_id)
            .map(|clip| {
                (
                    Some(clip.realized_ratio),
                    Some(clip.project_tempo_source),
                    clip.project_tempo_segment_id.clone(),
                )
            })
            .unwrap_or((None, None, None))
    }

    pub(crate) fn validate_registration(
        &self,
        registration: &RuntimeClipProcessingRegistration,
    ) -> Option<String> {
        if !registration.clip_gain.start_linear.is_finite()
            || !registration.clip_gain.end_linear.is_finite()
            || registration.clip_gain.start_linear < 0.0
            || registration.clip_gain.end_linear < 0.0
        {
            return Some("clip gain envelope must be finite and non-negative".to_string());
        }
        if registration.clip_gain.shape == RuntimeClipGainShape::Hold
            && (registration.clip_gain.start_linear - registration.clip_gain.end_linear).abs()
                > f32::EPSILON
        {
            return Some("hold clip gain shape requires identical start and end gain".to_string());
        }
        if u64::from(registration.fade_in.duration_samples)
            + u64::from(registration.fade_out.duration_samples)
            > u64::from(registration.duration_samples)
        {
            return Some("clip fades exceed clip duration".to_string());
        }
        None
    }

    pub(crate) fn snapshot(
        &self,
        media_pipeline: &RuntimeMediaPipelineStateModel,
        warp_pipeline: &RuntimeWarpPipelineStateModel,
        resolved_tempo: &RuntimeResolvedTempo,
    ) -> RuntimeClipProcessingPipelineSnapshot {
        let warp_snapshot = warp_pipeline.snapshot(resolved_tempo, media_pipeline);
        let clips = self
            .clips
            .values()
            .map(|registration| {
                self.snapshot_clip(registration, media_pipeline, &warp_snapshot.clips)
            })
            .collect::<Vec<_>>();
        let ready_clip_count = clips
            .iter()
            .filter(|clip| clip.readiness == RuntimeClipProcessingReadiness::Ready)
            .count();
        let pending_media_clip_count = clips
            .iter()
            .filter(|clip| clip.readiness == RuntimeClipProcessingReadiness::PendingMedia)
            .count();
        let pending_warp_clip_count = clips
            .iter()
            .filter(|clip| clip.readiness == RuntimeClipProcessingReadiness::PendingWarp)
            .count();
        let invalid_clip_count = clips
            .iter()
            .filter(|clip| clip.readiness == RuntimeClipProcessingReadiness::Invalid)
            .count();
        let faded_clip_count = clips
            .iter()
            .filter(|clip| clip.fade_in.duration_samples > 0 || clip.fade_out.duration_samples > 0)
            .count();
        let gain_shaped_clip_count = clips
            .iter()
            .filter(|clip| {
                clip.clip_gain.shape != RuntimeClipGainShape::Hold
                    || (clip.clip_gain.start_linear - 1.0).abs() > f32::EPSILON
                    || (clip.clip_gain.end_linear - 1.0).abs() > f32::EPSILON
            })
            .count();
        let warped_clip_count = clips
            .iter()
            .filter(|clip| clip.realized_warp_ratio.is_some())
            .count();
        let treatment_stage_count = clips.iter().map(|clip| clip.treatment_stages.len()).sum();

        RuntimeClipProcessingPipelineSnapshot {
            clip_count: clips.len(),
            ready_clip_count,
            pending_media_clip_count,
            pending_warp_clip_count,
            invalid_clip_count,
            faded_clip_count,
            gain_shaped_clip_count,
            warped_clip_count,
            treatment_stage_count,
            clips,
            summary: format!(
                "clip_processing clips={} ready={} pending_media={} pending_warp={} invalid={} faded={} gain_shaped={} warped={} treatment_stages={}",
                self.clips.len(),
                ready_clip_count,
                pending_media_clip_count,
                pending_warp_clip_count,
                invalid_clip_count,
                faded_clip_count,
                gain_shaped_clip_count,
                warped_clip_count,
                treatment_stage_count,
            ),
        }
    }
    pub(crate) fn reconcile_clips(&mut self, clips: Vec<RuntimeClipProcessingRegistration>) {
        let retained_ids = clips
            .iter()
            .map(|clip| clip.clip_id.clone())
            .collect::<BTreeSet<_>>();
        self.clips
            .retain(|clip_id, _| retained_ids.contains(clip_id));
        for clip in clips {
            self.clips.insert(clip.clip_id.clone(), clip);
        }
    }
}
