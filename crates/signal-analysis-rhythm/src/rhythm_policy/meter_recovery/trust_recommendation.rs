use signal_analysis::Confidence;

use crate::rhythm_policy::MeterWindowCandidate;
use crate::{
    MeterConfidenceBreakdown, MeterDetectionKind, MeterRecommendation, MeterSupportProfile,
    MeterTrustLevel,
};

pub(crate) fn meter_recovery_duration_strength(candidate: MeterWindowCandidate) -> Confidence {
    let recovered_beats = candidate.end_beat.saturating_sub(candidate.start_beat) as f32;
    let beat_span_strength = (recovered_beats / 16.0).clamp(0.0, 1.0);
    let window_strength = (candidate.supporting_windows as f32 / 3.0).clamp(0.0, 1.0);
    Confidence::new(0.7 * beat_span_strength + 0.3 * window_strength)
}

pub(crate) fn meter_support_profile(
    whole_track_strength: Option<Confidence>,
    segment_candidate: Option<MeterWindowCandidate>,
) -> MeterSupportProfile {
    MeterSupportProfile {
        whole_track_strength: whole_track_strength.unwrap_or(Confidence::new(0.0)),
        segment_recovery_strength: segment_candidate
            .map(|candidate| candidate.confidence)
            .unwrap_or(Confidence::new(0.0)),
        recovery_duration_strength: segment_candidate
            .map(meter_recovery_duration_strength)
            .unwrap_or(Confidence::new(0.0)),
    }
}

pub(crate) fn meter_trust_level(
    detection_kind: MeterDetectionKind,
    confidence: Confidence,
    support_profile: MeterSupportProfile,
    confidence_breakdown: MeterConfidenceBreakdown,
) -> MeterTrustLevel {
    match detection_kind {
        MeterDetectionKind::WholeTrack
            if confidence.0 >= 0.30
                && support_profile.whole_track_strength.0 >= 0.30
                && confidence_breakdown.support >= 0.80
                && confidence_breakdown.regularity >= 0.45
                && confidence_breakdown.phase_margin >= 0.25 =>
        {
            MeterTrustLevel::Stable
        }
        MeterDetectionKind::SegmentRecovery
            if confidence.0 >= 0.24
                && support_profile.segment_recovery_strength.0 >= 0.24
                && support_profile.recovery_duration_strength.0 >= 0.55
                && confidence_breakdown.recent_stability >= 0.14
                && confidence_breakdown.regularity >= 0.62 =>
        {
            MeterTrustLevel::Recovering
        }
        _ => MeterTrustLevel::Tentative,
    }
}

pub(crate) fn meter_recommendation(
    trust: MeterTrustLevel,
    detection_kind: MeterDetectionKind,
    confidence: Confidence,
    support_profile: MeterSupportProfile,
    confidence_breakdown: MeterConfidenceBreakdown,
) -> MeterRecommendation {
    match trust {
        MeterTrustLevel::Stable
            if detection_kind == MeterDetectionKind::WholeTrack
                && confidence.0 >= 0.38
                && support_profile.whole_track_strength.0 >= 0.38
                && confidence_breakdown.support >= 0.82
                && confidence_breakdown.phase_margin >= 0.30 =>
        {
            MeterRecommendation::Lock
        }
        MeterTrustLevel::Recovering => MeterRecommendation::Monitor,
        MeterTrustLevel::Tentative
            if confidence.0 >= 0.24
                && confidence_breakdown.support >= 0.72
                && confidence_breakdown.phase_margin >= 0.18 =>
        {
            MeterRecommendation::Monitor
        }
        _ => MeterRecommendation::Defer,
    }
}
