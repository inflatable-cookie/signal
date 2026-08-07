//! Render-cache bridge for promotion-gated stretch artifacts.

use super::build::build_offline_stretch_artifact_cache_handoff;
use super::types::{
    OfflineStretchArtifactBuildRequest, OfflineStretchArtifactCacheDecision,
    OfflineStretchArtifactCacheDecisionKind, OfflineStretchArtifactCacheHandoff,
    OfflineStretchArtifactMaterializeError, OfflineStretchArtifactPlanError,
    OfflineStretchArtifactRenderCacheBridge,
};

impl OfflineStretchArtifactRenderCacheBridge {
    /// Create an empty render-cache bridge.
    pub fn new() -> Self {
        Self::default()
    }

    /// Number of cache handoffs currently retained by this bridge.
    pub fn len(&self) -> usize {
        self.handoffs_by_hash.len()
    }

    /// Whether the bridge has no retained cache handoffs.
    pub fn is_empty(&self) -> bool {
        self.handoffs_by_hash.is_empty()
    }

    /// Return true when a stable cache identity hash is retained.
    pub fn contains_identity_hash(&self, cache_identity_hash: &str) -> bool {
        self.handoffs_by_hash.contains_key(cache_identity_hash)
    }

    /// Remove one retained cache handoff by stable identity hash.
    pub fn invalidate_identity_hash(
        &mut self,
        cache_identity_hash: &str,
    ) -> Option<OfflineStretchArtifactCacheHandoff> {
        self.invalidate_identity_hash_with_decision(cache_identity_hash)
            .map(|decision| decision.handoff)
    }

    /// Remove one retained cache handoff and return an invalidation decision.
    pub fn invalidate_identity_hash_with_decision(
        &mut self,
        cache_identity_hash: &str,
    ) -> Option<OfflineStretchArtifactCacheDecision> {
        self.handoffs_by_hash
            .remove(cache_identity_hash)
            .map(|handoff| OfflineStretchArtifactCacheDecision {
                kind: OfflineStretchArtifactCacheDecisionKind::Invalidated,
                handoff,
            })
    }

    /// Resolve a promotion-gated render-cache request against retained handoffs.
    ///
    /// Incomplete promotion evidence cannot write a new product-facing
    /// handoff. A miss returns
    /// [`OfflineStretchArtifactMaterializeError::NotReady`] and writes nothing.
    pub fn resolve(
        &mut self,
        request: OfflineStretchArtifactBuildRequest<'_>,
    ) -> Result<OfflineStretchArtifactCacheDecision, OfflineStretchArtifactMaterializeError> {
        let identity = request
            .identity_input
            .identity()
            .map_err(OfflineStretchArtifactPlanError::InvalidIdentity)?;
        if let Some(handoff) = self.handoffs_by_hash.get(&identity.stable_hash) {
            return Ok(OfflineStretchArtifactCacheDecision {
                kind: OfflineStretchArtifactCacheDecisionKind::Hit,
                handoff: handoff.clone(),
            });
        }

        let handoff = build_offline_stretch_artifact_cache_handoff(request)?;
        self.handoffs_by_hash
            .insert(handoff.cache_identity_hash.clone(), handoff.clone());
        Ok(OfflineStretchArtifactCacheDecision {
            kind: OfflineStretchArtifactCacheDecisionKind::Written,
            handoff,
        })
    }
}
