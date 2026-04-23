use signal_analysis::Confidence;

use super::{
    BeatAnalysisResult, MeterConfidenceBreakdown, MeterContinuityAction, MeterContinuitySource,
    MeterDetectionKind, MeterEstimate, MeterRecommendation, MeterRecoveryContext, MeterStateAction,
    MeterStateReason, MeterTrustLevel,
};

/// How a bar's position was determined.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BarSupportKind {
    /// Bar is fully supported by whole-track meter evidence.
    WholeTrack,
    /// Bar falls within the segment-recovery window.
    RecoveryWindow,
    /// Bar position was extrapolated from adjacent evidence.
    Extrapolated,
}

/// Time span of one bar within a rhythm structure summary.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BarSpan {
    /// Zero-based index of this bar within the track.
    pub bar_index: usize,
    /// Downbeat time of this bar, in seconds.
    pub start_seconds: f32,
    /// Estimated end time of this bar (start of the next bar), if known.
    pub end_seconds: Option<f32>,
    /// How this bar's position was determined.
    pub support: BarSupportKind,
}

/// Flattened meter continuity state for embedding in a structure summary.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RhythmStructureContinuitySummary {
    /// High-level meter state action.
    pub action: MeterStateAction,
    /// Reason for the meter state action.
    pub reason: MeterStateReason,
    /// Overall meter confidence.
    pub confidence: Confidence,
    /// Continuity action for bar length.
    pub bar_length_action: MeterContinuityAction,
    /// Confidence in the bar-length continuity.
    pub bar_length_confidence: Confidence,
    /// Continuity action for downbeat phase.
    pub downbeat_phase_action: MeterContinuityAction,
    /// Confidence in the downbeat-phase continuity.
    pub downbeat_phase_confidence: Confidence,
}

/// High-level rhythm structure description for a detected meter.
#[derive(Clone, Debug, PartialEq)]
pub struct RhythmStructureSummary {
    /// Number of beats per bar.
    pub beats_per_bar: usize,
    /// Whether the meter was derived whole-track or via segment recovery.
    pub detection_kind: MeterDetectionKind,
    /// Reliability of the meter estimate.
    pub trust: MeterTrustLevel,
    /// Suggested consumer action for the meter.
    pub recommendation: MeterRecommendation,
    /// Downbeat times in seconds, in ascending order.
    pub downbeat_positions_seconds: Vec<f32>,
    /// Span information for each detected bar.
    pub bars: Vec<BarSpan>,
    /// Total number of bars in `bars`.
    pub bar_count: usize,
    /// Number of bars that fall within the recovery window.
    pub recovered_bar_count: usize,
    /// Recovery context if a sub-segment recovery was used.
    pub recovery: Option<MeterRecoveryContext>,
    /// Meter continuity state at the time of this summary.
    pub continuity: RhythmStructureContinuitySummary,
}

/// Nature of any rhythmic structure ambiguity.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RhythmStructureAmbiguityKind {
    /// Two or more bar-length hypotheses have similar support.
    CompetingMeter,
    /// Two or more downbeat phase hypotheses have similar support.
    CompetingDownbeatPhase,
    /// The strongest downbeat phase is consistent with syncopation.
    SyncopatedDownbeatPhase,
    /// Downbeat accents are present but weak relative to other beats.
    WeakAccent,
    /// The primary evidence comes from a recovery window rather than the full track.
    RecoveryWindowFallback,
    /// Not enough evidence to characterise the ambiguity.
    InsufficientEvidence,
}

/// One competing meter/downbeat hypothesis in an ambiguity summary.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RhythmStructureCandidate {
    /// Number of beats per bar for this candidate.
    pub beats_per_bar: usize,
    /// Downbeat phase offset in beats relative to the beat grid.
    pub phase_offset_beats: usize,
    /// Detection confidence for this candidate.
    pub confidence: Confidence,
    /// Component confidence scores for this candidate.
    pub confidence_breakdown: MeterConfidenceBreakdown,
}

/// Summary of rhythmic structure ambiguity for a single analysis pass.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RhythmStructureAmbiguitySummary {
    /// Dominant type of ambiguity detected.
    pub kind: RhythmStructureAmbiguityKind,
    /// Confidence that the ambiguity is genuine (higher = more ambiguous).
    pub confidence: Confidence,
    /// The strongest meter/phase candidate, if one was identified.
    pub primary: Option<RhythmStructureCandidate>,
    /// The next-strongest candidate, if one exists.
    pub runner_up: Option<RhythmStructureCandidate>,
    /// Confidence remaining in a recovery window from a prior pass.
    pub trailing_recovery_confidence: Confidence,
}

/// Fallback meter continuity information when no structure summary is available.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RhythmStructureFallbackSummary {
    /// High-level meter state action at fallback time.
    pub action: MeterStateAction,
    /// Reason for the meter state action.
    pub reason: MeterStateReason,
    /// Overall meter confidence at fallback time.
    pub confidence: Confidence,
    /// Continuity action to apply for bar length.
    pub bar_length_action: MeterContinuityAction,
    /// Source of the bar-length value being carried.
    pub bar_length_source: MeterContinuitySource,
    /// Continuity action to apply for downbeat phase.
    pub downbeat_phase_action: MeterContinuityAction,
    /// Source of the downbeat-phase value being carried.
    pub downbeat_phase_source: MeterContinuitySource,
    /// Whether a recovery window is present and can be used.
    pub recovery_window_available: bool,
    /// Confidence remaining in any trailing recovery window.
    pub trailing_recovery_confidence: Confidence,
}

/// Complete rhythm structure assessment returned by
/// [`BeatAnalysisResult::rhythm_structure_assessment`].
#[derive(Clone, Debug, PartialEq)]
pub struct RhythmStructureAssessment {
    /// Full structure summary, or `None` if no meter was detected.
    pub structure: Option<RhythmStructureSummary>,
    /// Ambiguity information regardless of whether a structure was found.
    pub ambiguity: RhythmStructureAmbiguitySummary,
    /// Fallback continuity state for use when `structure` is `None`.
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
    /// Build a [`RhythmStructureSummary`] from the detected meter, or return
    /// `None` if no meter was found.
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

    /// Build a complete [`RhythmStructureAssessment`] including ambiguity and
    /// fallback continuity regardless of whether a meter was detected.
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
