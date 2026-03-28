use signal_analysis::Confidence;

use crate::{
    KeyDetectorConfig, LocalTonalTrackingSummary, TonalAmbiguityKind, TonalSegmentAmbiguitySummary,
    TonalSegmentSummary,
};

mod change_detection;
mod summary;

use change_detection::harmonic_changes;
use summary::local_tonal_ambiguities;

pub(super) fn summarize_changes_and_ambiguities(
    config: KeyDetectorConfig,
    mut segments: Vec<TonalSegmentSummary>,
) -> LocalTonalTrackingSummary {
    for segment in &mut segments {
        segment.ambiguity = segment_ambiguity(segment);
    }
    let changes = harmonic_changes(&segments);
    let ambiguities = local_tonal_ambiguities(&segments, &changes);

    LocalTonalTrackingSummary {
        window_seconds: config.section_window_seconds as f32,
        hop_seconds: config.section_hop_seconds as f32,
        segments,
        changes,
        ambiguities,
    }
}

fn segment_ambiguity(segment: &TonalSegmentSummary) -> Option<TonalSegmentAmbiguitySummary> {
    let best = segment.scoring.best?;
    let runner_up = segment.scoring.runner_up;
    let correlation_gap = runner_up
        .map(|candidate| (best.correlation - candidate.correlation).abs())
        .unwrap_or(best.correlation.abs());
    let ambiguity_confidence = Confidence::new((1.0 - segment.confidence.0).clamp(0.0, 1.0));

    if segment.key.is_none() || segment.confidence.0 < 0.10 || best.correlation < 0.45 {
        return Some(TonalSegmentAmbiguitySummary {
            kind: TonalAmbiguityKind::WeakTonalCenter,
            confidence: ambiguity_confidence,
            best_key: Some(best.key),
            alternate_key: runner_up.map(|candidate| candidate.key),
            correlation_gap,
        });
    }

    if let Some(runner_up) = runner_up {
        if runner_up.key != best.key && correlation_gap <= 0.08 {
            return Some(TonalSegmentAmbiguitySummary {
                kind: TonalAmbiguityKind::CompetingKeyCenters,
                confidence: ambiguity_confidence,
                best_key: Some(best.key),
                alternate_key: Some(runner_up.key),
                correlation_gap,
            });
        }
    }

    None
}
