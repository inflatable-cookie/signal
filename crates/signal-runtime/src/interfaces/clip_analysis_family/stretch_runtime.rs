use super::*;

impl RuntimeStretchClipSnapshot {
    /// Derives a stretch clip snapshot from a clip processing snapshot.
    pub fn from_clip_processing_snapshot(
        clip: &RuntimeClipProcessingSnapshot,
    ) -> RuntimeStretchClipSnapshot {
        let (engine_class, readiness, fallback_kind) = match clip.warp_mode {
            RuntimeWarpMode::Off => (
                RuntimeStretchEngineClass::Disabled,
                RuntimeStretchReadiness::Disabled,
                RuntimeStretchFallbackKind::None,
            ),
            RuntimeWarpMode::Repitch => (
                RuntimeStretchEngineClass::RatioOnly,
                match clip.readiness {
                    RuntimeClipProcessingReadiness::Ready => RuntimeStretchReadiness::Ready,
                    RuntimeClipProcessingReadiness::PendingMedia => {
                        RuntimeStretchReadiness::PendingMedia
                    }
                    RuntimeClipProcessingReadiness::PendingWarp => {
                        RuntimeStretchReadiness::PendingWarp
                    }
                    RuntimeClipProcessingReadiness::Invalid => RuntimeStretchReadiness::Degraded,
                },
                RuntimeStretchFallbackKind::None,
            ),
            RuntimeWarpMode::ElastiqueDraft => match clip.readiness {
                RuntimeClipProcessingReadiness::Ready => (
                    RuntimeStretchEngineClass::SampleDomain,
                    RuntimeStretchReadiness::Ready,
                    RuntimeStretchFallbackKind::None,
                ),
                RuntimeClipProcessingReadiness::PendingMedia => (
                    RuntimeStretchEngineClass::SampleDomain,
                    RuntimeStretchReadiness::PendingMedia,
                    RuntimeStretchFallbackKind::None,
                ),
                RuntimeClipProcessingReadiness::PendingWarp => (
                    RuntimeStretchEngineClass::SampleDomain,
                    RuntimeStretchReadiness::PendingWarp,
                    RuntimeStretchFallbackKind::None,
                ),
                RuntimeClipProcessingReadiness::Invalid => (
                    RuntimeStretchEngineClass::Fallback,
                    RuntimeStretchReadiness::Degraded,
                    if clip.realized_warp_ratio.is_some() {
                        RuntimeStretchFallbackKind::RatioOnly
                    } else {
                        RuntimeStretchFallbackKind::Bypass
                    },
                ),
            },
        };

        RuntimeStretchClipSnapshot {
            clip_id: clip.clip_id.clone(),
            media_asset_id: clip.media_asset_id.clone(),
            engine_class,
            readiness,
            fallback_kind,
            summary: format!(
                "clip={} engine={:?} readiness={:?} fallback={:?}",
                clip.clip_id, engine_class, readiness, fallback_kind
            ),
        }
    }
}

impl RuntimeStretchEngineSnapshot {
    /// Derives a stretch engine snapshot from the full clip processing pipeline snapshot.
    pub fn from_clip_processing_pipeline(
        clip_processing: &RuntimeClipProcessingPipelineSnapshot,
    ) -> RuntimeStretchEngineSnapshot {
        let clips = clip_processing
            .clips
            .iter()
            .map(RuntimeStretchClipSnapshot::from_clip_processing_snapshot)
            .collect::<Vec<_>>();
        let disabled_clip_count = clips
            .iter()
            .filter(|clip| clip.engine_class == RuntimeStretchEngineClass::Disabled)
            .count();
        let ready_clip_count = clips
            .iter()
            .filter(|clip| clip.readiness == RuntimeStretchReadiness::Ready)
            .count();
        let pending_media_clip_count = clips
            .iter()
            .filter(|clip| clip.readiness == RuntimeStretchReadiness::PendingMedia)
            .count();
        let pending_warp_clip_count = clips
            .iter()
            .filter(|clip| clip.readiness == RuntimeStretchReadiness::PendingWarp)
            .count();
        let degraded_clip_count = clips
            .iter()
            .filter(|clip| clip.readiness == RuntimeStretchReadiness::Degraded)
            .count();
        let sample_domain_clip_count = clips
            .iter()
            .filter(|clip| clip.engine_class == RuntimeStretchEngineClass::SampleDomain)
            .count();
        let ratio_only_clip_count = clips
            .iter()
            .filter(|clip| clip.engine_class == RuntimeStretchEngineClass::RatioOnly)
            .count();
        let fallback_clip_count = clips
            .iter()
            .filter(|clip| clip.engine_class == RuntimeStretchEngineClass::Fallback)
            .count();

        RuntimeStretchEngineSnapshot {
            clip_count: clips.len(),
            disabled_clip_count,
            ready_clip_count,
            pending_media_clip_count,
            pending_warp_clip_count,
            degraded_clip_count,
            sample_domain_clip_count,
            ratio_only_clip_count,
            fallback_clip_count,
            clips,
            summary: format!(
                "stretch clips={} disabled={} ready={} pending_media={} pending_warp={} degraded={} sample_domain={} ratio_only={} fallback={}",
                clip_processing.clip_count,
                disabled_clip_count,
                ready_clip_count,
                pending_media_clip_count,
                pending_warp_clip_count,
                degraded_clip_count,
                sample_domain_clip_count,
                ratio_only_clip_count,
                fallback_clip_count
            ),
        }
    }
}
