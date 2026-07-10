use signal_primitives::Sample;

use crate::COMPRESSION_SHORT_WINDOW_SELECTOR_ANALYSIS_HOP;

use super::{
    StretchHybridFrameTrace, StretchHybridOwner, StretchHybridTransitionTrace, TRANSITION_FRAMES,
};

pub(super) fn schedule_transitions(
    frames: &[StretchHybridFrameTrace],
    current_output: &[Sample],
    ratio: f64,
) -> Vec<StretchHybridTransitionTrace> {
    if current_output.is_empty() {
        return Vec::new();
    }
    let search_radius = (COMPRESSION_SHORT_WINDOW_SELECTOR_ANALYSIS_HOP as f64 * ratio)
        .round()
        .max(1.0) as usize;
    let mut transitions = Vec::new();
    for pair in frames.windows(2) {
        let from = pair[0].owner;
        let to = pair[1].owner;
        if from == to {
            continue;
        }
        let requested = pair[0].output_frame.saturating_add(pair[1].output_frame) / 2;
        let scheduled =
            lowest_safe_power_frame(frames, current_output, requested, search_radius, from, to);
        transitions.push(StretchHybridTransitionTrace {
            from,
            to,
            requested_output_frame: requested,
            scheduled_output_frame: scheduled,
            search_offset_frames: signed_frame_delta(scheduled, requested),
            crossfade_frames: TRANSITION_FRAMES,
        });
    }
    transitions
}

fn lowest_safe_power_frame(
    frames: &[StretchHybridFrameTrace],
    output: &[Sample],
    requested: usize,
    radius: usize,
    from: StretchHybridOwner,
    to: StretchHybridOwner,
) -> usize {
    let requested = requested.min(output.len() - 1);
    let start = requested.saturating_sub(radius);
    let end = requested.saturating_add(radius).min(output.len() - 1);
    (start..=end)
        .filter(|candidate| transition_candidate_is_safe(frames, *candidate, requested, from, to))
        .min_by(|left, right| {
            output[*left]
                .abs()
                .total_cmp(&output[*right].abs())
                .then_with(|| left.abs_diff(requested).cmp(&right.abs_diff(requested)))
                .then_with(|| left.cmp(right))
        })
        .unwrap_or(requested)
}

fn transition_candidate_is_safe(
    frames: &[StretchHybridFrameTrace],
    candidate: usize,
    requested: usize,
    from: StretchHybridOwner,
    to: StretchHybridOwner,
) -> bool {
    if to == StretchHybridOwner::Transient && candidate > requested {
        return false;
    }
    if from == StretchHybridOwner::Transient && candidate < requested {
        return false;
    }
    nearest_owner(frames, candidate) != StretchHybridOwner::Transient
}

pub(super) fn nearest_owner(
    frames: &[StretchHybridFrameTrace],
    output_frame: usize,
) -> StretchHybridOwner {
    frames
        .iter()
        .min_by_key(|frame| frame.output_frame.abs_diff(output_frame))
        .map(|frame| frame.owner)
        .unwrap_or(StretchHybridOwner::Mixed)
}

fn signed_frame_delta(frame: usize, reference: usize) -> i64 {
    if frame >= reference {
        frame.saturating_sub(reference).min(i64::MAX as usize) as i64
    } else {
        -(reference.saturating_sub(frame).min(i64::MAX as usize) as i64)
    }
}
