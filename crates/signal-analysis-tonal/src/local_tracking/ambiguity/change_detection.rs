use signal_analysis::Confidence;

use crate::{HarmonicChangeKind, HarmonicChangeSummary, TonalSegmentSummary};

pub(super) fn harmonic_changes(segments: &[TonalSegmentSummary]) -> Vec<HarmonicChangeSummary> {
    let mut changes = Vec::new();

    for pair in segments.windows(2) {
        let left = &pair[0];
        let right = &pair[1];
        let distance = chroma_distance(left.chroma, right.chroma);
        let distance_confidence = Confidence::new(distance);
        let key_changed = left.key != right.key && left.key.is_some() && right.key.is_some();
        let confidence = if key_changed {
            Confidence::new(
                ((left.confidence.0 + right.confidence.0) * 0.5 * distance.max(0.35))
                    .clamp(0.0, 1.0),
            )
        } else {
            Confidence::new(
                ((left.confidence.0 + right.confidence.0) * 0.25 * distance).clamp(0.0, 1.0),
            )
        };

        let kind = if key_changed {
            Some(HarmonicChangeKind::ConfirmedKeyChange)
        } else if distance >= 0.30 && (left.confidence.0 >= 0.15 || right.confidence.0 >= 0.15) {
            Some(HarmonicChangeKind::TonalDrift)
        } else {
            None
        };

        if let Some(kind) = kind {
            changes.push(HarmonicChangeSummary {
                kind,
                from_segment_index: left.index,
                to_segment_index: right.index,
                at_seconds: right.start_seconds,
                from_key: left.key,
                to_key: right.key,
                confidence,
                chroma_distance: distance_confidence,
            });
        }
    }

    changes
}

fn chroma_distance(lhs: [f32; 12], rhs: [f32; 12]) -> f32 {
    let total = lhs
        .iter()
        .zip(rhs.iter())
        .map(|(left, right)| (left - right).abs())
        .sum::<f32>();
    (0.5 * total).clamp(0.0, 1.0)
}
