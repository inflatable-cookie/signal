use signal_analysis::Confidence;

use crate::{
    HarmonicChangeKind, HarmonicChangeSummary, Key, LocalTonalAmbiguitySummary, TonalAmbiguityKind,
    TonalSegmentAmbiguitySummary, TonalSegmentSummary,
};

#[derive(Clone, Copy, Debug, PartialEq)]
struct StableKeyRun {
    key: Key,
    start_segment_index: usize,
    end_segment_index: usize,
    start_seconds: f32,
    end_seconds: f32,
    average_confidence: f32,
}

pub(super) fn local_tonal_ambiguities(
    segments: &[TonalSegmentSummary],
    changes: &[HarmonicChangeSummary],
) -> Vec<LocalTonalAmbiguitySummary> {
    if segments.is_empty() {
        return Vec::new();
    }

    let mut ambiguities = Vec::new();
    let first_segment = segments.first().expect("non-empty segments");
    let last_segment = segments.last().expect("non-empty segments");
    let average_segment_confidence = segments
        .iter()
        .map(|segment| segment.confidence.0)
        .sum::<f32>()
        / segments.len() as f32;
    let weak_segments: Vec<&TonalSegmentSummary> = segments
        .iter()
        .filter(|segment| {
            matches!(
                segment.ambiguity,
                Some(TonalSegmentAmbiguitySummary {
                    kind: TonalAmbiguityKind::WeakTonalCenter,
                    ..
                })
            )
        })
        .collect();
    let stable_runs = stable_key_runs(segments);
    let confirmed_changes: Vec<&HarmonicChangeSummary> = changes
        .iter()
        .filter(|change| change.kind == HarmonicChangeKind::ConfirmedKeyChange)
        .collect();
    let competing_segments: Vec<&TonalSegmentSummary> = segments
        .iter()
        .filter(|segment| {
            matches!(
                segment.ambiguity,
                Some(TonalSegmentAmbiguitySummary {
                    kind: TonalAmbiguityKind::CompetingKeyCenters,
                    ..
                })
            )
        })
        .collect();

    if (weak_segments.len() * 2 >= segments.len() || average_segment_confidence < 0.12)
        && confirmed_changes.is_empty()
        && stable_runs.len() <= 1
    {
        ambiguities.push(LocalTonalAmbiguitySummary {
            kind: TonalAmbiguityKind::WeakTonalCenter,
            confidence: Confidence::new(weak_segments.len() as f32 / segments.len() as f32),
            primary_key: segments.iter().find_map(|segment| segment.key),
            alternate_key: weak_segments.iter().find_map(|segment| {
                segment
                    .ambiguity
                    .and_then(|ambiguity| ambiguity.alternate_key)
            }),
            start_segment_index: first_segment.index,
            end_segment_index: last_segment.index,
            start_seconds: first_segment.start_seconds,
            end_seconds: last_segment.end_seconds,
        });
    }

    if confirmed_changes.len() == 1 {
        let change = confirmed_changes[0];
        ambiguities.push(LocalTonalAmbiguitySummary {
            kind: TonalAmbiguityKind::Modulation,
            confidence: change.confidence,
            primary_key: change.from_key,
            alternate_key: change.to_key,
            start_segment_index: change.from_segment_index,
            end_segment_index: change.to_segment_index,
            start_seconds: segments[change.from_segment_index].start_seconds,
            end_seconds: segments[change.to_segment_index].end_seconds,
        });
    } else if confirmed_changes.len() > 1 || stable_runs.len() > 2 || competing_segments.len() >= 2
    {
        let primary_key = stable_runs
            .first()
            .map(|run| run.key)
            .or_else(|| competing_segments.first().and_then(|segment| segment.key));
        let alternate_key = stable_runs.get(1).map(|run| run.key).or_else(|| {
            competing_segments.iter().find_map(|segment| {
                segment
                    .ambiguity
                    .and_then(|ambiguity| ambiguity.alternate_key)
            })
        });
        let ambiguity_strength = if !competing_segments.is_empty() {
            competing_segments
                .iter()
                .filter_map(|segment| segment.ambiguity.map(|ambiguity| ambiguity.confidence.0))
                .sum::<f32>()
                / competing_segments.len() as f32
        } else {
            (stable_runs.len() as f32 / segments.len() as f32).clamp(0.0, 1.0)
        };
        ambiguities.push(LocalTonalAmbiguitySummary {
            kind: TonalAmbiguityKind::MixedTonality,
            confidence: Confidence::new(ambiguity_strength.clamp(0.0, 1.0)),
            primary_key,
            alternate_key,
            start_segment_index: first_segment.index,
            end_segment_index: last_segment.index,
            start_seconds: first_segment.start_seconds,
            end_seconds: last_segment.end_seconds,
        });
    }

    ambiguities
}

fn stable_key_runs(segments: &[TonalSegmentSummary]) -> Vec<StableKeyRun> {
    let mut runs: Vec<StableKeyRun> = Vec::new();

    for segment in segments.iter().filter(|segment| segment.key.is_some()) {
        if segment.confidence.0 < 0.10 {
            continue;
        }
        if matches!(
            segment.ambiguity,
            Some(TonalSegmentAmbiguitySummary {
                kind: TonalAmbiguityKind::WeakTonalCenter,
                ..
            })
        ) {
            continue;
        }

        let key = segment.key.expect("filtered to some key");
        match runs.last_mut() {
            Some(run) if run.key == key => {
                let run_length = (run
                    .end_segment_index
                    .saturating_sub(run.start_segment_index)
                    + 1) as f32;
                run.end_segment_index = segment.index;
                run.end_seconds = segment.end_seconds;
                run.average_confidence = ((run.average_confidence * run_length)
                    + segment.confidence.0)
                    / (run_length + 1.0);
            }
            _ => {
                runs.push(StableKeyRun {
                    key,
                    start_segment_index: segment.index,
                    end_segment_index: segment.index,
                    start_seconds: segment.start_seconds,
                    end_seconds: segment.end_seconds,
                    average_confidence: segment.confidence.0,
                });
            }
        }
    }

    runs
}
