use signal_analysis::Confidence;

use super::{
    BeatAnalysisResult, MeterConfidenceBreakdown, MeterDetectionKind, MeterEstimate,
    MeterRecommendation, MeterRecoveryContext, MeterTrustLevel,
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
        })
    }
}
