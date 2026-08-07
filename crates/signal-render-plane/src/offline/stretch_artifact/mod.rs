//! Offline stretch-artifact planning and materialization.

mod bridge;
mod build;
mod errors;
mod planning;
mod rendering;
mod types;

pub use build::{
    build_offline_stretch_artifact_cache_handoff, build_offline_stretch_artifact_render_source,
};
pub use planning::plan_offline_stretch_artifact;
pub use rendering::materialize_offline_stretch_artifact_pcm;
pub use types::{
    OfflineStretchArtifactBuildRequest, OfflineStretchArtifactCacheDecision,
    OfflineStretchArtifactCacheDecisionKind, OfflineStretchArtifactCacheHandoff,
    OfflineStretchArtifactMaterializationReceipt, OfflineStretchArtifactMaterializeError,
    OfflineStretchArtifactPcm, OfflineStretchArtifactPlan, OfflineStretchArtifactPlanError,
    OfflineStretchArtifactReadiness, OfflineStretchArtifactRenderCacheBridge,
    OfflineStretchArtifactRenderSource, OfflineStretchArtifactScope,
};

#[cfg(test)]
pub use rendering::materialize_offline_stretch_artifact_pcm_with_chunk_config;
#[cfg(test)]
pub use types::OfflineStretchArtifactCapabilityStatus;
