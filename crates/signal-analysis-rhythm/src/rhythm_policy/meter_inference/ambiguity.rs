use signal_analysis::Confidence;

use crate::{
    RhythmStructureAmbiguityKind, RhythmStructureAmbiguitySummary, RhythmStructureCandidate,
};

use super::{
    meter_confidence_breakdown, meter_hypothesis_confidence, MeterHypothesis, MeterWindowCandidate,
};

pub(crate) fn rhythm_structure_candidate(
    hypothesis: MeterHypothesis,
    runner_up_score: f32,
) -> RhythmStructureCandidate {
    RhythmStructureCandidate {
        beats_per_bar: hypothesis.beats_per_bar,
        phase_offset_beats: hypothesis.phase_offset_beats,
        confidence: meter_hypothesis_confidence(hypothesis, runner_up_score),
        confidence_breakdown: meter_confidence_breakdown(hypothesis, runner_up_score),
    }
}

pub(crate) fn rhythm_structure_ambiguity_summary(
    hypotheses: &[MeterHypothesis],
    trailing_candidate: Option<MeterWindowCandidate>,
) -> RhythmStructureAmbiguitySummary {
    let trailing_recovery_confidence = trailing_candidate
        .map(|candidate| candidate.confidence)
        .unwrap_or(Confidence::new(0.0));

    let Some(best) = hypotheses.first().copied() else {
        return RhythmStructureAmbiguitySummary {
            kind: RhythmStructureAmbiguityKind::InsufficientEvidence,
            confidence: Confidence::new(trailing_recovery_confidence.0),
            primary: None,
            runner_up: None,
            trailing_recovery_confidence,
        };
    };

    let runner_up = hypotheses.get(1).copied();
    let runner_up_score = runner_up.map(|candidate| candidate.score).unwrap_or(0.0);
    let primary = Some(rhythm_structure_candidate(best, runner_up_score));
    let runner_up_summary = runner_up.map(|candidate| {
        let third_score = hypotheses.get(2).map(|entry| entry.score).unwrap_or(0.0);
        rhythm_structure_candidate(candidate, third_score)
    });

    let breakdown = meter_confidence_breakdown(best, runner_up_score);
    let ambiguity_confidence = Confidence::new(
        (0.45 * (1.0 - breakdown.phase_margin)
            + 0.20 * (1.0 - best.support_ratio).clamp(0.0, 1.0)
            + 0.15 * (1.0 - best.regularity).clamp(0.0, 1.0)
            + 0.10 * (1.0 - best.meter_support_ratio).clamp(0.0, 1.0)
            + 0.10 * trailing_recovery_confidence.0)
            .clamp(0.0, 1.0),
    );

    let primary_confidence = primary
        .map(|candidate| candidate.confidence.0)
        .unwrap_or(0.0);
    let kind = if trailing_recovery_confidence.0 >= 0.24
        && (primary_confidence <= 0.32 || ambiguity_confidence.0 >= 0.45)
    {
        RhythmStructureAmbiguityKind::RecoveryWindowFallback
    } else if best.support_ratio < 0.68 || best.regularity < 0.55 || best.meter_contrast_mean < 0.07
    {
        RhythmStructureAmbiguityKind::WeakAccent
    } else if let Some(runner_up) = runner_up {
        if runner_up.phase_offset_beats != best.phase_offset_beats
            && runner_up.beats_per_bar == best.beats_per_bar
        {
            if best.support_ratio < 0.78 || breakdown.phase_margin < 0.35 {
                RhythmStructureAmbiguityKind::SyncopatedDownbeatPhase
            } else {
                RhythmStructureAmbiguityKind::CompetingDownbeatPhase
            }
        } else if runner_up.beats_per_bar != best.beats_per_bar {
            RhythmStructureAmbiguityKind::CompetingMeter
        } else {
            RhythmStructureAmbiguityKind::InsufficientEvidence
        }
    } else {
        RhythmStructureAmbiguityKind::InsufficientEvidence
    };

    RhythmStructureAmbiguitySummary {
        kind,
        confidence: ambiguity_confidence,
        primary,
        runner_up: runner_up_summary,
        trailing_recovery_confidence,
    }
}
