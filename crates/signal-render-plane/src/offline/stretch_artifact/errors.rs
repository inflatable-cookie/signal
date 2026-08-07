//! Display and error-trait wiring for offline stretch-artifact failures.

use super::types::{OfflineStretchArtifactMaterializeError, OfflineStretchArtifactPlanError};

impl std::fmt::Display for OfflineStretchArtifactPlanError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OfflineStretchArtifactPlanError::InvalidIdentity(error) => {
                write!(formatter, "invalid stretch cache identity: {error:?}")
            }
            OfflineStretchArtifactPlanError::UnsupportedTier(tier) => write!(
                formatter,
                "offline stretch artifacts require OfflineHighQuality, got {tier:?}",
            ),
        }
    }
}

impl std::error::Error for OfflineStretchArtifactPlanError {}

impl std::fmt::Display for OfflineStretchArtifactMaterializeError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OfflineStretchArtifactMaterializeError::Plan(error) => write!(formatter, "{error}"),
            OfflineStretchArtifactMaterializeError::NotReady(readiness) => write!(
                formatter,
                "offline stretch artifact is not product-facing ready: {readiness:?}",
            ),
            OfflineStretchArtifactMaterializeError::UnsupportedChannelLayout { channels } => {
                write!(
                    formatter,
                    "offline stretch artifact PCM requires stereo source, got {channels} channels",
                )
            }
            OfflineStretchArtifactMaterializeError::SourceSampleRateMismatch {
                expected_hz,
                actual_hz,
            } => write!(
                formatter,
                "offline stretch artifact source sample rate mismatch: expected {expected_hz}, got {actual_hz}",
            ),
            OfflineStretchArtifactMaterializeError::UnsupportedPitchAutomation => write!(
                formatter,
                "offline stretch artifact materialization requires static pitch shift",
            ),
            OfflineStretchArtifactMaterializeError::UnsupportedOfflinePathDynamicRatio {
                path,
            } => write!(
                formatter,
                "offline stretch artifact path {path:?} does not support dynamic ratio materialization yet",
            ),
            OfflineStretchArtifactMaterializeError::UnsupportedOfflinePathPitchShift { path } => {
                write!(
                    formatter,
                    "offline stretch artifact path {path:?} does not support pitch-shift materialization yet",
                )
            }
            OfflineStretchArtifactMaterializeError::UnsupportedCacheHandoffScope { scope } => {
                write!(
                    formatter,
                    "offline stretch render-cache handoff requires RenderCache scope, got {scope:?}",
                )
            }
        }
    }
}

impl std::error::Error for OfflineStretchArtifactMaterializeError {}

impl From<OfflineStretchArtifactPlanError> for OfflineStretchArtifactMaterializeError {
    fn from(error: OfflineStretchArtifactPlanError) -> Self {
        Self::Plan(error)
    }
}
