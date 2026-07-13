use super::super::render::Render;
use super::evidence::{EventEvidence, SampleContribution};

const SEARCH: usize = 512;
const EXCLUSION: usize = 32;

pub(super) fn matched_peaks(samples: &[f64], targets: [usize; 2]) -> ([usize; 2], usize) {
    let start = targets[0].saturating_sub(SEARCH);
    let end = (targets[1] + SEARCH + 1).min(samples.len());
    let first = peak(samples, start, end, None);
    let second = peak(samples, start, end, Some(first));
    let mut actual = [first, second];
    actual.sort_unstable();
    let unmatched = actual
        .iter()
        .zip(targets)
        .filter(|(actual, target)| actual.abs_diff(*target) > SEARCH)
        .count();
    (actual, unmatched)
}

pub(super) fn event_evidence(
    source: usize,
    target: usize,
    actual_peak: usize,
    render: &Render,
) -> EventEvidence {
    let phase = render
        .phase_trace
        .iter()
        .find(|frame| frame.source == source as isize && frame.output == target as isize);
    let contributions = render
        .synthesis_trace
        .iter()
        .filter_map(|frame| {
            frame
                .event_samples
                .iter()
                .find(|sample| sample.source == source && sample.output == target as isize)
                .map(|sample| SampleContribution {
                    frame_source: frame.source,
                    frame_output: frame.output,
                    frame_length: frame.length,
                    dual_weight: sample.dual_weight,
                    value: sample.value,
                    frame_peak_output: frame.peak_output,
                    frame_peak_magnitude: frame.peak_magnitude,
                })
        })
        .collect::<Vec<_>>();
    let sum = contributions
        .iter()
        .fold([0.0_f64; 2], |sum, contribution| {
            [
                sum[0] + contribution.value[0],
                sum[1] + contribution.value[1],
            ]
        });
    let absolute_sum = contributions
        .iter()
        .map(|contribution| contribution.value[0].abs())
        .sum::<f64>();
    let target_value = render.samples[0][target];
    EventEvidence {
        source,
        target,
        attached: render
            .synthesis_trace
            .iter()
            .any(|frame| frame.source == source as isize && frame.output == target as isize),
        phase_found: phase.is_some(),
        event_assignment: phase.is_some_and(|frame| frame.phase.event_assignment),
        owner_counts: phase
            .map(|frame| {
                [
                    frame.phase.owner_births,
                    frame.phase.owner_matches,
                    frame.phase.owner_retirements,
                    frame.phase.region_assignments,
                ]
            })
            .unwrap_or([0; 4]),
        active_state_hash: phase
            .map(|frame| frame.phase.active_state_hash)
            .unwrap_or(0),
        actual_peak,
        peak_error: actual_peak.abs_diff(target),
        target_value,
        peak_value: render.samples[0][actual_peak],
        local_peaks: local_peaks(&render.samples[0], target),
        closure_error: [(sum[0] - target_value).abs(), sum[1].abs()],
        cancellation_ratio: absolute_sum / sum[0].abs().max(1.0e-15),
        contributions,
    }
}

fn local_peaks(samples: &[f64], center: usize) -> [usize; 3] {
    let start = center.saturating_sub(SEARCH);
    let end = (center + SEARCH + 1).min(samples.len());
    let first = peak(samples, start, end, None);
    let second = peak(samples, start, end, Some(first));
    let mut excluded = [first, second];
    excluded.sort_unstable();
    let third = (start..end)
        .filter(|index| {
            excluded
                .iter()
                .all(|other| index.abs_diff(*other) > EXCLUSION)
        })
        .max_by(|left, right| samples[*left].abs().total_cmp(&samples[*right].abs()))
        .unwrap_or(start);
    [first, second, third]
}

fn peak(samples: &[f64], start: usize, end: usize, exclude: Option<usize>) -> usize {
    (start..end)
        .filter(|index| exclude.is_none_or(|other| index.abs_diff(other) > EXCLUSION))
        .max_by(|left, right| samples[*left].abs().total_cmp(&samples[*right].abs()))
        .unwrap_or(start)
}
