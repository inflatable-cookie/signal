//! Offline stretch-artifact planning and capability evaluation.

use signal_dsp_stretch::{
    stretch_backend_plan, StretchBackendStatus, StretchBackendTier, StretchCacheIdentityInput,
    StretchPromotionReceipt, StretchRatioPoint,
};

use super::types::{
    OfflineStretchArtifactCapabilityStatus, OfflineStretchArtifactMaterializeError,
    OfflineStretchArtifactPlan, OfflineStretchArtifactPlanError, OfflineStretchArtifactReadiness,
    OfflineStretchArtifactScope,
};

/// Build a control-side artifact plan for an offline high-quality stretch
/// candidate.
///
/// This function does not render or promote anything. It gives cache/export
/// callers a deterministic identity and a typed answer for why the artifact
/// may or may not feed product-facing output yet.
pub fn plan_offline_stretch_artifact(
    scope: OfflineStretchArtifactScope,
    identity_input: &StretchCacheIdentityInput,
    promotion_receipt: StretchPromotionReceipt,
) -> Result<OfflineStretchArtifactPlan, OfflineStretchArtifactPlanError> {
    if identity_input.tier != StretchBackendTier::OfflineHighQuality {
        return Err(OfflineStretchArtifactPlanError::UnsupportedTier(
            identity_input.tier,
        ));
    }
    let identity = identity_input
        .identity()
        .map_err(OfflineStretchArtifactPlanError::InvalidIdentity)?;
    let backend = stretch_backend_plan(identity_input.tier);
    let capability_status = offline_stretch_artifact_capability_status(identity_input);
    let promotion_accepted = promotion_receipt
        .accepts_product_facing_path(identity_input.tier, identity_input.offline_path);
    let readiness = match (backend.status, promotion_accepted, capability_status) {
        (StretchBackendStatus::Planned, _, _) => {
            OfflineStretchArtifactReadiness::AwaitingImplementation
        }
        (StretchBackendStatus::Prototype, _, _) => {
            OfflineStretchArtifactReadiness::AwaitingCorpusEvidence
        }
        (StretchBackendStatus::Implemented, false, _) => {
            OfflineStretchArtifactReadiness::AwaitingCorpusEvidence
        }
        (
            StretchBackendStatus::Implemented,
            true,
            OfflineStretchArtifactCapabilityStatus::Supported,
        ) => OfflineStretchArtifactReadiness::Ready,
        (StretchBackendStatus::Implemented, true, _) => {
            OfflineStretchArtifactReadiness::UnsupportedCapability
        }
    };

    Ok(OfflineStretchArtifactPlan {
        scope,
        identity,
        tier: identity_input.tier,
        offline_path: identity_input.offline_path,
        readiness,
        capability_status,
        promotion_receipt,
        product_facing_allowed: readiness == OfflineStretchArtifactReadiness::Ready,
    })
}

pub(crate) fn static_or_initial_ratio(ratio_curve: &[StretchRatioPoint]) -> f64 {
    ratio_curve
        .iter()
        .find(|point| point.ratio.is_finite() && point.ratio > 0.0)
        .map(|point| point.ratio)
        .unwrap_or(1.0)
}

pub(crate) fn selector_offline_path_requires_static_materialization(
    path: signal_dsp_stretch::OfflineHighQualityPath,
) -> bool {
    use signal_dsp_stretch::OfflineHighQualityPath;
    matches!(
        path,
        OfflineHighQualityPath::CompressionShortWindowSelector
            | OfflineHighQualityPath::ExpansionShortWindowSelector
    )
}

pub(crate) fn offline_stretch_artifact_capability_status(
    identity_input: &StretchCacheIdentityInput,
) -> OfflineStretchArtifactCapabilityStatus {
    if identity_input.channel_layout.channels != 2 {
        return OfflineStretchArtifactCapabilityStatus::UnsupportedChannelLayout {
            channels: identity_input.channel_layout.channels,
        };
    }
    let pitch_shift = identity_input
        .pitch_curve
        .first()
        .map(|point| point.semitones)
        .unwrap_or(0.0);
    if identity_input
        .pitch_curve
        .iter()
        .any(|point| (point.semitones - pitch_shift).abs() > 1.0e-9)
    {
        return OfflineStretchArtifactCapabilityStatus::UnsupportedPitchAutomation;
    }
    if selector_offline_path_requires_static_materialization(identity_input.offline_path) {
        if ratio_curve_has_dynamic_changes(&identity_input.ratio_curve) {
            return OfflineStretchArtifactCapabilityStatus::UnsupportedOfflinePathDynamicRatio {
                path: identity_input.offline_path,
            };
        }
        if pitch_shift.abs() > 1.0e-9 {
            return OfflineStretchArtifactCapabilityStatus::UnsupportedOfflinePathPitchShift {
                path: identity_input.offline_path,
            };
        }
    }
    OfflineStretchArtifactCapabilityStatus::Supported
}

pub(crate) fn materialization_error_for_capability(
    capability_status: OfflineStretchArtifactCapabilityStatus,
) -> Option<OfflineStretchArtifactMaterializeError> {
    match capability_status {
        OfflineStretchArtifactCapabilityStatus::Supported => None,
        OfflineStretchArtifactCapabilityStatus::UnsupportedChannelLayout { channels } => {
            Some(OfflineStretchArtifactMaterializeError::UnsupportedChannelLayout { channels })
        }
        OfflineStretchArtifactCapabilityStatus::UnsupportedPitchAutomation => {
            Some(OfflineStretchArtifactMaterializeError::UnsupportedPitchAutomation)
        }
        OfflineStretchArtifactCapabilityStatus::UnsupportedOfflinePathDynamicRatio { path } => {
            Some(
                OfflineStretchArtifactMaterializeError::UnsupportedOfflinePathDynamicRatio { path },
            )
        }
        OfflineStretchArtifactCapabilityStatus::UnsupportedOfflinePathPitchShift { path } => {
            Some(OfflineStretchArtifactMaterializeError::UnsupportedOfflinePathPitchShift { path })
        }
    }
}

pub(crate) fn ratio_curve_has_dynamic_changes(ratio_curve: &[StretchRatioPoint]) -> bool {
    let mut valid_ratios = ratio_curve
        .iter()
        .filter(|point| point.ratio.is_finite() && point.ratio > 0.0)
        .map(|point| point.ratio);
    let Some(first) = valid_ratios.next() else {
        return false;
    };
    valid_ratios.any(|ratio| (ratio - first).abs() > 1.0e-9)
}

pub(crate) fn static_pitch_shift(
    identity_input: &StretchCacheIdentityInput,
) -> Result<f64, OfflineStretchArtifactMaterializeError> {
    let first = identity_input
        .pitch_curve
        .first()
        .map(|point| point.semitones)
        .unwrap_or(0.0);
    if identity_input
        .pitch_curve
        .iter()
        .any(|point| (point.semitones - first).abs() > 1.0e-9)
    {
        return Err(OfflineStretchArtifactMaterializeError::UnsupportedPitchAutomation);
    }
    Ok(first)
}
