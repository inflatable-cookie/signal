use signal_primitives::Sample;

use crate::{
    phase_vocoder::{phase_vocoder, transient_reset_phase_vocoder},
    stretch_mono_with_engine, COMPRESSION_SHORT_WINDOW_SELECTOR_ANALYSIS_HOP,
    COMPRESSION_SHORT_WINDOW_SELECTOR_WINDOW_SIZE, SUSTAINED_COHERENCE_REVIEW_ANALYSIS_HOP,
    SUSTAINED_COHERENCE_REVIEW_WINDOW_SIZE,
};

use super::{
    build_hybrid_trace, StretchHybridOwner, StretchHybridTrace, StretchHybridTransitionTrace,
    TRANSITION_FRAMES,
};

mod transition;
mod types;

use transition::{apply_transition, evaluate_transition, TransitionEvaluation};
pub use types::{
    StretchHybridRender, StretchHybridTransitionDecision, StretchHybridTransitionRejection,
};

#[cfg(test)]
use transition::{max_normalization_gain_db, MAX_NORMALIZATION_GAIN_DB};

#[derive(Clone, Copy, Debug)]
struct OwnerSpan {
    owner: StretchHybridOwner,
    first_frame: usize,
    last_frame: usize,
}

pub(crate) fn build_hybrid_render(
    input: &[Sample],
    mixed: &[Sample],
    ratio: f64,
) -> StretchHybridRender {
    let trace = build_hybrid_trace(input, mixed, ratio);
    if mixed.is_empty() || trace.transitions.is_empty() {
        return unchanged_render(mixed, trace);
    }

    let transient = stretch_mono_with_engine(
        input,
        ratio,
        COMPRESSION_SHORT_WINDOW_SELECTOR_WINDOW_SIZE,
        COMPRESSION_SHORT_WINDOW_SELECTOR_ANALYSIS_HOP,
        phase_vocoder,
    );
    let tonal = if ratio > 1.0 {
        stretch_mono_with_engine(
            input,
            ratio,
            SUSTAINED_COHERENCE_REVIEW_WINDOW_SIZE,
            SUSTAINED_COHERENCE_REVIEW_ANALYSIS_HOP,
            transient_reset_phase_vocoder,
        )
    } else {
        mixed.to_vec()
    };
    if transient.len() != mixed.len() || tonal.len() != mixed.len() {
        return unchanged_render(mixed, trace);
    }

    let mut samples = mixed.to_vec();
    let mut transition_decisions = Vec::new();
    let mut applied_span_count = 0usize;
    let mut rejected_span_count = 0usize;
    let mut last_applied_end = 0usize;

    for span in owner_spans(&trace) {
        let branch = branch_for_owner(span.owner, mixed, &transient, &tonal);
        let Some((entry, exit)) = span_transitions(&trace, span) else {
            rejected_span_count += 1;
            continue;
        };
        let entry_range = transition_range(entry.scheduled_output_frame, mixed.len());
        let exit_range = transition_range(exit.scheduled_output_frame, mixed.len());
        if entry_range.1 > exit_range.0 || entry_range.0 < last_applied_end {
            transition_decisions.push(rejected_decision(
                entry,
                StretchHybridTransitionRejection::SpanTooShort,
            ));
            transition_decisions.push(rejected_decision(
                exit,
                StretchHybridTransitionRejection::SpanTooShort,
            ));
            rejected_span_count += 1;
            continue;
        }

        let entry_evaluation = evaluate_transition(mixed, branch, entry_range);
        let exit_evaluation = evaluate_transition(branch, mixed, exit_range);
        if entry_evaluation.rejection.is_some() || exit_evaluation.rejection.is_some() {
            transition_decisions.push(decision(entry, entry_evaluation, false));
            transition_decisions.push(decision(exit, exit_evaluation, false));
            rejected_span_count += 1;
            continue;
        }

        apply_transition(
            &mut samples,
            mixed,
            branch,
            entry_range,
            entry_evaluation.correlation,
        );
        samples[entry_range.1..exit_range.0].copy_from_slice(&branch[entry_range.1..exit_range.0]);
        apply_transition(
            &mut samples,
            branch,
            mixed,
            exit_range,
            exit_evaluation.correlation,
        );
        transition_decisions.push(decision(entry, entry_evaluation, true));
        transition_decisions.push(decision(exit, exit_evaluation, true));
        applied_span_count += 1;
        last_applied_end = exit_range.1;
    }

    StretchHybridRender {
        samples,
        trace,
        transition_decisions,
        applied_span_count,
        rejected_span_count,
    }
}

fn unchanged_render(mixed: &[Sample], trace: StretchHybridTrace) -> StretchHybridRender {
    StretchHybridRender {
        samples: mixed.to_vec(),
        trace,
        transition_decisions: Vec::new(),
        applied_span_count: 0,
        rejected_span_count: 0,
    }
}

fn owner_spans(trace: &StretchHybridTrace) -> Vec<OwnerSpan> {
    let mut spans = Vec::new();
    let mut start = 0usize;
    while start < trace.frames.len() {
        let owner = trace.frames[start].owner;
        let mut end = start;
        while end + 1 < trace.frames.len() && trace.frames[end + 1].owner == owner {
            end += 1;
        }
        if owner != StretchHybridOwner::Mixed {
            spans.push(OwnerSpan {
                owner,
                first_frame: start,
                last_frame: end,
            });
        }
        start = end + 1;
    }
    spans
}

fn span_transitions(
    trace: &StretchHybridTrace,
    span: OwnerSpan,
) -> Option<(StretchHybridTransitionTrace, StretchHybridTransitionTrace)> {
    let entry_requested = midpoint(
        trace
            .frames
            .get(span.first_frame.checked_sub(1)?)?
            .output_frame,
        trace.frames.get(span.first_frame)?.output_frame,
    );
    let exit_requested = midpoint(
        trace.frames.get(span.last_frame)?.output_frame,
        trace.frames.get(span.last_frame + 1)?.output_frame,
    );
    let entry = *trace
        .transitions
        .iter()
        .find(|transition| transition.requested_output_frame == entry_requested)?;
    let exit = *trace
        .transitions
        .iter()
        .find(|transition| transition.requested_output_frame == exit_requested)?;
    Some((
        StretchHybridTransitionTrace {
            from: StretchHybridOwner::Mixed,
            to: span.owner,
            ..entry
        },
        StretchHybridTransitionTrace {
            from: span.owner,
            to: StretchHybridOwner::Mixed,
            ..exit
        },
    ))
}

fn branch_for_owner<'a>(
    owner: StretchHybridOwner,
    mixed: &'a [Sample],
    transient: &'a [Sample],
    tonal: &'a [Sample],
) -> &'a [Sample] {
    match owner {
        StretchHybridOwner::Transient => transient,
        StretchHybridOwner::Mixed => mixed,
        StretchHybridOwner::Tonal => tonal,
    }
}

fn transition_range(center: usize, output_len: usize) -> (usize, usize) {
    let start = center.saturating_sub(TRANSITION_FRAMES / 2);
    let end = start.saturating_add(TRANSITION_FRAMES).min(output_len);
    (start, end)
}

fn decision(
    transition: StretchHybridTransitionTrace,
    evaluation: TransitionEvaluation,
    applied: bool,
) -> StretchHybridTransitionDecision {
    StretchHybridTransitionDecision {
        transition,
        correlation: evaluation.correlation,
        max_normalization_gain_db: evaluation.max_normalization_gain_db,
        applied,
        rejection: evaluation.rejection,
    }
}

fn rejected_decision(
    transition: StretchHybridTransitionTrace,
    rejection: StretchHybridTransitionRejection,
) -> StretchHybridTransitionDecision {
    StretchHybridTransitionDecision {
        transition,
        correlation: 0.0,
        max_normalization_gain_db: 0.0,
        applied: false,
        rejection: Some(rejection),
    }
}

fn midpoint(left: usize, right: usize) -> usize {
    left.saturating_add(right) / 2
}

#[cfg(test)]
#[path = "render/tests.rs"]
mod tests;
