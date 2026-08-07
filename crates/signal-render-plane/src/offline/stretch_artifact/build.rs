//! Offline stretch-artifact render-source and cache-handoff builders.

use super::rendering::materialize_offline_stretch_artifact_pcm;
use super::types::{
    OfflineStretchArtifactBuildRequest, OfflineStretchArtifactCacheHandoff,
    OfflineStretchArtifactMaterializeError, OfflineStretchArtifactRenderSource,
    OfflineStretchArtifactScope,
};

/// Build a promotion-gated OfflineHighQuality render source.
pub fn build_offline_stretch_artifact_render_source(
    request: OfflineStretchArtifactBuildRequest<'_>,
) -> Result<OfflineStretchArtifactRenderSource, OfflineStretchArtifactMaterializeError> {
    let artifact = materialize_offline_stretch_artifact_pcm(
        request.scope,
        request.identity_input,
        request.promotion_receipt,
        request.source,
    )?;
    Ok(OfflineStretchArtifactRenderSource {
        source: crate::RenderSource::Samples(artifact.buffer.clone()),
        artifact,
    })
}

/// Build a promotion-gated OfflineHighQuality cache handoff.
///
/// This helper is scoped to [`OfflineStretchArtifactScope::RenderCache`].
pub fn build_offline_stretch_artifact_cache_handoff(
    request: OfflineStretchArtifactBuildRequest<'_>,
) -> Result<OfflineStretchArtifactCacheHandoff, OfflineStretchArtifactMaterializeError> {
    if request.scope != OfflineStretchArtifactScope::RenderCache {
        return Err(
            OfflineStretchArtifactMaterializeError::UnsupportedCacheHandoffScope {
                scope: request.scope,
            },
        );
    }
    let artifact_source = build_offline_stretch_artifact_render_source(request)?;
    let receipt = artifact_source.artifact.receipt.clone();
    Ok(OfflineStretchArtifactCacheHandoff {
        cache_identity_hash: receipt.cache_identity_hash.clone(),
        cache_identity_key: receipt.cache_identity_key.clone(),
        receipt,
        source: artifact_source.source,
    })
}
