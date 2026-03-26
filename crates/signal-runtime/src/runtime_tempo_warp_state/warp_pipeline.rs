use super::*;

impl RuntimeWarpPipelineStateModel {
    pub(crate) fn snapshot(
        &self,
        resolved_tempo: &RuntimeResolvedTempo,
        media_pipeline: &RuntimeMediaPipelineStateModel,
    ) -> RuntimeWarpPipelineSnapshot {
        let project_tempo_bpm =
            if resolved_tempo.tempo_bpm.is_finite() && resolved_tempo.tempo_bpm > 0.0 {
                resolved_tempo.tempo_bpm
            } else {
                120.0
            };
        let clips = self
            .clips
            .values()
            .map(|registration| {
                self.snapshot_clip(registration, resolved_tempo, &media_pipeline.assets)
            })
            .collect::<Vec<_>>();
        let ready_clip_count = clips
            .iter()
            .filter(|clip| clip.readiness == RuntimeWarpReadiness::Ready)
            .count();
        let degraded_clip_count = clips
            .iter()
            .filter(|clip| clip.readiness == RuntimeWarpReadiness::Degraded)
            .count();
        let bypassed_clip_count = clips
            .iter()
            .filter(|clip| clip.readiness == RuntimeWarpReadiness::Bypassed)
            .count();
        let active_warp_count = clips
            .iter()
            .filter(|clip| clip.mode != RuntimeWarpMode::Off)
            .count();

        RuntimeWarpPipelineSnapshot {
            clip_count: clips.len(),
            ready_clip_count,
            degraded_clip_count,
            bypassed_clip_count,
            active_warp_count,
            resolved_project_tempo_bpm: project_tempo_bpm,
            resolved_project_tempo_source: resolved_tempo.source,
            resolved_project_tempo_segment_id: resolved_tempo.active_segment_id.clone(),
            clips,
            summary: format!(
                "warp clips={} active={} ready={} degraded={} bypassed={} project_tempo={project_tempo_bpm:.2} source={:?} segment={:?}",
                self.clips.len(),
                active_warp_count,
                ready_clip_count,
                degraded_clip_count,
                bypassed_clip_count,
                resolved_tempo.source,
                resolved_tempo.active_segment_id,
            ),
        }
    }

    fn snapshot_clip(
        &self,
        registration: &RuntimeWarpClipRegistration,
        resolved_tempo: &RuntimeResolvedTempo,
        media_assets: &BTreeMap<String, RuntimeMediaPipelineAsset>,
    ) -> RuntimeWarpClipSnapshot {
        let project_tempo_bpm =
            if resolved_tempo.tempo_bpm.is_finite() && resolved_tempo.tempo_bpm > 0.0 {
                resolved_tempo.tempo_bpm
            } else {
                120.0
            };
        let mut realized_ratio = 1.0;
        let (readiness, last_error) = match registration.mode {
            RuntimeWarpMode::Off => (RuntimeWarpReadiness::Bypassed, None),
            RuntimeWarpMode::Repitch | RuntimeWarpMode::ElastiqueDraft => {
                match registration.source_tempo_bpm {
                    Some(source_tempo_bpm)
                        if source_tempo_bpm.is_finite() && source_tempo_bpm > 0.0 =>
                    {
                        realized_ratio = project_tempo_bpm / source_tempo_bpm;
                        if !realized_ratio.is_finite() || realized_ratio <= 0.0 {
                            (
                                RuntimeWarpReadiness::Degraded,
                                Some("warp ratio is invalid".to_string()),
                            )
                        } else if let Some(media_asset_id) = registration.media_asset_id.as_deref()
                        {
                            match media_assets.get(media_asset_id) {
                                Some(asset) if asset.state == RuntimeMediaAssetState::Ready => {
                                    if registration.mode == RuntimeWarpMode::ElastiqueDraft
                                        && !(0.5..=2.0).contains(&realized_ratio)
                                    {
                                        (
                                            RuntimeWarpReadiness::Degraded,
                                            Some(format!(
                                                "elastique draft ratio {realized_ratio:.3} outside baseline support"
                                            )),
                                        )
                                    } else {
                                        (RuntimeWarpReadiness::Ready, None)
                                    }
                                }
                                Some(asset) => (
                                    RuntimeWarpReadiness::Degraded,
                                    Some(format!("media asset not ready: {:?}", asset.state)),
                                ),
                                None => (
                                    RuntimeWarpReadiness::Degraded,
                                    Some(format!(
                                        "media asset `{media_asset_id}` missing from runtime cache"
                                    )),
                                ),
                            }
                        } else {
                            (
                                RuntimeWarpReadiness::Degraded,
                                Some("warp clip missing media asset".to_string()),
                            )
                        }
                    }
                    Some(_) => (
                        RuntimeWarpReadiness::Degraded,
                        Some("warp source tempo must be positive".to_string()),
                    ),
                    None => (
                        RuntimeWarpReadiness::Degraded,
                        Some("warp source tempo missing".to_string()),
                    ),
                }
            }
        };

        RuntimeWarpClipSnapshot {
            clip_id: registration.clip_id.clone(),
            media_asset_id: registration.media_asset_id.clone(),
            mode: registration.mode,
            source_tempo_bpm: registration.source_tempo_bpm,
            project_tempo_bpm,
            project_tempo_source: resolved_tempo.source,
            project_tempo_segment_id: resolved_tempo.active_segment_id.clone(),
            realized_ratio,
            anchor_timeline_samples: registration.anchor_timeline_samples,
            start_samples: registration.start_samples,
            duration_samples: registration.duration_samples,
            readiness,
            last_error: last_error.clone(),
            summary: format!(
                "clip={} mode={:?} readiness={:?} ratio={realized_ratio:.3} source_tempo={} project_tempo={project_tempo_bpm:.2}/{:?}/{:?} error={}",
                registration.clip_id,
                registration.mode,
                readiness,
                registration
                    .source_tempo_bpm
                    .map(|tempo| format!("{tempo:.2}"))
                    .unwrap_or_else(|| "none".to_string()),
                resolved_tempo.source,
                resolved_tempo.active_segment_id,
                last_error.as_deref().unwrap_or("none"),
            ),
        }
    }

    pub(crate) fn reconcile_clips(&mut self, clips: Vec<RuntimeWarpClipRegistration>) {
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
