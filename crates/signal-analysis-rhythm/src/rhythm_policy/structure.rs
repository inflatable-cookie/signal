use signal_analysis::Confidence;

use super::{
    BeatAnalysisResult, MeterConfidenceBreakdown, MeterContinuityAction, MeterContinuitySource,
    MeterDetectionKind, MeterEstimate, MeterRecommendation, MeterRecoveryContext, MeterStateAction,
    MeterStateReason, MeterTrustLevel,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BarSupportKind {
    WholeTrack,
    RecoveryWindow,
    Extrapolated,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BarSpan {
    pub bar_index: usize,
    pub start_seconds: f32,
    pub end_seconds: Option<f32>,
    pub support: BarSupportKind,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RhythmStructureContinuitySummary {
    pub action: MeterStateAction,
    pub reason: MeterStateReason,
    pub confidence: Confidence,
    pub bar_length_action: MeterContinuityAction,
    pub bar_length_confidence: Confidence,
    pub downbeat_phase_action: MeterContinuityAction,
    pub downbeat_phase_confidence: Confidence,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RhythmStructureSummary {
    pub beats_per_bar: usize,
    pub detection_kind: MeterDetectionKind,
    pub trust: MeterTrustLevel,
    pub recommendation: MeterRecommendation,
    pub downbeat_positions_seconds: Vec<f32>,
    pub bars: Vec<BarSpan>,
    pub bar_count: usize,
    pub recovered_bar_count: usize,
    pub recovery: Option<MeterRecoveryContext>,
    pub continuity: RhythmStructureContinuitySummary,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RhythmStructureAmbiguityKind {
    CompetingMeter,
    CompetingDownbeatPhase,
    SyncopatedDownbeatPhase,
    WeakAccent,
    RecoveryWindowFallback,
    InsufficientEvidence,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RhythmStructureCandidate {
    pub beats_per_bar: usize,
    pub phase_offset_beats: usize,
    pub confidence: Confidence,
    pub confidence_breakdown: MeterConfidenceBreakdown,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RhythmStructureAmbiguitySummary {
    pub kind: RhythmStructureAmbiguityKind,
    pub confidence: Confidence,
    pub primary: Option<RhythmStructureCandidate>,
    pub runner_up: Option<RhythmStructureCandidate>,
    pub trailing_recovery_confidence: Confidence,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RhythmStructureFallbackSummary {
    pub action: MeterStateAction,
    pub reason: MeterStateReason,
    pub confidence: Confidence,
    pub bar_length_action: MeterContinuityAction,
    pub bar_length_source: MeterContinuitySource,
    pub downbeat_phase_action: MeterContinuityAction,
    pub downbeat_phase_source: MeterContinuitySource,
    pub recovery_window_available: bool,
    pub trailing_recovery_confidence: Confidence,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RhythmStructureAssessment {
    pub structure: Option<RhythmStructureSummary>,
    pub ambiguity: RhythmStructureAmbiguitySummary,
    pub fallback: RhythmStructureFallbackSummary,
}

pub(crate) fn meter_bar_spans(meter: &MeterEstimate) -> Vec<BarSpan> {
    let mut bars = Vec::with_capacity(meter.downbeat_positions_seconds.len());

    for (bar_index, start_seconds) in meter.downbeat_positions_seconds.iter().copied().enumerate() {
        let end_seconds = meter
            .downbeat_positions_seconds
            .get(bar_index + 1)
            .copied()
            .or_else(|| estimated_final_bar_end(&meter.downbeat_positions_seconds));

        bars.push(BarSpan {
            bar_index,
            start_seconds,
            end_seconds,
            support: bar_support_kind(meter.recovery.as_ref(), start_seconds, end_seconds),
        });
    }

    bars
}

pub(crate) fn estimated_final_bar_end(downbeats: &[f32]) -> Option<f32> {
    if downbeats.len() < 2 {
        return None;
    }

    let mut intervals: Vec<f32> = downbeats
        .windows(2)
        .map(|pair| (pair[1] - pair[0]).max(0.0))
        .filter(|interval| *interval > 0.0)
        .collect();
    if intervals.is_empty() {
        return None;
    }

    intervals.sort_by(|lhs, rhs| lhs.partial_cmp(rhs).unwrap_or(core::cmp::Ordering::Equal));
    let median = intervals[intervals.len() / 2];
    downbeats.last().copied().map(|start| start + median)
}

fn bar_support_kind(
    recovery: Option<&MeterRecoveryContext>,
    start_seconds: f32,
    end_seconds: Option<f32>,
) -> BarSupportKind {
    let Some(recovery) = recovery else {
        return BarSupportKind::WholeTrack;
    };

    let overlaps_recovery = match end_seconds {
        Some(end_seconds) => {
            start_seconds < recovery.end_seconds && end_seconds > recovery.start_seconds
        }
        None => start_seconds >= recovery.start_seconds && start_seconds <= recovery.end_seconds,
    };

    if overlaps_recovery {
        BarSupportKind::RecoveryWindow
    } else {
        BarSupportKind::Extrapolated
    }
}

impl BeatAnalysisResult {
    pub fn rhythm_structure_summary(&self) -> Option<RhythmStructureSummary> {
        let meter = self.meter.as_ref()?;
        let bars = meter_bar_spans(meter);
        let recovered_bar_count = bars
            .iter()
            .filter(|bar| matches!(bar.support, BarSupportKind::RecoveryWindow))
            .count();

        Some(RhythmStructureSummary {
            beats_per_bar: meter.beats_per_bar,
            detection_kind: meter.detection_kind,
            trust: meter.trust,
            recommendation: meter.recommendation,
            downbeat_positions_seconds: meter.downbeat_positions_seconds.clone(),
            bar_count: bars.len(),
            bars,
            recovered_bar_count,
            recovery: meter.recovery.clone(),
            continuity: RhythmStructureContinuitySummary {
                action: self.meter_state.action,
                reason: self.meter_state.reason,
                confidence: self.meter_state.confidence,
                bar_length_action: self.meter_state.continuity.bar_length.action,
                bar_length_confidence: self.meter_state.continuity.bar_length.confidence,
                downbeat_phase_action: self.meter_state.continuity.downbeat_phase.action,
                downbeat_phase_confidence: self.meter_state.continuity.downbeat_phase.confidence,
            },
        })
    }

    pub fn rhythm_structure_assessment(&self) -> RhythmStructureAssessment {
        let structure = self.rhythm_structure_summary();
        let fallback = RhythmStructureFallbackSummary {
            action: self.meter_state.action,
            reason: self.meter_state.reason,
            confidence: self.meter_state.confidence,
            bar_length_action: self.meter_state.continuity.bar_length.action,
            bar_length_source: self.meter_state.continuity.bar_length.source,
            downbeat_phase_action: self.meter_state.continuity.downbeat_phase.action,
            downbeat_phase_source: self.meter_state.continuity.downbeat_phase.source,
            recovery_window_available: self
                .meter
                .as_ref()
                .and_then(|estimate| estimate.recovery.as_ref())
                .is_some()
                || self.structure_ambiguity.trailing_recovery_confidence.0 > 0.0,
            trailing_recovery_confidence: self.structure_ambiguity.trailing_recovery_confidence,
        };
        let mut ambiguity = self.structure_ambiguity;

        if structure.is_some()
            && matches!(
                ambiguity.kind,
                RhythmStructureAmbiguityKind::RecoveryWindowFallback
            )
            && matches!(
                fallback.bar_length_source,
                MeterContinuitySource::CurrentMeter
            )
        {
            ambiguity.kind = RhythmStructureAmbiguityKind::WeakAccent;
        } else if structure.is_none()
            && fallback.recovery_window_available
            && !matches!(ambiguity.kind, RhythmStructureAmbiguityKind::CompetingMeter)
        {
            ambiguity.kind = RhythmStructureAmbiguityKind::RecoveryWindowFallback;
        }

        RhythmStructureAssessment {
            structure,
            ambiguity,
            fallback,
        }
    }
}
